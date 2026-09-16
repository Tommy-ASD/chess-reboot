// src/components/game/GameParts.tsx
//
// Building blocks shared by every playable board page: the selected-piece
// panel, the promotion picker, the status banner, player bars and the move
// list. Each takes plain props so online, same-device and demo boards can
// compose them differently.

import type { ComponentChildren } from "preact";
import { useEffect, useRef } from "preact/hooks";
import type { BoardPlay } from "../board/useBoardPlay";
import { DuckSprite, PieceSprite } from "../board/PieceSprite";
import { describeSpecialMove } from "../../core/notation";
import type { StatusText } from "../../core/status";
import type { Color, GameMove } from "../../core/types";
import { pieceForSymbol, pieceSide } from "../../content/pieces";

/// What the selected piece is and what it can do beyond board clicks.
export function PiecePanel({ play, canMove, rows }: { play: BoardPlay; canMove: boolean; rows: number }) {
  if (play.duckPlacing) {
    return (
      <section class="panel piece-panel" aria-live="polite">
        <div class="piece-panel-head">
          <DuckSprite class="piece-panel-sprite" />
          <div>
            <h3 class="panel-title">{canMove ? "Place the duck" : "Waiting for the duck"}</h3>
            <p class="muted">
              {canMove
                ? "Tap any highlighted empty square. Nothing can pass the duck."
                : "The player to move is placing the duck."}
            </p>
          </div>
        </div>
      </section>
    );
  }

  const symbol = play.selectedPiece;
  if (!symbol) {
    return (
      <section class="panel piece-panel is-empty">
        <p class="muted">Tap a piece to see what it does{canMove ? " and where it can go" : ""}.</p>
      </section>
    );
  }

  const def = pieceForSymbol(symbol);
  return (
    <section class="panel piece-panel" aria-live="polite">
      <div class="piece-panel-head">
        <PieceSprite symbol={symbol} class="piece-panel-sprite" />
        <div>
          <h3 class="panel-title">{def?.name ?? symbol}</h3>
          {def && <p class="muted">{def.tagline}</p>}
        </div>
      </div>
      {def && (
        <p class="piece-panel-moves">
          <strong>Moves:</strong> {def.movement}{" "}
          <a href={`/pieces/${def.id}`} target="_blank" rel="noopener">
            Full rules
          </a>
        </p>
      )}

      {play.passengers.length > 0 && (
        <div class="passenger-picker">
          <span class="field-label">Who moves?</span>
          <div class="passenger-row">
            <button
              type="button"
              class={`passenger-tile${play.passengerIndex === null ? " is-active" : ""}`}
              onClick={() => play.setPassengerIndex(null)}
            >
              <PieceSprite symbol={symbol} />
              <span>Drive</span>
            </button>
            {play.passengers.map((p, i) => (
              <button
                key={i}
                type="button"
                class={`passenger-tile${play.passengerIndex === i ? " is-active" : ""}`}
                onClick={() => play.setPassengerIndex(i)}
              >
                <PieceSprite symbol={p} />
                <span>Get off</span>
              </button>
            ))}
          </div>
        </div>
      )}

      {canMove && play.specials.length > 0 && (
        <div class="special-actions">
          {play.specials.map((m, i) => (
            <button key={i} type="button" class="btn btn-secondary" disabled={play.busy} onClick={() => play.play(m)}>
              {describeSpecialMove(m, rows)}
            </button>
          ))}
        </div>
      )}
    </section>
  );
}

/// Pick a promotion piece. Rendered as a Board overlay.
export function PromotionPicker({ moves, pawn, onPick, onCancel }: {
  moves: GameMove[];
  pawn: string;
  onPick: (m: GameMove) => void;
  onCancel: () => void;
}) {
  const white = pieceSide(pawn) === "white";
  const symbols = { Queen: "Q", Rook: "R", Bishop: "B", Knight: "N" } as const;
  const first = useRef<HTMLButtonElement>(null);
  useEffect(() => first.current?.focus(), []);
  return (
    <div class="board-overlay" role="dialog" aria-label="Choose a promotion piece" onKeyDown={(e) => e.key === "Escape" && onCancel()}>
      <div class="overlay-card promotion-card">
        <h3 class="panel-title">Promote to</h3>
        <div class="promotion-choices">
          {moves.map((m, i) => {
            if (m.move_type.kind !== "Promotion") return null;
            const into = m.move_type.target.into;
            const s = symbols[into];
            return (
              <button key={into} ref={i === 0 ? first : undefined} type="button" class="promotion-choice" title={into} onClick={() => onPick(m)}>
                <PieceSprite symbol={white ? s : s.toLowerCase()} />
              </button>
            );
          })}
        </div>
        <button type="button" class="btn btn-ghost btn-sm" onClick={onCancel}>
          Cancel
        </button>
      </div>
    </div>
  );
}

export function StatusBanner({ text }: { text: StatusText | null }) {
  if (!text) return null;
  return (
    <div class={`status-banner status-${text.kind}`} role="status">
      <strong>{text.title}</strong>
      {text.detail && <span>{text.detail}</span>}
    </div>
  );
}

export function PlayerBar({ color, name, toMove, isYou, winner, children }: {
  color: Color;
  name: string;
  toMove: boolean;
  isYou?: boolean;
  winner?: boolean;
  children?: ComponentChildren;
}) {
  return (
    <div class={`player-bar${toMove ? " is-to-move" : ""}${winner ? " is-winner" : ""}`}>
      <span class={`player-dot player-dot-${color.toLowerCase()}`} aria-hidden="true" />
      <span class="player-name">
        {name}
        {isYou && <span class="chip chip-soft">you</span>}
      </span>
      {children}
      {winner && <span class="chip chip-success">Winner</span>}
      {toMove && !winner && <span class="to-move-label">to move</span>}
    </div>
  );
}

export function MoveList({ moves, empty = "No moves yet." }: { moves: string[]; empty?: string }) {
  const list = useRef<HTMLOListElement>(null);
  useEffect(() => {
    if (list.current) list.current.scrollTop = list.current.scrollHeight;
  }, [moves.length]);
  return (
    <section class="panel move-list-panel">
      <h3 class="panel-title">Moves</h3>
      {moves.length === 0 ? (
        <p class="muted">{empty}</p>
      ) : (
        <ol class="move-list" ref={list}>
          {moves.map((m, i) => (
            <li key={i}>{m}</li>
          ))}
        </ol>
      )}
    </section>
  );
}
