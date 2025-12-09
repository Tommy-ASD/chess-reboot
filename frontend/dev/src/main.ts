/// Builds with: npx tsc

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

            square.onclick = () => {
                handleSquareClick(rank, file);
            };

            boardEl.appendChild(square);
        }
    }
}

async function handleSquareClick(rank: number, file: number) {
    alert(`Square clicked: Rank ${8 - rank}, File ${String.fromCharCode(97 + file)}`);

    try {
        const fen = (document.getElementById("fen-input") as HTMLInputElement).value;
        const moves = await fetchMoves(fen, rank, file);

        console.log("Legal moves:", moves);
    } catch (err) {
        console.error("Error fetching moves:", err);
    }
}

async function fetchMoves(fen: string, rank: number, file: number): Promise<Coord[]> {
    const response = await fetch("http://localhost:8080/board/moves", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
            board_fen: fen,
            from: { file, rank }
        })
    });

    if (!response.ok) {
        throw new Error(`HTTP error ${response.status}`);
    }

    const data = await response.json();
    return data.moves; // Vec<Coord> from Rust
}

type Coord = { file: number; rank: number };


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
