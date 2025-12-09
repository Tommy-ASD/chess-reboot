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

// ---------------------------
// FEN Parsing
// ---------------------------

export function parseFEN(fen: string): (string | null)[][] {
    const rows = fen.split("/");

    if (rows.length !== 8) {
        throw new Error("Invalid FEN: must contain 8 rows");
    }

    return rows.map(row => {
        const squares: (string | null)[] = [];

        for (const char of row) {
            if (!isNaN(Number(char))) {
                // number → empty squares
                const count = Number(char);
                for (let i = 0; i < count; i++) {
                    squares.push(null);
                }
            } else {
                // piece letter
                squares.push(char);
            }
        }

        if (squares.length !== 8) {
            throw new Error("Invalid FEN row: " + row);
        }

        return squares;
    });
}