// ----------------------------------------------
// Type Definitions
// ----------------------------------------------

export type SquareType = "STANDARD" | "TURRET" | "VENT";

export type Square = {
  piece: string | null;        // "P", "q", "G(W=...)" etc
  squareType: SquareType;
  conditions: string[];        // ["FROZEN", ...]
};

// For pretty optional rendering
export const pieceToSymbol = (p: string): string => {
  const map: Record<string, string> = {
    "K": "♔", "Q": "♕", "R": "♖",
    "B": "♗", "N": "♘", "P": "♙",
    "k": "♚", "q": "♛", "r": "♜",
    "b": "♝", "n": "♞", "p": "♟",
  };
  return map[p] ?? p;
};

// ----------------------------------------------
// Top-level split (nested parentheses-safe)
// ----------------------------------------------

function splitTopLevel(input: string): string[] {
  const parts: string[] = [];
  let buf = "";
  let depth = 0;

  for (const ch of input) {
    if (ch === "(") {
      depth++;
      buf += ch;
    } else if (ch === ")") {
      depth--;
      buf += ch;
    }
    else if (ch === "," && depth === 0) {
      // split here
      parts.push(buf.trim());
      buf = "";
      continue;
    }
    else {
      buf += ch;
    }
  }

  if (buf.length > 0) {
    parts.push(buf.trim());
  }

  return parts;
}


function splitKeyValue(field: string): [string, string] {
  const idx = field.indexOf("=");
  if (idx === -1) return [field.trim(), ""];

  const key = field.slice(0, idx).trim();
  const value = field.slice(idx + 1).trim();
  return [key, value];
}


// ----------------------------------------------
// Parse ONE square from FEN
// ----------------------------------------------

export function fenToSquare(fen: string): Square {
  // Empty or "()"
  if (fen === "" || fen === "()") {
    return { piece: null, squareType: "STANDARD", conditions: [] };
  }

  // ------------------------------------------
  // Extended notation: (P=x,T=VENT,C=FROZEN)
  // ------------------------------------------
  if (fen.startsWith("(") && fen.endsWith(")")) {
    const inner = fen.slice(1, -1);
    const fields = splitTopLevel(inner);

    let piece: string | null = null;
    let squareType: SquareType = "STANDARD";
    const conditions: string[] = [];

    for (const field of fields) {
      const [key, value] = splitKeyValue(field);

      switch (key) {
        case "P":
          piece = value;
          console.log("Parsed piece:", piece, "from", field);
          break;

        case "T":
          if (value === "TURRET" || value === "VENT" || value === "STANDARD") {
            squareType = value;
          }
          break;

        case "C":
          conditions.push(value);
          break;

        default:
          console.warn("Unknown FEN square field:", field);
      }
    }

    return { piece, squareType, conditions };
  }

  // ------------------------------------------
  // Standard single-piece square
  // ------------------------------------------
  return {
    piece: fen,
    squareType: "STANDARD",
    conditions: [],
  };
}

// ----------------------------------------------
// Convert square → FEN
// ----------------------------------------------

export function squareToFEN(square: Square): string {
  const isStandardPiece =
    square.squareType === "STANDARD" &&
    square.conditions.length === 0 &&
    square.piece !== null &&
    square.piece.length === 1;

  // Simple single-piece format
  if (isStandardPiece) {
    return square.piece!;
  }

  // Extended format
  const parts: string[] = [];

  if (square.piece) parts.push(`P=${square.piece}`);
  if (square.squareType !== "STANDARD") parts.push(`T=${square.squareType}`);
  for (const c of square.conditions) parts.push(`C=${c}`);

  return `(${parts.join(",")})`;
}

// ----------------------------------------------
// Parse a FEN row (8 squares)
// ----------------------------------------------

export function parseFENRow(row: string): Square[] {
  const squares: Square[] = [];
  let i = 0;

  while (i < row.length) {
    const ch = row[i];

    // Digit → N empty squares
    if (/\d/.test(ch)) {
      const count = Number(ch);
      for (let k = 0; k < count; k++) {
        squares.push({
          piece: null,
          squareType: "STANDARD",
          conditions: [],
        });
      }
      i++;
      continue;
    }

    // Extended form
    if (ch === "(") {
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
      continue;
    }

    // Standard piece
    squares.push(fenToSquare(ch));
    i++;
  }

  if (squares.length !== 8) {
    throw new Error("Invalid FEN row: " + row);
  }

  return squares;
}

// ----------------------------------------------
// Parse whole board
// ----------------------------------------------

export function parseFEN(fen: string): Square[][] {
  const rows = fen.split("/");
  if (rows.length !== 8) {
    throw new Error("Invalid FEN: must have exactly 8 rows");
  }
  return rows.map(parseFENRow);
}

// ----------------------------------------------
// Convert board → FEN
// ----------------------------------------------

export function squaresToFEN(board: Square[][]): string {
  return board
    .map(row => {
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

      if (empty > 0) {
        fenRow += empty.toString();
      }

      return fenRow;
    })
    .join("/");
}
