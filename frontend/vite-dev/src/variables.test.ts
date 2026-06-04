import { afterEach, describe, expect, it } from "vitest";
import { boardOrientation, setBoardOrientation, squareDomIndex } from "./variables";

// squareDomIndex reads shared module state — reset it after every test.
afterEach(() => setBoardOrientation("white"));

describe("squareDomIndex — white orientation", () => {
  it("is row-major rank*cols + file", () => {
    expect(squareDomIndex(0, 0, 8, 8)).toBe(0);
    expect(squareDomIndex(7, 7, 8, 8)).toBe(63);
    expect(squareDomIndex(1, 4, 8, 8)).toBe(12);
    expect(squareDomIndex(6, 4, 8, 8)).toBe(52);
  });
});

describe("squareDomIndex — black orientation (180° rotation)", () => {
  it("reverses both axes", () => {
    setBoardOrientation("black");
    expect(squareDomIndex(0, 0, 8, 8)).toBe(63); // a8 → bottom-right slot
    expect(squareDomIndex(7, 7, 8, 8)).toBe(0); // h1 → top-left slot
    expect(squareDomIndex(6, 4, 8, 8)).toBe(11); // e2 from-square
    expect(squareDomIndex(4, 4, 8, 8)).toBe(27); // e4 to-square
  });

  it("is its own inverse — a DOM slot maps back to its logical square", () => {
    setBoardOrientation("black");
    for (let rank = 0; rank < 8; rank++) {
      for (let file = 0; file < 8; file++) {
        const dom = squareDomIndex(rank, file, 8, 8);
        const backRank = 7 - Math.floor(dom / 8);
        const backFile = 7 - (dom % 8);
        expect({ rank: backRank, file: backFile }).toEqual({ rank, file });
      }
    }
  });

  it("respects non-8 board dimensions", () => {
    setBoardOrientation("black");
    // 10 wide × 3 tall: (0,0) → (3-1)*10 + (10-1) = 29
    expect(squareDomIndex(0, 0, 3, 10)).toBe(29);
  });
});

describe("setBoardOrientation", () => {
  it("updates the exported live binding", () => {
    expect(boardOrientation).toBe("white");
    setBoardOrientation("black");
    expect(boardOrientation).toBe("black");
  });
});
