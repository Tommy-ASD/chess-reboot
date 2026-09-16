// src/pages/LocalGamePage.tsx
//
// Pass-and-play on one device. Moves go through the stateless engine API;
// the position history lives in the page, so undo and restart are free.

import { useState } from "preact/hooks";
import { useRoute } from "preact-iso";
import { Board } from "../components/board/Board";
import { useBoardPlay } from "../components/board/useBoardPlay";
import { MoveList, PiecePanel, PlayerBar, PromotionPicker, StatusBanner } from "../components/game/GameParts";
import { useLocalGame } from "../components/game/useLocalGame";
import { autoFlipLocal, showCoordinates, showMoveHints } from "../app/prefs";
import { describeStatus, isTerminal, statusWinner } from "../core/status";
import type { Orientation } from "../core/types";
import { resolveVariant, variantQuery } from "../content/userVariants";
import { NotFoundPage } from "./NotFoundPage";

export function LocalGamePage() {
  const { params, query } = useRoute();
  const variant = resolveVariant(params.variant, query);
  if (!variant) {
    return <NotFoundPage title="Variant not found" message="That variant doesn't exist, or its link is incomplete." />;
  }
  return <LocalGame key={variant.fen} name={variant.name} fen={variant.fen} setupHref={params.variant ? `/play/${params.variant}` : `/play?${variantQuery(variant)}`} />;
}

function LocalGame({ name, fen: startFen, setupHref }: { name: string; fen: string; setupHref: string }) {
  const game = useLocalGame(startFen);
  const over = isTerminal(game.status);
  const play = useBoardPlay({ fen: game.fen, canMove: !over, submit: game.submit });
  const [manual, setManual] = useState<Orientation>("white");

  const toMove = play.flags.sideToMove === "w" ? "White" : "Black";
  const orientation: Orientation = autoFlipLocal.value ? (toMove === "White" ? "white" : "black") : manual;
  const top = orientation === "white" ? "Black" : "White";
  const bottom = top === "White" ? "Black" : "White";
  const statusText = describeStatus(game.status);
  const winner = statusWinner(game.status);
  const rows = play.squares?.length ?? 8;

  if (!play.squares) {
    return <NotFoundPage title="Can't read this position" message="The variant's starting position isn't valid." />;
  }

  return (
    <div class="container page game-page">
      <div class="game-layout">
        <div class="game-board-col">
          <PlayerBar color={top} name={top} toMove={!over && toMove === top} winner={winner === top} />
          <Board
            squares={play.squares}
            orientation={orientation}
            selected={play.selected}
            targets={showMoveHints.value ? play.targets : []}
            lastMove={play.lastMove}
            emptyTargets={play.duckPlacing && !over}
            coords={showCoordinates.value}
            onSquareClick={play.onSquareClick}
            label={`${name} board`}
            class="game-board"
          >
            {play.promotion && play.selectedPiece && (
              <PromotionPicker moves={play.promotion} pawn={play.selectedPiece} onPick={play.play} onCancel={play.cancelPromotion} />
            )}
          </Board>
          <PlayerBar color={bottom} name={bottom} toMove={!over && toMove === bottom} winner={winner === bottom} />
        </div>

        <aside class="game-side">
          <header class="game-side-head">
            <div>
              <p class="eyebrow">On this device</p>
              <h1 class="game-title">{name}</h1>
            </div>
            <a class="btn btn-ghost btn-sm" href={setupHref}>
              Play online instead
            </a>
          </header>

          <StatusBanner text={statusText} />
          <PiecePanel play={play} canMove={!over} rows={rows} />

          <div class="button-row">
            <button type="button" class="btn btn-secondary" disabled={!game.canUndo || play.busy} onClick={game.undo}>
              Undo
            </button>
            <button
              type="button"
              class="btn btn-secondary"
              disabled={autoFlipLocal.value}
              title={autoFlipLocal.value ? "Auto-flip is on" : undefined}
              onClick={() => setManual(manual === "white" ? "black" : "white")}
            >
              Flip board
            </button>
            <button
              type="button"
              class="btn btn-ghost"
              disabled={!game.canUndo}
              onClick={() => {
                if (confirm("Restart from the starting position?")) game.restart();
              }}
            >
              Restart
            </button>
          </div>
          <label class="inline-check">
            <input type="checkbox" checked={autoFlipLocal.value} onChange={(e) => (autoFlipLocal.value = e.currentTarget.checked)} />
            Turn the board to face whoever is to move
          </label>

          <MoveList moves={game.moves} />
        </aside>
      </div>
    </div>
  );
}
