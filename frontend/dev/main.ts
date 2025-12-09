/// Builds with: npx tsc main.ts --watch

// Converts piece letters into unicode chess symbols (optional)
const pieceToSymbol = (p: string): string => {
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

function parseFEN(fen: string): (string | null)[][] {
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


// ---------------------------
// Rendering
// ---------------------------

function renderBoard(fen: string) {
    const boardEl = document.getElementById("board")!;
    boardEl.innerHTML = ""; // clear previous board

    const grid = parseFEN(fen);

    // Loop rank 8 → 1 (FEN order)
    for (let rank = 0; rank < 8; rank++) {
        for (let file = 0; file < 8; file++) {
            const piece = grid[rank][file];

            const square = document.createElement("div");
            square.classList.add("square");

            // light/dark checkered pattern
            const isDark = (rank + file) % 2 === 1;
            square.classList.add(isDark ? "dark" : "light");

            if (piece) {
                square.textContent = pieceToSymbol(piece);
            }

            boardEl.appendChild(square);
        }
    }
}


// ---------------------------
// UI Wiring
// ---------------------------

document.getElementById("load-btn")!.addEventListener("click", () => {
    const fen = (document.getElementById("fen-input") as HTMLInputElement).value;
    try {
        renderBoard(fen);
    } catch (e) {
        alert(e instanceof Error ? e.message : String(e));
    }
});

// Optional: live preview
document.getElementById("fen-input")!.addEventListener("input", (ev) => {
    const value = (ev.target as HTMLInputElement).value;
    try { renderBoard(value); } catch { }
});
