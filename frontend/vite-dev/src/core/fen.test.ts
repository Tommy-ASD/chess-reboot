import { describe, expect, it } from "vitest";
import {
  fenToSquare,
  formatLastMove,
  getBusPassengers,
  parseFEN,
  parseFENFlags,
  parseFENRow,
  parseLastMove,
  pieceToSymbol,
  serializeFullFEN,
  squareToFEN,
} from "./fen";

const STANDARD = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR";

describe("parseFEN", () => {
  it("parses the standard grid with pieces at the corners (rank 0 = top)", () => {
    const board = parseFEN(STANDARD);
    expect(board).toHaveLength(8);
    expect(board[0]).toHaveLength(8);
    expect(board[0][0].piece).toBe("r"); // a8 — black rook, top-left
    expect(board[7][7].piece).toBe("R"); // h1 — white rook, bottom-right
    expect(board[7][4].piece).toBe("K"); // e1 — white king
    expect(board[3][3].piece).toBeNull(); // empty middle
  });

  it("ignores trailing flag fields and parses only the grid", () => {
    const board = parseFEN(`${STANDARD} b KQkq - tr=full p=3 variants=duck_chess`);
    expect(board).toHaveLength(8);
    expect(board[0][0].piece).toBe("r");
  });

  it("supports non-8 board widths", () => {
    const board = parseFEN("K9/10/10"); // K + 9 empties = 10 wide, 3 rows
    expect(board).toHaveLength(3);
    expect(board[0]).toHaveLength(10);
    expect(board[0][0].piece).toBe("K");
  });

  it("rejects ragged rows (width mismatch)", () => {
    expect(() => parseFEN("8/7")).toThrow();
  });
});

describe("parseFENRow", () => {
  it("reads a multi-digit run length as one number, not two digits", () => {
    expect(parseFENRow("10")).toHaveLength(10); // ten empties, not 1 then 0
    expect(parseFENRow("15")).toHaveLength(15); // greedy digit run
    expect(parseFENRow("8")).toHaveLength(8);
  });

  it("parses an extended square mid-row", () => {
    const row = parseFENRow("(T=VENT)7");
    expect(row).toHaveLength(8);
    expect(row[0].squareType).toBe("VENT");
    expect(row[1].piece).toBeNull();
  });
});

describe("fenToSquare", () => {
  it("parses a plain piece", () => {
    expect(fenToSquare("Q")).toEqual({ piece: "Q", squareType: "STANDARD", conditions: [] });
  });

  it("parses an extended square with piece, type, and condition", () => {
    const sq = fenToSquare("(P=g,T=VENT,C=FROZEN)");
    expect(sq.piece).toBe("g");
    expect(sq.squareType).toBe("VENT");
    expect(sq.conditions).toContain("FROZEN");
  });

  it("parses the value-less DUCK flag", () => {
    const sq = fenToSquare("(DUCK)");
    expect(sq.duck).toBe(true);
    expect(sq.piece).toBeNull();
  });

  it("treats an empty square notation as empty", () => {
    expect(fenToSquare("()").piece).toBeNull();
  });

  it("falls back to STANDARD for an unknown square type", () => {
    expect(fenToSquare("(T=NONSENSE)").squareType).toBe("STANDARD");
  });
});

describe("squareToFEN round-trips fenToSquare", () => {
  for (const fen of ["Q", "(P=g,T=VENT,C=FROZEN)", "(DUCK)", "(T=BLOCK)"]) {
    it(`round-trips ${fen}`, () => {
      expect(squareToFEN(fenToSquare(fen))).toBe(fen);
    });
  }

  it("keeps a plain piece as a single character (not extended)", () => {
    expect(squareToFEN(fenToSquare("p"))).toBe("p");
  });
});

describe("parseFENFlags", () => {
  it("defaults a bare grid to white-to-move, full castling, no en passant", () => {
    const f = parseFENFlags(STANDARD);
    expect(f.sideToMove).toBe("w");
    expect(f.castling).toEqual({ K: true, Q: true, k: true, q: true });
    expect(f.enPassant).toBeNull();
    expect(f.variants).toEqual([]);
    expect(f.duckPhase).toBe("piece");
  });

  it("reads side-to-move, castling subset, ep, and ply", () => {
    const f = parseFENFlags(`${STANDARD} b Kq e3 tr=ply p=7`);
    expect(f.sideToMove).toBe("b");
    expect(f.castling).toEqual({ K: true, Q: false, k: false, q: true });
    expect(f.enPassant).toBe("e3");
    expect(f.plyCount).toBe(7);
  });

  it("reads prefix-scanned variants= and duck_phase= in any position", () => {
    const f = parseFENFlags(`${STANDARD} w KQkq - variants=duck_chess duck_phase=placing`);
    expect(f.variants).toContain("duck_chess");
    expect(f.duckPhase).toBe("placing");
  });
});

describe("serializeFullFEN round-trips parseFENFlags", () => {
  it("keeps a standard position byte-identical and flag-clean", () => {
    const board = parseFEN(STANDARD);
    const flags = parseFENFlags(STANDARD);
    const out = serializeFullFEN(board, flags);
    expect(out.startsWith(STANDARD)).toBe(true);
    expect(out).not.toContain("variants=");
    expect(out).not.toContain("duck_phase=");
    // Re-parsing the flags is stable.
    expect(parseFENFlags(out)).toEqual(flags);
  });

  it("emits variants= / duck_phase= only when non-default", () => {
    const board = parseFEN(STANDARD);
    const flags = parseFENFlags(`${STANDARD} w KQkq - variants=duck_chess duck_phase=placing`);
    const out = serializeFullFEN(board, flags);
    expect(out).toContain("variants=duck_chess");
    expect(out).toContain("duck_phase=placing");
  });
});

describe("getBusPassengers", () => {
  it("extracts passengers in order", () => {
    expect(getBusPassengers("BUS(P=(N,P,p))")).toEqual(["N", "P", "p"]);
  });

  it("returns [] for a bus with no passengers", () => {
    expect(getBusPassengers("BUS")).toEqual([]);
    expect(getBusPassengers("bus")).toEqual([]);
  });

  it("drops a nested carrier passenger (mirrors the engine)", () => {
    expect(getBusPassengers("BUS(P=(N,bus,p))")).toEqual(["N", "p"]);
  });
});

describe("pieceToSymbol", () => {
  it("maps known glyphs and passes unknown symbols through", () => {
    expect(pieceToSymbol("K")).toBe("♔");
    expect(pieceToSymbol("p")).toBe("♟");
    expect(pieceToSymbol("G")).toBe("G"); // custom multi-handled elsewhere
  });
});

describe("parseLastMove / formatLastMove", () => {
  const E2E4 = `${STANDARD} b KQkq e3 tr=full p=1 lm=(C=W,F=4-6,K=MOVE,T=4-4,P=P)`;

  it("parses the lm= from/to coords", () => {
    expect(parseLastMove(E2E4)).toEqual({
      from: { file: 4, rank: 6 },
      to: { file: 4, rank: 4 },
    });
  });

  it("returns null when there is no last move", () => {
    expect(parseLastMove(STANDARD)).toBeNull();
    expect(formatLastMove(STANDARD)).toBeNull();
  });

  it("formats algebraic-ish coordinate notation with the piece glyph", () => {
    // file 4 = 'e'; rank 6 → 8-6 = 2; rank 4 → 8-4 = 4.
    expect(formatLastMove(E2E4)).toBe("♙ e2→e4");
  });
});
