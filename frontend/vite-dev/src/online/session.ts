// src/online/session.ts
//
// One live online game: the latest server snapshot, the viewer's seat, the
// connection state and a running move list, plus the actions a seated
// player can take. Snapshots arrive from REST responses and WS pushes; both
// go through `apply`, which also raises toasts for the transitions a player
// should notice (opponent joined, your turn, draw offered, game over).

import { computed, signal, type ReadonlySignal, type Signal } from "@preact/signals";
import * as online from "../api/online";
import type { GameState } from "../api/online";
import { formatLastMove } from "../core/fen";
import type { Color, GameMove } from "../core/types";
import { toast } from "../ui/toast";

export type ConnectionState = "connecting" | "live" | "reconnecting";

export interface OnlineGame {
  state: Signal<GameState>;
  /** Captured from the join/get response: WS pushes carry `your_color: null`. */
  myColor: Color | null;
  connection: Signal<ConnectionState>;
  history: Signal<string[]>;
  isMyTurn: ReadonlySignal<boolean>;
  move(m: GameMove): Promise<void>;
  resign(): Promise<void>;
  offerDraw(): Promise<void>;
  declineDraw(): Promise<void>;
  rematch(): Promise<GameState>;
  close(): void;
}

export function openOnlineGame(initial: GameState): OnlineGame {
  const myColor = initial.your_color;
  const state = signal(initial);
  const connection = signal<ConnectionState>("connecting");
  const history = signal<string[]>(initial.ply > 0 ? [`Joined at move ${initial.ply}`] : []);
  let historyPly = initial.ply;

  const isMyTurn = computed(() => {
    const s = state.value;
    return !s.result && myColor !== null && myColor === s.side_to_move;
  });

  const apply = (next: GameState) => {
    notify(state.value, next, myColor);
    state.value = next;
    // Only the latest move is recoverable from a snapshot (`lm=`), so a
    // jump of several plies (lag, reconnect) is summarised, not replayed.
    if (next.ply > historyPly) {
      const gap = next.ply - historyPly - 1;
      const added: string[] = [];
      if (gap > 0) added.push(`${gap} move${gap === 1 ? "" : "s"} not shown`);
      added.push(formatLastMove(next.fen) ?? "—");
      history.value = [...history.value, ...added];
      historyPly = next.ply;
    }
  };

  const sub = online.subscribeGame(initial.id, apply, (up) => {
    const prev = connection.value;
    connection.value = up ? "live" : "reconnecting";
    if (up && prev === "reconnecting") toast("Reconnected", "success", 2000);
    if (!up && prev === "live") toast("Connection lost. Reconnecting…", "warn", 4000);
  });

  return {
    state,
    myColor,
    connection,
    history,
    isMyTurn,
    async move(m) {
      apply(await online.submitMove(initial.id, m));
    },
    async resign() {
      apply(await online.resignGame(initial.id));
    },
    async offerDraw() {
      apply(await online.offerDraw(initial.id));
    },
    async declineDraw() {
      apply(await online.declineDraw(initial.id));
    },
    rematch() {
      return online.rematchGame(initial.id);
    },
    close() {
      sub.close();
    },
  };
}

function notify(prev: GameState, next: GameState, my: Color | null) {
  if (my === null) return;
  const opponent = (s: GameState) => (my === "White" ? s.black : s.white);
  if (!opponent(prev) && opponent(next)) {
    toast(`${opponent(next)!.name || "Your opponent"} joined. Game on!`, "success");
  }

  if (!prev.result && next.result) {
    const r = next.result;
    toast(r.kind === "Draw" ? "Game drawn" : r.winner === my ? "You won!" : `${r.winner} wins`, "info", 6000);
  } else {
    const wasMine = !prev.result && my === prev.side_to_move;
    const isMine = !next.result && my === next.side_to_move;
    if (!wasMine && isMine && opponent(next)) toast("Your move", "info", 2500);
  }

  if (!prev.rematch && next.rematch) toast("A rematch is ready", "success", 5000);

  const oppOffer = (s: GameState) => s.draw_offer !== null && s.draw_offer !== my;
  if (!oppOffer(prev) && oppOffer(next)) toast("Your opponent offers a draw", "info", 5000);
}
