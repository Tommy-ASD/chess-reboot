import { describe, expect, it } from "vitest";
import { parseFEN, squaresToFEN } from "./fen";
import { invertPieceColor, shouldInvertColors, symmetrizeBoard } from "./symmetrize";

describe("invertPieceColor", () => {
  it("swaps the case of standard pieces", () => {
    expect(invertPieceColor("K")).toBe("k");
    expect(invertPieceColor("p")).toBe("P");
  });

  it("swaps multi-letter coloured symbols", () => {
    expect(invertPieceColor("BUS")).toBe("bus");
    expect(invertPieceColor("bus")).toBe("BUS");
  });

  it("leaves neutral trains alone but inverts their passengers", () => {
    expect(invertPieceColor("LOCO(ID=1,H=F,P=(K,r))")).toBe("LOCO(ID=1,H=F,P=(k,R))");
    expect(invertPieceColor("CART(ID=1,I=2)")).toBe("CART(ID=1,I=2)");
  });

  it("inverts a goblin and its kidnapped piece without touching its home", () => {
    expect(invertPieceColor("G(H=0-7,P=n)")).toBe("g(H=0-7,P=N)");
  });

  it("inverts nested passengers", () => {
    expect(invertPieceColor("BUS(P=(g(H=1-1,P=Q),p))")).toBe("bus(P=(G(H=1-1,P=q),P))");
  });
});

describe("shouldInvertColors", () => {
  it("auto inverts across the middle rank and keeps across the middle file", () => {
    expect(shouldInvertColors("top", "auto")).toBe(true);
    expect(shouldInvertColors("bottom", "auto")).toBe(true);
    expect(shouldInvertColors("left", "auto")).toBe(false);
    expect(shouldInvertColors("right", "auto")).toBe(false);
  });

  it("keep and invert override the direction", () => {
    expect(shouldInvertColors("top", "keep")).toBe(false);
    expect(shouldInvertColors("left", "invert")).toBe(true);
  });
});

describe("symmetrizeBoard", () => {
  it("builds the standard position from the white half, inverted", () => {
    const whiteHalf = parseFEN("8/8/8/8/8/8/PPPPPPPP/RNBQKBNR");
    const out = symmetrizeBoard(whiteHalf, "bottom", true);
    // A top/bottom reflection keeps the king on the e-file.
    expect(squaresToFEN(out)).toBe("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR");
  });

  it("keeps colours when asked to on a top/bottom copy", () => {
    const out = symmetrizeBoard(parseFEN("8/8/8/8/8/8/8/K7"), "bottom", false);
    expect(squaresToFEN(out)).toBe("K7/8/8/8/8/8/8/K7");
  });

  it("copies left onto right, reflected, keeping colours", () => {
    const out = symmetrizeBoard(parseFEN("RN6/8/8/8/8/8/8/8"), "left", false);
    expect(squaresToFEN(out)).toBe("RN4NR/8/8/8/8/8/8/8");
  });

  it("overwrites everything on the destination half, including blanks", () => {
    const out = symmetrizeBoard(parseFEN("7q/8/8/8/8/8/8/8"), "left", false);
    expect(squaresToFEN(out)).toBe("8/8/8/8/8/8/8/8");
  });

  it("copies right onto left and top onto bottom", () => {
    expect(squaresToFEN(symmetrizeBoard(parseFEN("6NR/8/8/8/8/8/8/8"), "right", false)))
      .toBe("RN4NR/8/8/8/8/8/8/8");
    expect(squaresToFEN(symmetrizeBoard(parseFEN("k7/8/8/8/8/8/8/8"), "top", true)))
      .toBe("k7/8/8/8/8/8/8/K7");
  });

  it("leaves the middle column untouched on an odd-width board", () => {
    const out = symmetrizeBoard(parseFEN("RNBQK/5"), "left", false);
    expect(squaresToFEN(out)).toBe("RNBNR/5");
  });

  it("reflects directional and coordinate payloads on the copy", () => {
    const board = parseFEN("(T=TRACK,D=E)(P=G(H=1-7))6/8/8/8/8/8/8/8");
    const out = symmetrizeBoard(board, "left", true);
    expect(out[0][7].extraFields?.D).toBe("W");
    expect(out[0][6].piece).toBe("g(H=6-7)");
    // Source half is untouched.
    expect(out[0][0].extraFields?.D).toBe("E");
    expect(out[0][1].piece).toBe("G(H=1-7)");
  });

  it("does not mutate the input board", () => {
    const fen = "8/8/8/8/8/8/PPPPPPPP/RNBQKBNR";
    const board = parseFEN(fen);
    symmetrizeBoard(board, "bottom", true);
    expect(squaresToFEN(board)).toBe(fen);
  });
});
