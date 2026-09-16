// src/components/game/DemoBoard.tsx
//
// A small sandbox game for trying a piece: real engine moves, both sides
// controlled by the viewer, one-click reset.

import { Board } from "../board/Board";
import { useBoardPlay } from "../board/useBoardPlay";
import { PiecePanel, PromotionPicker, StatusBanner } from "./GameParts";
import { useLocalGame } from "./useLocalGame";
import { showCoordinates } from "../../app/prefs";
import { describeStatus, isTerminal } from "../../core/status";

export function DemoBoard({ fen, hint }: { fen: string; hint: string }) {
  const game = useLocalGame(fen);
  const over = isTerminal(game.status);
  const play = useBoardPlay({ fen: game.fen, canMove: !over, submit: game.submit });
  if (!play.squares) return <div class="notice notice-danger">This demo's position couldn't be read.</div>;
  const toMove = play.flags.sideToMove === "w" ? "White" : "Black";

  return (
    <div class="demo">
      <p class="demo-hint">
        <strong>Try it:</strong> {hint}
      </p>
      <Board
        squares={play.squares}
        selected={play.selected}
        targets={play.targets}
        lastMove={play.lastMove}
        emptyTargets={play.duckPlacing}
        coords={showCoordinates.value}
        onSquareClick={play.onSquareClick}
        label="Demo board"
      >
        {play.promotion && play.selectedPiece && (
          <PromotionPicker moves={play.promotion} pawn={play.selectedPiece} onPick={play.play} onCancel={play.cancelPromotion} />
        )}
      </Board>
      <div class="demo-bar">
        <span class="muted">
          {over ? "Game over." : `${toMove} to move`} · you play both sides
        </span>
        <span class="button-row">
          <button type="button" class="btn btn-ghost btn-sm" disabled={!game.canUndo} onClick={game.undo}>
            Undo
          </button>
          <button type="button" class="btn btn-secondary btn-sm" disabled={!game.canUndo} onClick={game.restart}>
            Reset
          </button>
        </span>
      </div>
      <StatusBanner text={describeStatus(game.status)} />
      <PiecePanel play={play} canMove={!over} rows={play.squares.length} />
    </div>
  );
}
