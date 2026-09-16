import { describe, expect, it } from "vitest";
import { DEFAULT_FLAGS, parseFEN, parseFENFlags, squaresToFEN } from "./fen";
import { mirrorBoard, mirrorDir, mirrorFlags, mirrorPosition } from "./mirror";

const STANDARD = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR";

describe("mirrorBoard", () => {
  it("flips files left-to-right", () => {
    const grid = squaresToFEN(mirrorBoard(parseFEN(STANDARD), "horizontal"));
    // Every rank reverses: the king and queen swap places.
    expect(grid).toBe("rnbkqbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBKQBNR");
  });

  it("flips ranks top-to-bottom", () => {
    const grid = squaresToFEN(mirrorBoard(parseFEN(STANDARD), "vertical"));
    expect(grid).toBe("RNBQKBNR/PPPPPPPP/8/8/8/8/pppppppp/rnbqkbnr");
  });

  it("rotates 180 degrees when both axes are mirrored", () => {
    const grid = squaresToFEN(mirrorBoard(parseFEN(STANDARD), "both"));
    expect(grid).toBe("RNBKQBNR/PPPPPPPP/8/8/8/8/pppppppp/rnbkqbnr");
  });

  it("transposes across the diagonal, swapping width and height", () => {
    const board = parseFEN("Kq6/8/8/8/8/8/8/8"); // 8x8, K at (0,0), q at (1,0)
    const mirrored = mirrorBoard(board, "diagonal");
    expect(mirrored[0][0].piece).toBe("K");
    expect(mirrored[1][0].piece).toBe("q"); // file 1 becomes rank 1
    expect(mirrored[0][1].piece).toBeNull();
  });

  it("swaps dimensions on a non-square board", () => {
    const board = parseFEN("PPP/ppp"); // 3 wide, 2 tall
    const mirrored = mirrorBoard(board, "diagonal");
    expect(mirrored).toHaveLength(3);
    expect(mirrored[0]).toHaveLength(2);
    expect(squaresToFEN(mirrored)).toBe("Pp/Pp/Pp");
  });

  it("leaves the input board untouched", () => {
    const board = parseFEN(STANDARD);
    mirrorBoard(board, "both");
    expect(squaresToFEN(board)).toBe(STANDARD);
  });

  it("round-trips: mirroring twice on the same axis is the identity", () => {
    for (const axis of ["horizontal", "vertical", "both", "diagonal"] as const) {
      const once = mirrorBoard(parseFEN(STANDARD), axis);
      expect(squaresToFEN(mirrorBoard(once, axis))).toBe(STANDARD);
    }
  });

  it("preserves conditions, square types and duck flags", () => {
    const board = parseFEN("(P=P,T=VENT,C=FROZEN)7/7(DUCK)/8/8/8/8/8/8");
    const mirrored = mirrorBoard(board, "horizontal");
    const moved = mirrored[0][7];
    expect(moved.piece).toBe("P");
    expect(moved.squareType).toBe("VENT");
    expect(moved.conditions).toEqual(["FROZEN"]);
    expect(mirrored[1][0].duck).toBe(true);
  });
});

describe("mirrorDir", () => {
  it("flips east/west on a horizontal mirror and leaves north/south", () => {
    expect(mirrorDir("E", "horizontal")).toBe("W");
    expect(mirrorDir("W", "horizontal")).toBe("E");
    expect(mirrorDir("N", "horizontal")).toBe("N");
  });

  it("flips north/south on a vertical mirror", () => {
    expect(mirrorDir("N", "vertical")).toBe("S");
    expect(mirrorDir("E", "vertical")).toBe("E");
  });

  it("flips both on a 180 rotation", () => {
    expect(mirrorDir("N", "both")).toBe("S");
    expect(mirrorDir("E", "both")).toBe("W");
  });

  it("swaps the axes on a diagonal mirror", () => {
    expect(mirrorDir("N", "diagonal")).toBe("W");
    expect(mirrorDir("W", "diagonal")).toBe("N");
    expect(mirrorDir("S", "diagonal")).toBe("E");
    expect(mirrorDir("E", "diagonal")).toBe("S");
  });
});

describe("mirrorBoard payloads", () => {
  it("re-orients track directions", () => {
    const board = parseFEN("(T=TRACK,D=E)7/8/8/8/8/8/8/8");
    const mirrored = mirrorBoard(board, "horizontal");
    expect(mirrored[0][7].extraFields?.D).toBe("W");
  });

  it("re-orients junction branches and keeps the canonical format", () => {
    const board = parseFEN("(T=JUNCTION,ID=1,STATE=0,BRANCHES=(N,E))7/8/8/8/8/8/8/8");
    const mirrored = mirrorBoard(board, "vertical");
    expect(mirrored[7][0].extraFields?.BRANCHES).toBe("(S,E)");
  });

  it("leaves signal ids and targets alone (wiring is id-based)", () => {
    const board = parseFEN("(T=SWITCH,ID=2,TARGETS=(1,3))7/8/8/8/8/8/8/8");
    const mirrored = mirrorBoard(board, "both");
    expect(mirrored[7][7].extraFields?.ID).toBe("2");
    expect(mirrored[7][7].extraFields?.TARGETS).toBe("(1,3)");
  });

  it("mirrors a goblin's home coordinate", () => {
    const board = parseFEN("(P=G(H=0-7))7/8/8/8/8/8/8/8");
    expect(mirrorBoard(board, "horizontal")[0][7].piece).toBe("G(H=7-7)");
    expect(mirrorBoard(board, "vertical")[7][0].piece).toBe("G(H=0-0)");
    expect(mirrorBoard(board, "diagonal")[0][0].piece).toBe("G(H=7-0)");
  });

  it("mirrors a goblin home nested inside a carrier's passenger list", () => {
    const board = parseFEN("(P=BUS(P=(G(H=0-7))))7/8/8/8/8/8/8/8");
    expect(mirrorBoard(board, "horizontal")[0][7].piece).toBe("BUS(P=(G(H=7-7)))");
  });

  it("mirrors a locomotive's last-direction but not its F/R heading", () => {
    const board = parseFEN("(P=LOCO(ID=1,H=F,L=E),T=TRACK,D=E)7/8/8/8/8/8/8/8");
    const moved = mirrorBoard(board, "horizontal")[0][7];
    expect(moved.piece).toBe("LOCO(ID=1,H=F,L=W)");
    expect(moved.extraFields?.D).toBe("W");
  });
});

describe("mirrorFlags", () => {
  it("swaps kingside/queenside rights on a horizontal mirror", () => {
    const flags = { ...DEFAULT_FLAGS, castling: { K: true, Q: false, k: false, q: true } };
    expect(mirrorFlags(flags, "horizontal", 8, 8).castling)
      .toEqual({ K: false, Q: true, k: true, q: false });
  });

  it("leaves castling rights alone on a vertical mirror", () => {
    const flags = { ...DEFAULT_FLAGS, castling: { K: true, Q: false, k: false, q: true } };
    expect(mirrorFlags(flags, "vertical", 8, 8).castling)
      .toEqual({ K: true, Q: false, k: false, q: true });
  });

  it("mirrors the en-passant target square", () => {
    const flags = { ...DEFAULT_FLAGS, enPassant: "e3" };
    expect(mirrorFlags(flags, "horizontal", 8, 8).enPassant).toBe("d3");
    expect(mirrorFlags(flags, "vertical", 8, 8).enPassant).toBe("e6");
    expect(mirrorFlags(flags, "both", 8, 8).enPassant).toBe("d6");
  });

  it("leaves an unparseable en-passant field untouched", () => {
    const flags = { ...DEFAULT_FLAGS, enPassant: "zz" };
    expect(mirrorFlags(flags, "horizontal", 8, 8).enPassant).toBe("zz");
  });

  it("passes side to move, ply count and variants through unchanged", () => {
    const flags = {
      ...DEFAULT_FLAGS,
      sideToMove: "b" as const,
      plyCount: 7,
      variants: ["duck_chess"],
    };
    const out = mirrorFlags(flags, "both", 8, 8);
    expect(out.sideToMove).toBe("b");
    expect(out.plyCount).toBe(7);
    expect(out.variants).toEqual(["duck_chess"]);
  });
});

describe("mirrorPosition", () => {
  it("mirrors grid and flags together", () => {
    const fen = `${STANDARD} w KQk e3 tr=full p=4`;
    const { board, flags } = mirrorPosition(parseFEN(fen), parseFENFlags(fen), "horizontal");
    expect(squaresToFEN(board)).toBe("rnbkqbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBKQBNR");
    // "KQk" mirrored: white keeps both wings, black's kingside right
    // becomes a queenside one.
    expect(flags.castling).toEqual({ K: true, Q: true, k: false, q: true });
    expect(flags.enPassant).toBe("d3");
    expect(flags.plyCount).toBe(4);
  });
});
