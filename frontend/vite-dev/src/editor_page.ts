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
import { squareIconSvg, squareTypeIconByType } from "./signal_icons";
import {
  ALL_PRESSURE_TRIGGERS,
  ALL_TRACK_DIRS,
  getEmitterTargets,
  getGateId,
  getGateOpen,
  getJunctionBranches,
  getJunctionId,
  getJunctionState,
  getPlateTrigger,
  getReceiverId,
  isEmitter,
  isReceiver,
  nextSignalId,
  setEmitterTargets,
  setGateId,
  setGateOpen,
  setJunctionBranches,
  setJunctionId,
  setJunctionState,
  setPlateTrigger,
  toggleEmitterTarget,
  type PressureTrigger,
} from "./signal_payload";
import type { Coord, Square, SquareType } from "./variables";

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
  | { kind: "erase-all" }
  /// Plan-08 inspect/wire brush. Clicking a substrate square shows its
  /// details panel; clicking an emitter then clicking receivers toggles
  /// them in/out of the emitter's targets.
  | { kind: "inspect" };

let activeBrush: Brush | null = null;
let activeBrushButton: HTMLButtonElement | null = null;

/// Coord currently under inspection (rendered with `.selected-inspect`).
/// Null = nothing selected. When set to an emitter, the renderer also
/// rings its wired receivers; clicking other receivers toggles them.
let inspectedSquare: Coord | null = null;

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
const SQUARE_TYPES: SquareType[] = [
  "STANDARD",
  "VENT",
  "TURRET",
  "SWITCH",
  "JUNCTION",
  "GATE",
  "PLATE",
];

/// Lower-cased class suffix per square type so render code can do
/// `cell.classList.add(`type-${TYPE_CLASS[sq.squareType]}`)` without a
/// switch each time. Standard is excluded — it has no class.
const TYPE_CLASS: Record<Exclude<SquareType, "STANDARD">, string> = {
  VENT: "vent",
  TURRET: "turret",
  SWITCH: "switch",
  JUNCTION: "junction",
  GATE: "gate",
  PLATE: "plate",
};
const CONDITIONS = ["FROZEN", "BRAINROT"];

const PRESETS: { name: string; fen: string }[] = [
  { name: "Empty board", fen: EMPTY_FEN },
  { name: "Standard chess", fen: STARTING_FEN },
  { name: "Goblin test", fen: "(P=g(H=0-0))nbqkbn(P=g(H=7-0))/pppppppp/8/8/8/8/PPPPPPPP/(P=G(H=0-7))NBQKBN(P=G(H=7-7)) w KQkq -" },
  { name: "Vent test", fen: "(T=VENT)7/8/8/8/8/8/8/8 w - -" },
  { name: "Frozen test", fen: "(C=FROZEN)7/8/8/8/8/8/8/8 w - -" },
  {
    name: "Signal substrate test",
    fen: "(T=SWITCH,TARGETS=(3,7))(T=JUNCTION,ID=3,STATE=0,BRANCHES=(N,E))(T=GATE,ID=7,OPEN=0)(T=PLATE,TARGETS=(3),FIRES=B)4/8/8/8/8/8/8/8 w - -",
  },
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
    case "inspect":        return "Inspect / wire signal squares";
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
      delete sq.extraFields;
      // Auto-populate payload defaults so freshly-painted substrate
      // squares are immediately well-formed and the user only has to
      // tweak what they want to change.
      if (activeBrush.squareType === "JUNCTION") {
        setJunctionId(sq, nextSignalId(board));
        setJunctionState(sq, 0);
        // Default to a 4-way junction so the SVG renders something useful;
        // the user can prune via the details panel.
        setJunctionBranches(sq, [...ALL_TRACK_DIRS]);
      } else if (activeBrush.squareType === "GATE") {
        setGateId(sq, nextSignalId(board));
        setGateOpen(sq, true);
      } else if (activeBrush.squareType === "SWITCH") {
        setEmitterTargets(sq, []);
      } else if (activeBrush.squareType === "PLATE") {
        setEmitterTargets(sq, []);
        setPlateTrigger(sq, "ANY");
      }
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
      delete sq.extraFields;
      break;
    case "erase-all":
      sq.piece = null;
      sq.conditions = [];
      sq.squareType = "STANDARD";
      delete sq.extraFields;
      break;
    case "inspect": {
      // Two-mode: if an emitter is currently inspected AND the user
      // clicks a receiver, toggle the receiver's ID in the emitter's
      // targets. Otherwise, just inspect the clicked square.
      const inspected = inspectedSquare
        ? board[inspectedSquare.rank]?.[inspectedSquare.file] ?? null
        : null;
      if (inspected && isEmitter(inspected) && isReceiver(sq)) {
        const rid = getReceiverId(sq);
        if (rid !== null) {
          toggleEmitterTarget(inspected, rid);
        }
      } else {
        inspectedSquare = { file, rank };
      }
      break;
    }
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
      if (sq.squareType !== "STANDARD") {
        cell.classList.add(`type-${TYPE_CLASS[sq.squareType]}`);
        const svg = squareIconSvg(sq);
        if (svg) {
          const iconWrap = document.createElement("div");
          iconWrap.className = "square-icon";
          iconWrap.innerHTML = svg;
          cell.appendChild(iconWrap);
        }
      }

      // Inspect-mode highlights: the currently-inspected square gets a
      // ring; if it's an emitter, every receiver whose ID is in its
      // targets gets a "wired" ring.
      if (inspectedSquare && inspectedSquare.rank === rank && inspectedSquare.file === file) {
        cell.classList.add("selected-inspect");
      } else if (inspectedSquare) {
        const insp = board[inspectedSquare.rank]?.[inspectedSquare.file];
        if (insp && isEmitter(insp) && isReceiver(sq)) {
          const targets = getEmitterTargets(insp);
          const rid = getReceiverId(sq);
          if (rid !== null && targets.includes(rid)) {
            cell.classList.add("wired-target");
          }
        }
      }

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
  renderInspector();
  renderWiringOverlay();
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
    inspectedSquare = null;
    renderBoard();
    syncFlagsToControls();
    renderInspector();
    renderWiringOverlay();
  } catch (e) {
    console.warn("Invalid FEN ignored:", e);
  }
}

// ---------------------------
// Inspector panel (substrate payload editor)
// ---------------------------

/// Render the details panel for the currently-inspected square. If
/// nothing is inspected (or the inspected square is Standard / has no
/// payload), the panel shows a short usage hint.
function renderInspector() {
  const host = document.getElementById("inspector");
  if (!host) return;
  host.innerHTML = "";

  if (!inspectedSquare) {
    host.appendChild(hint(
      "Pick the Inspect tool, then click a Switch / Junction / Gate / Plate to edit it.",
    ));
    return;
  }
  const { rank, file } = inspectedSquare;
  const sq = board[rank]?.[file];
  if (!sq) return;

  const coordLabel = document.createElement("div");
  coordLabel.className = "inspector-coord";
  coordLabel.textContent = `(${file}, ${rank}) — ${sq.squareType}`;
  host.appendChild(coordLabel);

  switch (sq.squareType) {
    case "SWITCH":
      host.appendChild(renderTargetsEditor(sq, "SWITCH"));
      host.appendChild(hint(
        "With this Switch selected, click any Junction or Gate to wire / un-wire it.",
      ));
      break;
    case "JUNCTION":
      host.appendChild(renderJunctionEditor(sq));
      break;
    case "GATE":
      host.appendChild(renderGateEditor(sq));
      break;
    case "PLATE":
      host.appendChild(renderTargetsEditor(sq, "PLATE"));
      host.appendChild(renderPlateTriggerEditor(sq));
      host.appendChild(hint(
        "With this Plate selected, click any Junction or Gate to wire / un-wire it.",
      ));
      break;
    default:
      host.appendChild(hint(`${sq.squareType} squares have no editable payload.`));
  }
}

function hint(text: string): HTMLElement {
  const p = document.createElement("p");
  p.className = "inspector-hint";
  p.textContent = text;
  return p;
}

function renderTargetsEditor(sq: Square, kind: "SWITCH" | "PLATE"): HTMLElement {
  const wrap = document.createElement("div");
  wrap.className = "inspector-section";
  const label = document.createElement("label");
  label.textContent = "Targets (signal IDs):";
  wrap.appendChild(label);
  const input = document.createElement("input");
  input.type = "text";
  input.placeholder = "e.g. 1,3,7";
  input.value = getEmitterTargets(sq).join(",");
  input.addEventListener("change", () => {
    const ids = input.value
      .split(",")
      .map(x => Number.parseInt(x.trim(), 10))
      .filter(n => Number.isFinite(n) && n >= 0);
    setEmitterTargets(sq, ids);
    syncFromState();
  });
  wrap.appendChild(input);
  void kind; // both kinds use the same TARGETS shape
  return wrap;
}

function renderJunctionEditor(sq: Square): HTMLElement {
  const wrap = document.createElement("div");
  wrap.className = "inspector-section";

  wrap.appendChild(numericField("ID", getJunctionId(sq), v => {
    setJunctionId(sq, v);
    syncFromState();
  }));
  wrap.appendChild(numericField("State", getJunctionState(sq), v => {
    setJunctionState(sq, v);
    syncFromState();
  }));

  const branchesLabel = document.createElement("div");
  branchesLabel.className = "inspector-sublabel";
  branchesLabel.textContent = "Branches:";
  wrap.appendChild(branchesLabel);

  const current = getJunctionBranches(sq);
  const branchRow = document.createElement("div");
  branchRow.className = "inspector-branches";
  for (const dir of ALL_TRACK_DIRS) {
    const btn = document.createElement("button");
    btn.className = "branch-toggle";
    btn.textContent = dir;
    if (current.includes(dir)) btn.classList.add("on");
    btn.onclick = () => {
      const next = getJunctionBranches(sq);
      const idx = next.indexOf(dir);
      if (idx === -1) next.push(dir);
      else next.splice(idx, 1);
      setJunctionBranches(sq, next);
      syncFromState();
    };
    branchRow.appendChild(btn);
  }
  wrap.appendChild(branchRow);

  return wrap;
}

function renderGateEditor(sq: Square): HTMLElement {
  const wrap = document.createElement("div");
  wrap.className = "inspector-section";

  wrap.appendChild(numericField("ID", getGateId(sq), v => {
    setGateId(sq, v);
    syncFromState();
  }));

  const row = document.createElement("label");
  row.className = "inspector-toggle";
  const cb = document.createElement("input");
  cb.type = "checkbox";
  cb.checked = getGateOpen(sq);
  cb.addEventListener("change", () => {
    setGateOpen(sq, cb.checked);
    syncFromState();
  });
  row.appendChild(cb);
  row.append(" Open");
  wrap.appendChild(row);

  return wrap;
}

function renderPlateTriggerEditor(sq: Square): HTMLElement {
  const wrap = document.createElement("div");
  wrap.className = "inspector-section";
  const label = document.createElement("div");
  label.className = "inspector-sublabel";
  label.textContent = "Fires for:";
  wrap.appendChild(label);

  const current = getPlateTrigger(sq);
  const row = document.createElement("div");
  row.className = "inspector-branches";
  for (const trig of ALL_PRESSURE_TRIGGERS) {
    const btn = document.createElement("button");
    btn.className = "branch-toggle";
    btn.textContent = triggerLabel(trig);
    if (trig === current) btn.classList.add("on");
    btn.onclick = () => {
      setPlateTrigger(sq, trig);
      syncFromState();
    };
    row.appendChild(btn);
  }
  wrap.appendChild(row);
  return wrap;
}

function triggerLabel(trig: PressureTrigger): string {
  switch (trig) {
    case "ANY": return "Any";
    case "W":   return "White";
    case "B":   return "Black";
  }
}

function numericField(label: string, value: number, onCommit: (v: number) => void): HTMLElement {
  const row = document.createElement("label");
  row.className = "inspector-field";
  const text = document.createElement("span");
  text.textContent = `${label}: `;
  row.appendChild(text);
  const input = document.createElement("input");
  input.type = "number";
  input.min = "0";
  input.value = String(value);
  input.addEventListener("change", () => {
    const n = Number.parseInt(input.value, 10);
    if (Number.isFinite(n) && n >= 0) onCommit(n);
  });
  row.appendChild(input);
  return row;
}

// ---------------------------
// Wiring overlay (SVG over board)
// ---------------------------

/// Redraw the SVG overlay that shows emitter→receiver wires. Only drawn
/// for the currently-inspected emitter (otherwise the board gets noisy).
function renderWiringOverlay() {
  const overlay = document.getElementById("wiring-overlay") as SVGSVGElement | null;
  if (!overlay) return;
  while (overlay.firstChild) overlay.removeChild(overlay.firstChild);

  if (!inspectedSquare) return;
  const insp = board[inspectedSquare.rank]?.[inspectedSquare.file];
  if (!insp || !isEmitter(insp)) return;

  const cells = document.querySelectorAll<HTMLElement>("#board .square");
  const cols = board[0]?.length ?? 0;
  if (cols === 0) return;
  const boardRect = (document.getElementById("board") as HTMLElement).getBoundingClientRect();

  // viewBox = pixel coords relative to the board origin.
  overlay.setAttribute("viewBox", `0 0 ${boardRect.width} ${boardRect.height}`);
  overlay.style.width = `${boardRect.width}px`;
  overlay.style.height = `${boardRect.height}px`;

  const srcIdx = inspectedSquare.rank * cols + inspectedSquare.file;
  const srcCenter = cellCenter(cells[srcIdx], boardRect);

  const targets = getEmitterTargets(insp);
  for (let r = 0; r < board.length; r++) {
    for (let c = 0; c < cols; c++) {
      const sq = board[r][c];
      if (!isReceiver(sq)) continue;
      const rid = getReceiverId(sq);
      if (rid === null || !targets.includes(rid)) continue;
      const destIdx = r * cols + c;
      const destCenter = cellCenter(cells[destIdx], boardRect);
      const line = document.createElementNS("http://www.w3.org/2000/svg", "line");
      line.setAttribute("x1", String(srcCenter.x));
      line.setAttribute("y1", String(srcCenter.y));
      line.setAttribute("x2", String(destCenter.x));
      line.setAttribute("y2", String(destCenter.y));
      overlay.appendChild(line);
      const dot = document.createElementNS("http://www.w3.org/2000/svg", "circle");
      dot.setAttribute("cx", String(destCenter.x));
      dot.setAttribute("cy", String(destCenter.y));
      dot.setAttribute("r", "4");
      overlay.appendChild(dot);
    }
  }
}

function cellCenter(cell: HTMLElement | undefined, boardRect: DOMRect): { x: number; y: number } {
  if (!cell) return { x: 0, y: 0 };
  const r = cell.getBoundingClientRect();
  return {
    x: r.left - boardRect.left + r.width / 2,
    y: r.top - boardRect.top + r.height / 2,
  };
}

// ---------------------------
// Build palettes
// ---------------------------

function makeBrushButton(
  label: string,
  brush: Brush,
  opts: { glyph?: string; img?: string; svg?: string; accent?: string } = {},
): HTMLButtonElement {
  const btn = document.createElement("button");
  btn.className = "brush-btn";
  if (opts.accent) btn.style.setProperty("--brush-accent", opts.accent);
  if (opts.img) {
    const img = document.createElement("img");
    img.src = opts.img;
    img.className = "brush-img";
    btn.appendChild(img);
  } else if (opts.svg) {
    const wrap = document.createElement("span");
    wrap.className = "brush-svg";
    wrap.innerHTML = opts.svg;
    btn.appendChild(wrap);
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

/// Accent color per substrate type — kept in lockstep with the matching
/// `.type-*` rules in style.css so the palette previews match the board.
const TYPE_ACCENT: Record<SquareType, string> = {
  STANDARD: "rgba(180, 180, 180, 0.7)",
  TURRET:   "rgba(180, 90, 30, 0.95)",
  VENT:     "rgba(60, 60, 60, 0.85)",
  SWITCH:   "rgba(255, 196, 0, 0.95)",
  JUNCTION: "rgba(120, 200, 255, 0.95)",
  GATE:     "rgba(220, 80, 80, 0.95)",
  PLATE:    "rgba(170, 130, 255, 0.95)",
};

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
    const svg = squareTypeIconByType(t) ?? undefined;
    div.appendChild(makeBrushButton(t, { kind: "type", squareType: t }, {
      glyph: t[0],
      svg,
      accent: TYPE_ACCENT[t],
    }));
  }
  // Inspect / wiring tool: not a square-type brush but lives next to the
  // type palette since it's about substrate squares.
  div.appendChild(makeBrushButton("INSPECT", { kind: "inspect" }, {
    glyph: "?",
    accent: "rgba(255, 255, 255, 0.95)",
  }));
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
