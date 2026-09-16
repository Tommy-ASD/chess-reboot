// src/pages/PlaySetupPage.tsx
//
// `/play/<variant>`: "How do you want to play this variant?" Invite a friend
// (private online game + link), play on this device, or open the game to
// the lobby. The breadcrumb's variant name switches between built-ins.
//
// `/play/` (no variant): the variant index to choose from.

import { useMemo, useState } from "preact/hooks";
import { useLocation, useRoute } from "preact-iso";
import { Board } from "../components/board/Board";
import { DuckSprite, PieceSprite } from "../components/board/PieceSprite";
import { CopyField, Segmented } from "../components/ui/Controls";
import { VariantCard } from "../components/VariantCard";
import { VariantSwitcher } from "../components/VariantSwitcher";
import { displayName } from "../app/prefs";
import * as online from "../api/online";
import { parseFEN } from "../core/fen";
import type { Color } from "../core/types";
import { pieceById } from "../content/pieces";
import { resolveVariant, savedVariants, shareUrl, toVariant, variantQuery } from "../content/userVariants";
import { OFFICIAL_VARIANTS } from "../content/variants";
import { showError } from "../ui/toast";
import { NotFoundPage } from "./NotFoundPage";

type SeatChoice = "random" | "White" | "Black";

export function PlaySetupPage() {
  const { params, query } = useRoute();
  if (!params.variant && !query.fen) return <VariantIndex />;
  return <PlaySetup />;
}

function VariantIndex() {
  const mine = savedVariants.value;
  return (
    <div class="container page">
      <header class="page-head">
        <p class="eyebrow">Play</p>
        <h1 class="page-title">Choose a variant</h1>
        <p class="page-lede">Start with Classic, or pick something stranger. You'll choose how to play on the next screen.</p>
      </header>
      <div class="variant-grid">
        {OFFICIAL_VARIANTS.map((v) => (
          <VariantCard key={v.id} variant={v} href={`/play/${v.id}`} />
        ))}
      </div>

      <section class="section-tight">
        <h2 class="section-title">Your variants</h2>
        {mine.length === 0 ? (
          <p class="section-lede">
            Nothing saved yet. <a href="/lab">Build one in the Lab</a> and it will show up here.
          </p>
        ) : (
          <div class="variant-grid variant-grid-spaced">
            {mine.map((v) => (
              <VariantCard key={v.id} variant={toVariant(v)} href={`/play/${v.id}`} />
            ))}
          </div>
        )}
      </section>
    </div>
  );
}

function PlaySetup() {
  const { params, query } = useRoute();
  const { route } = useLocation();
  const variant = resolveVariant(params.variant, query);
  const [seat, setSeat] = useState<SeatChoice>("random");
  const [creating, setCreating] = useState<"invite" | "open" | null>(null);

  const squares = useMemo(() => {
    if (!variant) return null;
    try {
      return parseFEN(variant.fen);
    } catch {
      return null;
    }
  }, [variant?.fen]);

  if (!variant) {
    return <NotFoundPage title="Variant not found" message="That variant doesn't exist, or its link is incomplete." />;
  }

  const isShared = !params.variant;
  const localHref = isShared ? `/local?${variantQuery({ name: variant.name, fen: variant.fen, author: variant.author, description: variant.description })}` : `/local/${variant.id}`;
  const pieces = variant.pieces.map((id) => pieceById(id)).filter((p) => p !== undefined);

  const create = async (kind: "invite" | "open") => {
    const color: Color = seat === "random" ? (Math.random() < 0.5 ? "White" : "Black") : seat;
    setCreating(kind);
    try {
      const state = await online.createGame({
        name: variant.name,
        public: kind === "open",
        starting_fen: variant.fen,
        color,
      });
      route(`/game/${state.id}`);
    } catch (err) {
      showError(err);
      setCreating(null);
    }
  };

  return (
    <div class="container page">
      <nav class="breadcrumb" aria-label="Breadcrumb">
        <a href="/play/">Variants</a>
        <span aria-hidden="true">/</span>
        <VariantSwitcher current={variant} />
      </nav>

      <div class="setup-grid">
        <div class="setup-board">
          {squares ? (
            <Board squares={squares} orientation={seat === "Black" ? "black" : "white"} label={`${variant.name} starting position`} />
          ) : (
            <div class="notice notice-danger">This variant's position couldn't be read.</div>
          )}
        </div>

        <div class="setup-info">
          <header class="page-head">
            <h1 class="page-title">
              {variant.name}
              {variant.status === "experimental" && <span class="chip chip-warn">Experimental</span>}
            </h1>
            <p class="page-lede">{variant.description || variant.tagline}</p>
            {variant.source === "community" && (params.variant || query.by) && <p class="muted">Community variant by {variant.author}</p>}
          </header>

          {variant.rules.length > 0 && (
            <section class="panel">
              <h2 class="panel-title">How it's different</h2>
              <ul class="rule-list">
                {variant.rules.map((r) => (
                  <li key={r}>{r}</li>
                ))}
              </ul>
              {variant.id !== "classic" && pieces.length > 0 && (
                <div class="know-pieces">
                  {pieces.map((p) => (
                    <a key={p.id} class="piece-chip piece-chip-compact" href={`/pieces/${p.id}`} target="_blank" rel="noopener">
                      {p.symbols ? <PieceSprite symbol={p.symbols.white} /> : <DuckSprite />}
                      <span>
                        <strong>{p.name}</strong>
                        <span class="muted">How it works ↗</span>
                      </span>
                    </a>
                  ))}
                </div>
              )}
            </section>
          )}

          <section class="panel setup-options">
            <h2 class="panel-title">How do you want to play?</h2>

            <div class="field-row">
              <label class="field">
                <span class="field-label">Your name</span>
                <input
                  class="input"
                  value={displayName.value}
                  maxLength={24}
                  placeholder="Anonymous"
                  autoComplete="nickname"
                  onInput={(e) => (displayName.value = e.currentTarget.value)}
                />
              </label>
              <div class="field">
                <span class="field-label">Play as</span>
                <Segmented<SeatChoice>
                  label="Play as"
                  value={seat}
                  onChange={setSeat}
                  options={[
                    { value: "random", label: "Random" },
                    { value: "White", label: "White" },
                    { value: "Black", label: "Black" },
                  ]}
                />
              </div>
            </div>

            <div class="option-cards">
              <button type="button" class="option-card is-primary" disabled={creating !== null} onClick={() => create("invite")}>
                <span class="option-card-title">{creating === "invite" ? "Creating game…" : "Invite a friend"}</span>
                <span class="option-card-text">Get a link to send. The game starts when they open it.</span>
              </button>
              <a class="option-card" href={localHref}>
                <span class="option-card-title">Play on this device</span>
                <span class="option-card-text">Pass and play: take turns on one screen.</span>
              </a>
              <button type="button" class="option-card" disabled={creating !== null} onClick={() => create("open")}>
                <span class="option-card-title">{creating === "open" ? "Creating game…" : "Open to anyone"}</span>
                <span class="option-card-text">List it under “Happening now” so anyone can join.</span>
              </button>
            </div>
          </section>

          {variant.source === "community" && (
            <section class="panel">
              <CopyField label="Share this variant" value={shareUrl({ name: variant.name, fen: variant.fen, author: variant.author, description: variant.description })} />
            </section>
          )}
        </div>
      </div>
    </div>
  );
}
