import { describe, expect, it } from "vitest";
import { describeSpecialMove, squareName } from "./notation";

describe("squareName", () => {
  it("maps engine coords (rank 0 at the top) to algebraic names", () => {
    expect(squareName({ file: 0, rank: 7 }, 8)).toBe("a1");
    expect(squareName({ file: 4, rank: 4 }, 8)).toBe("e4");
    expect(squareName({ file: 7, rank: 0 }, 8)).toBe("h8");
  });

  it("follows the board height on non-standard boards", () => {
    expect(squareName({ file: 2, rank: 0 }, 10)).toBe("c10");
  });
});

describe("describeSpecialMove", () => {
  it("names the tornado's target square", () => {
    expect(
      describeSpecialMove({ from: { file: 3, rank: 7 }, move_type: { kind: "PlaceTornado", target: { target: { file: 4, rank: 6 } } } }, 8),
    ).toBe("Summon a tornado on e2");
  });

  it("uses player language for brainrot", () => {
    expect(describeSpecialMove({ from: { file: 0, rank: 0 }, move_type: { kind: "PhaseShift" } }, 8)).toBe("Grow brainrot");
  });
});
