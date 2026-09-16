// src/mirror.ts
//
// Board mirroring for the FEN editor. Reflecting a position is not a
// pure array shuffle: several payloads encode *direction* (`D=` on
// Track, `BRANCHES=` on Junction, `L=` on a Locomotive) or an absolute
// *coordinate* (`H=file-rank`, the Goblin's home square), and both have
// to be transformed alongside the geometry or the mirrored board plays
// differently from its reflection.
//
// Signal wiring (`ID=` / `TARGETS=`) is deliberately left alone — it's
// id-based, not positional, so it survives any reflection unchanged.

import type { BoardFlags, CastlingRights } from "./fen";
import {
  ALL_TRACK_DIRS,
  getJunctionBranches,
  setJunctionBranches,
  type TrackDir,
} from "./signal_payload";
import type { Square } from "./types";

/// The four reflections the editor offers. `diagonal` is the transpose
/// across the a8–h1 diagonal — it swaps the file and rank axes, so on a
/// non-square board it also swaps the dimensions.
export type MirrorAxis = "horizontal" | "vertical" | "both" | "diagonal";

export const MIRROR_AXES: readonly MirrorAxis[] = [
  "horizontal",
  "vertical",
  "both",
  "diagonal",
];

/// Where the square at output `(rank, file)` comes from in the source
/// grid. Every other transform in this module is derived from this one
/// mapping, so the geometry only exists in one place.
function sourceOf(
  axis: MirrorAxis,
  rank: number,
  file: number,
  rows: number,
  cols: number,
): { rank: number; file: number } {
  switch (axis) {
    case "horizontal": return { rank, file: cols - 1 - file };
    case "vertical":   return { rank: rows - 1 - rank, file };
    case "both":       return { rank: rows - 1 - rank, file: cols - 1 - file };
    case "diagonal":   return { rank: file, file: rank };
  }
}

/// Map a cardinal direction through the reflection. Horizontal flips
/// the file axis (E↔W), vertical the rank axis (N↔S), and the diagonal
/// swaps the two axes (N↔W, S↔E).
export function mirrorDir(dir: TrackDir, axis: MirrorAxis): TrackDir {
  const flipFile = (d: TrackDir): TrackDir => (d === "E" ? "W" : d === "W" ? "E" : d);
  const flipRank = (d: TrackDir): TrackDir => (d === "N" ? "S" : d === "S" ? "N" : d);
  switch (axis) {
    case "horizontal": return flipFile(dir);
    case "vertical":   return flipRank(dir);
    case "both":       return flipRank(flipFile(dir));
    case "diagonal":
      return dir === "N" ? "W" : dir === "W" ? "N" : dir === "S" ? "E" : "S";
  }
}

/// Mirror an absolute board coordinate. Same mapping as `sourceOf` —
/// a reflection is its own inverse for the three axis flips, and the
/// transpose likewise, so one function serves both directions.
function mirrorCoord(
  file: number,
  rank: number,
  axis: MirrorAxis,
  rows: number,
  cols: number,
): { file: number; rank: number } {
  const src = sourceOf(axis, rank, file, rows, cols);
  return { file: src.file, rank: src.rank };
}

export function cloneSquare(sq: Square): Square {
  const next: Square = {
    piece: sq.piece,
    squareType: sq.squareType,
    conditions: [...sq.conditions],
  };
  if (sq.duck) next.duck = true;
  if (sq.extraFields) next.extraFields = { ...sq.extraFields };
  return next;
}

function isTrackDir(v: string | undefined): v is TrackDir {
  return v !== undefined && (ALL_TRACK_DIRS as readonly string[]).includes(v);
}

/// Rewrite the direction/coordinate payloads carried by a piece symbol.
/// Both patterns are matched with a regex rather than a parse so they
/// also reach payloads nested inside a carrier's passenger list (a
/// kidnapping Goblin riding a Bus keeps a valid home square).
///
/// `H=` is overloaded: the Goblin writes `H=<file>-<rank>` (a coord)
/// and a Locomotive writes `H=F` / `H=R` (a heading). Requiring digits
/// keeps the two apart — a heading has no coordinate to mirror.
function mirrorPiecePayload(
  piece: string,
  axis: MirrorAxis,
  rows: number,
  cols: number,
): string {
  return piece
    .replace(/H=(\d+)-(\d+)/g, (_m, f: string, r: string) => {
      const c = mirrorCoord(Number(f), Number(r), axis, rows, cols);
      return `H=${c.file}-${c.rank}`;
    })
    // `L=` is the Locomotive's "direction I entered this tile through".
    .replace(/L=([NSEW])(?=[,)]|$)/g, (_m, d: string) =>
      `L=${mirrorDir(d as TrackDir, axis)}`);
}

/// Mirror one square's contents, returning a fresh copy. Exported for
/// the unit tests; the board walk below is the real entry point.
export function mirrorSquare(
  sq: Square,
  axis: MirrorAxis,
  rows: number,
  cols: number,
): Square {
  const next = cloneSquare(sq);

  if (next.piece) next.piece = mirrorPiecePayload(next.piece, axis, rows, cols);

  if (next.extraFields) {
    const d = next.extraFields.D;
    if (isTrackDir(d)) next.extraFields.D = mirrorDir(d, axis);

    // `BRANCHES=(N,E)` — go through the payload helpers so the list
    // comes back out in the canonical parenthesised form.
    if (next.extraFields.BRANCHES !== undefined) {
      const dirs = getJunctionBranches(next);
      if (dirs.length > 0) {
        setJunctionBranches(next, dirs.map(d => mirrorDir(d, axis)));
      }
    }
  }

  return next;
}

/// Reflect the whole grid. The returned board is a fresh deep copy —
/// nothing is shared with the input, so the caller can swap it in
/// wholesale.
export function mirrorBoard(board: Square[][], axis: MirrorAxis): Square[][] {
  const rows = board.length;
  const cols = board[0]?.length ?? 0;
  if (rows === 0 || cols === 0) return board.map(row => row.map(cloneSquare));

  const outRows = axis === "diagonal" ? cols : rows;
  const outCols = axis === "diagonal" ? rows : cols;

  const next: Square[][] = [];
  for (let r = 0; r < outRows; r++) {
    const row: Square[] = [];
    for (let c = 0; c < outCols; c++) {
      const src = sourceOf(axis, r, c, rows, cols);
      row.push(mirrorSquare(board[src.rank][src.file], axis, rows, cols));
    }
    next.push(row);
  }
  return next;
}

function mirrorCastling(c: CastlingRights, axis: MirrorAxis): CastlingRights {
  // A horizontal flip carries each king to the other wing, so what was
  // a kingside right becomes a queenside one. A vertical flip moves the
  // armies between the near and far rank but doesn't change either
  // side's *colour*, so its rights travel with it untouched. Castling
  // is meaningless across the transpose — left as-is rather than
  // silently invented.
  const swapWings = axis === "horizontal" || axis === "both";
  if (!swapWings) return { ...c };
  return { K: c.Q, Q: c.K, k: c.q, q: c.k };
}

/// Mirror an algebraic en-passant target ("e3"). Returns the input
/// unchanged when it isn't a square this board can express — a
/// hand-typed placeholder shouldn't be mangled into a plausible-looking
/// wrong square.
function mirrorEnPassant(
  ep: string | null,
  axis: MirrorAxis,
  rows: number,
  cols: number,
): string | null {
  if (ep === null) return null;
  const m = /^([a-z])(\d+)$/.exec(ep.trim());
  if (!m) return ep;
  const file = m[1].charCodeAt(0) - 97;
  const rankFromBottom = Number(m[2]);
  // Algebraic ranks count from the bottom; the grid indexes from the top.
  const rank = rows - rankFromBottom;
  if (file < 0 || file >= cols || rank < 0 || rank >= rows) return ep;

  const c = mirrorCoord(file, rank, axis, rows, cols);
  const outRows = axis === "diagonal" ? cols : rows;
  const outCols = axis === "diagonal" ? rows : cols;
  if (c.file >= outCols || c.rank >= outRows) return ep;
  return `${String.fromCharCode(97 + c.file)}${outRows - c.rank}`;
}

/// Mirror the flag fields that reference board geometry. Side to move,
/// ply count, tick rate, variants and duck phase are all orientation-
/// independent and pass through unchanged.
export function mirrorFlags(
  flags: BoardFlags,
  axis: MirrorAxis,
  rows: number,
  cols: number,
): BoardFlags {
  return {
    ...flags,
    castling: mirrorCastling(flags.castling, axis),
    enPassant: mirrorEnPassant(flags.enPassant, axis, rows, cols),
    variants: [...flags.variants],
  };
}

/// Mirror a full position — grid plus the geometry-bearing flags.
export function mirrorPosition(
  board: Square[][],
  flags: BoardFlags,
  axis: MirrorAxis,
): { board: Square[][]; flags: BoardFlags } {
  const rows = board.length;
  const cols = board[0]?.length ?? 0;
  return {
    board: mirrorBoard(board, axis),
    flags: mirrorFlags(flags, axis, rows, cols),
  };
}

/// Button labels + tooltips for the editor's mirror row.
export const MIRROR_LABELS: Record<MirrorAxis, { short: string; title: string }> = {
  horizontal: { short: "⇄ Files", title: "Mirror left ↔ right (flip the file axis)" },
  vertical:   { short: "⇅ Ranks", title: "Mirror top ↔ bottom (flip the rank axis)" },
  both:       { short: "⟳ 180°", title: "Mirror both axes (rotate the board 180°)" },
  diagonal:   { short: "⤡ Diagonal", title: "Mirror across the diagonal (swaps width and height)" },
};
