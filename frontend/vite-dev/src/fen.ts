// src/fen.ts

import type { Square, SquareType } from "./variables";

// ----------------------------------------------
// Full-FEN flag fields (used by the dedicated editor)
// ----------------------------------------------

export type Side = "w" | "b";

export type CastlingRights = {
  K: boolean; // white kingside
  Q: boolean; // white queenside
  k: boolean; // black kingside
  q: boolean; // black queenside
};

export type BoardFlags = {
  sideToMove: Side;
  castling: CastlingRights;
  /** Algebraic square ("e3") or null when no en-passant target. */
  enPassant: string | null;
};

export const DEFAULT_FLAGS: BoardFlags = {
  sideToMove: "w",
  castling: { K: true, Q: true, k: true, q: true },
  enPassant: null,
};

/// Split a full FEN ("<grid> <stm> <castling> <ep>") into its grid +
/// flags. Missing trailing fields default to white-to-move with all
/// castle rights and no en-passant target — matching `fen_to_board`'s
/// fallback in the engine, so a bare grid round-trips identically.
export function parseFENFlags(fen: string): BoardFlags {
  const parts = fen.trim().split(/\s+/);
  // parts[0] is the grid; the rest are flag fields.
  const stm = parts[1];
  const castle = parts[2];
  const ep = parts[3];

  const sideToMove: Side = stm === "b" ? "b" : "w";

  const castling: CastlingRights = castle === undefined
    ? { K: true, Q: true, k: true, q: true }
    : castle === "-"
      ? { K: false, Q: false, k: false, q: false }
      : {
        K: castle.includes("K"),
        Q: castle.includes("Q"),
        k: castle.includes("k"),
        q: castle.includes("q"),
      };

  const enPassant = ep === undefined || ep === "-" ? null : ep;

  return { sideToMove, castling, enPassant };
}

function castlingToFEN(c: CastlingRights): string {
  let s = "";
  if (c.K) s += "K";
  if (c.Q) s += "Q";
  if (c.k) s += "k";
  if (c.q) s += "q";
  return s.length === 0 ? "-" : s;
}

/// Build a full FEN string from a board grid + flags.
export function serializeFullFEN(board: Square[][], flags: BoardFlags): string {
  const grid = squaresToFEN(board);
  const stm = flags.sideToMove;
  const castling = castlingToFEN(flags.castling);
  const ep = flags.enPassant ?? "-";
  return `${grid} ${stm} ${castling} ${ep}`;
}

// For pretty optional rendering
export const pieceToSymbol = (p: string): string => {
  const map: Record<string, string> = {
    "K": "♔", "Q": "♕", "R": "♖",
    "B": "♗", "N": "♘", "P": "♙",
    "k": "♚", "q": "♛", "r": "♜",
    "b": "♝", "n": "♞", "p": "♟",
  };
  return map[p] ?? p;
};

export const pieceToImage = (p: string): string | undefined => {
  // for custom pieces
  // currently, skibidi and bus
  // currently let's just take letters before first paranthesis
  const base = p.split("(")[0];
  // if first char is lowercase, turn entire base to lowercase
  // likewise, if uppercase, turn entire base to uppercase
  // this is so "Bus" and "BUS" both map to same image
  // only the first letter matters for color
  if (base[0] >= "a" && base[0] <= "z") {
    base.toLowerCase();
  } else if (base[0] >= "A" && base[0] <= "Z") {
    base.toUpperCase();
  }
  // map base to image filename
  const map: Record<string, string> = {
    "G": "/img/pieces/Goblin white.png",
    "g": "/img/pieces/Goblin black.png",
    "BUS": "/img/pieces/Bus white.png",
    "bus": "/img/pieces/Bus black.png",
  };
  console.log("pieceToImage:", p, "->", map[base]);
  return map[base];
}


// ----------------------------------------------
// Bus passenger extraction
// ----------------------------------------------

/// Parse a Bus FEN piece string into its passenger symbols, in order.
/// e.g. "BUS(P=(N,P,p))" -> ["N", "P", "p"]
/// "BUS" or "bus" with no inner content -> []
/// Mirrors the Rust parser in engine/src/pieces/fairy/bus.rs.
export function getBusPassengers(busPiece: string): string[] {
  const openIdx = busPiece.indexOf("(");
  if (openIdx === -1) return [];

  // Find matching close paren
  let depth = 0;
  let closeIdx = -1;
  for (let i = openIdx; i < busPiece.length; i++) {
    const ch = busPiece[i];
    if (ch === "(") depth++;
    else if (ch === ")") {
      depth--;
      if (depth === 0) { closeIdx = i; break; }
    }
  }
  if (closeIdx === -1) return [];

  const inner = busPiece.slice(openIdx + 1, closeIdx);
  // Inner is e.g. "P=(N,P,p)". Find the P=... field.
  for (const field of splitTopLevel(inner)) {
    const [key, value] = splitKeyValue(field);
    if (key !== "P") continue;
    // value is "(N,P,p)" — strip the wrapping parens
    if (!value.startsWith("(") || !value.endsWith(")")) return [];
    return splitTopLevel(value.slice(1, -1));
  }
  return [];
}


// ----------------------------------------------
// Top-level split (nested parentheses-safe)
// ----------------------------------------------

function splitTopLevel(input: string): string[] {
  const parts: string[] = [];
  let buf = "";
  let depth = 0;

  for (const ch of input) {
    if (ch === "(") {
      depth++;
      buf += ch;
    } else if (ch === ")") {
      depth--;
      buf += ch;
    }
    else if (ch === "," && depth === 0) {
      // split here
      parts.push(buf.trim());
      buf = "";
      continue;
    }
    else {
      buf += ch;
    }
  }

  if (buf.length > 0) {
    parts.push(buf.trim());
  }

  return parts;
}


function splitKeyValue(field: string): [string, string] {
  const idx = field.indexOf("=");
  if (idx === -1) return [field.trim(), ""];

  const key = field.slice(0, idx).trim();
  const value = field.slice(idx + 1).trim();
  return [key, value];
}


// ----------------------------------------------
// Parse ONE square from FEN
// ----------------------------------------------

export function fenToSquare(fen: string): Square {
  // Empty or "()"
  if (fen === "" || fen === "()") {
    return { piece: null, squareType: "STANDARD", conditions: [] };
  }

  // ------------------------------------------
  // Extended notation: (P=x,T=VENT,C=FROZEN)
  // ------------------------------------------
  if (fen.startsWith("(") && fen.endsWith(")")) {
    const inner = fen.slice(1, -1);
    const fields = splitTopLevel(inner);

    let piece: string | null = null;
    let squareType: SquareType = "STANDARD";
    const conditions: string[] = [];

    for (const field of fields) {
      const [key, value] = splitKeyValue(field);

      switch (key) {
        case "P":
          piece = value;
          console.log("Parsed piece:", piece, "from", field);
          break;

        case "T":
          if (value === "TURRET" || value === "VENT" || value === "STANDARD") {
            squareType = value;
          }
          break;

        case "C":
          conditions.push(value);
          break;

        default:
          console.warn("Unknown FEN square field:", field);
      }
    }

    return { piece, squareType, conditions };
  }

  // ------------------------------------------
  // Standard single-piece square
  // ------------------------------------------
  return {
    piece: fen,
    squareType: "STANDARD",
    conditions: [],
  };
}

// ----------------------------------------------
// Convert square → FEN
// ----------------------------------------------

export function squareToFEN(square: Square): string {
  const isStandardPiece =
    square.squareType === "STANDARD" &&
    square.conditions.length === 0 &&
    square.piece !== null &&
    square.piece.length === 1;

  // Simple single-piece format
  if (isStandardPiece) {
    return square.piece!;
  }

  // Extended format
  const parts: string[] = [];

  if (square.piece) parts.push(`P=${square.piece}`);
  if (square.squareType !== "STANDARD") parts.push(`T=${square.squareType}`);
  for (const c of square.conditions) parts.push(`C=${c}`);

  return `(${parts.join(",")})`;
}

// ----------------------------------------------
// Parse a FEN row (arbitrary width)
// ----------------------------------------------

/// Parse a single FEN row into squares. Width is whatever the row encodes
/// — `pppppppp` is 8 wide, `pppppppppp` is 10. Multi-digit run-length
/// empties (`10`, `15`) are supported for boards wider than 9 columns.
/// The whole-board parser (`parseFEN`) enforces that every row in a
/// given FEN has the same width.
export function parseFENRow(row: string): Square[] {
  const squares: Square[] = [];
  let i = 0;

  while (i < row.length) {
    const ch = row[i];

    // Digits → N empty squares. Consume the whole digit run greedily so
    // "10" reads as ten empty squares, not "1" then "0".
    if (/\d/.test(ch)) {
      let j = i;
      while (j < row.length && /\d/.test(row[j])) j++;
      const count = Number(row.slice(i, j));
      for (let k = 0; k < count; k++) {
        squares.push({
          piece: null,
          squareType: "STANDARD",
          conditions: [],
        });
      }
      i = j;
      continue;
    }

    // Extended form
    if (ch === "(") {
      let depth = 0;
      let buf = "";

      while (i < row.length) {
        const c = row[i];
        buf += c;

        if (c === "(") depth++;
        if (c === ")") depth--;

        i++;

        if (depth === 0) break;
      }

      squares.push(fenToSquare(buf));
      continue;
    }

    // Standard piece
    squares.push(fenToSquare(ch));
    i++;
  }

  return squares;
}

// ----------------------------------------------
// Parse whole board
// ----------------------------------------------

/// Parse a FEN grid into a 2D array of squares. Trailing flag fields
/// (`<stm> <castling> <ep>`) are ignored — they're handled separately by
/// `parseFENFlags`. Width and height are inferred from the input: a 1×1
/// "K" is valid, so is a 16×12 grid. All rows in a given FEN must have
/// the same width or the parse rejects.
export function parseFEN(fen: string): Square[][] {
  const grid = fen.split(/\s+/, 1)[0];
  const rows = grid.split("/");
  if (rows.length === 0) {
    throw new Error("Invalid FEN: empty grid");
  }
  const parsed = rows.map(parseFENRow);
  const width = parsed[0].length;
  for (let r = 1; r < parsed.length; r++) {
    if (parsed[r].length !== width) {
      throw new Error(
        `Invalid FEN: row ${r} has ${parsed[r].length} squares but row 0 has ${width}`,
      );
    }
  }
  return parsed;
}

// ----------------------------------------------
// Convert board → FEN
// ----------------------------------------------

export function squaresToFEN(board: Square[][]): string {
  return board
    .map(row => {
      let fenRow = "";
      let empty = 0;

      for (const sq of row) {
        const fen = squareToFEN(sq);

        if (fen === "" || fen === "()") {
          empty++;
        } else {
          if (empty > 0) {
            fenRow += empty.toString();
            empty = 0;
          }
          fenRow += fen;
        }
      }

      if (empty > 0) {
        fenRow += empty.toString();
      }

      return fenRow;
    })
    .join("/");
}
