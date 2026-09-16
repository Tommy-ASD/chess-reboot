import { describe, expect, it } from "vitest";
import { describeResult, describeStatus, isTerminal, statusWinner } from "./status";

describe("describeStatus", () => {
  it("shows nothing while the game goes on", () => {
    expect(describeStatus({ status: "Ongoing" })).toBeNull();
  });

  it("treats check as informational and mate as game over", () => {
    expect(describeStatus({ status: "Check", data: { side_to_move: "Black" } })?.kind).toBe("check");
    expect(describeStatus({ status: "Checkmate", data: { winner: "White" } })).toMatchObject({ title: "White wins", kind: "over" });
    expect(isTerminal({ status: "Stalemate" })).toBe(true);
    expect(isTerminal({ status: "Check", data: { side_to_move: "White" } })).toBe(false);
  });
});

describe("describeResult", () => {
  it("prefers the board's own ending when there is one", () => {
    const text = describeResult({ kind: "Decisive", winner: "White" }, { status: "Checkmate", data: { winner: "White" } });
    expect(text.detail).toBe("Checkmate.");
  });

  it("names the resigning side", () => {
    expect(describeResult({ kind: "Resignation", winner: "Black" }, { status: "Ongoing" }).detail).toBe("White resigned.");
  });

  it("describes an agreed draw", () => {
    expect(describeResult({ kind: "Draw" }, { status: "Ongoing" }).title).toBe("Draw");
  });
});

describe("statusWinner", () => {
  it("reads the winner off decided boards only", () => {
    expect(statusWinner({ status: "BrainrotWin", data: { winner: "Black" } })).toBe("Black");
    expect(statusWinner({ status: "Stalemate" })).toBeNull();
    expect(statusWinner({ status: "Check", data: { side_to_move: "White" } })).toBeNull();
  });
});
