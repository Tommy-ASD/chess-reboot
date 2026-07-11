// src/fen_page.ts
//
// Entry point for the read-only FEN renderer (fen.html). It draws a
// position and nothing more: paste or pick a FEN, see the board. No move
// fetching, no backend calls — the visual pass is shared with the play
// surface via `board_render`, so what you see here is exactly what you'd
// see mid-game.
//
// Two hand-off links carry the current FEN onward: "Play this position"
// (play.html) and "Edit this position" (editor.html), both via `?fen=`.

import { initBoardResize } from "./board_size";
import { renderBoardInto } from "./board_render";
import { FEN_PRESETS } from "./presets";
import { boardOrientation, setBoardOrientation } from "./variables";

const boardEl = document.getElementById("board")!;
const fenInput = document.getElementById("fen-input") as HTMLInputElement;

/// Draw the current FEN. Read-only: `renderBoardInto` with no click
/// handler leaves the squares inert (and tags the board `.readonly`).
function render(fen: string) {
  try {
    renderBoardInto(boardEl, fen);
  } catch {
    // Mid-edit FENs are frequently unparseable (a half-typed grid); keep
    // the last good board rather than blanking it on every keystroke.
  }
}

// --- FEN input: explicit Render + live preview on every keystroke ---

document.getElementById("load-btn")!.addEventListener("click", () => render(fenInput.value));
fenInput.addEventListener("input", () => render(fenInput.value));

// --- Presets (shared catalogue with the play page) ---

function populateFENList() {
  const list = document.getElementById("fen-list")!;
  list.innerHTML = "";
  for (const { name, fen } of FEN_PRESETS) {
    const li = document.createElement("li");
    li.textContent = name;
    li.onclick = () => {
      fenInput.value = fen;
      render(fen);
    };
    list.appendChild(li);
  }
}

// --- Flip (orientation is the shared board-wide switch) ---

document.getElementById("flip-local-btn")!.addEventListener("click", () => {
  setBoardOrientation(boardOrientation === "white" ? "black" : "white");
  render(fenInput.value);
});

// --- Hand-offs: forward the current FEN to Play / Edit via `?fen=` ---

function wireHandoff(id: string, page: string) {
  const link = document.getElementById(id) as HTMLAnchorElement | null;
  if (!link) return;
  link.addEventListener("click", (ev) => {
    ev.preventDefault();
    window.location.href = `${page}?fen=${encodeURIComponent(fenInput.value)}`;
  });
}

wireHandoff("play-from-here", "/play.html");
wireHandoff("open-editor-link", "/editor.html");

// --- Boot ---

populateFENList();
initBoardResize({
  sliderSelector: "#board-size-slider",
  valueLabelSelector: "#board-size-value",
});

// Initial position: honour `?fen=` (a hand-off from Play / Editor), else
// draw the standard start so the board isn't blank on first paint.
const DEFAULT_FEN = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR";
const initialFen = new URLSearchParams(window.location.search).get("fen") ?? DEFAULT_FEN;
fenInput.value = initialFen;
render(initialFen);
