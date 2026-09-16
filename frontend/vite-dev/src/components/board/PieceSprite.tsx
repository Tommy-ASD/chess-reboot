// src/components/board/PieceSprite.tsx
//
// Draws one piece symbol: sprite art where it exists, otherwise a glyph.
// Used on the board and anywhere a piece appears in the UI (cards, the
// piece panel, the Lab palette), so every surface draws pieces the same way.

import { pieceArt, pieceForSymbol, pieceSide } from "../../content/pieces";

interface PieceSpriteProps {
  symbol: string;
  /** Degrees; train carts turn to follow their track. */
  rotate?: number;
  class?: string;
  /** Accessible name; defaults to "<side> <piece name>". */
  label?: string;
}

export function PieceSprite({ symbol, rotate = 0, class: extra, label }: PieceSpriteProps) {
  const art = pieceArt(symbol);
  const side = pieceSide(symbol);
  const def = pieceForSymbol(symbol);
  const name = label ?? `${side === "neutral" ? "" : side + " "}${def?.name ?? symbol}`.trim();
  const cls = `piece piece-${side}${extra ? " " + extra : ""}`;
  const style = rotate ? { transform: `rotate(${rotate}deg)` } : undefined;

  if (art.kind === "image") {
    return (
      <span class={cls} role="img" aria-label={name}>
        <img class="piece-img" src={art.src} alt="" draggable={false} style={style} />
      </span>
    );
  }
  return (
    <span class={`${cls}${art.tinted ? " piece-tinted" : " piece-emoji"}`} role="img" aria-label={name}>
      <span class="piece-glyph" style={style}>
        {art.text}
      </span>
    </span>
  );
}

/// The duck, drawn like a piece.
export function DuckSprite({ class: extra }: { class?: string }) {
  return (
    <span class={`piece piece-neutral piece-emoji${extra ? " " + extra : ""}`} role="img" aria-label="duck">
      <span class="piece-glyph">{"\u{1F986}"}</span>
    </span>
  );
}
