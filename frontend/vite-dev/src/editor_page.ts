// src/editor_page.ts
//
// Entry point for the dedicated board-editor page (editor.html).
// Manages a brush-based painting model: pick one brush from the palette,
// click squares to apply it. State (board grid + flags) round-trips via
// a single FEN string — copy/paste/edit and everything stays in sync.

import { initBoardResize, setBoardDimensions } from "./board_size";
import {
  DEFAULT_FLAGS,
  parseFEN,
  parseFENFlags,
  pieceToImage,
  pieceToSymbol,
  serializeFullFEN,
  type BoardFlags,
  type Side,
} from "./fen";
import type { Square, SquareType } from "./variables";

// ---------------------------
// Brush model
// ---------------------------

type Brush =
  | { kind: "piece"; piece: string }
  | { kind: "type"; squareType: SquareType }
  | { kind: "condition"; condition: string }
  | { kind: "erase-piece" }
  | { kind: "erase-conditions" }
  | { kind: "erase-type" }
  | { kind: "erase-all" };

let activeBrush: Brush | null = null;
let activeBrushButton: HTMLButtonElement | null = null;

const STARTING_FEN = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq -";
const EMPTY_FEN = "8/8/8/8/8/8/8/8 w - -";

let board: Square[][] = parseFEN(EMPTY_FEN);
let flags: BoardFlags = { ...DEFAULT_FLAGS };

// ---------------------------
// Palette definitions
// ---------------------------

const WHITE_PIECES = ["K", "Q", "R", "B", "N", "P"];
const BLACK_PIECES = ["k", "q", "r", "b", "n", "p"];
const CUSTOM_PIECES = ["G", "g", "BUS", "bus"];
const SQUARE_TYPES: SquareType[] = ["STANDARD", "VENT", "TURRET"];
const CONDITIONS = ["FROZEN", "BRAINROT"];

const PRESETS: { name: string; fen: string }[] = [
  { name: "Empty board", fen: EMPTY_FEN },
  { name: "Standard chess", fen: STARTING_FEN },
  { name: "Goblin test", fen: "(P=g(H=0-0))nbqkbn(P=g(H=7-0))/pppppppp/8/8/8/8/PPPPPPPP/(P=G(H=0-7))NBQKBN(P=G(H=7-7)) w KQkq -" },
  { name: "Vent test", fen: "(T=VENT)7/8/8/8/8/8/8/8 w - -" },
  { name: "Frozen test", fen: "(C=FROZEN)7/8/8/8/8/8/8/8 w - -" },
];

// ---------------------------
// Helpers
// ---------------------------

function $(id: string): HTMLElement {
  const el = document.getElementById(id);
  if (!el) throw new Error(`Missing element: #${id}`);
  return el;
}

function describeBrush(b: Brush): string {
  switch (b.kind) {
    case "piece":          return `Place: ${pieceToSymbol(b.piece)} (${b.piece})`;
    case "type":           return `Square type: ${b.squareType}`;
    case "condition":      return `Toggle condition: ${b.condition}`;
    case "erase-piece":    return "Erase piece";
    case "erase-conditions": return "Erase conditions";
    case "erase-type":     return "Reset square type";
    case "erase-all":      return "Erase everything on square";
  }
}

function setActiveBrush(brush: Brush, btn: HTMLButtonElement) {
  activeBrush = brush;
  if (activeBrushButton) activeBrushButton.classList.remove("active");
  activeBrushButton = btn;
  btn.classList.add("active");
  $("active-brush").textContent = describeBrush(brush);
}

// ---------------------------
// Apply brush to a square
// ---------------------------

function applyBrush(rank: number, file: number) {
  if (!activeBrush) return;
  const sq = board[rank][file];

  switch (activeBrush.kind) {
    case "piece":
      sq.piece = activeBrush.piece;
      break;
    case "type":
      sq.squareType = activeBrush.squareType;
      break;
    case "condition": {
      const c = activeBrush.condition;
      sq.conditions = sq.conditions.includes(c)
        ? sq.conditions.filter(x => x !== c)
        : [...sq.conditions, c];
      break;
    }
    case "erase-piece":
      sq.piece = null;
      break;
    case "erase-conditions":
      sq.conditions = [];
      break;
    case "erase-type":
      sq.squareType = "STANDARD";
      break;
    case "erase-all":
      sq.piece = null;
      sq.conditions = [];
      sq.squareType = "STANDARD";
      break;
  }

  syncFromState();
}

// ---------------------------
// Render
// ---------------------------

function renderBoard() {
  const boardEl = $("board");
  boardEl.innerHTML = "";

  const rows = board.length;
  const cols = board[0]?.length ?? 0;
  setBoardDimensions(cols, rows);

  for (let rank = 0; rank < rows; rank++) {
    for (let file = 0; file < cols; file++) {
      const sq = board[rank][file];
      const cell = document.createElement("div");
      cell.classList.add("square");
      cell.classList.add((rank + file) % 2 === 1 ? "dark" : "light");

      if (sq.piece) {
        const imgPath = pieceToImage(sq.piece);
        if (imgPath) {
          const img = document.createElement("img");
          img.src = imgPath;
          img.alt = sq.piece;
          img.classList.add("piece-image");
          cell.appendChild(img);
        } else {
          cell.textContent = pieceToSymbol(sq.piece);
        }
      }
      if (sq.conditions.includes("FROZEN")) cell.classList.add("cond-frozen");
      if (sq.conditions.includes("BRAINROT")) cell.classList.add("cond-brainrot");
      if (sq.squareType === "VENT") cell.classList.add("type-vent");

      cell.onclick = () => applyBrush(rank, file);
      boardEl.appendChild(cell);
    }
  }
}

/// Push the in-memory state out to the FEN input + flag controls,
/// then re-render the board. Single source of truth: `board` + `flags`.
function syncFromState() {
  (document.getElementById("fen-input") as HTMLInputElement).value =
    serializeFullFEN(board, flags);
  syncFlagsToControls();
  renderBoard();
}

function syncFlagsToControls() {
  for (const btn of document.querySelectorAll("#stm-toggle button")) {
    btn.classList.toggle("active",
      (btn as HTMLButtonElement).dataset.color === flags.sideToMove);
  }
  (document.getElementById("castle-K") as HTMLInputElement).checked = flags.castling.K;
  (document.getElementById("castle-Q") as HTMLInputElement).checked = flags.castling.Q;
  (document.getElementById("castle-k") as HTMLInputElement).checked = flags.castling.k;
  (document.getElementById("castle-q") as HTMLInputElement).checked = flags.castling.q;
  (document.getElementById("ep-target") as HTMLInputElement).value = flags.enPassant ?? "";
  (document.getElementById("board-cols") as HTMLInputElement).value = String(board[0]?.length ?? 0);
  (document.getElementById("board-rows") as HTMLInputElement).value = String(board.length);
}

/// Resize the board grid to the requested dimensions, preserving any
/// pieces / square types / conditions in the overlap region. Cells
/// outside the new bounds are dropped; new cells are blank standard
/// squares. Anchored to the top-left so coordinates stay stable in the
/// common "grow" direction.
function resizeBoard(newCols: number, newRows: number) {
  const cols = Math.max(1, Math.round(newCols));
  const rows = Math.max(1, Math.round(newRows));
  const next: Square[][] = [];
  for (let r = 0; r < rows; r++) {
    const row: Square[] = [];
    for (let c = 0; c < cols; c++) {
      const existing = board[r]?.[c];
      row.push(existing ?? {
        piece: null,
        squareType: "STANDARD",
        conditions: [],
      });
    }
    next.push(row);
  }
  board = next;
  syncFromState();
}

/// Pull state from the FEN input. Used when the user types or pastes.
function loadFEN(fen: string) {
  try {
    board = parseFEN(fen);
    flags = parseFENFlags(fen);
    renderBoard();
    syncFlagsToControls();
  } catch (e) {
    console.warn("Invalid FEN ignored:", e);
  }
}

// ---------------------------
// Build palettes
// ---------------------------

function makeBrushButton(label: string, brush: Brush, opts: { glyph?: string; img?: string } = {}): HTMLButtonElement {
  const btn = document.createElement("button");
  btn.className = "brush-btn";
  if (opts.img) {
    const img = document.createElement("img");
    img.src = opts.img;
    img.className = "brush-img";
    btn.appendChild(img);
  } else {
    const span = document.createElement("span");
    span.className = "brush-glyph";
    span.textContent = opts.glyph ?? label;
    btn.appendChild(span);
  }
  const cap = document.createElement("span");
  cap.className = "brush-caption";
  cap.textContent = label;
  btn.appendChild(cap);
  btn.onclick = () => setActiveBrush(brush, btn);
  return btn;
}

function buildPiecePalette(containerId: string, pieces: string[]) {
  const div = $(containerId);
  div.innerHTML = "";
  for (const p of pieces) {
    const img = pieceToImage(p);
    div.appendChild(makeBrushButton(p, { kind: "piece", piece: p }, {
      glyph: pieceToSymbol(p),
      img: img,
    }));
  }
}

function buildTypePalette() {
  const div = $("palette-types");
  div.innerHTML = "";
  for (const t of SQUARE_TYPES) {
    div.appendChild(makeBrushButton(t, { kind: "type", squareType: t }, { glyph: t[0] }));
  }
}

function buildConditionPalette() {
  const div = $("palette-conditions");
  div.innerHTML = "";
  for (const c of CONDITIONS) {
    div.appendChild(makeBrushButton(c, { kind: "condition", condition: c }, { glyph: c[0] }));
  }
}

function buildErasePalette() {
  const div = $("palette-erase");
  div.innerHTML = "";
  div.appendChild(makeBrushButton("Piece", { kind: "erase-piece" }, { glyph: "✕" }));
  div.appendChild(makeBrushButton("Conditions", { kind: "erase-conditions" }, { glyph: "✕" }));
  div.appendChild(makeBrushButton("Type", { kind: "erase-type" }, { glyph: "✕" }));
  div.appendChild(makeBrushButton("All", { kind: "erase-all" }, { glyph: "✖" }));
}

function buildPresets() {
  const ul = $("presets");
  ul.innerHTML = "";
  for (const { name, fen } of PRESETS) {
    const li = document.createElement("li");
    li.textContent = name;
    li.onclick = () => {
      (document.getElementById("fen-input") as HTMLInputElement).value = fen;
      loadFEN(fen);
    };
    ul.appendChild(li);
  }
}

// ---------------------------
// Wire UI
// ---------------------------

function wireFENInput() {
  const input = document.getElementById("fen-input") as HTMLInputElement;
  input.addEventListener("input", () => loadFEN(input.value));
}

function wireDimensionControls() {
  const cols = document.getElementById("board-cols") as HTMLInputElement;
  const rows = document.getElementById("board-rows") as HTMLInputElement;
  const apply = () => resizeBoard(Number(cols.value), Number(rows.value));
  cols.addEventListener("change", apply);
  rows.addEventListener("change", apply);
}

function wireFlagControls() {
  for (const btn of document.querySelectorAll<HTMLButtonElement>("#stm-toggle button")) {
    btn.onclick = () => {
      flags.sideToMove = (btn.dataset.color as Side) ?? "w";
      syncFromState();
    };
  }
  for (const key of ["K", "Q", "k", "q"] as const) {
    const cb = document.getElementById(`castle-${key}`) as HTMLInputElement;
    cb.addEventListener("change", () => {
      flags.castling[key] = cb.checked;
      syncFromState();
    });
  }
  const ep = document.getElementById("ep-target") as HTMLInputElement;
  ep.addEventListener("input", () => {
    const v = ep.value.trim();
    flags.enPassant = v === "" || v === "-" ? null : v;
    (document.getElementById("fen-input") as HTMLInputElement).value =
      serializeFullFEN(board, flags);
  });
}

function wireTopButtons() {
  document.getElementById("copy-fen")!.addEventListener("click", async () => {
    const fen = (document.getElementById("fen-input") as HTMLInputElement).value;
    try {
      await navigator.clipboard.writeText(fen);
      flashButton("copy-fen", "Copied!");
    } catch {
      flashButton("copy-fen", "Failed");
    }
  });

  document.getElementById("paste-fen")!.addEventListener("click", async () => {
    try {
      const text = await navigator.clipboard.readText();
      (document.getElementById("fen-input") as HTMLInputElement).value = text;
      loadFEN(text);
    } catch {
      flashButton("paste-fen", "Blocked");
    }
  });

  document.getElementById("reset-standard")!.addEventListener("click", () => {
    (document.getElementById("fen-input") as HTMLInputElement).value = STARTING_FEN;
    loadFEN(STARTING_FEN);
  });

  document.getElementById("clear-board")!.addEventListener("click", () => {
    (document.getElementById("fen-input") as HTMLInputElement).value = EMPTY_FEN;
    loadFEN(EMPTY_FEN);
  });
}

function flashButton(id: string, text: string) {
  const btn = document.getElementById(id) as HTMLButtonElement;
  const original = btn.textContent;
  btn.textContent = text;
  setTimeout(() => { btn.textContent = original; }, 900);
}

// ---------------------------
// Boot
// ---------------------------

initBoardResize({
  sliderSelector: "#board-size-slider",
  valueLabelSelector: "#board-size-value",
});

buildPiecePalette("palette-white", WHITE_PIECES);
buildPiecePalette("palette-black", BLACK_PIECES);
buildPiecePalette("palette-custom", CUSTOM_PIECES);
buildTypePalette();
buildConditionPalette();
buildErasePalette();
buildPresets();
wireFENInput();
wireFlagControls();
wireDimensionControls();
wireTopButtons();

// Honour ?fen=... so the play page can hand off the current position.
const urlFen = new URLSearchParams(window.location.search).get("fen");
const initialFen = urlFen ?? STARTING_FEN;
(document.getElementById("fen-input") as HTMLInputElement).value = initialFen;
loadFEN(initialFen);
