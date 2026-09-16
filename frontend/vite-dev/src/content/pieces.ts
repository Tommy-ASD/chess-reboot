// src/content/pieces.ts
//
// The piece registry: every piece the site can explain or draw. The
// home page, the piece guide, the in-game piece panel and the Lab palette
// all read from here, so a new piece is one entry (plus engine support).
//
// Rules text is written for players, not engineers — the engine doc
// comments in `engine/src/pieces/` are the authority on edge cases.

import { pieceToImage } from "../core/fen";

export type PieceFamily = "classic" | "chess2" | "trains" | "extras";

export const FAMILY_LABELS: Record<PieceFamily, { title: string; blurb: string }> = {
  classic: { title: "Classic pieces", blurb: "The pieces you already know. They behave exactly as in standard chess." },
  chess2: { title: "Chess 2 pieces", blurb: "New pieces with new ideas: kidnapping, brainrot, jumping and weather." },
  trains: { title: "Trains", blurb: "Neutral machines that run on tracks by themselves and carry passengers." },
  extras: { title: "Variant extras", blurb: "Things that aren't pieces but live on the board in some variants." },
};

export interface PieceDemo {
  /** A small, engine-valid position that shows the piece off. */
  fen: string;
  /** What to try, in one or two sentences. */
  hint: string;
}

export interface PieceDef {
  id: string;
  name: string;
  family: PieceFamily;
  /** FEN symbols drawn for each side. `null` for things with no symbol (the duck). */
  symbols: { white: string; black: string } | null;
  /** One line, shown on cards and in the game's piece panel. */
  tagline: string;
  /** "Moves like…" summary. */
  movement: string;
  /** Plain-language rules, one idea per bullet. */
  rules: string[];
  status: "stable" | "experimental";
  demo?: PieceDemo;
}

export const PIECES: PieceDef[] = [
  {
    id: "king",
    name: "King",
    family: "classic",
    symbols: { white: "K", black: "k" },
    tagline: "Protect it at all costs.",
    movement: "One square in any direction.",
    rules: [
      "Moves one square in any direction.",
      "Can never move into check.",
      "Castles with a rook when neither has moved and the path is safe.",
      "Checkmate the enemy king to win.",
    ],
    status: "stable",
  },
  {
    id: "queen",
    name: "Queen",
    family: "classic",
    symbols: { white: "Q", black: "q" },
    tagline: "The strongest classic piece.",
    movement: "Any distance in a straight line or diagonal.",
    rules: ["Slides any number of squares horizontally, vertically or diagonally.", "Can't jump over pieces."],
    status: "stable",
    demo: { fen: "4k3/8/8/8/3Q4/8/8/4K3 w - - tr=full p=0", hint: "Tap the queen to see how far she reaches." },
  },
  {
    id: "rook",
    name: "Rook",
    family: "classic",
    symbols: { white: "R", black: "r" },
    tagline: "Straight lines, long range.",
    movement: "Any distance horizontally or vertically.",
    rules: ["Slides any number of squares horizontally or vertically.", "Takes part in castling."],
    status: "stable",
  },
  {
    id: "bishop",
    name: "Bishop",
    family: "classic",
    symbols: { white: "B", black: "b" },
    tagline: "Diagonals only, one colour forever.",
    movement: "Any distance diagonally.",
    rules: ["Slides any number of squares diagonally.", "Stays on the square colour it started on."],
    status: "stable",
  },
  {
    id: "knight",
    name: "Knight",
    family: "classic",
    symbols: { white: "N", black: "n" },
    tagline: "The only classic piece that jumps.",
    movement: "An L-shape: two squares one way, one square the other.",
    rules: ["Moves in an L-shape.", "Jumps over anything in between."],
    status: "stable",
  },
  {
    id: "pawn",
    name: "Pawn",
    family: "classic",
    symbols: { white: "P", black: "p" },
    tagline: "Small, stubborn, and full of potential.",
    movement: "Forward one square (two on its first move); captures diagonally.",
    rules: [
      "Moves forward one square, or two from its starting rank.",
      "Captures one square diagonally forward.",
      "Can capture en passant right after an enemy pawn's double step.",
      "Promotes to a queen, rook, bishop or knight on the far rank.",
    ],
    status: "stable",
  },
  {
    id: "goblin",
    name: "Goblin",
    family: "chess2",
    symbols: { white: "G", black: "g" },
    tagline: "Kidnaps a piece and drags it home to switch sides.",
    movement: "Like a queen while free; like a king while carrying a captive.",
    rules: [
      "Moves like a queen until it captures something.",
      "Instead of removing the captured piece, the Goblin kidnaps it.",
      "While carrying a captive it moves like a king.",
      "Reach the Goblin's home square and the captive joins your side.",
      "Capture a Goblin that's carrying someone and the captive is dropped where your piece came from, and you move again.",
    ],
    status: "experimental",
    demo: {
      fen: "4k3/8/2p5/8/4(P=G(H=0-7))3/8/8/4K3 w - - tr=full p=0",
      hint: "Capture the pawn on c6 to kidnap it, then walk the Goblin home to a1. Black's king moves in between.",
    },
  },
  {
    id: "skibidi",
    name: "Skibidi",
    family: "chess2",
    symbols: { white: "S", black: "s" },
    tagline: "Spreads brainrot that freezes everything nearby.",
    movement: "One square in any direction; can only capture other Skibidis.",
    rules: [
      "Moves like a king, but can only capture other Skibidis.",
      "Spend a turn to grow its brainrot: phase 1 does nothing, phases 2–4 freeze pieces in a growing radius.",
      "Frozen pieces (yours too) can't move.",
      "Moving the Skibidi resets it to phase 1. Without an enemy Skibidi on the board it tops out at phase 3.",
      "Another Skibidi entering the radius neutralises it back to phase 1.",
      "If brainrot leaves your opponent with no moves, you win by brainrot instead of stalemate.",
    ],
    status: "experimental",
    demo: {
      fen: "4k3/8/8/3pp3/4S3/8/8/4K3 w - - tr=full p=0",
      hint: "Select the Skibidi and use “Grow brainrot” a couple of times. Watch the black pawns freeze.",
    },
  },
  {
    id: "monkey",
    name: "Monkey",
    family: "chess2",
    symbols: { white: "M", black: "m" },
    tagline: "Hops over pieces, chaining jumps together.",
    movement: "One square in any direction, or a chain of jumps over neighbouring pieces.",
    rules: [
      "Steps one square in any direction, but can't capture with a plain step.",
      "Can jump over an adjacent piece, and keep jumping from wherever it lands.",
      "Captures whatever it lands on at the end of a jump.",
    ],
    status: "experimental",
    demo: {
      fen: "4k3/8/8/8/3pP3/4M3/8/4K3 w - - tr=full p=0",
      hint: "Tap the Monkey: it can vault the pawns in front of it to land two squares away.",
    },
  },
  {
    id: "bus",
    name: "Bus",
    family: "chess2",
    symbols: { white: "BUS", black: "bus" },
    tagline: "Carries up to five of your pieces across the board.",
    movement: "Like a rook; can't capture.",
    rules: [
      "Moves like a rook but can never capture.",
      "Your pieces can board it by moving onto it.",
      "Passengers get off by making their own normal move from the Bus's square.",
      "If the Bus is captured, everyone inside is lost with it.",
    ],
    status: "experimental",
    demo: {
      fen: "4k3/8/8/8/8/8/8/(P=BUS(P=(N,P)))3K3 w - - tr=full p=0",
      hint: "Drive the Bus up the board, or select it and pick a passenger to let them off.",
    },
  },
  {
    id: "stormcaller",
    name: "Stormcaller",
    family: "chess2",
    symbols: { white: "W", black: "w" },
    tagline: "Summons tornadoes that pull pieces in and trap them.",
    movement: "One square in any direction; can't capture.",
    rules: [
      "Moves like a king but can't capture.",
      "Spend a turn to place a tornado on a neighbouring square.",
      "While a tornado is up, any side that can move a piece onto it must do so.",
      "A piece that enters a tornado is trapped there until it dies down, and an enemy that can reach it must capture it.",
      "Kings ignore tornadoes completely.",
    ],
    status: "experimental",
    demo: {
      fen: "4k3/8/8/8/8/8/8/3WK3 w - - tr=full p=0",
      hint: "Select the Stormcaller and choose a square to summon a tornado on.",
    },
  },
  {
    id: "locomotive",
    name: "Locomotive",
    family: "trains",
    symbols: { white: "LOCO(ID=1,H=F)", black: "LOCO(ID=1,H=F)" },
    tagline: "Runs along its track on its own, crushing what's in the way.",
    movement: "Moves automatically along track tiles every tick.",
    rules: [
      "Belongs to nobody: neither side moves it.",
      "Advances along its track on a schedule set by the position.",
      "Pulls carriages behind it and carries passengers like a Bus.",
      "Anything parked in its path gets run over.",
    ],
    status: "experimental",
    demo: {
      fen: "4k3/8/8/3(T=TRACK,D=E,P=LOCO(ID=1,H=F,P=(N)))(T=TRACK,D=E)3/3(T=TRACK,D=E,P=CART(ID=1,I=1))(T=TRACK,D=E)3/8/8/4K3 w - - tr=ply p=0",
      hint: "Make any move and the train rolls one step around its loop. The knight riding on it can hop off.",
    },
  },
  {
    id: "carriage",
    name: "Carriage",
    family: "trains",
    symbols: { white: "CART(ID=1,I=1)", black: "CART(ID=1,I=1)" },
    tagline: "Follows its locomotive and carries passengers.",
    movement: "Follows the cart in front of it.",
    rules: ["Trails behind its locomotive along the track.", "Carries passengers who can step off with their own moves."],
    status: "experimental",
  },
  {
    id: "duck",
    name: "The Duck",
    family: "extras",
    symbols: null,
    tagline: "A blocker both players move. From Duck Chess.",
    movement: "Placed on any empty square after every move.",
    rules: [
      "After moving a piece, you must move the duck to a different empty square.",
      "Nothing can move through or onto the duck.",
      "There's no check: you win by capturing the king.",
    ],
    status: "stable",
  },
];

const BY_ID = new Map(PIECES.map((p) => [p.id, p]));

const BY_SYMBOL_KEY: Record<string, string> = {
  k: "king",
  q: "queen",
  r: "rook",
  b: "bishop",
  n: "knight",
  p: "pawn",
  g: "goblin",
  s: "skibidi",
  m: "monkey",
  bus: "bus",
  w: "stormcaller",
  loco: "locomotive",
  cart: "carriage",
};

export function pieceById(id: string): PieceDef | undefined {
  return BY_ID.get(id);
}

/** The registry entry for a FEN piece symbol like `G(H=0-7)` or `bus`. */
export function pieceForSymbol(symbol: string): PieceDef | undefined {
  const key = symbol.split("(")[0].toLowerCase();
  const id = BY_SYMBOL_KEY[key];
  return id ? BY_ID.get(id) : undefined;
}

export type PieceSide = "white" | "black" | "neutral";

/** Which side a symbol belongs to, following the engine's casing rules. */
export function pieceSide(symbol: string): PieceSide {
  const base = symbol.split("(")[0];
  const lower = base.toLowerCase();
  if (lower === "loco" || lower === "cart") return "neutral";
  if (lower === "bus") return base === "BUS" ? "white" : "black";
  return base === base.toUpperCase() ? "white" : "black";
}

/// How to draw a piece. Sprites where we have art; otherwise a glyph.
/// Classic glyphs are tinted per side by CSS; emoji placeholders can't be
/// tinted, so they get a side ring instead. Replacing a placeholder with
/// real art only needs an entry in `pieceToImage`.
export type PieceArt =
  | { kind: "image"; src: string }
  | { kind: "glyph"; text: string; tinted: boolean };

const CLASSIC_GLYPHS: Record<string, string> = {
  k: "♚",
  q: "♛",
  r: "♜",
  b: "♝",
  n: "♞",
  p: "♟",
};

const PLACEHOLDER_GLYPHS: Record<string, string> = {
  s: "\u{1F6BD}", // 🚽
  m: "\u{1F412}", // 🐒
  w: "\u{1F329}️", // 🌩️
};

export function pieceArt(symbol: string): PieceArt {
  const src = pieceToImage(symbol);
  if (src) return { kind: "image", src };
  const key = symbol.split("(")[0].toLowerCase();
  // U+FE0E asks for text presentation so the pawn doesn't turn into an emoji.
  if (CLASSIC_GLYPHS[key]) return { kind: "glyph", text: CLASSIC_GLYPHS[key] + "︎", tinted: true };
  if (PLACEHOLDER_GLYPHS[key]) return { kind: "glyph", text: PLACEHOLDER_GLYPHS[key], tinted: false };
  return { kind: "glyph", text: symbol.split("(")[0], tinted: true };
}

/// Top-level `KEY=value` fields of a symbol's payload: `G(H=0-7,P=p)` →
/// `{ H: "0-7", P: "p" }`. Nested parentheses stay inside their value.
export function symbolFields(symbol: string): Record<string, string> {
  const open = symbol.indexOf("(");
  if (open === -1 || !symbol.endsWith(")")) return {};
  const inner = symbol.slice(open + 1, -1);
  const out: Record<string, string> = {};
  let depth = 0;
  let start = 0;
  const push = (end: number) => {
    const part = inner.slice(start, end);
    const eq = part.indexOf("=");
    if (eq > 0) out[part.slice(0, eq).trim()] = part.slice(eq + 1).trim();
  };
  for (let i = 0; i < inner.length; i++) {
    const ch = inner[i];
    if (ch === "(") depth++;
    else if (ch === ")") depth--;
    else if (ch === "," && depth === 0) {
      push(i);
      start = i + 1;
    }
  }
  push(inner.length);
  return out;
}

/** A Skibidi's brainrot phase (1–4), or null for other pieces. */
export function skibidiPhase(symbol: string): number | null {
  if (pieceForSymbol(symbol)?.id !== "skibidi") return null;
  const phase = Number(symbolFields(symbol).PHASE ?? 1);
  return Number.isFinite(phase) ? phase : 1;
}

/** The piece a Goblin is carrying, or null. */
export function goblinCaptive(symbol: string): string | null {
  if (pieceForSymbol(symbol)?.id !== "goblin") return null;
  return symbolFields(symbol).P ?? null;
}
