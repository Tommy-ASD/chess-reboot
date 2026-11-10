// src/main.ts
//
// Play page entry point. Dev: `npm run dev` (Vite). Production
// build: `npm run build` (runs `tsc && vite build`). See package.json.

import { initBoardResize, setBoardDimensions } from "./board_size";
import { clearSelection, highlightMoves, isAllowedSquare, isSpecialMove } from "./board_helpers";
import { getBusPassengers, parseFEN, pieceToImage, pieceToSymbol } from "./fen";
import { renderCarrierPassengerOverlay } from "./passenger_overlay";
import { squareIconSvg } from "./signal_icons";
import { isTrainCart, trainCartRotationDegrees } from "./train_payload";
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

  // Visually mark the selected square. Read `--cols` from the
  // board's CSS variable rather than hardcoding 8 — plan 09 added
  // variable board dimensions, so any non-8-wide board would pick
  // the wrong DOM cell here. `board_helpers.ts::highlightMoves`
  // already follows this convention.
  const squareEls = document.querySelectorAll(".square");
  squareEls.forEach(s => s.classList.remove("selected"));
  const cols =
    Number(
      getComputedStyle(document.documentElement)
        .getPropertyValue("--cols")
        .trim(),
    ) || currentBoard[0]?.length || 8;
  squareEls[rank * cols + file]?.classList.add("selected");

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

      case "PlaceTornado": {
        // Stormcaller stamps a tornado on an adjacent square. There's
        // one PlaceTornado per in-bounds neighbour, so the target must
        // be in the label or the actions are indistinguishable. Render
        // the target algebraically (file letter + rank counted from the
        // bottom, matching the engine's `format_coord`).
        const t = m.move_type.target.target;
        const fileLetter = String.fromCharCode(97 + t.file);
        const rankNum = currentBoard.length - t.rank;
        li.textContent = `Place Tornado → ${fileLetter}${rankNum}`;
        break;
      }

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

// The first group are bare grids — the engine's parser fills in
// default flag fields (stm=w, castling=KQkq, ep=-, tr=full, p=0) when
// they're absent. The PUZZLES below use the full FEN form because a
// puzzle needs an explicit side-to-move and castling off ("-") to have
// a single intended solution. The editor's PRESETS in `editor_page.ts`
// also use the full form. All round-trip through the engine identically.
//
// Internal coords (confirmed against api/src/main.rs's fool's-mate
// test): FEN row 0 = rank 0 = Black's back rank (a8..h8); row 7 =
// rank 7 = White's back rank. file 0..7 = a..h, left to right.
const FEN_PRESETS: { name: string; fen: string }[] = [
  { name: "Empty Board", fen: "8/8/8/8/8/8/8/8" },
  { name: "Standard Chess", fen: "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR" },
  { name: "Goblin Test", fen: "(P=g(H=0-0))nbqkbn(P=g(H=7-0))/pppppppp/8/8/8/8/PPPPPPPP/(P=G(H=0-7))NBQKBN(P=G(H=7-7))" },
  { name: "Vent Test", fen: "(T=VENT)7/8/8/8/8/8/8/8" },
  { name: "Frozen Test", fen: "(C=FROZEN)7/8/8/8/8/8/8/8" },

  // ================================================================
  // PUZZLE LIBRARY — accumulated, not replaced. Every entry below was
  // verified end-to-end against the running engine during the session
  // it was built (enumerate legal moves via /board/moves, replay via
  // /board/new_state, confirm the result). Solutions are in the
  // comments so the visible name doesn't spoil them. Coords are
  // (file,rank); rank 0 is Black's back rank.
  // ================================================================

  // ---- One-move mechanism tactics -------------------------------

  // "The Brainrot Wall" (mate in 1). Black's bishop, knight AND queen
  // all look like they defend the back rank, but the White Skibidi on
  // c5 is PHASE=4 — its radius-3 brainrot disk freezes b7/c6/e6 solid.
  // Solution: Ra1-a8#  (rook (0,7)->(0,0)). Verified Checkmate{White};
  // the result FEN paints the brainrot diamond.
  { name: "Puzzle: The Brainrot Wall (W #1)",
    fen: "6k1/1b3ppp/2n1q3/2(P=S(PHASE=4))5/8/5N2/5PPP/R5K1 w - -" },

  // "Goblin Guillotine" (mate in 1). The free Goblin on b1 moves like
  // a queen; it rifles up the open b-file into a contact mate that
  // can't be blocked, run from, or taken (b7 guarded twice).
  // Solution: Gb1-b7#  (goblin (1,7)->(1,1)). Verified Checkmate{White}.
  { name: "Puzzle: Goblin Guillotine (W #1)",
    fen: "k6r/6pp/2K2n2/N3q3/8/8/6PP/1(P=G(H=1-7))2R3 w - -" },

  // "Deus Ex Machina" (mate in 1, by a piece that never nears the
  // king). A closed Gate on h4 blocks the h1 rook's shot at Kh8
  // (pre-move: Ongoing, not even check). The knight leaps onto the
  // pressure Plate on d4; the plate fires *before* the turn ends, the
  // remote Gate flips open, and the rook's ray completes for mate.
  // Solution: Nc6-d4  (2,2)->(3,4). Verified Ongoing -> Checkmate{White}.
  { name: "Puzzle: Deus Ex Machina (W #1)",
    fen: "6pk/6p1/2N5/8/3(T=PLATE,TARGETS=(7),FIRES=W)3(T=GATE,ID=7,OPEN=0)/8/8/K6R w - -" },

  // "Caught on the Rails" (mate in 1, unique). Quiet position. The
  // Monkey on d8 is the only blocker of the a8 rook's 8th-rank shot;
  // BLOCK terrain on c7/e7 fences it in. Its two king-steps just slide
  // along the rank (stay Ongoing); only the Monkey *leaping* over its
  // own d7 pawn vacates the file and discovers mate.
  // Solution: M d8-d6  (3,0)->(3,2). Verified the unique Checkmate{White}.
  { name: "Puzzle: Caught on the Rails (W #1)",
    fen: "R2M3k/2(T=BLOCK)P(T=BLOCK)1pp/8/8/8/8/8/K7 w - -" },

  // ---- Mechanism contraptions / sandboxes -----------------------

  // "The Contraption". A White rook is sealed in a BLOCK chamber whose
  // only door is the closed Gate (ID=9) on d4. The Bus on f2 (carries
  // a knight, slides like a rook) drives straight up onto the pressure
  // Plate on f6; the plate fires, the Gate flips open, the rook frees.
  // Solution: Bus f2-f6  (5,6)->(5,2). Verified: Gate flips OPEN 0->1.
  { name: "Puzzle: The Contraption — free the rook (W)",
    fen: "k7/8/2(T=BLOCK)(T=BLOCK)(T=BLOCK)(T=PLATE,TARGETS=(9),FIRES=W)2/2(T=BLOCK)(P=R)(T=BLOCK)3/2(T=BLOCK)(T=GATE,ID=9,OPEN=0)(T=BLOCK)3/8/5(P=BUS(P=(N)))2/K7 w - -" },

  // "Break the Spell". The entire White army (rook/knight/bishop/queen
  // /3 pawns) is frozen in the radius-3 brainrot aura of the cornered
  // Black Skibidi at PHASE=4 — every piece has ZERO legal moves. The
  // lone mobile White Skibidi marches INTO the aura; the neutralization
  // rule resets the Black Skibidi to phase 1 and the army thaws.
  // Solution: White Skibidi d7-c7  (3,1)->(2,1). Verified: post-move
  // FEN has zero BRAINROT; the frozen queen goes from 0 to 16 moves.
  { name: "Puzzle: Break the Spell (W)",
    fen: "(P=s(PHASE=4))(P=R,C=BRAINROT)(P=N,C=BRAINROT)(P=B,C=BRAINROT)3k/(P=P,C=BRAINROT)(P=P,C=BRAINROT)(C=BRAINROT)S4/(P=P,C=BRAINROT)(C=BRAINROT)5K/(P=Q,C=BRAINROT)7/8/8/8/8 w - -" },

  // "The Roundhouse" (sandbox). A working 12-tile rectangular rail
  // loop, tr=ply: a LOCO + 2 CARTs that roll one tile every ply.
  // Make any move and watch it circle — neighbour auto-connect bends
  // the rails round the corners. Verified: loco runs E along the top,
  // auto-turns S at the corner, carts chain-follow, never derails.
  { name: "Sandbox: The Roundhouse (train loop)",
    fen: "7k/8/1(T=TRACK,D=E,P=LOCO(ID=1,H=F))(T=TRACK,D=E)(T=TRACK,D=E)(T=TRACK,D=E)3/1(T=TRACK,D=E,P=CART(ID=1,I=1))2(T=TRACK,D=E)3/1(T=TRACK,D=E,P=CART(ID=1,I=2))2(T=TRACK,D=E)3/1(T=TRACK,D=E)(T=TRACK,D=E)(T=TRACK,D=E)(T=TRACK,D=E)3/7P/6K1 w - - tr=ply p=0" },

  // ---- Forced multi-move (tornado compulsion) -------------------
  // The compulsion: "if the side to move can reach a tornado square,
  // every move that does NOT land on it is illegal." This makes the
  // opponent's reply genuinely forced.

  // "The Summoning" (mate in 2). Kh8 smothered by its own knight/pawns;
  // the rook on f2 guards f7. Verified: 24 White moves, none mates in 1.
  //   1. Stormcaller e6 -> PlaceTornado f7.  Black's ONLY legal reply
  //      (king frozen, compulsion) is 1... Rf2-f7 (rook trapped).
  //   2. Ne5 x f7#  — White spears the trapped rook; smothered mate.
  { name: "Puzzle: The Summoning (W, mate in 2)",
    fen: "6nk/6pp/4W3/4N3/8/8/5r2/K7 w - -" },

  // "The Undertow" (mate in 2). Back-rank: Kg8 walled by f7/g7/h7,
  // Ng6 covering f8+h8, Re8 guarding the 8th. Verified: 39 White
  // moves, none mates in 1.
  //   1. Stormcaller b7 -> PlaceTornado a8.  ONLY reply 1... Re8-a8
  //      (rook compelled off the back rank into the corner storm).
  //   2. Ra1 x a8#  — White is itself compelled to capture it, and
  //      that capture *is* the back-rank mate.
  { name: "Puzzle: The Undertow (W, mate in 2)",
    fen: "4r1k1/1W3ppp/6N1/8/8/8/8/R5K1 w - -" },

  // ---- Forced mate-in-5 zugzwang studies ------------------------
  // Black is in total zugzwang: a free Goblin cages the cornered king
  // (covers all 3 escape squares, never checks it), so Black's only
  // move ever is a lone pawn shuffle. A lone queen + far king can't
  // mate, so White marches its king 4 squares, then mates. Verified:
  // 31 White moves (none mates in 1); exactly 1 Black reply per turn.

  // 1.Kf3 h5 2.Ke4 h4 3.Kd5 h3 4.Kc6 h2 5.Gb7#
  { name: "Puzzle: The Procession (W, mate in 5)",
    fen: "k7/8/1(P=G(H=1-2))5p/8/8/8/6K1/8 w - -" },

  // Mirrored to the a1 corner: 1.Kf6 h5 2.Ke5 h4 3.Kd4 h3 4.Kc3 h2 5.Gb2#
  { name: "Puzzle: The Long Walk (W, mate in 5)",
    fen: "8/6K1/7p/8/8/1(P=G(H=1-5))6/8/k7 w - -" },
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
