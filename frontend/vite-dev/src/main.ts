// src/main.ts
/// Builds with: npx tsc
/// Runs with: npx serve

import { parseFEN, pieceToSymbol } from "./fen";

let selectedSquare: Coord | null = null;
let allowedSquares: Coord[] = []; // returned from API

function isAllowedSquare(c: Coord): boolean {
  return allowedSquares.some(m => m.rank === c.rank && m.file === c.file);
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
      const square_data = grid[rank][file];

      const square = document.createElement("div");
      square.classList.add("square");

      // light/dark checkered pattern
      const isDark = (rank + file) % 2 === 1;
      square.classList.add(isDark ? "dark" : "light");

      if (square_data) {
        if (square_data.piece) {
          square.textContent = pieceToSymbol(square_data.piece);
        }
        if (square_data.conditions.includes("FROZEN")) {
          square.classList.add("cond-frozen");
        }
        if (square_data.squareType === "VENT") {
          square.classList.add("type-vent");
        }

      }

      square.onclick = () => {
        handleSquareClick(rank, file);
      };

      boardEl.appendChild(square);
    }
  }
}

/// Handler attached to each square on the board
/// On click, fetches legal moves from backend and highlights them
async function handleSquareClick(rank: number, file: number) {
  const clicked = { rank, file };

  // if the user clicks the selected square again, clear selection
  if (selectedSquare && selectedSquare.rank === rank && selectedSquare.file === file) {
    console.log("Pressed twice; clearing selection");
    clearSelection();
    return;
  }

  // if the user clicks an allowed square, make the move
  if (isAllowedSquare(clicked)) {
    console.log("Move:", selectedSquare, "->", clicked);

    try {
      const fen = (document.getElementById("fen-input") as HTMLInputElement).value;
      const newFen = await makeMove(fen, selectedSquare!, clicked);
      console.log("New FEN:", newFen);
      (document.getElementById("fen-input") as HTMLInputElement).value = newFen;
      renderBoard(newFen);
    } catch (err) {
      console.error("Error making move:", err);
    }

    return;
  }

  selectedSquare = clicked;

  try {
    const fen = (document.getElementById("fen-input") as HTMLInputElement).value;
    allowedSquares = await fetchMoves(fen, rank, file);

    console.log("Legal moves:", allowedSquares);

    highlightMoves(allowedSquares);
  } catch (err) {
    console.error("Error fetching moves:", err);
  }
}

/// Calls the backend API to get legal moves for a piece at (file, rank) on the board described by fen
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

/// Simple helper to highlight squares given a list of coordinates
function highlightMoves(moves: Coord[]) {
  const squares = document.querySelectorAll(".square");
  squares.forEach(sq => sq.classList.remove("highlight"));

  for (const m of moves) {
    const index = m.rank * 8 + m.file;
    squares[index].classList.add("highlight");
  }
}

/// Clears any selected square and highlighted moves
function clearSelection() {
  selectedSquare = null;
  allowedSquares = [];

  const squares = document.querySelectorAll(".square");
  squares.forEach(s => s.classList.remove("selected", "highlight"));
}

/// Attempts to make a move
/// API call's at `POST /board/new_state` with body:
/// {
///   board_fen: string,
///   from: { file: number, rank: number },
///   to: { file: number, rank: number }
/// }
/// Returns the new FEN string on success
async function makeMove(fen: string, from: Coord, to: Coord): Promise<string> {
  const response = await fetch("http://localhost:8080/board/new_state", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      board_fen: fen,
      from,
      to
    })
  });
  console.log("Response:", response);

  if (!response.ok) {
    throw new Error(`HTTP error ${response.status}`);
  }

  const data = await response.json();
  console.log("Move response data:", data);
  return data.new_board_fen; // new FEN string from Rust
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
