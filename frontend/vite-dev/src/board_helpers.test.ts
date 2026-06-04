import { afterEach, describe, expect, it } from "vitest";
import { castleKingDest, isAllowedSquare, isSpecialMove, visibleMoveTargets } from "./board_helpers";
import { setAllowedMoves, setSelectedPassengerIndex, type Coord, type GameMove } from "./variables";

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

describe("isAllowedSquare (reads module state)", () => {
  afterEach(() => {
    setAllowedMoves([]);
    setSelectedPassengerIndex(null);
  });

  it("matches a destination present in the allowed moves", () => {
    setAllowedMoves([moveTo(at(4, 6), at(4, 4))]);
    expect(isAllowedSquare(at(4, 4))).toBe(true);
    expect(isAllowedSquare(at(0, 0))).toBe(false);
  });
});
