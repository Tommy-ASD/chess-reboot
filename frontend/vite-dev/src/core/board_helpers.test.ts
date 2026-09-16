import { describe, expect, it } from "vitest";
import { castleKingDest, findMoveForTarget, isAllowedSquare, isSpecialMove, promotionsAt, visibleMoveTargets } from "./board_helpers";
import type { Coord, GameMove } from "./types";

const at = (file: number, rank: number): Coord => ({ file, rank });

const moveTo = (from: Coord, to: Coord): GameMove => ({ from, move_type: { kind: "MoveTo", target: to } });
const promotion = (from: Coord, to: Coord): GameMove => ({
  from,
  move_type: { kind: "Promotion", target: { target: to, into: "Queen" } },
});
const enPassant = (from: Coord, to: Coord, captured: Coord): GameMove => ({
  from,
  move_type: { kind: "EnPassant", target: { target: to, captured } },
});
const castle = (from: Coord, side: "Kingside" | "Queenside"): GameMove => ({
  from,
  move_type: { kind: "Castle", target: { side } },
});
const pieceInCarrier = (from: Coord, idx: number, to: Coord): GameMove => ({
  from,
  move_type: {
    kind: "PieceInCarrier",
    target: { piece_index: idx, move_type: { kind: "MoveTo", target: to } },
  },
});

describe("castleKingDest", () => {
  it("lands the king on the g-file (6) for kingside", () => {
    expect(castleKingDest(castle(at(4, 7), "Kingside"))).toEqual({ file: 6, rank: 7 });
  });
  it("lands the king on the c-file (2) for queenside", () => {
    expect(castleKingDest(castle(at(4, 0), "Queenside"))).toEqual({ file: 2, rank: 0 });
  });
  it("returns null for a non-castle move", () => {
    expect(castleKingDest(moveTo(at(4, 6), at(4, 4)))).toBeNull();
  });
});

describe("isSpecialMove", () => {
  it("is false for board-clickable moves (incl. the core chess moves)", () => {
    expect(isSpecialMove(moveTo(at(0, 0), at(0, 1)))).toBe(false);
    expect(isSpecialMove(castle(at(4, 7), "Kingside"))).toBe(false);
    expect(isSpecialMove(promotion(at(4, 1), at(4, 0)))).toBe(false);
    expect(isSpecialMove(enPassant(at(4, 3), at(5, 2), at(5, 3)))).toBe(false);
  });
  it("is true for side-panel moves", () => {
    expect(isSpecialMove({ from: at(0, 0), move_type: { kind: "PhaseShift" } })).toBe(true);
    expect(isSpecialMove({ from: at(0, 0), move_type: { kind: "ThrowSwitch", target: { switch: at(0, 0) } } })).toBe(true);
    expect(isSpecialMove({ from: at(0, 0), move_type: { kind: "PlaceTornado", target: { target: at(1, 1) } } })).toBe(true);
  });
});

describe("visibleMoveTargets — struct-variant wire nesting", () => {
  it("reads each core move's destination from the right nested field", () => {
    const moves = [
      moveTo(at(4, 6), at(4, 4)), // → target
      promotion(at(4, 1), at(4, 0)), // → target.target
      enPassant(at(4, 3), at(5, 2), at(5, 3)), // → target.target
      castle(at(4, 7), "Kingside"), // → castleKingDest (g-file)
    ];
    const targets = visibleMoveTargets(moves, null);
    expect(targets.map((t) => t.target)).toEqual([
      { file: 4, rank: 4 },
      { file: 4, rank: 0 },
      { file: 5, rank: 2 },
      { file: 6, rank: 7 },
    ]);
    expect(targets.every((t) => t.kind === "move")).toBe(true);
  });

  it("hides PieceInCarrier deploys until that passenger is picked", () => {
    const moves = [pieceInCarrier(at(5, 3), 0, at(7, 2))];
    expect(visibleMoveTargets(moves, null)).toEqual([]);
    expect(visibleMoveTargets(moves, 0)).toEqual([{ target: { file: 7, rank: 2 }, kind: "deploy" }]);
    // A different passenger index shows nothing.
    expect(visibleMoveTargets(moves, 1)).toEqual([]);
  });
});

describe("isAllowedSquare", () => {
  it("matches a destination present in the allowed moves", () => {
    const moves = [moveTo(at(4, 6), at(4, 4))];
    expect(isAllowedSquare(at(4, 4), moves, null)).toBe(true);
    expect(isAllowedSquare(at(0, 0), moves, null)).toBe(false);
  });

  it("only matches passenger deploys when that passenger is picked", () => {
    const moves = [pieceInCarrier(at(5, 3), 0, at(7, 2))];
    expect(isAllowedSquare(at(7, 2), moves, null)).toBe(false);
    expect(isAllowedSquare(at(7, 2), moves, 0)).toBe(true);
  });
});

describe("promotionsAt", () => {
  it("returns only the promotions landing on the clicked square", () => {
    const moves = [promotion(at(4, 1), at(4, 0)), promotion(at(4, 1), at(3, 0)), moveTo(at(0, 6), at(0, 5))];
    expect(promotionsAt(at(4, 0), moves)).toEqual([moves[0]]);
    expect(promotionsAt(at(0, 5), moves)).toEqual([]);
  });
});

describe("findMoveForTarget", () => {
  it("resolves a castle click on the king's landing square", () => {
    const m = castle(at(4, 7), "Kingside");
    expect(findMoveForTarget(at(6, 7), [m], null)).toBe(m);
  });

  it("resolves the picked passenger's deploy, not the carrier's own move", () => {
    const drive = moveTo(at(5, 3), at(7, 2));
    const deploy = pieceInCarrier(at(5, 3), 0, at(7, 2));
    expect(findMoveForTarget(at(7, 2), [drive, deploy], null)).toBe(drive);
    expect(findMoveForTarget(at(7, 2), [drive, deploy], 0)).toBe(deploy);
  });

  it("returns null when nothing lands there", () => {
    expect(findMoveForTarget(at(1, 1), [moveTo(at(0, 0), at(0, 1))], null)).toBeNull();
  });
});
