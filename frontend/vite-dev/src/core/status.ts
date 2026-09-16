// src/core/status.ts
//
// Player-facing wording for engine statuses and online results.

import type { Color, GameStatus } from "./types";

export interface StatusText {
  title: string;
  detail?: string;
  /** `check` is informational; `over` means the game has ended. */
  kind: "check" | "over";
}

/// Banner text for a `GameStatus`, or null while the game simply goes on.
/// Exhaustive over the engine's variants: adding one there is a TS error
/// here until it's worded.
export function describeStatus(status: GameStatus): StatusText | null {
  switch (status.status) {
    case "Ongoing":
      return null;
    case "Check":
      return { title: "Check", detail: `${status.data.side_to_move} must answer it.`, kind: "check" };
    case "Checkmate":
      return { title: `${status.data.winner} wins`, detail: "Checkmate.", kind: "over" };
    case "Stalemate":
      return { title: "Draw", detail: "Stalemate: no legal moves, but no check.", kind: "over" };
    case "BrainrotWin":
      return { title: `${status.data.winner} wins`, detail: "Brainrot: the opponent is frozen with no moves.", kind: "over" };
    case "BrainrotLockout":
      return { title: `${status.data.winner} wins`, detail: "Brainrot lockout: nothing can stop a phase-4 Skibidi now.", kind: "over" };
    case "Win":
      return { title: `${status.data.winner} wins`, detail: "The king was captured.", kind: "over" };
  }
}

export function isTerminal(status: GameStatus): boolean {
  return describeStatus(status)?.kind === "over";
}

/// The winning side of a decided board, or null (ongoing, check, draw).
export function statusWinner(status: GameStatus): Color | null {
  switch (status.status) {
    case "Checkmate":
    case "BrainrotWin":
    case "BrainrotLockout":
    case "Win":
      return status.data.winner;
    default:
      return null;
  }
}

/// Online games can end without a terminal board (resignation, agreed
/// draw); the server records those as a `GameResult`.
export type GameResult =
  | { kind: "Decisive"; winner: Color }
  | { kind: "Resignation"; winner: Color }
  | { kind: "Draw" };

export function describeResult(result: GameResult, status: GameStatus): StatusText {
  const fromBoard = describeStatus(status);
  if (fromBoard?.kind === "over") return fromBoard;
  switch (result.kind) {
    case "Draw":
      return { title: "Draw", detail: "Agreed by both players.", kind: "over" };
    case "Resignation":
      return { title: `${result.winner} wins`, detail: `${result.winner === "White" ? "Black" : "White"} resigned.`, kind: "over" };
    case "Decisive":
      return { title: `${result.winner} wins`, kind: "over" };
  }
}
