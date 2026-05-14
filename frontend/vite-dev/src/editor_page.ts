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
import { renderCarrierPassengerOverlay } from "./passenger_overlay";
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
  getTrackDir,
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
  setTrackDir,
  toggleEmitterTarget,
  type PressureTrigger,
  type TrackDir,
} from "./signal_payload";
import {
  ALL_TRAIN_HEADINGS,
  highestChainIndex,
  firstOrphanTrainId,
  highestTrainId,
  isTrainCart,
  parseTrainCart,
  serializeTrainCart,
  trainCartRotationDegrees,
  type TrainCart,
} from "./train_payload";
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

const STARTING_FEN = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - tr=full p=0";
const EMPTY_FEN = "8/8/8/8/8/8/8/8 w - - tr=full p=0";

let board: Square[][] = parseFEN(EMPTY_FEN);
let flags: BoardFlags = { ...DEFAULT_FLAGS };

// ---------------------------
// Palette definitions
// ---------------------------

const WHITE_PIECES = ["K", "Q", "R", "B", "N", "P"];
const BLACK_PIECES = ["k", "q", "r", "b", "n", "p"];
/// Train brushes carry minimal payload — the inspector tunes the rest.
/// `train_id=1` keeps successive placements in the same train by default;
/// pick a different ID in the inspector for a separate train.
const CUSTOM_PIECES = ["G", "g", "BUS", "bus", "LOCO(ID=1,H=F)", "CART(ID=1,I=1)"];
const SQUARE_TYPES: SquareType[] = [
  "STANDARD",
  "VENT",
  "TURRET",
  "BLOCK",
  "SWITCH",
  "JUNCTION",
  "GATE",
  "PLATE",
  "TRACK",
];

/// Lower-cased class suffix per square type so render code can do
/// `cell.classList.add(`type-${TYPE_CLASS[sq.squareType]}`)` without a
/// switch each time. Standard is excluded — it has no class.
const TYPE_CLASS: Record<Exclude<SquareType, "STANDARD">, string> = {
  VENT: "vent",
  TURRET: "turret",
  BLOCK: "block",
  SWITCH: "switch",
  JUNCTION: "junction",
  GATE: "gate",
  PLATE: "plate",
  TRACK: "track",
};
const CONDITIONS = ["FROZEN", "BRAINROT"];

const PRESETS: { name: string; fen: string }[] = [
  { name: "Empty board", fen: EMPTY_FEN },
  { name: "Standard chess", fen: STARTING_FEN },
  { name: "Goblin test", fen: "(P=g(H=0-0))nbqkbn(P=g(H=7-0))/pppppppp/8/8/8/8/PPPPPPPP/(P=G(H=0-7))NBQKBN(P=G(H=7-7)) w KQkq -" },
  { name: "Vent test", fen: "(T=VENT)7/8/8/8/8/8/8/8 w - -" },
  { name: "Block test", fen: "(T=BLOCK)7/8/8/8/8/8/8/8 w - -" },
  { name: "Frozen test", fen: "(C=FROZEN)7/8/8/8/8/8/8/8 w - -" },
  {
    name: "Signal substrate test",
    fen: "(T=SWITCH,TARGETS=(3,7))(T=JUNCTION,ID=3,STATE=0,BRANCHES=(N,E))(T=GATE,ID=7,OPEN=0)(T=PLATE,TARGETS=(3),FIRES=B)4/8/8/8/8/8/8/8 w - -",
  },
  {
    // Loco at the western end of a 5-tile east-pointing rail. Tiles
    // downstream of the loco's start tile don't need their D set —
    // neighbor-detection picks the right exit at each tick.
    name: "Train test (straight east)",
    fen: "8/8/8/8/(T=TRACK,D=E,P=LOCO(ID=1,H=F))(T=TRACK,D=E)(T=TRACK,D=E)(T=TRACK,D=E)(T=TRACK,D=E)3/8/8/8 w - - tr=ply p=0",
  },
  {
    // 4-tile loop in a 2×2 square. Loco starts on the NW tile facing
    // east; neighbor-detection bends the rails into a closed loop.
    // The cart trails one chain step behind the loco. Connection
    // graph: NW↔NE (top edge), NE↔SE (right edge), SE↔SW (bottom),
    // SW↔NW (left). Loco rolls E→S→W→N→E... and the cart follows.
    name: "Train test (loop + cart)",
    fen: "8/8/8/3(T=TRACK,D=E,P=LOCO(ID=1,H=F))(T=TRACK,D=E)3/3(T=TRACK,D=E,P=CART(ID=1,I=1))(T=TRACK,D=E)3/8/8/8 w - - tr=ply p=0",
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

/// Rewrite the LOCO/CART brush template so each placement allocates a
/// fresh train_id (LOCO) or the next chain_index for the most recent
/// train (CART). Non-train brushes pass through unchanged. Keeps the
/// common case "I want a new train" / "I want the next carriage"
/// one-click without dragging the user into the inspector.
function withSmartTrainPayload(piece: string): string {
  const cart = parseTrainCart(piece);
  if (!cart) return piece;
  if (cart.kind === "LOCO") {
    // Adopt any existing orphan train (carts placed before this LOCO)
    // so the user doesn't end up with a silently disconnected chain.
    // If no orphan exists, allocate a fresh ID.
    const orphan = firstOrphanTrainId(board);
    cart.trainId = orphan !== null ? orphan : highestTrainId(board) + 1;
  } else {
    // Attach to the highest train_id currently on the board (assumed
    // to be the loco the user just placed). If no loco exists yet,
    // fall back to train 1 so the carriage still has a sensible ID
    // — and warn the user. The LOCO branch above will adopt this
    // train_id when a matching LOCO is later placed, so the orphan
    // is recoverable.
    const max = highestTrainId(board);
    if (max === 0) {
      console.warn(
        "Placing a CART before any LOCO — assigning train_id=1. The next " +
          "LOCO you place will adopt this train automatically; until then " +
          "the carriage will be orphaned and won't move.",
      );
    }
    const target = Math.max(1, max);
    cart.trainId = target;
    cart.chainIndex = highestChainIndex(board, target) + 1;
  }
  return serializeTrainCart(cart);
}

/// When painting a Locomotive onto a Track tile, re-orient the tile's
/// `D` field to point away from same-train carriages. The engine uses
/// `D` on the first tick, and the default E often points at empty
/// space; this rule eliminates the most common "paint a chain and the
/// train won't move" surprise. Looks at the 4 cardinal neighbors and
/// picks a track exit that isn't already occupied by a cart of the
/// same train.
function pickLocoStartDir(
  boardGrid: Square[][],
  file: number,
  rank: number,
  trainId: number,
): TrackDir | null {
  const cardinal: { dir: TrackDir; df: number; dr: number }[] = [
    { dir: "N", df: 0, dr: -1 },
    { dir: "E", df: 1, dr: 0 },
    { dir: "S", df: 0, dr: 1 },
    { dir: "W", df: -1, dr: 0 },
  ];
  const behind: TrackDir[] = [];
  const candidates: TrackDir[] = [];
  for (const { dir, df, dr } of cardinal) {
    const sq = boardGrid[rank + dr]?.[file + df];
    if (!sq) continue;
    if (sq.squareType !== "TRACK" && sq.squareType !== "JUNCTION") continue;
    if (sq.piece) {
      const c = parseTrainCart(sq.piece);
      if (c && c.trainId === trainId) {
        behind.push(dir);
        continue;
      }
    }
    candidates.push(dir);
  }
  if (candidates.length === 0) return null;
  // If we have a "behind" direction, prefer its opposite — that's the
  // intuitive "forward" out of a chain.
  for (const b of behind) {
    const opposite: TrackDir =
      b === "N" ? "S" : b === "S" ? "N" : b === "E" ? "W" : "E";
    if (candidates.includes(opposite)) return opposite;
  }
  return candidates[0];
}

function applyBrush(rank: number, file: number) {
  if (!activeBrush) return;
  const sq = board[rank][file];

  switch (activeBrush.kind) {
    case "piece": {
      sq.piece = withSmartTrainPayload(activeBrush.piece);
      // Plan 09 polish: a freshly-painted LOCO on a Track tile gets
      // its tile's `D` re-oriented so the first tick moves the train
      // out of the chain, not into empty space. CART placement
      // doesn't touch D — carts inherit direction from the loco at
      // tick time, so their tile's D is engine-irrelevant.
      if (sq.piece && sq.squareType === "TRACK") {
        const placed = parseTrainCart(sq.piece);
        if (placed && placed.kind === "LOCO") {
          const newDir = pickLocoStartDir(board, file, rank, placed.trainId);
          if (newDir !== null) setTrackDir(sq, newDir);
        }
      }
      break;
    }
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
      } else if (activeBrush.squareType === "TRACK") {
        // East by default — the inspector lets the user rotate it.
        setTrackDir(sq, "E");
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
          if (isTrainCart(sq.piece)) {
            const deg = trainCartRotationDegrees(sq.piece, board, file, rank);
            if (deg !== 0) img.style.transform = `rotate(${deg}deg)`;
          }
          cell.appendChild(img);
        } else {
          cell.textContent = pieceToSymbol(sq.piece);
        }
        renderCarrierPassengerOverlay(cell, sq.piece);
      }
      if (sq.conditions.includes("FROZEN")) cell.classList.add("cond-frozen");
      if (sq.conditions.includes("BRAINROT")) cell.classList.add("cond-brainrot");
      if (sq.squareType !== "STANDARD") {
        cell.classList.add(`type-${TYPE_CLASS[sq.squareType]}`);
        const svg = squareIconSvg(sq, { board, file, rank });
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

  // Plan 09: train tick rate + ply count. The mode select switches the
  // discriminated union; only `EveryNPly` exposes the `n` input.
  const tickMode = document.getElementById("train-tick-mode") as HTMLSelectElement | null;
  const tickN = document.getElementById("train-tick-n") as HTMLInputElement | null;
  const ply = document.getElementById("ply-count") as HTMLInputElement | null;
  if (tickMode) tickMode.value = flags.trainTickRate.kind;
  if (tickN) {
    tickN.value = flags.trainTickRate.kind === "EveryNPly" ? String(flags.trainTickRate.n) : "";
    tickN.disabled = flags.trainTickRate.kind !== "EveryNPly";
  }
  if (ply) ply.value = String(flags.plyCount);
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
  // Clear the inspected coord if it's now outside the new bounds —
  // otherwise the inspector silently disappears (renderInspector
  // early-returns on missing square) while the user wonders why
  // their click doesn't show anything.
  if (
    inspectedSquare &&
    (inspectedSquare.rank >= rows || inspectedSquare.file >= cols)
  ) {
    inspectedSquare = null;
  }
  syncFromState();
}

/// Pull state from the FEN input. Used when the user types or pastes.
function loadFEN(fen: string) {
  try {
    board = parseFEN(fen);
    flags = parseFENFlags(fen);
    // Preserve `inspectedSquare` across re-parses — typing into the FEN
    // box (which fires `input` per keystroke) used to clear the
    // selection mid-edit. Drop it if (a) the parsed board no longer
    // covers the coord (smaller board parsed in), or (b) the inspected
    // tile is now a STANDARD square with no train cart on it — in
    // either case there's nothing left to inspect at that coord.
    if (inspectedSquare) {
      const inBounds =
        inspectedSquare.rank < board.length &&
        inspectedSquare.file < (board[0]?.length ?? 0);
      if (!inBounds) {
        inspectedSquare = null;
      } else {
        const sq = board[inspectedSquare.rank][inspectedSquare.file];
        const hasInspectable =
          sq.squareType !== "STANDARD" || (sq.piece && isTrainCart(sq.piece));
        if (!hasInspectable) {
          inspectedSquare = null;
        }
      }
    }
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
    case "TRACK":
      host.appendChild(renderTrackEditor(sq));
      break;
    default:
      if (!isTrainCart(sq.piece)) {
        host.appendChild(hint(`${sq.squareType} squares have no editable payload.`));
      }
  }

  // Train carts are *pieces*, not square types — so they can sit on a
  // TRACK tile alongside the square-payload editor. Append the cart
  // editor whenever the square holds a LOCO or CART.
  if (isTrainCart(sq.piece)) {
    host.appendChild(renderTrainCartEditor(sq));
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
    btn.title = triggerTooltip(trig);
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
    case "N":   return "Neutral";
  }
}

function triggerTooltip(trig: PressureTrigger): string {
  switch (trig) {
    case "ANY": return "Fires for any piece";
    case "W":   return "Fires only for white pieces";
    case "B":   return "Fires only for black pieces";
    case "N":   return "Fires only for train carts (Neutral)";
  }
}

function renderTrackEditor(sq: Square): HTMLElement {
  const wrap = document.createElement("div");
  wrap.className = "inspector-section";

  const label = document.createElement("div");
  label.className = "inspector-sublabel";
  label.textContent = "Direction (outgoing):";
  wrap.appendChild(label);

  const current = getTrackDir(sq);
  const row = document.createElement("div");
  row.className = "inspector-branches";
  for (const dir of ALL_TRACK_DIRS) {
    const btn = document.createElement("button");
    btn.className = "branch-toggle";
    btn.textContent = dir;
    if (dir === current) btn.classList.add("on");
    btn.onclick = () => {
      setTrackDir(sq, dir);
      syncFromState();
    };
    row.appendChild(btn);
  }
  wrap.appendChild(row);
  return wrap;
}

/// Inspector for the LOCO / CART piece sitting on the inspected square.
/// Reads the structured form via `parseTrainCart`, lets the user edit
/// the fields, and writes back through `serializeTrainCart` so the FEN
/// always sees the canonical engine-ordered shape.
function renderTrainCartEditor(sq: Square): HTMLElement {
  const wrap = document.createElement("div");
  wrap.className = "inspector-section";
  const cart = parseTrainCart(sq.piece ?? "");
  if (!cart) {
    wrap.appendChild(hint("(invalid train symbol)"));
    return wrap;
  }

  const heading = document.createElement("div");
  heading.className = "inspector-sublabel";
  heading.textContent = cart.kind === "LOCO" ? "Locomotive" : "Carriage";
  wrap.appendChild(heading);

  const commit = (mutate: (c: TrainCart) => void) => {
    mutate(cart);
    sq.piece = serializeTrainCart(cart);
    syncFromState();
  };

  wrap.appendChild(numericField("Train ID", cart.trainId, v => {
    commit(c => { c.trainId = v; });
  }));

  if (cart.kind === "LOCO") {
    const subLabel = document.createElement("div");
    subLabel.className = "inspector-sublabel";
    subLabel.textContent = "Heading (first tick only):";
    wrap.appendChild(subLabel);

    const row = document.createElement("div");
    row.className = "inspector-branches";
    for (const h of ALL_TRAIN_HEADINGS) {
      const btn = document.createElement("button");
      btn.className = "branch-toggle";
      btn.textContent = h === "F" ? "Forward" : "Reverse";
      if (h === cart.heading) btn.classList.add("on");
      btn.onclick = () => commit(c => { c.heading = h; });
      row.appendChild(btn);
    }
    wrap.appendChild(row);

    // Last-direction selector. After the first tick the engine uses
    // neighbor-detection: it exits through the side that isn't
    // `lastDir`. The "Reset" choice clears the field so the loco
    // bootstraps from the tile's `D` again. Useful for hand-tuning a
    // mid-game scenario where the loco is already partway through a
    // rail and the rail's stored `D` would send it the wrong way.
    const lastLabel = document.createElement("div");
    lastLabel.className = "inspector-sublabel";
    lastLabel.textContent = "Entered from (last_dir):";
    wrap.appendChild(lastLabel);

    const lastRow = document.createElement("div");
    lastRow.className = "inspector-branches";

    const resetBtn = document.createElement("button");
    resetBtn.className = "branch-toggle";
    resetBtn.textContent = "—";
    resetBtn.title = "Clear last_dir; engine uses the tile's D on the next tick.";
    if (cart.lastDir === null) resetBtn.classList.add("on");
    resetBtn.onclick = () => commit(c => { c.lastDir = null; });
    lastRow.appendChild(resetBtn);

    for (const d of ALL_TRACK_DIRS) {
      const btn = document.createElement("button");
      btn.className = "branch-toggle";
      btn.textContent = d;
      if (cart.lastDir === d) btn.classList.add("on");
      btn.onclick = () => commit(c => { c.lastDir = d; });
      lastRow.appendChild(btn);
    }
    wrap.appendChild(lastRow);
  } else {
    // Chain index 0 is reserved for the locomotive head — the engine
    // warns and rewrites I=0 to I=1 at parse time, so disallow it in
    // the UI for byte-identical FEN round-trip.
    wrap.appendChild(numericField("Chain index", cart.chainIndex, v => {
      commit(c => { c.chainIndex = v; });
    }, { min: 1 }));
  }

  // Passenger list editor — comma-separated symbols. Same shape as the
  // engine's `P=(...)` field. Empty input means "no passengers".
  // Unknown symbols are dropped with a visible warning rather than
  // silently — otherwise a typo round-trips through the engine
  // (`symbol_to_piece` returns `None`) and the passenger vanishes.
  const passLabel = document.createElement("div");
  passLabel.className = "inspector-sublabel";
  passLabel.textContent = "Passengers (symbols, comma-separated):";
  wrap.appendChild(passLabel);
  const passInput = document.createElement("input");
  passInput.type = "text";
  passInput.placeholder = "e.g. P,N";
  passInput.value = cart.passengers.join(",");
  const passHint = document.createElement("p");
  passHint.className = "inspector-hint";
  passInput.addEventListener("change", () => {
    const raw = passInput.value
      .split(",")
      .map(s => s.trim())
      .filter(s => s.length > 0);
    // Validate each — a passenger's leading prefix (before any
    // parens) must be a recognised piece kind that the engine's
    // `PieceType::symbol_to_piece` accepts. Nested carriers
    // (Bus/LOCO/CART as passengers) are also rejected: the engine
    // refuses them at FEN parse, and the move filter never produces
    // them, so accepting them here would diverge from the engine.
    //
    // Note: Monkey (M/m) is intentionally absent — the engine's
    // `symbol_to_piece` doesn't have a Monkey arm, so a Monkey
    // passenger would parse on the frontend but get silently
    // dropped on engine round-trip. Keep the validator's accept-set
    // in sync with the engine's parse-set.
    const known = new Set([
      "p", "r", "n", "b", "q", "k",
      "g", "s",
    ]);
    const customCarriers = new Set(["bus", "loco", "cart"]);
    const accepted: string[] = [];
    const rejected: string[] = [];
    for (const sym of raw) {
      const prefix = sym.split("(")[0].toLowerCase();
      if (customCarriers.has(prefix)) {
        rejected.push(`${sym} (nested carrier)`);
      } else if (known.has(prefix)) {
        accepted.push(sym);
      } else {
        rejected.push(sym);
      }
    }
    if (rejected.length > 0) {
      passHint.textContent = `Dropped: ${rejected.join(", ")}`;
      console.warn("Inspector dropped invalid passengers:", rejected);
    } else {
      passHint.textContent = "";
    }
    commit(c => { c.passengers = accepted; });
  });
  wrap.appendChild(passInput);
  wrap.appendChild(passHint);

  return wrap;
}


function numericField(
  label: string,
  value: number,
  onCommit: (v: number) => void,
  opts?: { min?: number },
): HTMLElement {
  const row = document.createElement("label");
  row.className = "inspector-field";
  const text = document.createElement("span");
  text.textContent = `${label}: `;
  row.appendChild(text);
  const input = document.createElement("input");
  input.type = "number";
  const min = opts?.min ?? 0;
  input.min = String(min);
  input.value = String(value);
  input.addEventListener("change", () => {
    const n = Number.parseInt(input.value, 10);
    if (Number.isFinite(n) && n >= min) onCommit(n);
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
  BLOCK:    "rgba(90, 70, 50, 0.95)",
  SWITCH:   "rgba(255, 196, 0, 0.95)",
  JUNCTION: "rgba(120, 200, 255, 0.95)",
  GATE:     "rgba(220, 80, 80, 0.95)",
  PLATE:    "rgba(170, 130, 255, 0.95)",
  TRACK:    "rgba(200, 170, 110, 0.95)",
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
  const apply = () => {
    // Guard against empty / non-numeric input. `Number("")` is 0,
    // `Number("abc")` is NaN. Both would erase the board via
    // resizeBoard. Fall back to the current dimension on bad input
    // and snap the input value back so the user sees the rejection.
    const cParsed = Number(cols.value);
    const rParsed = Number(rows.value);
    const safeCols =
      Number.isFinite(cParsed) && cParsed >= 1
        ? Math.round(cParsed)
        : board[0]?.length ?? 8;
    const safeRows =
      Number.isFinite(rParsed) && rParsed >= 1
        ? Math.round(rParsed)
        : board.length;
    cols.value = String(safeCols);
    rows.value = String(safeRows);
    resizeBoard(safeCols, safeRows);
  };
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

  const tickMode = document.getElementById("train-tick-mode") as HTMLSelectElement | null;
  const tickN = document.getElementById("train-tick-n") as HTMLInputElement | null;
  if (tickMode) {
    tickMode.addEventListener("change", () => {
      const value = tickMode.value;
      if (value === "EveryNPly") {
        const current = flags.trainTickRate.kind === "EveryNPly" ? flags.trainTickRate.n : 2;
        flags.trainTickRate = { kind: "EveryNPly", n: current };
      } else if (value === "EveryPly") {
        flags.trainTickRate = { kind: "EveryPly" };
      } else {
        flags.trainTickRate = { kind: "EveryFullTurn" };
      }
      syncFromState();
    });
  }
  if (tickN) {
    tickN.addEventListener("change", () => {
      const n = Number.parseInt(tickN.value, 10);
      if (Number.isFinite(n) && n > 0) {
        flags.trainTickRate = { kind: "EveryNPly", n };
        syncFromState();
      }
    });
  }
  const ply = document.getElementById("ply-count") as HTMLInputElement | null;
  if (ply) {
    ply.addEventListener("change", () => {
      const n = Number.parseInt(ply.value, 10);
      if (Number.isFinite(n) && n >= 0) {
        flags.plyCount = n;
        syncFromState();
      }
    });
  }
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
