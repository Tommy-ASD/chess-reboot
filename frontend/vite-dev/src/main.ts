// src/main.ts
//
// Play page entry point. Dev: `npm run dev` (Vite). Production
// build: `npm run build` (runs `tsc && vite build`). See package.json.

import { initBoardResize, setBoardDimensions } from "./board_size";
import { castleKingDest, clearSelection, highlightMoves, isAllowedSquare, isSpecialMove } from "./board_helpers";
import { formatLastMove, getBusPassengers, parseFEN, parseFENFlags, parseLastMove, pieceToImage, pieceToSymbol } from "./fen";
import { renderCarrierPassengerOverlay } from "./passenger_overlay";
import { squareIconSvg } from "./signal_icons";
import { isTrainCart, trainCartRotationDegrees } from "./train_payload";
import { allowedMoves, boardOrientation, currentBoard, selectedPassengerIndex, selectedSquare, setAllowedMoves, setBoardOrientation, setCurrentBoard, setSelectedPassengerIndex, setSelectedSquare, squareDomIndex, type Color, type Coord, type GameMove, type GameStatus } from "./variables";
import { API_BASE } from "./config";
import * as online from "./online";
import type { CreateGameRequest, GameResult, GameState, Subscription } from "./online";
import { getName, setName } from "./player";
import { toast } from "./toast";



// ---------------------------
// Rendering
// ---------------------------

function renderBoard(fen: string) {
  const boardEl = document.getElementById("board")!;
  boardEl.innerHTML = ""; // clear previous board

  // Reset the status banner on every re-render; callers that know the
  // post-position status (Load, post-move) re-show it immediately after.
  renderStatus(null);

  setCurrentBoard(parseFEN(fen));
  const rows = currentBoard.length;
  const cols = currentBoard[0]?.length ?? 0;
  setBoardDimensions(cols, rows);

  // Render in DOM (row-major) order, mapping each slot to its logical
  // square for the current orientation. A black-oriented board is a 180°
  // rotation, so both axes reverse. Logical coords drive piece lookup, the
  // checker pattern, and the click handler — so move logic never has to
  // know which way the board faces.
  const flipped = boardOrientation === "black";
  const showCoords = rows === 8 && cols === 8;
  for (let dr = 0; dr < rows; dr++) {
    for (let df = 0; df < cols; df++) {
      const rank = flipped ? rows - 1 - dr : dr;
      const file = flipped ? cols - 1 - df : df;
      const square_data = currentBoard[rank][file];

      const square = document.createElement("div");
      square.classList.add("square");

      // light/dark checkered pattern (keyed on logical coords, so each
      // square keeps its colour when the board flips)
      const isDark = (rank + file) % 2 === 1;
      square.classList.add(isDark ? "dark" : "light");

      if (square_data) {
        if (square_data.piece) {
          // check if pieceToImage returns other than undefined
          // and if it does, use an img element instead of textContent
          const imgPath = pieceToImage(square_data.piece);
          if (imgPath) {
            const img = document.createElement("img");
            img.src = imgPath;
            img.alt = square_data.piece;
            img.classList.add("piece-image");
            if (isTrainCart(square_data.piece)) {
              const deg = trainCartRotationDegrees(
                square_data.piece,
                currentBoard,
                file,
                rank,
              );
              if (deg !== 0) img.style.transform = `rotate(${deg}deg)`;
            }
            square.appendChild(img);
          } else {
            square.textContent = pieceToSymbol(square_data.piece);
          }
          renderCarrierPassengerOverlay(square, square_data.piece);
        }
        // Plan 11 (Duck Chess): the duck is colourless and never shares a
        // square with a piece, so render it as the square's glyph.
        if (square_data.duck) {
          const d = document.createElement("span");
          d.className = "duck-glyph";
          d.textContent = "\u{1F986}"; // 🦆
          square.appendChild(d);
        }
        if (square_data.conditions.includes("FROZEN")) {
          square.classList.add("cond-frozen");
        }
        if (square_data.conditions.includes("BRAINROT")) {
          square.classList.add("cond-brainrot");
        }
        // Plan 13: `TORNADO` carries a `:<remaining>` countdown payload,
        // so match the prefix rather than exact equality. The swirl is a
        // CSS overlay (.cond-tornado::after); the countdown is value-
        // bearing, so the badge is built here.
        const tornado = square_data.conditions.find(
          (c) => c === "TORNADO" || c.startsWith("TORNADO:"),
        );
        if (tornado) {
          square.classList.add("cond-tornado");
          const remaining = tornado.split(":")[1];
          if (remaining) {
            const badge = document.createElement("span");
            badge.className = "tornado-countdown";
            badge.textContent = remaining;
            square.appendChild(badge);
          }
        }
        // Plan 08: substrate types render with a per-type accent border
        // (via `type-{lowercase}`) plus an SVG icon overlay.
        if (square_data.squareType !== "STANDARD") {
          square.classList.add(`type-${square_data.squareType.toLowerCase()}`);
          const svg = squareIconSvg(square_data, {
            board: currentBoard,
            file,
            rank,
          });
          if (svg) {
            const wrap = document.createElement("div");
            wrap.className = "square-icon";
            wrap.innerHTML = svg;
            square.appendChild(wrap);
          }
        }

      }

      // Coordinate labels on the board edges (standard 8×8 only — fairy
      // boards have arbitrary dimensions). Appended AFTER the piece, whose
      // `textContent` assignment would otherwise wipe these child nodes.
      // File letter on the bottom DOM row, rank number on the left DOM
      // column; both read the logical square so they rotate with the
      // orientation.
      if (showCoords) {
        if (dr === rows - 1) square.appendChild(coordLabel("file", "abcdefgh"[file]));
        if (df === 0) square.appendChild(coordLabel("rank", String(8 - rank)));
      }

      square.onclick = () => handleSquareClick(rank, file);

      boardEl.appendChild(square);
    }
  }

  // Tint the squares of the most recent move (from the FEN's `lm=` marker)
  // so the opponent's move is obvious in online play.
  highlightLastMove(fen, rows, cols);

  // Plan 11: if the position is a Duck Chess placement half-turn, light up
  // the empty squares as duck targets and show the hint.
  setupDuckPlacementMode();
}

/// A board-edge coordinate label (file letter / rank number). Absolutely
/// positioned within its corner cell via the `coord-*` CSS classes.
function coordLabel(kind: "file" | "rank", text: string): HTMLSpanElement {
  const span = document.createElement("span");
  span.className = `coord-label coord-${kind}`;
  span.textContent = text;
  return span;
}

/// Tint the from/to squares of the last move (`.last-move`), honoring the
/// current orientation via `squareDomIndex`.
function highlightLastMove(fen: string, rows: number, cols: number) {
  const lm = parseLastMove(fen);
  if (!lm) return;
  const squares = document.querySelectorAll("#board .square");
  for (const c of [lm.from, lm.to]) {
    squares[squareDomIndex(c.rank, c.file, rows, cols)]?.classList.add("last-move");
  }
}

/// Handler attached to each square on the board
/// On click, fetches legal moves from backend and highlights them
async function handleSquareClick(rank: number, file: number) {
  const clicked = { rank, file };

  // Any square click dismisses a pending promotion picker.
  hidePromotionPicker();

  // Online mode: the board is view-only unless it's the player's turn
  // (also blocks interaction while spectating or after game over). During
  // the player's own Duck-Chess placement half-turn `side_to_move` stays
  // on their colour, so the placement branch below still passes this gate.
  if (onlineSession && !isMyTurnOnline()) return;

  // Plan 11 (Duck Chess) placement half-turn: clicking an empty square
  // places (first turn) or moves the duck. Piece selection is disabled —
  // pieces can't move this half, and the engine returns no piece moves.
  if (duckPlacementActive()) {
    if (isEmptyForDuck(clicked)) {
      await placeDuckAt(clicked);
    }
    return;
  }

  // if the user clicks the selected square again, clear selection
  if (selectedSquare && selectedSquare.rank === rank && selectedSquare.file === file) {
    console.log("Pressed twice; clearing selection");
    clearSelection();
    return;
  }

  // if the user clicks an allowed square, make the move
  if (isAllowedSquare(clicked)) {
    console.log("Move:", selectedSquare, "->", clicked, "passenger:", selectedPassengerIndex);

    // Promotion needs a piece choice: if the clicked square is a promotion
    // target, show the picker and defer the move until the user picks.
    const promotions = allowedMoves.filter(
      (m) =>
        m.move_type.kind === "Promotion" &&
        m.move_type.target.target.file === clicked.file &&
        m.move_type.target.target.rank === clicked.rank,
    );
    if (promotions.length > 0) {
      showPromotionPicker(promotions);
      return;
    }

    const moveToExecute = findMoveForTarget(clicked, allowedMoves, selectedPassengerIndex);
    if (!moveToExecute) {
      console.error("isAllowedSquare matched but findMoveForTarget returned null");
      return;
    }
    await executeMove(moveToExecute);
    return;
  }

  setSelectedSquare(clicked);
  setSelectedPassengerIndex(null);

  // Visually mark the selected square. Read `--cols` from the
  // board's CSS variable rather than hardcoding 8 — plan 09 added
  // variable board dimensions, so any non-8-wide board would pick
  // the wrong DOM cell here. `board_helpers.ts::highlightMoves`
  // already follows this convention.
  const squareEls = document.querySelectorAll(".square");
  squareEls.forEach(s => s.classList.remove("selected"));
  const rootStyle = getComputedStyle(document.documentElement);
  const cols = Number(rootStyle.getPropertyValue("--cols").trim()) || currentBoard[0]?.length || 8;
  const rows = Number(rootStyle.getPropertyValue("--rows").trim()) || currentBoard.length || 8;
  squareEls[squareDomIndex(rank, file, rows, cols)]?.classList.add("selected");

  try {
    const fen = currentFen();
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
      if (m.move_type.kind === "EnPassant" && sameCoord(m.move_type.target.target, clicked)) return m;
      if (m.move_type.kind === "Castle") {
        const dest = castleKingDest(m);
        if (dest && sameCoord(dest, clicked)) return m;
      }
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

/// Apply a chosen move via the backend, then re-render the board + status
/// and clear the selection. `MoveTo` goes through `makeMove`; everything
/// else — Castle / Promotion / EnPassant / carrier / special — through
/// `makeSpecialMove`, which posts the `GameMove` as-is.
async function executeMove(move: GameMove) {
  try {
    const result = await submitMove(move);
    console.log("New FEN:", result.newFen);
    // Clear the prior selection BEFORE re-rendering: `clearSelection`
    // strips `.highlight` from squares, so running it after `renderBoard`
    // would wipe the duck-placement highlights that `renderBoard` adds
    // when the new position is a duck half-turn.
    clearSelection();
    renderBoard(result.newFen);
    renderStatus(result.status);
    if (onlineSession) renderGamePanel();
  } catch (err) {
    showError(err);
  }
}

/// The single move sink. Online: POST to the game (the server enforces
/// turn/colour, the engine validates legality) and adopt the returned
/// snapshot. Local: the original stateless `/board/new_state` dispatch
/// (`MoveTo` → `makeMove`, everything else → `makeSpecialMove`). Either
/// way `#fen-input` is updated so the FEN-reading helpers — and a later
/// switch back to Local mode — see the current position.
async function submitMove(move: GameMove): Promise<MoveResult> {
  if (onlineSession) {
    const state = await online.submitMove(onlineSession.gameId, move);
    onlineSession.state = state;
    (document.getElementById("fen-input") as HTMLInputElement).value = state.fen;
    return { newFen: state.fen, status: state.status };
  }
  const fen = currentFen();
  const result =
    move.move_type.kind === "MoveTo"
      ? await makeMove(fen, move.from, move.move_type.target)
      : await makeSpecialMove(fen, move);
  (document.getElementById("fen-input") as HTMLInputElement).value = result.newFen;
  return result;
}

/// Show the promotion picker for the four `Promotion` moves that share a
/// target square. Picking a piece submits that specific move. The glyph
/// case follows the promoting pawn's colour.
function showPromotionPicker(moves: GameMove[]) {
  const picker = document.getElementById("promotion-picker")!;
  picker.innerHTML = "";
  picker.classList.remove("hidden");

  const from = moves[0].from;
  const pawn = currentBoard[from.rank]?.[from.file]?.piece ?? "P";
  const white = pawn === pawn.toUpperCase();
  const glyphs: Record<string, [string, string]> = {
    Queen: ["♕", "♛"],
    Rook: ["♖", "♜"],
    Bishop: ["♗", "♝"],
    Knight: ["♘", "♞"],
  };

  const label = document.createElement("span");
  label.className = "promotion-label";
  label.textContent = "Promote to:";
  picker.appendChild(label);

  for (const m of moves) {
    if (m.move_type.kind !== "Promotion") continue;
    const into = m.move_type.target.into;
    const btn = document.createElement("button");
    btn.className = "promotion-choice";
    btn.textContent = glyphs[into]?.[white ? 0 : 1] ?? into;
    btn.title = into;
    btn.onclick = () => {
      hidePromotionPicker();
      executeMove(m);
    };
    picker.appendChild(btn);
  }
}

/// Hide + clear the promotion picker. Safe to call when it's already
/// hidden (every square click calls it to dismiss a stale picker).
function hidePromotionPicker() {
  const picker = document.getElementById("promotion-picker");
  if (!picker) return;
  picker.classList.add("hidden");
  picker.innerHTML = "";
}

/// Plan 11: is the current position a Duck Chess duck-placement half-turn?
/// Read straight off the FEN flags (variants + duck_phase).
function duckPlacementActive(): boolean {
  const fen = currentFen();
  const flags = parseFENFlags(fen);
  return flags.variants.includes("duck_chess") && flags.duckPhase === "placing";
}

/// Plan 11: a square that can receive the duck — empty of both a piece and
/// the duck. Walkability isn't checked client-side; the engine's
/// `validate_duck_move` is the final authority and rejects bad targets.
function isEmptyForDuck(c: Coord): boolean {
  const sq = currentBoard[c.rank]?.[c.file];
  return !!sq && !sq.piece && !sq.duck;
}

/// Plan 11: the duck's current square, or null before its first placement.
function findDuckOnBoard(): Coord | null {
  for (let rank = 0; rank < currentBoard.length; rank++) {
    const row = currentBoard[rank];
    for (let file = 0; file < row.length; file++) {
      if (row[file]?.duck) return { file, rank };
    }
  }
  return null;
}

/// Plan 11: place (first turn) or move the duck to `clicked`, then apply
/// via the backend (which validates legality).
async function placeDuckAt(clicked: Coord) {
  const existing = findDuckOnBoard();
  const move: GameMove = existing
    ? { from: existing, move_type: { kind: "MoveDuck", target: { to: clicked } } }
    : { from: clicked, move_type: { kind: "PlaceDuck", target: { to: clicked } } };
  await executeMove(move);
}

/// Plan 11: during a duck-placement half-turn, highlight every empty
/// square as a placement target and show the hint. Called at the end of
/// `renderBoard`; hides the hint and does nothing otherwise.
function setupDuckPlacementMode() {
  const hint = document.getElementById("duck-hint")!;
  if (!duckPlacementActive()) {
    hint.classList.add("hidden");
    return;
  }
  hint.textContent = "🦆 Duck Chess — click a highlighted empty square to place the duck.";
  hint.classList.remove("hidden");

  const squares = document.querySelectorAll("#board .square");
  const rootStyle = getComputedStyle(document.documentElement);
  const cols = Number(rootStyle.getPropertyValue("--cols").trim()) || currentBoard[0]?.length || 8;
  const rows = Number(rootStyle.getPropertyValue("--rows").trim()) || currentBoard.length || 8;
  for (let rank = 0; rank < currentBoard.length; rank++) {
    const row = currentBoard[rank];
    for (let file = 0; file < row.length; file++) {
      if (!row[file]?.piece && !row[file]?.duck) {
        squares[squareDomIndex(rank, file, rows, cols)]?.classList.add("highlight");
      }
    }
  }
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

      // Plan 13: the Stormcaller's tornado placement. Each candidate
      // targets a distinct in-range square, so label it with the target.
      // (Struct variant → coord nests under `target.target`.)
      case "PlaceTornado":
        li.textContent = `Place Tornado → (${m.move_type.target.target.file}, ${m.move_type.target.target.rank})`;
        break;

      default:
        li.textContent = JSON.stringify(m.move_type);
        break;
    }

    // Route through the shared sink so online games POST to the server
    // (turn already gated) and local play keeps the stateless dispatch.
    li.onclick = () => executeMove(m);

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
  toast(msg, "error", 5000);
}

/// Render the game-status banner (plans 04/06). `Ongoing` or `null` hides
/// it; `Check` shows an info banner; the terminal outcomes
/// (checkmate / stalemate / brainrot win / brainrot lockout) show a
/// game-over banner.
function renderStatus(status: GameStatus | null) {
  const el = document.getElementById("game-status")!;
  const described = status ? describeStatus(status) : null;
  if (!described) {
    el.className = "game-status hidden";
    el.textContent = "";
    return;
  }
  el.className = `game-status ${described.kind}`;
  el.textContent = described.text;
}

/// Map a `GameStatus` to banner text + severity. Returns `null` for
/// `Ongoing` (nothing to show). The `switch` is exhaustive over the
/// engine's `GameStatus` variants — adding one there surfaces a TS error
/// here until it's handled.
function describeStatus(
  status: GameStatus,
): { text: string; kind: "info" | "over" } | null {
  switch (status.status) {
    case "Ongoing":
      return null;
    case "Check":
      return { text: `Check — ${status.data.side_to_move} to move`, kind: "info" };
    case "Checkmate":
      return { text: `Checkmate — ${status.data.winner} wins`, kind: "over" };
    case "Stalemate":
      return { text: "Stalemate — draw", kind: "over" };
    case "BrainrotWin":
      return { text: `Brainrot win — ${status.data.winner} wins`, kind: "over" };
    case "BrainrotLockout":
      return { text: `Brainrot lockout — ${status.data.winner} wins`, kind: "over" };
    case "Win":
      return { text: `King captured — ${status.data.winner} wins`, kind: "over" };
  }
}

/// Calls the backend API to get legal moves for a piece at (file, rank) on the board described by fen
async function fetchMoves(fen: string, rank: number, file: number): Promise<GameMove[]> {
  const response = await fetch(`${API_BASE}/board/moves`, {
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

/// Query the post-position game status for a FEN (plan 06 `/board/status`).
/// Used on explicit Load so a terminal position surfaces its banner
/// without first requiring a move. (Live-edit previews skip this to avoid
/// a request per keystroke.)
async function fetchStatus(fen: string): Promise<GameStatus> {
  const response = await fetch(`${API_BASE}/board/status`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ board_fen: fen }),
  });

  if (!response.ok) {
    throw await consumeError(response, "fetchStatus");
  }

  const data = await response.json();
  return data.status;
}

/// Result of a state-changing API call: the post-move FEN plus the
/// engine's post-move `GameStatus` (folded into `/board/new_state`).
type MoveResult = { newFen: string; status: GameStatus };

async function makeSpecialMove(fen: string, move: GameMove): Promise<MoveResult> {
  const response = await fetch(`${API_BASE}/board/new_state`, {
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
  return { newFen: data.new_board_fen, status: data.status };
}


/// Attempts to make a move
/// API call's at `POST /board/new_state` with body:
/// {
///   board_fen: string,
///   from: { file: number, rank: number },
///   to: { file: number, rank: number }
/// }
/// Returns the new FEN string on success
async function makeMove(fen: string, from: Coord, to: Coord): Promise<MoveResult> {
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
  const response = await fetch(`${API_BASE}/board/new_state`, body);
  console.log("Response:", response);

  if (!response.ok) {
    throw await consumeError(response, "makeMove");
  }

  const data = await response.json();
  console.log("Move response data:", data);
  return { newFen: data.new_board_fen, status: data.status };
}


// ---------------------------
// Online multiplayer (lobby + live game)
// ---------------------------

/// Canonical Duck-Chess starting position (standard set + the variant
/// flag; `duck_phase` defaults to "piece" — a piece moves first, then the
/// duck is placed). Matches the engine's canonical FEN encoding.
const DUCK_CHESS_FEN =
  "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - tr=full p=0 variants=duck_chess";

/// The active online game (null in Local mode / the lobby). `myColor` is
/// the seat captured from the create/join REST response — WS pushes carry
/// `your_color: null`, so colour is never read off the live `state`.
type OnlineSession = {
  gameId: string;
  code: string;
  myColor: Color | null;
  state: GameState;
  sub: Subscription;
};

let onlineSession: OnlineSession | null = null;
let lobbySub: Subscription | null = null;

/// Tracks the game WS connection so we only toast genuine drops/recoveries —
/// not the initial connect or an intentional leave. Reset per game.
let connState: "init" | "up" | "down" = "init";

function setConnStatus(connected: boolean) {
  const dot = document.getElementById("conn-status");
  if (dot) {
    dot.className = "conn-status " + (connected ? "live" : "down");
    dot.title = connected ? "Live" : "Reconnecting…";
  }
  if (connected) {
    if (connState === "down") toast("Reconnected", "success", 2000);
    connState = "up";
  } else {
    if (connState === "up") toast("Connection lost — reconnecting…", "warn", 4000);
    connState = "down";
  }
}

/// The FEN the board is currently showing: the online session's when
/// seated in a game, else the local FEN input. Board helpers read this
/// (not `#fen-input` directly) so online play drives off the server's
/// authoritative position.
function currentFen(): string {
  if (onlineSession) return onlineSession.state.fen;
  return (document.getElementById("fen-input") as HTMLInputElement).value;
}

/// Online turn gate: always true in Local mode; online, true only while
/// seated, on-move, and the game is still live.
function isMyTurnOnline(): boolean {
  if (!onlineSession) return true;
  const s = onlineSession.state;
  if (s.result) return false;
  return onlineSession.myColor != null && onlineSession.myColor === s.side_to_move;
}

/// Adopt a server `GameState` (initial snapshot or a WS push) as the live
/// board: sync `#fen-input`, re-render the board + status, refresh the
/// game panel. `myColor` is preserved (pushes carry `your_color: null`).
function applyOnlineState(state: GameState) {
  if (!onlineSession) return;
  notifyStateChange(onlineSession.state, state);
  onlineSession.state = state;
  (document.getElementById("fen-input") as HTMLInputElement).value = state.fen;
  clearSelection();
  renderBoard(state.fen);
  renderStatus(state.status);
  renderOnlineOutcome(state);
  renderGamePanel();
  trackHistory(state);
}

/// Emit toast cues for the meaningful transitions between two snapshots of
/// the same game: an opponent taking a seat, the game ending, and the turn
/// passing to us.
function notifyStateChange(prev: GameState, next: GameState) {
  if (!onlineSession) return;
  const my = onlineSession.myColor;

  const oppSeat = (s: GameState) =>
    my === "White" ? s.black : my === "Black" ? s.white : null;
  if (!oppSeat(prev) && oppSeat(next)) {
    toast(`${oppSeat(next)!.name || "Opponent"} joined`, "success");
  }

  if (!prev.result && next.result) {
    const r = next.result;
    const msg =
      r.kind === "Draw"
        ? "Game drawn"
        : r.kind === "Resignation"
          ? `${r.winner} wins by resignation`
          : `${r.winner} wins`;
    toast(msg, "info", 6000);
  } else {
    // Turn just passed to us (suppressed when the game has ended).
    const wasMine = !prev.result && my != null && my === prev.side_to_move;
    const isMine = !next.result && my != null && my === next.side_to_move;
    if (!wasMine && isMine) toast("Your turn", "info", 2500);
  }

  // The opponent offered a rematch.
  if (!prev.rematch && next.rematch) toast("Rematch ready — jump in!", "success", 5000);

  // The opponent offered a draw.
  const prevOppDraw = prev.draw_offer != null && my != null && prev.draw_offer !== my;
  const nextOppDraw = next.draw_offer != null && my != null && next.draw_offer !== my;
  if (!prevOppDraw && nextOppDraw) toast("Opponent offers a draw", "info", 5000);
}

/// The `#game-status` banner is driven by `GameStatus`, which has no
/// variant for a resignation (the board isn't mated — the outcome lives in
/// `result`). When a game has a `result` but `renderStatus` left the banner
/// hidden (resignation / draw), surface the result here so game-over is
/// unmistakable in the banner too. Board outcomes (checkmate / Win /
/// stalemate) already showed via `renderStatus`, so leave those alone.
function renderOnlineOutcome(state: GameState) {
  if (!state.result) return;
  const el = document.getElementById("game-status")!;
  if (!el.classList.contains("hidden")) return;
  const text =
    state.result.kind === "Draw"
      ? "Game over — draw"
      : state.result.kind === "Resignation"
        ? `${state.result.winner} wins by resignation`
        : `${state.result.winner} wins`;
  el.className = "game-status over";
  el.textContent = text;
}

// --- mode + lobby ---

function setMode(mode: "local" | "online") {
  const isOnline = mode === "online";
  document.body.classList.toggle("online", isOnline);
  document.getElementById("online-panel")!.classList.toggle("hidden", !isOnline);
  const tabOnline = document.getElementById("tab-online")!;
  const tabLocal = document.getElementById("tab-local")!;
  tabOnline.classList.toggle("active", isOnline);
  tabLocal.classList.toggle("active", !isOnline);
  tabOnline.setAttribute("aria-selected", String(isOnline));
  tabLocal.setAttribute("aria-selected", String(!isOnline));
  if (isOnline) {
    startLobby();
  } else {
    stopLobby();
    if (onlineSession) leaveOnlineGame();
  }
}

function startLobby() {
  refreshPublicGames();
  refreshLiveGames();
  if (!lobbySub) {
    lobbySub = online.subscribeLobby((games) => {
      renderPublicGames(games);
      refreshLiveGames();
    });
  }
}

function stopLobby() {
  lobbySub?.close();
  lobbySub = null;
}

async function refreshPublicGames() {
  try {
    renderPublicGames(await online.listGames());
  } catch (err) {
    console.error("listGames failed", err);
  }
}

function hostName(g: GameState): string {
  return (g.white ?? g.black)?.name || "Anonymous";
}

function describeGame(g: GameState): string {
  const seat = g.white ? "Black seat open" : "White seat open";
  const variant = g.fen.includes("variants=duck_chess") ? "Duck Chess" : "Chess";
  return `${variant} · ${seat} · host ${hostName(g)}`;
}

function renderPublicGames(games: GameState[]) {
  const list = document.getElementById("public-games")!;
  const empty = document.getElementById("public-games-empty")!;
  list.innerHTML = "";
  empty.classList.toggle("hidden", games.length > 0);
  for (const g of games) {
    const li = document.createElement("li");
    const label = document.createElement("span");
    label.className = "game-label";
    label.textContent = `${g.name || "Game"} — ${describeGame(g)}`;
    const btn = document.createElement("button");
    btn.className = "join-mini";
    btn.textContent = "Join";
    btn.onclick = () => joinByIdOrCode(g.id);
    li.append(label, btn);
    list.appendChild(li);
  }
}

async function refreshLiveGames() {
  try {
    renderLiveGames(await online.listLive());
  } catch (err) {
    console.error("listLive failed", err);
  }
}

function renderLiveGames(games: GameState[]) {
  const list = document.getElementById("live-games")!;
  const empty = document.getElementById("live-games-empty")!;
  list.innerHTML = "";
  empty.classList.toggle("hidden", games.length > 0);
  for (const g of games) {
    const li = document.createElement("li");
    const label = document.createElement("span");
    label.className = "game-label";
    const variant = g.fen.includes("variants=duck_chess") ? "Duck Chess" : "Chess";
    label.textContent =
      `${g.name || "Game"} — ${variant} · ${g.white?.name ?? "?"} vs ${g.black?.name ?? "?"} · move ${g.ply}`;
    const btn = document.createElement("button");
    btn.className = "join-mini";
    btn.textContent = "Watch";
    btn.onclick = () => spectateGame(g.id);
    li.append(label, btn);
    list.appendChild(li);
  }
}

// --- create / join / leave ---

function syncName() {
  setName((document.getElementById("player-name") as HTMLInputElement).value);
}

async function createOnlineGame() {
  syncName();
  const preset = (document.getElementById("create-preset") as HTMLSelectElement).value;
  const color = (document.getElementById("create-color") as HTMLSelectElement).value as Color;
  const name = (document.getElementById("create-name") as HTMLInputElement).value.trim();
  const isPublic = (document.getElementById("create-public") as HTMLInputElement).checked;
  const req: CreateGameRequest = { public: isPublic, color };
  if (name) req.name = name;
  if (preset === "duck") req.starting_fen = DUCK_CHESS_FEN;
  try {
    enterOnlineGame(await online.createGame(req));
  } catch (err) {
    showError(err);
  }
}

async function joinByIdOrCode(idOrCode: string) {
  const key = idOrCode.trim();
  if (!key) return;
  syncName();
  try {
    enterOnlineGame(await online.joinGame(key));
  } catch (err) {
    showError(err);
  }
}

/// Open a game as a spectator: `getGame` (not join), so `your_color` stays
/// null — the board renders read-only and the seat actions stay hidden.
async function spectateGame(idOrCode: string) {
  const key = idOrCode.trim();
  if (!key) return;
  syncName();
  try {
    enterOnlineGame(await online.getGame(key));
  } catch (err) {
    showError(err);
  }
}

function enterOnlineGame(state: GameState) {
  onlineSession?.sub.close();
  connState = "init";
  const sub = online.subscribeGame(state.id, applyOnlineState, setConnStatus);
  onlineSession = {
    gameId: state.id,
    code: state.code,
    myColor: state.your_color,
    state,
    sub,
  };
  // Seat the board from the player's perspective (Black plays flipped);
  // spectators keep the default White-up view.
  setBoardOrientation(state.your_color === "Black" ? "black" : "white");
  resetHistory(state);
  setGameUrlParam(state.id);
  showGameView(true);
  applyOnlineState(state);
}

function leaveOnlineGame() {
  onlineSession?.sub.close();
  onlineSession = null;
  clearGameUrlParam();
  showGameView(false);
  refreshPublicGames();
  // Back to the local White-up view.
  setBoardOrientation("white");
  renderBoard(currentFen());
}

/// Manual board flip (works in Local and Online). Re-renders the current
/// position the other way up; re-applies the online status/panel that
/// `renderBoard` resets.
function flipBoard() {
  setBoardOrientation(boardOrientation === "white" ? "black" : "white");
  clearSelection();
  renderBoard(currentFen());
  if (onlineSession) {
    renderStatus(onlineSession.state.status);
    renderOnlineOutcome(onlineSession.state);
    renderGamePanel();
  }
}

function showGameView(inGame: boolean) {
  document.getElementById("online-lobby")!.classList.toggle("hidden", inGame);
  document.getElementById("online-game")!.classList.toggle("hidden", !inGame);
}

// --- live game panel ---

function resultWinner(r: GameResult): Color | null {
  return r.kind === "Draw" ? null : r.winner;
}

function turnPillText(s: GameState): string {
  if (s.result) {
    return s.result.kind === "Draw" ? "Draw" : `${s.result.winner} wins`;
  }
  if (!onlineSession?.myColor) return `${s.side_to_move} to move`;
  return isMyTurnOnline() ? "Your move" : "Opponent's move";
}

function updateChip(id: string, color: Color, name: string | null, s: GameState) {
  const chip = document.getElementById(id)!;
  chip.querySelector(".chip-name")!.textContent = name ?? "waiting…";
  chip.classList.toggle("active", !s.result && s.side_to_move === color);
  chip.classList.toggle("winner", Boolean(s.result && resultWinner(s.result) === color));
}

function renderGamePanel() {
  if (!onlineSession) return;
  const s = onlineSession.state;
  updateChip("chip-white", "White", s.white?.name ?? null, s);
  updateChip("chip-black", "Black", s.black?.name ?? null, s);
  const pill = document.getElementById("turn-pill")!;
  pill.textContent = turnPillText(s);
  pill.className =
    "turn-pill" + (s.result ? " over" : isMyTurnOnline() ? " mine" : " theirs");
  document.getElementById("share-line")!.textContent =
    `Code ${s.code} · ` + (onlineSession.myColor ? `you are ${onlineSession.myColor}` : "spectating");
  const resignBtn = document.getElementById("resign-btn") as HTMLButtonElement;
  resignBtn.disabled = Boolean(s.result) || onlineSession.myColor == null;

  // Rematch affordances: offer one once the game is over (to seated
  // players); once a rematch exists, both sides get a jump-across button.
  const over = Boolean(s.result);
  const seated = onlineSession.myColor != null;
  document
    .getElementById("rematch-btn")!
    .classList.toggle("hidden", !(over && seated && s.rematch == null));
  document
    .getElementById("goto-rematch-btn")!
    .classList.toggle("hidden", s.rematch == null);

  // Draw-offer affordances (seated players, live game only). I see an
  // "Offer draw" button (a pending self-offer disables it); the opponent
  // sees Accept / Decline.
  const liveSeated = !over && seated;
  const myColor = onlineSession.myColor;
  const iOffered = liveSeated && s.draw_offer != null && s.draw_offer === myColor;
  const oppOffered =
    liveSeated && s.draw_offer != null && myColor != null && s.draw_offer !== myColor;
  const offerBtn = document.getElementById("offer-draw-btn") as HTMLButtonElement;
  offerBtn.classList.toggle("hidden", !(liveSeated && (s.draw_offer == null || iOffered)));
  offerBtn.disabled = iOffered;
  offerBtn.textContent = iOffered ? "Draw offered…" : "½ Offer draw";
  document.getElementById("accept-draw-btn")!.classList.toggle("hidden", !oppOffered);
  document.getElementById("decline-draw-btn")!.classList.toggle("hidden", !oppOffered);
}

function flashCopied() {
  const btn = document.getElementById("copy-link-btn")!;
  const prev = btn.textContent;
  btn.textContent = "Copied!";
  setTimeout(() => (btn.textContent = prev), 1200);
}

// --- move history ---

let moveHistory: string[] = [];
let historyMaxPly = 0;

/// Start a fresh history for a game. Joining mid-game, we lack the earlier
/// moves, so seed the high-water mark at the entry ply and note it.
function resetHistory(state: GameState) {
  moveHistory = [];
  historyMaxPly = state.ply;
  if (state.ply > 0) moveHistory.push(`· joined at move ${state.ply} ·`);
  renderMoveHistory();
}

/// Append the move(s) that advanced the game since the last render. Only the
/// latest move is recoverable (from `lm=`), so a ply jump (lag / mid-game
/// join) is summarised rather than reconstructed.
function trackHistory(state: GameState) {
  if (state.ply <= historyMaxPly) return;
  const gap = state.ply - historyMaxPly - 1;
  if (gap > 0) moveHistory.push(`· ${gap} move${gap === 1 ? "" : "s"} not shown ·`);
  moveHistory.push(`${state.ply}. ${formatLastMove(state.fen) ?? "—"}`);
  historyMaxPly = state.ply;
  renderMoveHistory();
}

function renderMoveHistory() {
  const el = document.getElementById("move-history");
  if (!el) return;
  el.innerHTML = "";
  if (moveHistory.length === 0) {
    const li = document.createElement("li");
    li.className = "mh-empty";
    li.textContent = "No moves yet";
    el.appendChild(li);
  } else {
    for (const m of moveHistory) {
      const li = document.createElement("li");
      li.textContent = m;
      el.appendChild(li);
    }
  }
  el.scrollTop = el.scrollHeight;
}

// --- share URL ---

function gameShareUrl(id: string): string {
  const u = new URL(window.location.href);
  u.searchParams.set("game", id);
  return u.toString();
}

function setGameUrlParam(id: string) {
  window.history.replaceState({}, "", gameShareUrl(id));
}

function clearGameUrlParam() {
  const u = new URL(window.location.href);
  u.searchParams.delete("game");
  u.searchParams.delete("code");
  window.history.replaceState({}, "", u.toString());
}

// ---------------------------
// UI Wiring
// ---------------------------

document.getElementById("load-btn")!.addEventListener("click", async () => {
  const fen = (document.getElementById("fen-input") as HTMLInputElement).value;
  try {
    renderBoard(fen);
    // Surface the loaded position's status (e.g. loading an already-
    // checkmated FEN shows the banner without needing to make a move).
    renderStatus(await fetchStatus(fen));
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

// Presets here are bare grids — the engine's parser fills in default
// flag fields (stm=w, castling=KQkq, ep=-, tr=full, p=0) when they're
// absent. The editor's PRESETS in `editor_page.ts` use the canonical
// full form. Both round-trip through the engine identically.
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


// ---------------------------
// Online UI wiring
// ---------------------------

(document.getElementById("player-name") as HTMLInputElement).value = getName();
document.getElementById("player-name")!.addEventListener("change", syncName);
document.getElementById("tab-local")!.addEventListener("click", () => setMode("local"));
document.getElementById("tab-online")!.addEventListener("click", () => setMode("online"));
document.getElementById("create-btn")!.addEventListener("click", () => createOnlineGame());
document.getElementById("join-btn")!.addEventListener("click", () =>
  joinByIdOrCode((document.getElementById("join-code") as HTMLInputElement).value),
);
document.getElementById("resign-btn")!.addEventListener("click", async () => {
  if (!onlineSession || !confirm("Resign this game?")) return;
  try {
    applyOnlineState(await online.resignGame(onlineSession.gameId));
  } catch (err) {
    showError(err);
  }
});
document.getElementById("leave-btn")!.addEventListener("click", () => leaveOnlineGame());
document.getElementById("flip-btn")!.addEventListener("click", () => flipBoard());
document.getElementById("flip-local-btn")!.addEventListener("click", () => flipBoard());
document.getElementById("rematch-btn")!.addEventListener("click", async () => {
  if (!onlineSession) return;
  try {
    enterOnlineGame(await online.rematchGame(onlineSession.gameId));
  } catch (err) {
    showError(err);
  }
});
document.getElementById("goto-rematch-btn")!.addEventListener("click", async () => {
  const rid = onlineSession?.state.rematch;
  if (!rid) return;
  try {
    enterOnlineGame(await online.getGame(rid));
  } catch (err) {
    showError(err);
  }
});
document.getElementById("offer-draw-btn")!.addEventListener("click", async () => {
  if (!onlineSession) return;
  try {
    applyOnlineState(await online.offerDraw(onlineSession.gameId));
  } catch (err) {
    showError(err);
  }
});
document.getElementById("accept-draw-btn")!.addEventListener("click", async () => {
  if (!onlineSession) return;
  try {
    applyOnlineState(await online.offerDraw(onlineSession.gameId));
  } catch (err) {
    showError(err);
  }
});
document.getElementById("decline-draw-btn")!.addEventListener("click", async () => {
  if (!onlineSession) return;
  try {
    applyOnlineState(await online.declineDraw(onlineSession.gameId));
  } catch (err) {
    showError(err);
  }
});
document.getElementById("copy-link-btn")!.addEventListener("click", async () => {
  if (!onlineSession) return;
  const url = gameShareUrl(onlineSession.gameId);
  try {
    await navigator.clipboard.writeText(url);
    flashCopied();
  } catch {
    window.prompt("Copy this link:", url);
  }
});

// Deep link: `?game=<id>` (or `?code=<code>`) auto-opens Online mode and
// joins, so a shared link / reload lands straight back in the game.
{
  const target =
    new URLSearchParams(window.location.search).get("game") ??
    new URLSearchParams(window.location.search).get("code");
  if (target) {
    setMode("online");
    joinByIdOrCode(target);
  }
}
