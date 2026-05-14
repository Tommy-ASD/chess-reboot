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

/// Plan 09: serialized form of the engine's `TrainTickRate`. We keep it
/// as a string union rather than a discriminated union since the field
/// has no other payload than the optional `<n>` for `EveryNPly`.
export type TrainTickRate =
  | { kind: "EveryPly" }
  | { kind: "EveryFullTurn" }
  | { kind: "EveryNPly"; n: number };

export type BoardFlags = {
  sideToMove: Side;
  castling: CastlingRights;
  /** Algebraic square ("e3") or null when no en-passant target. */
  enPassant: string | null;
  /// Plan 09: how often the train tick fires. Default `EveryFullTurn`
  /// matches the engine's `BoardFlags::train_tick_rate` default.
  trainTickRate: TrainTickRate;
  /// Plan 09: monotonic ply counter, bumped at every successful move.
  plyCount: number;
};

export const DEFAULT_FLAGS: BoardFlags = {
  sideToMove: "w",
  castling: { K: true, Q: true, k: true, q: true },
  enPassant: null,
  trainTickRate: { kind: "EveryFullTurn" },
  plyCount: 0,
};

/// Split a full FEN ("<grid> <stm> <castling> <ep>") into its grid +
/// flags. Missing trailing fields default to white-to-move with all
/// castle rights and no en-passant target — matching `fen_to_board`'s
/// fallback in the engine, so a bare grid round-trips identically.
export function parseFENFlags(fen: string): BoardFlags {
  const parts = fen.trim().split(/\s+/);
  // parts[0] is the grid; the rest are flag fields:
  //   [1] stm, [2] castling, [3] ep, [4] train-tick rate, [5] ply count.
  // Plan 09 appended trainTickRate + plyCount; both fall back to engine
  // defaults when absent, so older grid-only FENs still round-trip.
  const stm = parts[1];
  const castle = parts[2];
  const ep = parts[3];
  const tr = parts[4];
  const p = parts[5];

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
  const trainTickRate = parseTrainTickRate(tr) ?? { kind: "EveryFullTurn" };
  const plyCount = parsePlyCount(p) ?? 0;

  return { sideToMove, castling, enPassant, trainTickRate, plyCount };
}

function parseTrainTickRate(field: string | undefined): TrainTickRate | null {
  if (field === undefined) return null;
  const body = field.startsWith("tr=") ? field.slice(3) : field;
  if (body === "full") return { kind: "EveryFullTurn" };
  if (body === "ply") return { kind: "EveryPly" };
  if (body.endsWith("ply")) {
    const n = Number.parseInt(body.slice(0, -3), 10);
    if (Number.isFinite(n) && n > 0) return { kind: "EveryNPly", n };
  }
  return null;
}

function parsePlyCount(field: string | undefined): number | null {
  if (field === undefined) return null;
  const body = field.startsWith("p=") ? field.slice(2) : field;
  const n = Number.parseInt(body, 10);
  return Number.isFinite(n) && n >= 0 ? n : null;
}

function trainTickRateToFEN(rate: TrainTickRate): string {
  switch (rate.kind) {
    case "EveryPly":      return "tr=ply";
    case "EveryFullTurn": return "tr=full";
    case "EveryNPly":
      // Canonicalize `EveryNPly(1)` to `tr=ply` to match the engine's
      // parser + encoder (both normalize: `ply_count % 1 == 0` is
      // always true, so `EveryNPly(1)` is behaviorally identical to
      // `EveryPly`). Without this, a server round-trip would byte-
      // shift the FEN: editor emits `tr=1ply`, engine parses to
      // `EveryPly`, engine re-encodes as `tr=ply`.
      return rate.n === 1 ? "tr=ply" : `tr=${rate.n}ply`;
  }
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
  const tr = trainTickRateToFEN(flags.trainTickRate);
  const p = `p=${flags.plyCount}`;
  return `${grid} ${stm} ${castling} ${ep} ${tr} ${p}`;
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
  // For custom pieces (multi-char or payload-bearing symbols). Strip
  // any parenthesised payload first so "LOCO(ID=1,H=F)" reduces to
  // "LOCO".
  const base = p.split("(")[0];
  // Train carts are color-blind (Neutral) — single sprite per kind.
  // The engine accepts both "LOCO"/"loco" so route either casing to
  // the same image.
  const baseUpper = base.toUpperCase();
  if (baseUpper === "LOCO") return "/img/pieces/locomotive.svg";
  if (baseUpper === "CART") return "/img/pieces/carriage.svg";
  // map base to image filename for non-train custom pieces.
  const map: Record<string, string> = {
    "G":    "/img/pieces/Goblin white.png",
    "g":    "/img/pieces/Goblin black.png",
    "BUS":  "/img/pieces/Bus white.png",
    "bus":  "/img/pieces/Bus black.png",
  };
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
    // Mirror the engine's nested-carrier rejection: a Bus passenger
    // that's itself a carrier (BUS / LOCO / CART) is dropped at FEN
    // parse on the engine side. Filter here so the editor doesn't
    // display state the engine then silently strips on round-trip.
    const raw = splitTopLevel(value.slice(1, -1));
    return raw.filter(sym => {
      const prefix = sym.split("(")[0].toLowerCase();
      const isCarrier =
        prefix === "bus" || prefix === "loco" || prefix === "cart";
      if (isCarrier) {
        console.warn(
          `getBusPassengers: dropping nested carrier passenger '${sym}'`,
        );
      }
      return !isCarrier;
    });
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

/// Engine-recognized `T=` values. Anything outside this set is treated as
/// unknown and parses to `STANDARD` with a warning.
const KNOWN_SQUARE_TYPES = new Set<SquareType>([
  "STANDARD",
  "TURRET",
  "VENT",
  "SWITCH",
  "JUNCTION",
  "GATE",
  "PLATE",
  "TRACK",
]);

/// Variant-payload keys we know about but the editor doesn't yet model
/// individually. We round-trip them verbatim via `Square.extraFields`.
const PAYLOAD_KEYS = new Set<string>([
  "ID",
  "STATE",
  "BRANCHES",
  "TARGETS",
  "OPEN",
  "FIRES",
  "D",
]);

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
    const extraFields: Record<string, string> = {};

    for (const field of fields) {
      const [key, value] = splitKeyValue(field);

      switch (key) {
        case "P":
          piece = value;
          break;

        case "T":
          if (KNOWN_SQUARE_TYPES.has(value as SquareType)) {
            squareType = value as SquareType;
          } else {
            console.warn(`Unknown square type "${value}"; treating as STANDARD`);
          }
          break;

        case "C":
          conditions.push(value);
          break;

        default:
          if (PAYLOAD_KEYS.has(key)) {
            // Known variant-payload field — preserve verbatim so the FEN
            // round-trips even though the editor doesn't render it yet.
            // OPEN is canonicalized to "1"/"0" to match engine-side
            // strict parsing (fen.rs Gate: anything other than "0"/"1"
            // is treated as closed). Without this, a pasted FEN with
            // `OPEN=garbage` would render closed in the editor but
            // still emit `OPEN=garbage` on the next round-trip.
            if (key === "OPEN") {
              extraFields[key] = value === "1" ? "1" : "0";
            } else {
              extraFields[key] = value;
            }
          } else {
            console.warn("Unknown FEN square field:", field);
          }
      }
    }

    const sq: Square = { piece, squareType, conditions };
    if (Object.keys(extraFields).length > 0) sq.extraFields = extraFields;
    return sq;
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

/// Canonical write order for variant-payload fields. Matches the engine
/// encoder so server↔client FEN strings stay byte-identical when nothing
/// else has changed. Parsers on both sides are order-agnostic but
/// determinism makes diffing easier.
const PAYLOAD_FIELD_ORDER: readonly string[] = [
  "ID",
  "STATE",
  "BRANCHES",
  "TARGETS",
  "OPEN",
  "FIRES",
  "D",
];

export function squareToFEN(square: Square): string {
  const isStandardPiece =
    square.squareType === "STANDARD" &&
    square.conditions.length === 0 &&
    (!square.extraFields || Object.keys(square.extraFields).length === 0) &&
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

  if (square.extraFields) {
    for (const key of PAYLOAD_FIELD_ORDER) {
      const v = square.extraFields[key];
      if (v !== undefined) parts.push(`${key}=${v}`);
    }
  }

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
