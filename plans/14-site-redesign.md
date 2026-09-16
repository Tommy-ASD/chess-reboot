# Plan 14 — Chess 2 site redesign

Turn the frontend from a FEN-renderer dev tool into a site a newcomer can
land on and play. Branch: `site-redesign` (worktree `worktrees/site-redesign`).

## Decisions (from the design review)

- **Name:** "Chess 2" (for now).
- **Tone:** polished, but user choice first — theming must make adding a
  theme a one-file job. Theme system is modelled on Hearth's
  (`Bonfire/Hearth/docs/THEMES.md`): one CSS file per theme, a manifest,
  per-theme variable overrides, and a custom-CSS layer.
- **Landing:** visitors land on *variants* (official + community-made),
  not on a single game.
- **Identity:** anonymous (existing `X-Player-Id` UUID) for now; accounts
  and persistence later.
- **Framework:** Preact + `@preact/signals`, routing via `preact-iso`.

## Site map

| Route | Purpose |
|---|---|
| `/` | Home: pitch, variant gallery, live games, community variants, pieces strip |
| `/play/:variant` (`/play?fen=&name=` for custom) | Game setup: invite a friend / this device / open lobby |
| `/game/:id` | Online game (seated, invite-join card, or spectating) |
| `/local/:variant` (`/local?fen=`) | Same-device game with undo / restart |
| `/pieces`, `/pieces/:id` | Piece guide, each with a try-it board |
| `/lab` | Position editor (the old editor) + play / save / share a variant |
| `/settings` | Themes, colour customizer, custom CSS, display name, board prefs |

Legacy URLs: `/?game=<id>` / `?code=` redirect to `/game/:id`;
`/editor.html?fen=` redirects to `/lab?fen=`.

## Styling stack (applied in this order)

1. `src/styles/base.css` — structure only; every colour/font is a token.
2. Active theme `src/themes/<id>.css` — defines the tokens, may restyle selectors.
3. `<style id="chess2-theme-vars">` — the user's per-theme colour overrides.
4. `<style id="chess2-custom-css">` — the user's hand-written CSS.

Theme authoring guide: `frontend/vite-dev/docs/THEMES.md`.

## Content registries

- `src/content/pieces.ts` — every piece: name, symbols, rules text, demo position.
- `src/content/variants.ts` — official variants: name, pitch, starting FEN, featured pieces.
- `src/content/userVariants.ts` — variants saved from the Lab (localStorage) +
  shareable variant links (the FEN travels in the URL — no backend needed).

Every surface (home cards, guide, in-game piece panel, Lab palette) reads the
registries, so they can't drift apart.

## Out of scope for this pass

- Unified piece art (sprites are still a mix of glyphs / PNG / SVG).
- Server-side variant sharing, accounts, persistence.
- Theme versioning / sync (Hearth has it; not needed yet).
- An engine-backed "Chess 2" starting position — the official list ships
  Classic, Duck Chess and clearly-labelled experimental showcases.
