// src/content/variants.ts
//
// Official variants — what visitors land on. One entry per variant; the
// home gallery, the play-setup page and the lobby labels all read this.
// Every `fen` must be engine-valid (the API rejects anything else).
//
// Community variants (saved from the Lab or opened from a share link)
// live in `userVariants.ts` and share the `Variant` shape.

export interface Variant {
  id: string;
  name: string;
  /** One line for cards. */
  tagline: string;
  description: string;
  /** Starting position, full engine FEN. */
  fen: string;
  /** Piece ids (see pieces.ts) worth introducing before playing. */
  pieces: string[];
  /** How it differs from classic chess, one idea per bullet. */
  rules: string[];
  status: "stable" | "experimental";
  source: "official" | "community";
  author: string;
}

export const OFFICIAL_VARIANTS: Variant[] = [
  {
    id: "classic",
    name: "Classic",
    tagline: "Standard chess. The place to start.",
    description: "Regular chess with the regular rules: castling, en passant, promotion, checkmate.",
    fen: "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - tr=full p=0",
    pieces: ["king", "queen", "rook", "bishop", "knight", "pawn"],
    rules: ["Nothing new. Checkmate the king to win."],
    status: "stable",
    source: "official",
    author: "Chess 2",
  },
  {
    id: "duck",
    name: "Duck Chess",
    tagline: "After every move, you move the duck. Nobody can pass it.",
    description:
      "A shared rubber duck blocks one square. Each turn is two steps: move a piece, then move the duck somewhere new.",
    fen: "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - tr=full p=0 variants=duck_chess",
    pieces: ["duck"],
    rules: [
      "Each turn: move a piece, then place the duck on a different empty square.",
      "Nothing can move onto or through the duck.",
      "There's no check. Capture the king to win.",
    ],
    status: "stable",
    source: "official",
    author: "Chess 2",
  },
  {
    id: "goblin",
    name: "Goblin Chess",
    tagline: "Your rooks are Goblins. Captured pieces get kidnapped, not killed.",
    description:
      "The rooks are replaced by Goblins that carry their captures home and turn them to your side. No castling.",
    fen: "(P=g(H=0-0))nbqkbn(P=g(H=7-0))/pppppppp/8/8/8/8/PPPPPPPP/(P=G(H=0-7))NBQKBN(P=G(H=7-7)) w - - tr=full p=0",
    pieces: ["goblin"],
    rules: [
      "Goblins start in the rook corners, and that corner is their home.",
      "A Goblin that captures carries its captive home to recruit it.",
      "There's no castling.",
    ],
    status: "experimental",
    source: "official",
    author: "Chess 2",
  },
  {
    id: "storm",
    name: "Storm Chess",
    tagline: "Your knights are Stormcallers. Tornadoes decide where the fight happens.",
    description:
      "The knights are replaced by Stormcallers, who can't capture but summon tornadoes that compel both sides to move onto them.",
    fen: "rwbqkbwr/pppppppp/8/8/8/8/PPPPPPPP/RWBQKBWR w KQkq - tr=full p=0",
    pieces: ["stormcaller"],
    rules: [
      "Stormcallers start on the knight squares.",
      "A tornado forces any side that can reach it to move there, and traps what arrives.",
    ],
    status: "experimental",
    source: "official",
    author: "Chess 2",
  },
];

export function officialVariant(id: string): Variant | undefined {
  return OFFICIAL_VARIANTS.find((v) => v.id === id);
}

/// A short label for a game's position, used in lobby lists. Matches an
/// official variant by its starting FEN's piece layout and rule flags;
/// otherwise names the ruleset from the flags alone.
export function describeFen(fen: string): string {
  const grid = fen.trim().split(/\s+/)[0];
  const duck = /\bvariants=[^\s]*duck_chess/.test(fen);
  const match = OFFICIAL_VARIANTS.find(
    (v) => v.fen.split(/\s+/)[0] === grid && /variants=[^\s]*duck_chess/.test(v.fen) === duck,
  );
  if (match) return match.name;
  return duck ? "Duck Chess (custom position)" : "Custom position";
}
