// src/fen.ts

// Converts piece letters into unicode chess symbols (optional)
export const pieceToSymbol = (p: string): string => {
  const map: Record<string, string> = {
    "K": "♔", "Q": "♕", "R": "♖",
    "B": "♗", "N": "♘", "P": "♙",
    "k": "♚", "q": "♛", "r": "♜",
    "b": "♝", "n": "♞", "p": "♟",
  };

  return map[p] ?? p;
};

export type Square = {
  piece: string | null;
  squareType: "STANDARD" | "TURRET" | "VENT";
  conditions: string[]; // ["FROZEN"]
};

// ---------------------------
// FEN Parsing
// ---------------------------

export function fenToSquare(fen: string): Square {
  // Empty or "()"
  if (fen === "" || fen === "()") {
    return { piece: null, squareType: "STANDARD", conditions: [] };
  }

  // Extended notation "(P=x,T=VENT,C=FROZEN)"
  if (fen.startsWith("(") && fen.endsWith(")")) {
    const inner = fen.slice(1, -1);
    const parts = inner.split(",");
    let piece: string | null = null;
    let squareType: Square["squareType"] = "STANDARD";
    const conditions: string[] = [];

    for (const part of parts) {
      const [key, value] = part.split("=");

      switch (key) {
        case "P": piece = value; break;
        case "T": squareType = value as any; break;
        case "C": conditions.push(value); break;
      }
    }

    return { piece, squareType, conditions };
  }

  // Standard piece single char
  return {
    piece: fen,
    squareType: "STANDARD",
    conditions: []
  };
}

export function squareToFEN(square: Square): string {
  const isStandard =
    square.squareType === "STANDARD" &&
    square.conditions.length === 0 &&
    square.piece !== null &&
    square.piece.length === 1;

  if (isStandard) {
    return square.piece!;
  }

  const parts: string[] = [];

  if (square.piece) parts.push(`P=${square.piece}`);
  if (square.squareType !== "STANDARD") parts.push(`T=${square.squareType}`);
  for (const cond of square.conditions) parts.push(`C=${cond}`);

  return `(${parts.join(",")})`;
}

export function parseFENRow(row: string): Square[] {
  const squares: Square[] = [];
  let i = 0;

  while (i < row.length) {
    const ch = row[i];

    if (/\d/.test(ch)) {
      // empty squares
      const count = Number(ch);
      for (let k = 0; k < count; k++) {
        squares.push({ piece: null, squareType: "STANDARD", conditions: [] });
      }
      i++;
    }
    else if (ch === "(") {
      // parse until matching ')'
      let depth = 0;
      let buf = "";

      while (i < row.length) {
        const c = row[i];
        buf += c;
        if (c === "(") depth++;
        if (c === ")") depth--;
        i++;
        if (depth === 0) break;
      }

      squares.push(fenToSquare(buf));
    }
    else {
      // single piece
      squares.push(fenToSquare(ch));
      i++;
    }
  }

  if (squares.length !== 8)
    throw new Error("Invalid row length: " + row);

  return squares;
}

export function parseFEN(fen: string): Square[][] {
  const rows = fen.split("/");

  if (rows.length !== 8)
    throw new Error("Invalid FEN: must have 8 rows");

  return rows.map(parseFENRow);
}


export function squaresToFEN(board: Square[][]): string {
  return board.map(row => {
    let fenRow = "";
    let empty = 0;

    for (const sq of row) {
      const fen = squareToFEN(sq);

      if (fen === "" || fen === "()") {
        empty++;
      } else {
        if (empty > 0) {
          fenRow += empty.toString();
          empty = 0;
        }
        fenRow += fen;
      }
    }

    if (empty > 0) fenRow += empty.toString();

    return fenRow;
  }).join("/");
}
