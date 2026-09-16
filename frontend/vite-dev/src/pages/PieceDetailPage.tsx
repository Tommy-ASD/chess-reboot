// src/pages/PieceDetailPage.tsx
//
// One piece: rules in plain language, a board to try it on, and the
// variants it appears in.

import { useRoute } from "preact-iso";
import { DuckSprite, PieceSprite } from "../components/board/PieceSprite";
import { DemoBoard } from "../components/game/DemoBoard";
import { FAMILY_LABELS, PIECES, pieceById } from "../content/pieces";
import { OFFICIAL_VARIANTS } from "../content/variants";
import { NotFoundPage } from "./NotFoundPage";

export function PieceDetailPage() {
  const { params } = useRoute();
  const piece = pieceById(params.id);
  if (!piece) return <NotFoundPage title="Piece not found" message="There's no piece by that name in the guide." />;

  const index = PIECES.indexOf(piece);
  const prev = PIECES[index - 1];
  const next = PIECES[index + 1];
  const variants = OFFICIAL_VARIANTS.filter((v) => v.pieces.includes(piece.id));

  return (
    <div class="container page">
      <nav class="breadcrumb" aria-label="Breadcrumb">
        <a href="/pieces">Pieces</a>
        <span aria-hidden="true">/</span>
        <span>{FAMILY_LABELS[piece.family].title}</span>
      </nav>

      <div class="piece-detail">
        <div class="piece-detail-info">
          <header class="piece-detail-head">
            <span class="piece-detail-sprites">
              {piece.symbols ? (
                <>
                  <PieceSprite symbol={piece.symbols.white} />
                  {piece.symbols.black !== piece.symbols.white && <PieceSprite symbol={piece.symbols.black} />}
                </>
              ) : (
                <DuckSprite />
              )}
            </span>
            <div>
              <h1 class="page-title">
                {piece.name}
                {piece.status === "experimental" && <span class="chip chip-warn">Experimental</span>}
              </h1>
              <p class="page-lede">{piece.tagline}</p>
            </div>
          </header>

          <section class="panel">
            <h2 class="panel-title">How it moves</h2>
            <p>{piece.movement}</p>
            <h2 class="panel-title">Rules</h2>
            <ul class="rule-list">
              {piece.rules.map((r) => (
                <li key={r}>{r}</li>
              ))}
            </ul>
            {piece.status === "experimental" && (
              <p class="muted small">Experimental pieces are still being balanced, so details may change.</p>
            )}
          </section>

          {variants.length > 0 && (
            <section class="panel">
              <h2 class="panel-title">Play it in</h2>
              <div class="button-row">
                {variants.map((v) => (
                  <a key={v.id} class="btn btn-secondary" href={`/play/${v.id}`}>
                    {v.name}
                  </a>
                ))}
              </div>
            </section>
          )}

          <nav class="pager" aria-label="More pieces">
            {prev ? (
              <a class="pager-link" href={`/pieces/${prev.id}`}>
                ← {prev.name}
              </a>
            ) : (
              <span />
            )}
            {next && (
              <a class="pager-link" href={`/pieces/${next.id}`}>
                {next.name} →
              </a>
            )}
          </nav>
        </div>

        <div class="piece-detail-demo">
          {piece.demo ? (
            <DemoBoard key={piece.id} fen={piece.demo.fen} hint={piece.demo.hint} />
          ) : (
            <div class="panel empty-state">
              <p class="muted">
                No demo board for the {piece.name.toLowerCase()} yet.
                {variants.length > 0 ? " Try it in a real game instead." : ""}
              </p>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
