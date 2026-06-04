// src/online.ts
//
// The online-session module: a thin REST client for the multiplayer
// endpoints (create / list / join / get / move / resign) and a pair of
// auto-reconnecting WebSocket subscriptions (a single game, and the
// public lobby). Actions go over REST (validated request/response);
// live updates arrive over WS as server-pushed `GameState` snapshots.
//
// This module is transport only — it holds no UI and no coupling into
// the board renderer. `main.ts` owns the active session and routes the
// board through it.

import { API_BASE, WS_BASE } from "./config";
import { authHeaders } from "./player";
import type { Color, GameMove, GameStatus } from "./variables";

/// A seated player as the server serializes it (`PlayerSlot`).
export type OnlinePlayer = { id: string; name: string };

/// Why a game ended (`GameResult`, internally tagged on `kind`). `null`
/// on a live `GameState` means in progress.
export type GameResult =
  | { kind: "Decisive"; winner: Color }
  | { kind: "Resignation"; winner: Color }
  | { kind: "Draw" };

/// Mirrors the server's `GameState` snapshot (REST responses + WS pushes).
/// Field names are the Rust struct's, serialized verbatim. NOTE:
/// `your_color` is populated only on REST responses to the seated player;
/// WS pushes use `snapshot(None)` and carry `your_color: null`, so the
/// caller must remember its colour from create/join rather than trust the
/// pushed value.
export type GameState = {
  id: string;
  code: string;
  name: string;
  public: boolean;
  white: OnlinePlayer | null;
  black: OnlinePlayer | null;
  fen: string;
  side_to_move: Color;
  ply: number;
  status: GameStatus;
  result: GameResult | null;
  /// Set once a rematch has been created from this finished game (its id);
  /// both players learn it from the old game's live feed.
  rematch: string | null;
  /// The colour with an outstanding draw offer, if any.
  draw_offer: Color | null;
  your_color: Color | null;
};

/// Body for `POST /games`. `public` defaults false server-side; a custom
/// `starting_fen` enables Duck Chess / variant games; `color` is the
/// host's seat preference (defaults White).
export type CreateGameRequest = {
  name?: string;
  public: boolean;
  starting_fen?: string;
  color?: Color;
};

// ---------------------------------------------------------------------
// REST
// ---------------------------------------------------------------------

/// Read a non-2xx body into an `Error`, preferring the server's
/// structured `{ code, message }` shape and falling back to raw text.
async function readError(res: Response, context: string): Promise<Error> {
  let text = "";
  try {
    text = await res.text();
  } catch {
    return new Error(`${context}: HTTP ${res.status}`);
  }
  try {
    const body = JSON.parse(text) as { code?: string; message?: string };
    const code = body.code ? ` [${body.code}]` : "";
    return new Error(`${body.message ?? text}${code}`);
  } catch {
    return new Error(`${context}: HTTP ${res.status} — ${text || "(no body)"}`);
  }
}

async function api<T>(path: string, init?: RequestInit): Promise<T> {
  const res = await fetch(`${API_BASE}${path}`, {
    ...init,
    headers: {
      "Content-Type": "application/json",
      ...authHeaders(),
      ...(init?.headers ?? {}),
    },
  });
  if (!res.ok) throw await readError(res, path);
  return (await res.json()) as T;
}

export function createGame(req: CreateGameRequest): Promise<GameState> {
  return api<GameState>("/games", { method: "POST", body: JSON.stringify(req) });
}

export function listGames(): Promise<GameState[]> {
  return api<GameState[]>("/games");
}

/// Seat the caller in `idOrCode` (the server accepts a game UUID or the
/// short share code).
export function joinGame(idOrCode: string): Promise<GameState> {
  return api<GameState>(`/games/${encodeURIComponent(idOrCode)}/join`, {
    method: "POST",
  });
}

export function getGame(idOrCode: string): Promise<GameState> {
  return api<GameState>(`/games/${encodeURIComponent(idOrCode)}`);
}

export function submitMove(idOrCode: string, move: GameMove): Promise<GameState> {
  return api<GameState>(`/games/${encodeURIComponent(idOrCode)}/move`, {
    method: "POST",
    body: JSON.stringify({ game_move: move }),
  });
}

export function resignGame(idOrCode: string): Promise<GameState> {
  return api<GameState>(`/games/${encodeURIComponent(idOrCode)}/resign`, {
    method: "POST",
  });
}

/// Create (or re-fetch) a swapped-colours rematch of a finished game. Both
/// players are pre-seated; the returned state is the new game.
export function rematchGame(idOrCode: string): Promise<GameState> {
  return api<GameState>(`/games/${encodeURIComponent(idOrCode)}/rematch`, {
    method: "POST",
  });
}

/// Public in-progress games (both seats filled, not finished) — spectatable.
export function listLive(): Promise<GameState[]> {
  return api<GameState[]>("/games/live");
}

/// Offer a draw, or accept the opponent's outstanding offer.
export function offerDraw(idOrCode: string): Promise<GameState> {
  return api<GameState>(`/games/${encodeURIComponent(idOrCode)}/draw`, {
    method: "POST",
  });
}

/// Decline / withdraw a draw offer.
export function declineDraw(idOrCode: string): Promise<GameState> {
  return api<GameState>(`/games/${encodeURIComponent(idOrCode)}/draw/decline`, {
    method: "POST",
  });
}

// ---------------------------------------------------------------------
// WebSocket (server -> client push, auto-reconnecting)
// ---------------------------------------------------------------------

/// A live subscription. Call `close()` to tear it down (e.g. leaving a
/// game); after that no reconnect is attempted.
export type Subscription = { close(): void };

/// Connect to `url`, parse each text frame as JSON, and invoke
/// `onMessage`. Reconnects with capped exponential backoff on an
/// unexpected close, until `close()` is called.
function subscribe(
  url: string,
  onMessage: (data: unknown) => void,
  onStatus?: (connected: boolean) => void,
): Subscription {
  let ws: WebSocket | null = null;
  let closed = false;
  let attempt = 0;
  let timer: ReturnType<typeof setTimeout> | null = null;

  const connect = (): void => {
    if (closed) return;
    ws = new WebSocket(url);
    ws.onopen = () => {
      attempt = 0;
      onStatus?.(true);
    };
    ws.onmessage = (ev) => {
      try {
        onMessage(JSON.parse(ev.data as string));
      } catch (err) {
        console.error(`ws ${url}: bad message`, err);
      }
    };
    ws.onclose = () => {
      // An intentional `close()` is not a dropped connection — stay quiet.
      if (closed) return;
      onStatus?.(false);
      const delay = Math.min(1000 * 2 ** attempt, 10000);
      attempt += 1;
      timer = setTimeout(connect, delay);
    };
    ws.onerror = () => {
      // `onclose` always follows `onerror`; reconnect is handled there.
    };
  };

  connect();
  return {
    close() {
      closed = true;
      if (timer) clearTimeout(timer);
      ws?.close();
    },
  };
}

/// Subscribe to a single game; `onState` fires on connect (initial
/// snapshot) and on every server-side change.
export function subscribeGame(
  idOrCode: string,
  onState: (state: GameState) => void,
  onStatus?: (connected: boolean) => void,
): Subscription {
  return subscribe(
    `${WS_BASE}/ws/games/${encodeURIComponent(idOrCode)}`,
    (data) => onState(data as GameState),
    onStatus,
  );
}

/// Subscribe to the public lobby; `onList` fires on connect and whenever
/// the public-games list changes (create / join / finish).
export function subscribeLobby(
  onList: (games: GameState[]) => void,
): Subscription {
  return subscribe(`${WS_BASE}/ws/lobby`, (data) => onList(data as GameState[]));
}
