// src/symmetrize.ts
//
// "Duplicate half" for the FEN editor: set up one side of the board,
// then copy it — reflected — onto the other side. Unlike `mirror.ts`,
// which reflects the whole position, this keeps the source half as-is
// and overwrites only the destination half.
//
// The copy is a true reflection, so it reuses `mirrorBoard` for the
// geometry: track directions, junction branches, loco last-direction
// and Goblin home squares all come out pointing the right way.
// On top of that, piece colours can optionally be inverted — the
// natural default when copying one army across the middle rank.

import { cloneSquare, mirrorBoard, type MirrorAxis } from "./mirror";
import type { Square } from "./types";

/// Which half is the source. Its reflection overwrites the opposite
/// half; on an odd dimension the middle row/column is left untouched.
export type HalfSource = "left" | "right" | "top" | "bottom";

export const HALF_SOURCES: readonly HalfSource[] = ["left", "right", "top", "bottom"];

/// How piece colours are treated on the copied half.
///  - `auto`:   invert when copying across the middle rank (top ↔
///              bottom — one army facing the other), keep when copying
///              across the middle file (left ↔ right — same army).
///  - `keep`:   never invert.
///  - `invert`: always invert.
export type ColorMode = "auto" | "keep" | "invert";

export const HALF_LABELS: Record<HalfSource, { short: string; title: string }> = {
  left:   { short: "Left → Right", title: "Copy the left half onto the right half, reflected" },
  right:  { short: "Right → Left", title: "Copy the right half onto the left half, reflected" },
  top:    { short: "Top → Bottom", title: "Copy the top half onto the bottom half, reflected" },
  bottom: { short: "Bottom → Top", title: "Copy the bottom half onto the top half, reflected" },
};

function axisOf(source: HalfSource): MirrorAxis {
  return source === "left" || source === "right" ? "horizontal" : "vertical";
}

/// Resolve a colour mode to a yes/no for a given copy direction.
export function shouldInvertColors(source: HalfSource, mode: ColorMode): boolean {
  if (mode === "keep") return false;
  if (mode === "invert") return true;
  return axisOf(source) === "vertical";
}

// ----------------------------------------------
// Colour inversion
// ----------------------------------------------

/// Index of the `)` matching the `(` at `open`, or -1 when unbalanced.
function findMatchingParen(s: string, open: number): number {
  let depth = 0;
  for (let i = open; i < s.length; i++) {
    if (s[i] === "(") depth++;
    else if (s[i] === ")") {
      depth--;
      if (depth === 0) return i;
    }
  }
  return -1;
}

/// Split on commas that aren't inside parentheses.
function splitTopLevel(input: string): string[] {
  const parts: string[] = [];
  let buf = "";
  let depth = 0;
  for (const ch of input) {
    if (ch === "(") depth++;
    else if (ch === ")") depth--;
    if (ch === "," && depth === 0) {
      parts.push(buf);
      buf = "";
    } else {
      buf += ch;
    }
  }
  if (buf.length > 0) parts.push(buf);
  return parts;
}

/// Swap a piece symbol's colour. Colour is the case of the leading
/// symbol (`K`/`k`, `G`/`g`, `BUS`/`bus`), matching the engine's
/// `symbol_to_piece`. Trains (`LOCO` / `CART`) are Neutral and keep
/// their symbol.
///
/// Only the `P=` field of the payload is touched: it holds the pieces a
/// carrier transports (`BUS(P=(N,p))`) or a Goblin has kidnapped
/// (`g(H=0-0,P=N)`), and those swap sides with their carrier. Every
/// other field (`H=F` headings, `H=0-7` homes, ids) passes through
/// verbatim — blindly flipping case would corrupt them.
export function invertPieceColor(piece: string): string {
  const open = piece.indexOf("(");
  const base = open === -1 ? piece : piece.slice(0, open);

  const upper = base.toUpperCase();
  const neutral = upper === "LOCO" || upper === "CART";
  const nextBase = neutral ? base : base === upper ? base.toLowerCase() : upper;

  if (open === -1) return nextBase;
  const close = findMatchingParen(piece, open);
  if (close === -1) return nextBase + piece.slice(open);

  const fields = splitTopLevel(piece.slice(open + 1, close)).map(field => {
    const eq = field.indexOf("=");
    if (eq === -1 || field.slice(0, eq).trim() !== "P") return field;
    const value = field.slice(eq + 1).trim();
    if (value.startsWith("(") && value.endsWith(")")) {
      const inner = splitTopLevel(value.slice(1, -1)).map(p => invertPieceColor(p.trim()));
      return `P=(${inner.join(",")})`;
    }
    return `P=${invertPieceColor(value)}`;
  });

  return `${nextBase}(${fields.join(",")})${piece.slice(close + 1)}`;
}

// ----------------------------------------------
// Duplicate half
// ----------------------------------------------

/// Is `(rank, file)` in the half that gets overwritten?
function inDestination(
  source: HalfSource,
  rank: number,
  file: number,
  rows: number,
  cols: number,
): boolean {
  const halfCols = Math.floor(cols / 2);
  const halfRows = Math.floor(rows / 2);
  switch (source) {
    case "left":   return file >= cols - halfCols;
    case "right":  return file < halfCols;
    case "top":    return rank >= rows - halfRows;
    case "bottom": return rank < halfRows;
  }
}

/// Copy the `source` half onto the opposite half, reflected, optionally
/// inverting piece colours on the copy. Returns a fresh board; the
/// source half and any odd middle row/column are unchanged copies of
/// the input.
export function symmetrizeBoard(
  board: Square[][],
  source: HalfSource,
  invertColors: boolean,
): Square[][] {
  const rows = board.length;
  const cols = board[0]?.length ?? 0;
  const reflected = mirrorBoard(board, axisOf(source));

  return board.map((row, rank) =>
    row.map((sq, file) => {
      if (!inDestination(source, rank, file, rows, cols)) return cloneSquare(sq);
      const copy = reflected[rank][file];
      if (invertColors && copy.piece) copy.piece = invertPieceColor(copy.piece);
      return copy;
    }),
  );
}
