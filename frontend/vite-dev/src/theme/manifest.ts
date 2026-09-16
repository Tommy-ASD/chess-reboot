// src/theme/manifest.ts
//
// The theme registry. A theme is one CSS file in `src/themes/<id>.css`
// plus one entry here — see docs/THEMES.md. The applier (`apply.ts`)
// resolves the id to the CSS file through `import.meta.glob`, so a
// manifest entry without a matching file falls back to the default.

export type ColorScheme = "light" | "dark";

export interface ThemeManifest {
  /** Must match the CSS filename: `src/themes/<id>.css`. */
  id: string;
  name: string;
  author: string;
  /** One or two sentences shown on the theme card. */
  description: string;
  tags: string[];
  /** Drives the native `color-scheme` (scrollbars, form controls) and
   *  which theme "Match system" picks. */
  scheme: ColorScheme;
  /** Three representative colours for the card swatch: [background, accent, secondary]. */
  colors: [string, string, string];
  /** The board's [light square, dark square] for the card's mini board. */
  board: [string, string];
}

export const THEMES: ThemeManifest[] = [
  {
    id: "walnut",
    name: "Walnut",
    author: "Chess 2",
    description: "Warm parchment and polished walnut. A quiet club room with a good board.",
    tags: ["light", "warm", "classic"],
    scheme: "light",
    colors: ["#F6F0E6", "#7A4A24", "#C8963E"],
    board: ["#EEDAB6", "#B58863"],
  },
  {
    id: "walnut-night",
    name: "Walnut Night",
    author: "Chess 2",
    description: "The same club room after hours. Deep browns, amber lamplight, easy on the eyes.",
    tags: ["dark", "warm", "classic"],
    scheme: "dark",
    colors: ["#15110D", "#E0A55E", "#8C6A45"],
    board: ["#C9A97F", "#8A6440"],
  },
  {
    id: "slate",
    name: "Slate",
    author: "Chess 2",
    description: "Cool slate and tournament green. Crisp, modern and high-contrast.",
    tags: ["dark", "cool", "modern"],
    scheme: "dark",
    colors: ["#11161C", "#4FBF8B", "#7A93AD"],
    board: ["#DCE3D5", "#6F9A76"],
  },
];

/** Fallback whenever a requested theme can't be resolved. */
export const DEFAULT_THEME_ID = "walnut";

/** What "Match system" resolves to for each OS colour scheme. */
export const SYSTEM_THEMES: Record<ColorScheme, string> = {
  light: "walnut",
  dark: "walnut-night",
};

export function findTheme(id: string): ThemeManifest | undefined {
  return THEMES.find((t) => t.id === id);
}
