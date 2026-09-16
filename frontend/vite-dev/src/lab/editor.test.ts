import { describe, expect, it } from "vitest";
import { parseFEN, squaresToFEN } from "../core/fen";
import { applyBrush, resizeBoard } from "./editor";

describe("applyBrush", () => {
  it("never mutates the board it was given", () => {
    const board = parseFEN("8/8/8/8/8/8/8/8");
    const before = squaresToFEN(board);
    applyBrush(board, { kind: "piece", piece: "Q" }, { file: 3, rank: 4 }, null);
    expect(squaresToFEN(board)).toBe(before);
  });

  it("places pieces and erases them again", () => {
    const board = parseFEN("8/8/8/8/8/8/8/8");
    const placed = applyBrush(board, { kind: "piece", piece: "Q" }, { file: 3, rank: 4 }, null).board;
    expect(placed[4][3].piece).toBe("Q");
    const erased = applyBrush(placed, { kind: "erase-piece" }, { file: 3, rank: 4 }, null).board;
    expect(erased[4][3].piece).toBeNull();
  });

  it("keeps a single duck on the board", () => {
    const board = parseFEN("8/8/8/8/8/8/8/8");
    const first = applyBrush(board, { kind: "duck" }, { file: 0, rank: 0 }, null).board;
    const second = applyBrush(first, { kind: "duck" }, { file: 7, rank: 7 }, null).board;
    expect(second.flat().filter((s) => s.duck)).toHaveLength(1);
    expect(second[7][7].duck).toBe(true);
  });

  it("toggles a tornado regardless of its countdown", () => {
    const board = parseFEN("(C=TORNADO:1)7/8/8/8/8/8/8/8");
    const cleared = applyBrush(board, { kind: "condition", condition: "TORNADO:3" }, { file: 0, rank: 0 }, null).board;
    expect(cleared[0][0].conditions).toEqual([]);
  });

  it("gives each new locomotive its own train id", () => {
    const board = parseFEN("8/8/8/8/8/8/8/8");
    const one = applyBrush(board, { kind: "piece", piece: "LOCO(ID=1,H=F)" }, { file: 0, rank: 0 }, null).board;
    const two = applyBrush(one, { kind: "piece", piece: "LOCO(ID=1,H=F)" }, { file: 5, rank: 5 }, null).board;
    expect(one[0][0].piece).toContain("ID=1");
    expect(two[5][5].piece).toContain("ID=2");
  });

  it("wires a receiver to the inspected emitter", () => {
    const board = parseFEN("(T=SWITCH,TARGETS=())(T=GATE,ID=7,OPEN=1)6/8/8/8/8/8/8/8");
    const { board: wired, inspected } = applyBrush(board, { kind: "inspect" }, { file: 1, rank: 0 }, { file: 0, rank: 0 });
    expect(inspected).toEqual({ file: 0, rank: 0 });
    expect(squaresToFEN(wired)).toContain("TARGETS=(7)");
  });
});

describe("resizeBoard", () => {
  it("keeps the top-left overlap and pads with empty squares", () => {
    const board = parseFEN("K7/8/8/8/8/8/8/7k");
    const small = resizeBoard(board, 2, 2);
    expect(small).toHaveLength(2);
    expect(small[0][0].piece).toBe("K");
    const big = resizeBoard(small, 3, 3);
    expect(big[2][2].piece).toBeNull();
  });
});
