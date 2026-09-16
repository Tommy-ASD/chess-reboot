# Writing a Chess 2 theme

A theme is **one CSS file plus one manifest entry**:

```
src/themes/your-theme-id.css   ← tokens (+ optional selector overrides)
src/theme/manifest.ts          ← add an entry to THEMES
```

Nothing else needs to change. The theme picker in Settings, the colour
customizer and "Match system" all read the manifest, and the applier finds
the CSS file by id.

## Built-in themes

| Id | Name | Scheme | Notes |
|---|---|---|---|
| `walnut` | Walnut | light | Default light. Parchment, walnut, Palatino display type |
| `walnut-night` | Walnut Night | dark | Default dark. Same family, amber accent |
| `slate` | Slate | dark | Cool slate + tournament green, sans display type, tighter corners |

Start by copying whichever is closest.

## How styles are layered

Applied in this order; each layer overrides only what it sets:

1. `src/styles/base.css` (+ `base/*.css`): layout and structure. It reads
   **only** tokens for colour, type, radii and shadows.
2. The active theme: `<link id="chess2-theme">` → `src/themes/<id>.css`.
3. The user's colour tweaks for that theme: `<style id="chess2-theme-vars">`.
4. The user's custom CSS: `<style id="chess2-custom-css">`.

Only the active theme's file is linked, so themes can define tokens on
`:root` without scoping.

## 1. Add a manifest entry

```ts
// src/theme/manifest.ts
{
  id: "your-theme-id",            // must match the CSS filename
  name: "Your Theme",
  author: "your-handle",
  description: "One or two sentences shown on the theme card.",
  tags: ["dark", "cool"],
  scheme: "dark",                 // "light" | "dark": native controls + Match system
  colors: ["#101418", "#4FBF8B", "#7A93AD"], // card swatch: [background, accent, secondary]
  board: ["#DCE3D5", "#6F9A76"],  // card mini-board: [light square, dark square]
},
```

To make it a "Match system" default, point `SYSTEM_THEMES.light` or
`SYSTEM_THEMES.dark` at its id.

## 2. Create the CSS file

Every token below is required. The canonical list lives in
`src/theme/vars.ts` (`THEME_VAR_GROUPS` + `THEME_SHAPE_VARS`).

```css
:root {
  /* Surfaces */
  --page-bg:  #…;   /* any CSS background: colour, gradient(s), image */
  --bg:       #…;   /* solid fallback / page colour */
  --surface:  #…;   /* cards, panels, inputs */
  --surface2: #…;   /* raised / hover surfaces */
  --surface3: #…;   /* sunken areas, alternating sections */

  /* Lines */
  --border:  …;     /* subtle */
  --border2: …;     /* stronger: inputs, secondary buttons */

  /* Accent */
  --accent:      #…;  /* primary buttons, links, active states */
  --accent2:     #…;  /* hover for the above */
  --accent-text: #…;  /* text on an --accent background */
  --accent-soft: …;   /* translucent accent tint: selections, chips */
  --highlight:   #…;  /* secondary pop colour */

  /* Text */
  --text:  #…;
  --text2: #…;  /* secondary copy */
  --muted: #…;  /* labels, hints, placeholders */

  /* Status */
  --success: #…;
  --warning: #…;
  --danger:  #…;

  /* Board */
  --board-light:   #…;
  --board-dark:    #…;
  --board-frame:   #…;
  --piece-white:   #…;  /* fill of white glyph pieces */
  --piece-black:   #…;  /* fill of black glyph pieces */
  --piece-outline: …;   /* outline around white pieces */
  --move-hint:     …;   /* legal-move dots and capture rings (translucent) */
  --selected:      …;   /* selected square tint (translucent) */
  --last-move:     …;   /* last move tint (translucent) */

  /* Type */
  --font-body:    …;
  --font-display: …;    /* headings, titles */
  --font-mono:    …;    /* FENs, codes */

  /* Shape */
  --radius:    …;
  --radius-sm: …;
  --shadow-sm: …;
  --shadow:    …;
  --shadow-lg: …;
}
```

### Going further than tokens

Below the tokens a theme may restyle **any selector**. The built-ins use this
for small flourishes (`.brand-mark` glow in Walnut Night, heavier headings in
Slate). Stable hooks worth knowing:

| Area | Selectors |
|---|---|
| Shell | `.topbar`, `.brand-mark`, `.brand-name`, `.topnav-link`, `.footer` |
| Type | `.hero-title`, `.page-title`, `.section-title`, `.eyebrow` |
| Controls | `.btn`, `.btn-primary`, `.btn-secondary`, `.panel`, `.chip`, `.input` |
| Board | `.board-frame`, `.board`, `.sq-light`, `.sq-dark`, `.sq.is-selected`, `.sq.is-target`, `.piece-glyph` |
| Cards | `.variant-card`, `.piece-card`, `.option-card` |

`html[data-theme="<id>"]` is also set, for rules that must only apply under a
specific theme (useful in custom CSS).

## Tips

- **Check both boards.** Glyph pieces sit on both square colours; make sure
  `--piece-black` reads on `--board-dark` and `--piece-white` on `--board-light`.
- **Keep hints translucent.** `--move-hint`, `--selected` and `--last-move`
  are painted over the squares.
- **Test the dark-on-accent case.** Primary buttons and the "Your move"
  callout put `--accent-text` on `--accent`.
- **Fonts:** use system stacks or fonts you ship with the app; themes
  shouldn't make network requests.

## What users can do without a theme file

- **Settings → Customize:** colour pickers for every colour token, stored per
  theme in localStorage (`chess2.theme-overrides`).
- **Settings → Custom CSS:** arbitrary CSS applied last
  (`chess2.custom-css`).

## Not yet

Hearth-style saved custom themes (named override sets with version history)
and theme sharing are natural next steps; the override store is already keyed
by theme id, so a saved theme is "base theme id + overrides".
