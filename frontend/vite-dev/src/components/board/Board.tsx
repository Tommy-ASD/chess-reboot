// src/components/board/Board.tsx
//
// The board view. Pure rendering: it draws a grid of squares plus whatever
// selection / hint / last-move state it's handed, and reports clicks as
// logical coords. Game rules live in the engine; interaction state lives in
// `useBoardPlay` (play) or the Lab's editor state (editing).
//
// Sizing: the board fills its container's width, keeps square cells for
// any grid shape, and scales pieces with `cqw` units. `--cols` / `--rows`
// are set on the element itself, so several boards can share a page.

import type { ComponentChildren } from "preact";
import { getCarrierPassengers, isTrainCart, trainCartRotationDegrees } from "../../core/train_payload";
import { squareIconSvg } from "../../core/signal_icons";
import type { BoardTarget } from "../../core/board_helpers";
import { squareName } from "../../core/notation";
import type { Coord, Orientation, Square } from "../../core/types";
import { goblinCaptive, pieceForSymbol, pieceSide, skibidiPhase } from "../../content/pieces";
import { DuckSprite, PieceSprite } from "./PieceSprite";

export interface BoardProps {
  squares: Square[][];
  orientation?: Orientation;
  selected?: Coord | null;
  targets?: BoardTarget[];
  lastMove?: { from: Coord; to: Coord } | null;
  /** Highlight every empty square (Duck Chess placement). */
  emptyTargets?: boolean;
  coords?: boolean;
  /** Extra classes per square, keyed `"file,rank"` (Lab inspect / wiring marks). */
  marks?: Record<string, string>;
  onSquareClick?: (c: Coord) => void;
  label?: string;
  class?: string;
  /** Rendered over the board (e.g. a waiting-for-opponent card). */
  children?: ComponentChildren;
}

const key = (file: number, rank: number) => `${file},${rank}`;

export function Board({
  squares,
  orientation = "white",
  selected = null,
  targets = [],
  lastMove = null,
  emptyTargets = false,
  coords = false,
  marks,
  onSquareClick,
  label = "Chess board",
  class: extra,
  children,
}: BoardProps) {
  const rows = squares.length;
  const cols = squares[0]?.length ?? 0;
  const flipped = orientation === "black";
  const standard = rows === 8 && cols === 8;
  const interactive = Boolean(onSquareClick);

  const targetKinds = new Map<string, BoardTarget["kind"]>();
  for (const t of targets) targetKinds.set(key(t.target.file, t.target.rank), t.kind);

  const cells = [];
  for (let dr = 0; dr < rows; dr++) {
    for (let df = 0; df < cols; df++) {
      // DOM order is row-major; map each slot to its logical square for the
      // current orientation (a 180° flip reverses both axes).
      const rank = flipped ? rows - 1 - dr : dr;
      const file = flipped ? cols - 1 - df : df;
      const sq = squares[rank][file];
      const k = key(file, rank);
      const piece = sq.piece;
      const occupied = Boolean(piece) || Boolean(sq.duck);

      const classes = ["sq", (rank + file) % 2 === 1 ? "sq-dark" : "sq-light"];
      if (occupied) classes.push("has-piece");
      if (selected && selected.file === file && selected.rank === rank) classes.push("is-selected");
      if (lastMove && ((lastMove.from.file === file && lastMove.from.rank === rank) || (lastMove.to.file === file && lastMove.to.rank === rank))) {
        classes.push("is-last-move");
      }
      const target = targetKinds.get(k);
      if (target) classes.push("is-target", `target-${target}`);
      else if (emptyTargets && !occupied && sq.squareType !== "BLOCK") classes.push("is-target", "target-move");
      if (sq.squareType !== "STANDARD") classes.push(`type-${sq.squareType.toLowerCase()}`);
      if (sq.conditions.includes("FROZEN")) classes.push("cond-frozen");
      if (sq.conditions.includes("BRAINROT")) classes.push("cond-brainrot");
      const tornado = sq.conditions.find((c) => c === "TORNADO" || c.startsWith("TORNADO:"));
      if (tornado) classes.push("cond-tornado");
      if (marks?.[k]) classes.push(marks[k]);

      const icon = sq.squareType !== "STANDARD" ? squareIconSvg(sq, { board: squares, file, rank }) : null;
      const content = (
        <>
          {icon && <span class="sq-icon" dangerouslySetInnerHTML={{ __html: icon }} />}
          {piece && <BoardPiece symbol={piece} squares={squares} file={file} rank={rank} />}
          {sq.duck && <DuckSprite />}
          {tornado?.includes(":") && <span class="sq-badge sq-tornado-count">{tornado.split(":")[1]}</span>}
          {coords && standard && dr === rows - 1 && <span class="sq-coord sq-coord-file">{"abcdefgh"[file]}</span>}
          {coords && standard && df === 0 && <span class="sq-coord sq-coord-rank">{8 - rank}</span>}
        </>
      );

      if (interactive) {
        cells.push(
          <button
            key={k}
            type="button"
            class={classes.join(" ")}
            aria-label={squareLabel(sq, file, rank, rows)}
            aria-pressed={classes.includes("is-selected")}
            onClick={() => onSquareClick!({ file, rank })}
          >
            {content}
          </button>,
        );
      } else {
        cells.push(
          <div key={k} class={classes.join(" ")}>
            {content}
          </div>,
        );
      }
    }
  }

  return (
    <div
      class={`board-frame${interactive ? " is-interactive" : ""}${extra ? " " + extra : ""}`}
      style={{ "--cols": cols, "--rows": rows }}
    >
      <div class="board" role="group" aria-label={label}>
        {cells}
      </div>
      {children}
    </div>
  );
}

function BoardPiece({ symbol, squares, file, rank }: { symbol: string; squares: Square[][]; file: number; rank: number }) {
  const rotate = isTrainCart(symbol) ? trainCartRotationDegrees(symbol, squares, file, rank) : 0;
  const passengers = getCarrierPassengers(symbol);
  const captive = goblinCaptive(symbol);
  const phase = skibidiPhase(symbol);
  return (
    <>
      <PieceSprite symbol={symbol} rotate={rotate} />
      {passengers.length > 0 && (
        <span class="sq-passengers" data-count={Math.min(passengers.length, 6)}>
          {passengers.slice(0, 6).map((p, i) => (
            <PieceSprite key={i} symbol={p} class="passenger" />
          ))}
        </span>
      )}
      {captive && (
        <span class="sq-captive" title="Kidnapped">
          <PieceSprite symbol={captive} class="passenger" />
        </span>
      )}
      {phase !== null && phase > 1 && <span class="sq-badge sq-phase">{phase}</span>}
    </>
  );
}

function squareLabel(sq: Square, file: number, rank: number, rows: number): string {
  const name = squareName({ file, rank }, rows);
  if (sq.duck) return `${name}, duck`;
  if (!sq.piece) return name;
  const side = pieceSide(sq.piece);
  const def = pieceForSymbol(sq.piece);
  return `${name}, ${side === "neutral" ? "" : side + " "}${def?.name ?? sq.piece}`;
}
