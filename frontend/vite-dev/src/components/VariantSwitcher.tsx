// src/components/VariantSwitcher.tsx
//
// The current variant's name as a dropdown: pick any built-in variant to
// switch the page to it, or paste a FEN to play a custom position. Lives in
// the play-setup breadcrumb.

import { useEffect, useLayoutEffect, useRef, useState } from "preact/hooks";
import { useLocation } from "preact-iso";
import { DuckSprite, PieceSprite } from "./board/PieceSprite";
import { fetchStatus } from "../api/board";
import { parseFEN } from "../core/fen";
import { pieceById } from "../content/pieces";
import { variantQuery } from "../content/userVariants";
import { OFFICIAL_VARIANTS, type Variant } from "../content/variants";

export function VariantSwitcher({ current }: { current: Variant }) {
  const { route } = useLocation();
  const [open, setOpen] = useState(false);
  const root = useRef<HTMLDivElement>(null);
  const button = useRef<HTMLButtonElement>(null);
  const options = useRef<(HTMLButtonElement | null)[]>([]);
  const pasteButton = useRef<HTMLButtonElement>(null);
  const fenInput = useRef<HTMLInputElement>(null);
  const [pasting, setPasting] = useState(false);
  const [fen, setFen] = useState("");
  const [fenError, setFenError] = useState("");
  const [checking, setChecking] = useState(false);

  useLayoutEffect(() => {
    if (pasting) fenInput.current?.focus();
  }, [pasting]);

  const close = (refocus: boolean) => {
    setOpen(false);
    setPasting(false);
    setFenError("");
    if (refocus) button.current?.focus();
  };

  const startPasting = async () => {
    setPasting(true);
    setFenError("");
    // Opening the box is a fresh paste: prefill from the clipboard when the
    // browser allows it (replacing any old draft); otherwise the user pastes
    // into the box.
    try {
      const text = (await navigator.clipboard.readText()).trim();
      if (text) setFen(text);
    } catch {
      // Permission denied or unsupported: nothing to prefill.
    }
  };

  const submitFen = async () => {
    const text = fen.trim();
    if (!text) return;
    try {
      const board = parseFEN(text);
      if (board.length === 0 || (board[0]?.length ?? 0) === 0) throw new Error();
    } catch {
      setFenError("That doesn't look like a FEN.");
      fenInput.current?.focus();
      return;
    }
    // The engine is the authority on whether the position is playable.
    setChecking(true);
    try {
      await fetchStatus(text);
    } catch (err) {
      const reason = err instanceof Error ? err.message : String(err);
      setFenError(/reach the game server/.test(reason) ? reason : `The engine can't play that position: ${reason}`);
      setChecking(false);
      // The Play button was disabled while checking, which drops focus.
      fenInput.current?.focus();
      return;
    }
    setChecking(false);
    close(true);
    setFen("");
    route(`/play?${variantQuery({ name: "Custom position", fen: text })}`);
  };

  // Close on outside click.
  useEffect(() => {
    if (!open) return;
    const onDown = (e: PointerEvent) => {
      if (!root.current?.contains(e.target as Node)) setOpen(false);
    };
    document.addEventListener("pointerdown", onDown);
    return () => document.removeEventListener("pointerdown", onDown);
  }, [open]);

  // On open, focus the current variant (or the first option) — right after
  // render, and only if focus hasn't already moved into the menu.
  useLayoutEffect(() => {
    if (!open || document.activeElement !== button.current) return;
    const i = OFFICIAL_VARIANTS.findIndex((v) => v.id === current.id);
    // A pasted / shared position isn't a built-in: start on "Paste FEN".
    (i === -1 && current.id === "shared" ? pasteButton.current : options.current[Math.max(0, i)])?.focus();
  }, [open]);

  const pick = (v: Variant) => {
    close(true);
    if (v.id !== current.id || current.source !== "official") route(`/play/${v.id}`);
  };

  const onMenuKey = (e: KeyboardEvent) => {
    // Inside the FEN box: leave arrows / Home / End to the input; Escape
    // closes just the box.
    if (e.target instanceof Element && e.target.closest(".variant-paste")) {
      if (e.key === "Escape") {
        e.preventDefault();
        setPasting(false);
        setFenError("");
        pasteButton.current?.focus();
      }
      return;
    }
    const items = [...options.current, pasteButton.current].filter((o): o is HTMLButtonElement => o !== null);
    const at = items.indexOf(document.activeElement as HTMLButtonElement);
    const move = (to: number) => {
      e.preventDefault();
      items[(to + items.length) % items.length]?.focus();
    };
    if (e.key === "ArrowDown") move(at + 1);
    else if (e.key === "ArrowUp") move(at - 1);
    else if (e.key === "Home") move(0);
    else if (e.key === "End") move(items.length - 1);
    else if (e.key === "Escape") {
      e.preventDefault();
      close(true);
    } else if (e.key === "Tab") setOpen(false);
  };

  return (
    <div class="variant-switcher" ref={root}>
      <button
        ref={button}
        type="button"
        class={`variant-switcher-button${open ? " is-open" : ""}`}
        aria-haspopup="true"
        aria-expanded={open}
        aria-label={`Variant: ${current.name}. Choose another variant`}
        onClick={() => setOpen(!open)}
        onKeyDown={(e) => {
          if (e.key !== "ArrowDown" && e.key !== "ArrowUp") return;
          e.preventDefault();
          if (!open) setOpen(true);
          // Already open (keys can outrun the focus-on-open effect): step into the menu.
          else options.current[e.key === "ArrowDown" ? 0 : OFFICIAL_VARIANTS.length - 1]?.focus();
        }}
      >
        <span>{current.name}</span>
        <svg class="variant-switcher-chevron" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
          <path d="m6 9 6 6 6-6" />
        </svg>
      </button>

      {open && (
        <div class="variant-menu" role="menu" aria-label="Built-in variants" onKeyDown={onMenuKey}>
          <p class="variant-menu-heading field-label">Built-in variants</p>
          {OFFICIAL_VARIANTS.map((v, i) => {
            const selected = v.id === current.id && current.source === "official";
            const featured = v.id === "classic" ? [] : v.pieces.map((id) => pieceById(id)).filter((p) => p !== undefined);
            return (
              <button
                key={v.id}
                ref={(el) => {
                  options.current[i] = el;
                }}
                type="button"
                role="menuitemradio"
                aria-checked={selected}
                class={`variant-option${selected ? " is-selected" : ""}`}
                onClick={() => pick(v)}
              >
                <span class="variant-option-icons" aria-hidden="true">
                  {featured.length > 0 ? (
                    featured.slice(0, 2).map((p) => (p.symbols ? <PieceSprite key={p.id} symbol={p.symbols.white} /> : <DuckSprite key={p.id} />))
                  ) : (
                    <PieceSprite symbol="N" />
                  )}
                </span>
                <span class="variant-option-text">
                  <span class="variant-option-name">
                    {v.name}
                    {v.status === "experimental" && <span class="chip chip-warn">Experimental</span>}
                  </span>
                  <span class="variant-option-tagline">{v.tagline}</span>
                </span>
                <svg class="variant-option-check" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
                  <path d="M20 6 9 17l-5-5" />
                </svg>
              </button>
            );
          })}
          <div class="variant-menu-divider" role="separator" />
          <button
            ref={pasteButton}
            type="button"
            role="menuitem"
            aria-expanded={pasting}
            class={`variant-option variant-option-paste${pasting || current.id === "shared" ? " is-selected" : ""}`}
            onClick={() => (pasting ? setPasting(false) : startPasting())}
          >
            <span class="variant-option-icons" aria-hidden="true">
              <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <rect x="8" y="2" width="8" height="4" rx="1" />
                <path d="M16 4h2a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2h2" />
              </svg>
            </span>
            <span class="variant-option-text">
              <span class="variant-option-name">Paste FEN</span>
              <span class="variant-option-tagline">Play any position, from the Lab or anywhere else.</span>
            </span>
          </button>
          {pasting && (
            <form
              class="variant-paste"
              onSubmit={(e) => {
                e.preventDefault();
                submitFen();
              }}
            >
              <input
                class={`input input-mono input-sm${fenError ? " is-invalid" : ""}`}
                placeholder="rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq -"
                aria-label="FEN"
                aria-invalid={Boolean(fenError)}
                spellcheck={false}
                autoComplete="off"
                value={fen}
                ref={fenInput}
                onInput={(e) => {
                  setFen(e.currentTarget.value);
                  setFenError("");
                }}
              />
              <button type="submit" class="btn btn-primary btn-sm" disabled={!fen.trim() || checking}>
                {checking ? "Checking…" : "Play"}
              </button>
              {fenError && <p class="field-error">{fenError}</p>}
            </form>
          )}
          <a class="variant-menu-all" href="/play/" onClick={() => setOpen(false)}>
            Compare all variants →
          </a>
        </div>
      )}
    </div>
  );
}
