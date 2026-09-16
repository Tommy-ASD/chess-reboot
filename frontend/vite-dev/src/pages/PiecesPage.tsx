// src/pages/PiecesPage.tsx
//
// The piece guide index, grouped by family.

import { DuckSprite, PieceSprite } from "../components/board/PieceSprite";
import { FAMILY_LABELS, PIECES, type PieceFamily } from "../content/pieces";

const ORDER: PieceFamily[] = ["chess2", "trains", "extras", "classic"];

export function PiecesPage() {
  return (
    <div class="container page">
      <header class="page-head">
        <p class="eyebrow">Piece guide</p>
        <h1 class="page-title">Meet the pieces</h1>
        <p class="page-lede">What every piece does, in plain words, with a board to try each one on.</p>
      </header>

      {ORDER.map((family) => {
        const pieces = PIECES.filter((p) => p.family === family);
        return (
          <section key={family} class="section-tight">
            <h2 class="section-title">{FAMILY_LABELS[family].title}</h2>
            <p class="section-lede">{FAMILY_LABELS[family].blurb}</p>
            <div class="piece-grid">
              {pieces.map((p) => (
                <a key={p.id} class="piece-card" href={`/pieces/${p.id}`}>
                  <span class="piece-card-sprites">
                    {p.symbols ? (
                      <>
                        <PieceSprite symbol={p.symbols.white} />
                        {p.symbols.black !== p.symbols.white && <PieceSprite symbol={p.symbols.black} />}
                      </>
                    ) : (
                      <DuckSprite />
                    )}
                  </span>
                  <span class="piece-card-body">
                    <span class="piece-card-title">
                      {p.name}
                      {p.status === "experimental" && <span class="chip chip-warn">Experimental</span>}
                    </span>
                    <span class="muted">{p.tagline}</span>
                  </span>
                </a>
              ))}
            </div>
          </section>
        );
      })}
    </div>
  );
}
