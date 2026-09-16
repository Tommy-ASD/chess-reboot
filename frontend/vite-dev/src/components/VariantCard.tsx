// src/components/VariantCard.tsx
//
// A variant in the gallery: a live preview of its starting position, its
// pitch, and the pieces it introduces. The whole card links to setup.

import { useMemo } from "preact/hooks";
import { Board } from "./board/Board";
import { PieceSprite, DuckSprite } from "./board/PieceSprite";
import { parseFEN } from "../core/fen";
import { pieceById } from "../content/pieces";
import type { Variant } from "../content/variants";

export function VariantCard({ variant, href }: { variant: Variant; href: string }) {
  const squares = useMemo(() => {
    try {
      return parseFEN(variant.fen);
    } catch {
      return null;
    }
  }, [variant.fen]);
  const featured = variant.pieces.map((id) => pieceById(id)).filter((p) => p !== undefined);
  const showFeatured = variant.id !== "classic" && featured.length > 0;

  return (
    <a class="variant-card" href={href}>
      <div class="variant-card-board">
        {squares ? <Board squares={squares} label={`${variant.name} starting position`} /> : <div class="board-placeholder">Invalid position</div>}
      </div>
      <div class="variant-card-body">
        <div class="variant-card-title-row">
          <h3 class="variant-card-title">{variant.name}</h3>
          {variant.status === "experimental" && <span class="chip chip-warn">Experimental</span>}
        </div>
        <p class="variant-card-tagline">{variant.tagline}</p>
        <div class="variant-card-foot">
          {showFeatured ? (
            <span class="variant-card-pieces" aria-label="Featured pieces">
              {featured.map((p) =>
                p.symbols ? <PieceSprite key={p.id} symbol={p.symbols.white} /> : <DuckSprite key={p.id} />,
              )}
              <span class="muted">{featured.map((p) => p.name).join(", ")}</span>
            </span>
          ) : (
            <span class="muted">{variant.source === "community" ? `by ${variant.author}` : "All classic pieces"}</span>
          )}
          <span class="variant-card-cta">Play →</span>
        </div>
      </div>
    </a>
  );
}
