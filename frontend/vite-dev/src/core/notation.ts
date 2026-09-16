// src/core/notation.ts
//
// Human-readable square and move names. Engine coords are (file, rank)
// with rank 0 at the top (black's back rank on a standard board), so the
// algebraic rank is `rows - rank`.

import type { Coord, GameMove } from "./types";

const FILES = "abcdefghijklmnopqrstuvwxyz";

/// `e4`-style name for a square on a `rows`-tall board. Boards wider than
/// 26 files fall back to a numeric file.
export function squareName(c: Coord, rows: number): string {
  const file = c.file < FILES.length ? FILES[c.file] : `f${c.file}`;
  return `${file}${rows - c.rank}`;
}

const PROMOTION_NAMES = { Queen: "queen", Rook: "rook", Bishop: "bishop", Knight: "knight" } as const;

/// Button label for a move that isn't made by clicking a destination
/// (see `isSpecialMove`).
export function describeSpecialMove(m: GameMove, rows: number): string {
  switch (m.move_type.kind) {
    case "PhaseShift":
      return "Grow brainrot";
    case "ThrowSwitch":
      return "Throw the switch";
    case "PlaceTornado":
      return `Summon a tornado on ${squareName(m.move_type.target.target, rows)}`;
    case "Promotion":
      return `Promote to ${PROMOTION_NAMES[m.move_type.target.into]}`;
    default:
      return m.move_type.kind;
  }
}
