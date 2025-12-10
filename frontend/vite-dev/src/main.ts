// src/main.ts

/// Builds with: npx tsc
/// Runs with: npx serve

import { parseFEN, pieceToSymbol } from "./fen";

let selectedSquare: Coord | null = null;
let allowedMoves: GameMove[] = []; // returned from API


function isAllowedSquare(c: Coord): boolean {
  // filter allowedMoves for MoveTo moves only
  let allowedCoords = allowedMoves
    .filter(m => isMoveTo(m))
    .map(m => m.move_type.target);

  return allowedCoords.some(ac => ac.file === c.file && ac.rank === c.rank);
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
      clearSelection();
    } catch (err) {
      console.error("Error making move:", err);
    }

    return;
  }

  selectedSquare = clicked;

  try {
    const fen = (document.getElementById("fen-input") as HTMLInputElement).value;
    allowedMoves = (await fetchMoves(fen, rank, file));

    console.log("Legal moves:", allowedMoves);

    highlightMoves(allowedMoves);
  } catch (err) {
    console.error("Error fetching moves:", err);
  }
}

/// Calls the backend API to get legal moves for a piece at (file, rank) on the board described by fen
async function fetchMoves(fen: string, rank: number, file: number): Promise<GameMove[]> {
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

type MoveType =
  | { kind: "MoveTo"; target: Coord }
  | { kind: "PhaseShift" };


type GameMove = { from: Coord; move_type: MoveType };

function isMoveTo(m: GameMove): m is GameMove & { move_type: { kind: "MoveTo"; target: Coord } } {
  return m.move_type.kind === "MoveTo";
}

/// Simple helper to highlight squares given a list of coordinates
function highlightMoves(moves: GameMove[]) {
  const squares = document.querySelectorAll(".square");
  squares.forEach(sq => sq.classList.remove("highlight"));

  // filter moves for MoveTo only
  let moveCoords = moves
    .filter(m => isMoveTo(m))
    .map(m => m.move_type.target);

  for (const m of moveCoords) {
    const index = m.rank * 8 + m.file;
    squares[index].classList.add("highlight");
  }
}

/// Clears any selected square and highlighted moves
function clearSelection() {
  selectedSquare = null;
  allowedMoves = [];

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
  let body = {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      board_fen: fen,
      game_move: {
        from,
        move_type: { kind: "MoveTo", target: to }
      }
    })
  };
  console.log("Making move with body:", body);
  const response = await fetch("http://localhost:8080/board/new_state", body);
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


// ------------------------------------------
// FEN PRESET LIST
// ------------------------------------------

const FEN_PRESETS: { name: string; fen: string }[] = [
  { name: "Empty Board", fen: "8/8/8/8/8/8/8/8" },
  { name: "Standard Chess", fen: "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR" },
  { name: "Goblin Test", fen: "(P=g(H=0-0))nbqkbn(P=g(H=7-0))/pppppppp/8/8/8/8/PPPPPPPP/(P=G(H=0-7))NBQKBN(P=G(H=7-7))" },
  { name: "Vent Test", fen: "(T=VENT)7/8/8/8/8/8/8/8" },
  { name: "Frozen Test", fen: "(C=FROZEN)7/8/8/8/8/8/8/8" },
];

function populateFENList() {
  const list = document.getElementById("fen-list")!;
  list.innerHTML = "";

  for (const { name, fen } of FEN_PRESETS) {
    const li = document.createElement("li");
    li.textContent = name;

    li.onclick = () => {
      const input = document.getElementById("fen-input") as HTMLInputElement;
      input.value = fen;
      renderBoard(fen);
    };

    list.appendChild(li);
  }
}

populateFENList();
