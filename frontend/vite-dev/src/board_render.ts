// src/board_render.ts
//
// Shared, presentation-only board renderer. Draws a FEN into a board
// element — pieces, ducks, square conditions, tornado swirls, substrate
// types + icons, coordinate labels and the last-move tint. It is
// deliberately free of any game logic: no move fetching, no click
// wiring, no status banners.
//
// Two pages consume it:
//   * the interactive play surface (main.ts) passes an `onSquareClick`
//     handler and layers selection / highlight / duck-placement state on
//     top afterwards;
//   * the read-only FEN renderer (fen_page.ts) passes nothing, so the
//     board is purely a picture.
//
// Orientation is read from the shared `boardOrientation` in variables.ts
// so both pages flip through the same switch.

import { setBoardDimensions } from "./board_size";
import { parseFEN, parseLastMove, pieceToImage, pieceToSymbol } from "./fen";
import { renderCarrierPassengerOverlay } from "./passenger_overlay";
import { squareIconSvg } from "./signal_icons";
import { isTrainCart, trainCartRotationDegrees } from "./train_payload";
import { boardOrientation, currentBoard, setCurrentBoard, squareDomIndex } from "./variables";

export type RenderBoardOptions = {
  /// When provided, each square is made clickable and forwards its
  /// logical (rank, file) to this handler. Omit for a read-only board.
  onSquareClick?: (rank: number, file: number) => void;
};

/// Render `fen` into `boardEl`. Sets the shared `currentBoard` (so the
/// caller's move logic sees the same grid) and the board's `--cols` /
/// `--rows` sizing variables. Returns nothing; the caller owns any
/// interactive overlays (selection, move dots, duck placement).
export function renderBoardInto(
  boardEl: HTMLElement,
  fen: string,
  opts: RenderBoardOptions = {},
): void {
  boardEl.innerHTML = ""; // clear previous board

  setCurrentBoard(parseFEN(fen));
  const board = currentBoard;
  const rows = board.length;
  const cols = board[0]?.length ?? 0;
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
      const square_data = board[rank][file];

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
                board,
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
            board,
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

      // Interactivity is opt-in: a read-only render leaves the square
      // inert (and the board element carries a `.readonly` class so the
      // cursor doesn't imply clickability).
      if (opts.onSquareClick) {
        const handler = opts.onSquareClick;
        square.onclick = () => handler(rank, file);
      }

      boardEl.appendChild(square);
    }
  }

  boardEl.classList.toggle("readonly", !opts.onSquareClick);

  // Tint the squares of the most recent move (from the FEN's `lm=` marker)
  // so the opponent's move is obvious in online play.
  highlightLastMove(fen, rows, cols);
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
