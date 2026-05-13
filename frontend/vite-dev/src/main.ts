// src/main.ts

/// Builds with: npx tsc
/// Runs with: npx serve

import { initBoardResize, setBoardDimensions } from "./board_size";
import { clearSelection, highlightMoves, isAllowedSquare, isSpecialMove } from "./board_helpers";
import { getBusPassengers, parseFEN, pieceToImage, pieceToSymbol } from "./fen";
import { squareIconSvg } from "./signal_icons";
import { allowedMoves, currentBoard, selectedPassengerIndex, selectedSquare, setAllowedMoves, setCurrentBoard, setSelectedPassengerIndex, setSelectedSquare, type Coord, type GameMove } from "./variables";



// ---------------------------
// Rendering
// ---------------------------

function renderBoard(fen: string) {
  const boardEl = document.getElementById("board")!;
  boardEl.innerHTML = ""; // clear previous board

  setCurrentBoard(parseFEN(fen));
  const rows = currentBoard.length;
  const cols = currentBoard[0]?.length ?? 0;
  setBoardDimensions(cols, rows);

  for (let rank = 0; rank < rows; rank++) {
    for (let file = 0; file < cols; file++) {
      const square_data = currentBoard[rank][file];

      const square = document.createElement("div");
      square.classList.add("square");

      // light/dark checkered pattern
      const isDark = (rank + file) % 2 === 1;
      square.classList.add(isDark ? "dark" : "light");

      if (square_data) {
        console.log(square_data);
        if (square_data.piece) {
          // check if pieceToImage returns other than undefined
          // and if it does, use an img element instead of textContent
          const imgPath = pieceToImage(square_data.piece);
          if (imgPath) {
            const img = document.createElement("img");
            img.src = imgPath;
            img.alt = square_data.piece;
            img.classList.add("piece-image");
            square.appendChild(img);
          } else {
            square.textContent = pieceToSymbol(square_data.piece);
          }
        }
        if (square_data.conditions.includes("FROZEN")) {
          square.classList.add("cond-frozen");
        }
        if (square_data.conditions.includes("BRAINROT")) {
          square.classList.add("cond-brainrot");
        }
        // Plan 08: substrate types render with a per-type accent border
        // (via `type-{lowercase}`) plus an SVG icon overlay.
        if (square_data.squareType !== "STANDARD") {
          square.classList.add(`type-${square_data.squareType.toLowerCase()}`);
          const svg = squareIconSvg(square_data);
          if (svg) {
            const wrap = document.createElement("div");
            wrap.className = "square-icon";
            wrap.innerHTML = svg;
            square.appendChild(wrap);
          }
        }

      }

      square.onclick = () => handleSquareClick(rank, file);

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
    console.log("Move:", selectedSquare, "->", clicked, "passenger:", selectedPassengerIndex);

    const moveToExecute = findMoveForTarget(clicked, allowedMoves, selectedPassengerIndex);
    if (!moveToExecute) {
      console.error("isAllowedSquare matched but findMoveForTarget returned null");
      return;
    }

    try {
      const fen = (document.getElementById("fen-input") as HTMLInputElement).value;
      const newFen = moveToExecute.move_type.kind === "MoveTo"
        ? await makeMove(fen, selectedSquare!, clicked)
        : await makeSpecialMove(fen, moveToExecute);
      console.log("New FEN:", newFen);
      (document.getElementById("fen-input") as HTMLInputElement).value = newFen;
      renderBoard(newFen);
      clearSelection();
    } catch (err) {
      showError(err);
    }

    return;
  }

  setSelectedSquare(clicked);
  setSelectedPassengerIndex(null);

  // Visually mark the selected square
  const squareEls = document.querySelectorAll(".square");
  squareEls.forEach(s => s.classList.remove("selected"));
  squareEls[rank * 8 + file]?.classList.add("selected");

  try {
    const fen = (document.getElementById("fen-input") as HTMLInputElement).value;
    setAllowedMoves((await fetchMoves(fen, rank, file)));

    console.log("Legal moves:", allowedMoves);

    highlightMoves(allowedMoves);
    renderSpecialActions(allowedMoves);
    renderCarrierPanel(allowedMoves, rank, file);
  } catch (err) {
    showError(err);
  }
}

/// Given a clicked target square, find the corresponding GameMove from
/// the allowed list, respecting whether we're showing the carrier's own
/// moves or a specific passenger's deploys.
function findMoveForTarget(clicked: Coord, moves: GameMove[], passengerIdx: number | null): GameMove | null {
  const sameCoord = (a: Coord, b: Coord) => a.file === b.file && a.rank === b.rank;
  for (const m of moves) {
    if (passengerIdx === null) {
      if (m.move_type.kind === "MoveTo" && sameCoord(m.move_type.target, clicked)) return m;
      if (m.move_type.kind === "MoveIntoCarrier" && sameCoord(m.move_type.target, clicked)) return m;
    } else {
      if (m.move_type.kind === "PieceInCarrier"
        && m.move_type.target.piece_index === passengerIdx
        && m.move_type.target.move_type.kind === "MoveTo"
        && sameCoord(m.move_type.target.move_type.target, clicked)) {
        return m;
      }
    }
  }
  return null;
}

/// The side-actions panel: catch-all for moves that don't fit the
/// "click a destination on the board" model — currently PhaseShift,
/// future Promotion menu, etc. Carrier moves are NOT special; they
/// flow through renderCarrierPanel + board highlights instead.
function renderSpecialActions(moves: GameMove[]) {
  const list = document.getElementById("special-actions")!;
  list.innerHTML = "";

  const specials = moves.filter(isSpecialMove);

  for (const m of specials) {
    const li = document.createElement("li");

    switch (m.move_type.kind) {
      case "PhaseShift":
        li.textContent = "Increase Brainrot Radius (PhaseShift)";
        break;

      case "ThrowSwitch":
        li.textContent = "Throw Switch";
        break;

      default:
        li.textContent = JSON.stringify(m.move_type);
        break;
    }

    li.onclick = async () => {
      const fen = (document.getElementById("fen-input") as HTMLInputElement).value;
      try {
        const newFen = await makeSpecialMove(fen, m);
        (document.getElementById("fen-input") as HTMLInputElement).value = newFen;
        renderBoard(newFen);
        clearSelection();
      } catch (err) {
        showError(err);
      }
    };

    list.appendChild(li);
  }
}

/// Two-step passenger picker. When the selected piece is a carrier with
/// passengers, render one tile per passenger plus a "Drive" tile that
/// flips back to the carrier's own moves. Clicking a tile switches the
/// board highlights between drive-mode and deploy-mode for that passenger.
function renderCarrierPanel(moves: GameMove[], rank: number, file: number) {
  const panel = document.getElementById("carrier-moves")!;
  panel.innerHTML = "";

  // Only show the panel when there are passenger-deploy moves available.
  const hasDeployMoves = moves.some(m => m.move_type.kind === "PieceInCarrier");
  if (!hasDeployMoves) return;

  const square = currentBoard[rank]?.[file];
  if (!square || !square.piece) return;

  const passengers = getBusPassengers(square.piece);
  if (passengers.length === 0) return;

  panel.appendChild(makePassengerTile({
    glyph: "\u{1F68C}", // bus emoji as the "drive" icon
    label: "Drive",
    isActive: selectedPassengerIndex === null,
    extraClass: "drive-tile",
    onPick: () => {
      setSelectedPassengerIndex(null);
      highlightMoves(allowedMoves);
      renderCarrierPanel(moves, rank, file);
    },
  }));

  for (let i = 0; i < passengers.length; i++) {
    const piece = passengers[i];
    const idx = i;
    panel.appendChild(makePassengerTile({
      glyph: pieceToSymbol(piece),
      glyphImage: pieceToImage(piece),
      label: `#${idx}`,
      isActive: selectedPassengerIndex === idx,
      onPick: () => {
        setSelectedPassengerIndex(idx);
        highlightMoves(allowedMoves);
        renderCarrierPanel(moves, rank, file);
      },
    }));
  }
}

function makePassengerTile(opts: {
  glyph: string;
  glyphImage?: string;
  label: string;
  isActive: boolean;
  extraClass?: string;
  onPick: () => void;
}): HTMLButtonElement {
  const btn = document.createElement("button");
  btn.className = "passenger-tile" + (opts.isActive ? " active" : "") + (opts.extraClass ? " " + opts.extraClass : "");

  if (opts.glyphImage) {
    const img = document.createElement("img");
    img.src = opts.glyphImage;
    img.className = "passenger-glyph-img";
    btn.appendChild(img);
  } else {
    const span = document.createElement("span");
    span.className = "passenger-glyph";
    span.textContent = opts.glyph;
    btn.appendChild(span);
  }

  const label = document.createElement("span");
  label.className = "passenger-label";
  label.textContent = opts.label;
  btn.appendChild(label);

  btn.onclick = opts.onPick;
  return btn;
}

/// Structured failure body the backend returns on 4xx responses to
/// `/board/new_state`. Mirrors `MakeMoveErrorBody` in `api/src/main.rs`.
type MakeMoveErrorBody = {
  code: string;
  message: string;
  details: unknown;
  side_to_move: "White" | "Black";
  received: unknown;
};

/// Read a non-2xx response, prefer JSON for structured engine errors,
/// fall back to plain text. Always logs the full body so a quick console
/// glance shows the engine's diagnostic context.
async function consumeError(response: Response, context: string): Promise<Error> {
  let text = "";
  try {
    text = await response.text();
  } catch {
    return new Error(`${context}: HTTP ${response.status} (no body)`);
  }
  try {
    const parsed = JSON.parse(text) as MakeMoveErrorBody;
    console.error(`${context}: server returned ${response.status}`, parsed);
    const msg = parsed.message ?? text;
    const code = parsed.code ? ` [${parsed.code}]` : "";
    return new Error(`${context}${code}: ${msg}`);
  } catch {
    console.error(`${context}: server returned ${response.status} with non-JSON body:`, text);
    return new Error(`${context}: HTTP ${response.status} — ${text || "(empty body)"}`);
  }
}

/// Show an error to the user. Currently a plain alert (cheap and
/// impossible to miss); upgrade to an in-page toast once we have one.
function showError(err: unknown) {
  const msg = err instanceof Error ? err.message : String(err);
  console.error("Surfaced to user:", msg);
  alert(msg);
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
    throw await consumeError(response, "fetchMoves");
  }

  const data = await response.json();
  return data.moves; // Vec<Coord> from Rust
}

async function makeSpecialMove(fen: string, move: GameMove): Promise<string> {
  const response = await fetch("http://localhost:8080/board/new_state", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      board_fen: fen,
      game_move: move
    })
  });

  if (!response.ok) {
    throw await consumeError(response, "makeSpecialMove");
  }

  const data = await response.json();
  return data.new_board_fen;
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
  const body = {
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
    throw await consumeError(response, "makeMove");
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
initBoardResize({
  sliderSelector: "#board-size-slider",
  valueLabelSelector: "#board-size-value",
});

// Make the "Edit this position" link forward the current FEN to the editor.
const editorLink = document.getElementById("open-editor-link") as HTMLAnchorElement | null;
if (editorLink) {
  editorLink.addEventListener("click", (ev) => {
    ev.preventDefault();
    const fen = (document.getElementById("fen-input") as HTMLInputElement).value;
    const url = `/editor.html?fen=${encodeURIComponent(fen)}`;
    window.location.href = url;
  });
}

// Auto-load standard chess position so the board isn't empty on first paint
const DEFAULT_FEN = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR";
(document.getElementById("fen-input") as HTMLInputElement).value = DEFAULT_FEN;
renderBoard(DEFAULT_FEN);
