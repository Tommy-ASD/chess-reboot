// src/api/board.ts
//
// Stateless engine calls: legal moves for a square, apply a move to a FEN,
// and the status of a position. Used by same-device games, piece demos and
// the Lab. Online games move through `online.ts` instead, where the server
// owns the position.

import { API_BASE } from "./config";
import type { Coord, GameMove, GameStatus } from "../core/types";

export type MoveResult = { fen: string; status: GameStatus };

/// Structured failure body the backend returns on 4xx. Mirrors
/// `MakeMoveErrorBody` in `api/src/main.rs`.
type ErrorBody = { code?: string; message?: string };

async function readError(response: Response, context: string): Promise<Error> {
  let text = "";
  try {
    text = await response.text();
  } catch {
    return new Error(`${context}: HTTP ${response.status}`);
  }
  try {
    const parsed = JSON.parse(text) as ErrorBody;
    console.error(`${context}: server returned ${response.status}`, parsed);
    const code = parsed.code ? ` [${parsed.code}]` : "";
    return new Error(`${parsed.message ?? text}${code}`);
  } catch {
    return new Error(`${context}: HTTP ${response.status} — ${text || "(empty body)"}`);
  }
}

/// Network-level failure (server down, CORS, offline) gets a message a
/// player can act on instead of "Failed to fetch".
async function post(path: string, body: unknown, context: string): Promise<Response> {
  let response: Response;
  try {
    response = await fetch(`${API_BASE}${path}`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(body),
    });
  } catch {
    throw new Error("Can't reach the game server. Is it running?");
  }
  if (!response.ok) throw await readError(response, context);
  return response;
}

export async function fetchMoves(fen: string, from: Coord): Promise<GameMove[]> {
  const response = await post("/board/moves", { board_fen: fen, from }, "fetchMoves");
  const data = (await response.json()) as { moves: GameMove[] };
  return data.moves;
}

export async function fetchStatus(fen: string): Promise<GameStatus> {
  const response = await post("/board/status", { board_fen: fen }, "fetchStatus");
  const data = (await response.json()) as { status: GameStatus };
  return data.status;
}

export async function applyMove(fen: string, move: GameMove): Promise<MoveResult> {
  const response = await post("/board/new_state", { board_fen: fen, game_move: move }, "applyMove");
  const data = (await response.json()) as { new_board_fen: string; status: GameStatus };
  return { fen: data.new_board_fen, status: data.status };
}
