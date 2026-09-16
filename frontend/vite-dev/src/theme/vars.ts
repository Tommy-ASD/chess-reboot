// src/theme/vars.ts
//
// The theme token contract: every CSS custom property a theme must define,
// grouped for the Settings customizer. `src/styles/base/*.css` reads only
// these tokens for colour, type and shape, so a theme that defines them all
// styles the whole site. Keep this list, docs/THEMES.md and the built-in
// theme files in sync.

export interface VarMeta {
  /** CSS custom property name, e.g. `--accent`. */
  name: string;
  label: string;
}

export interface VarGroup {
  label: string;
  vars: VarMeta[];
}

/** Colour tokens exposed in the customizer, in display order. */
export const THEME_VAR_GROUPS: VarGroup[] = [
  {
    label: "Surfaces",
    vars: [
      { name: "--bg", label: "Page" },
      { name: "--surface", label: "Card" },
      { name: "--surface2", label: "Raised" },
      { name: "--surface3", label: "Sunken" },
    ],
  },
  {
    label: "Lines",
    vars: [
      { name: "--border", label: "Border" },
      { name: "--border2", label: "Border strong" },
    ],
  },
  {
    label: "Accent",
    vars: [
      { name: "--accent", label: "Primary" },
      { name: "--accent2", label: "Hover" },
      { name: "--accent-text", label: "Text on primary" },
      { name: "--accent-soft", label: "Tint" },
      { name: "--highlight", label: "Highlight" },
    ],
  },
  {
    label: "Text",
    vars: [
      { name: "--text", label: "Primary" },
      { name: "--text2", label: "Secondary" },
      { name: "--muted", label: "Muted" },
    ],
  },
  {
    label: "Status",
    vars: [
      { name: "--success", label: "Success" },
      { name: "--warning", label: "Warning" },
      { name: "--danger", label: "Danger" },
    ],
  },
  {
    label: "Board",
    vars: [
      { name: "--board-light", label: "Light squares" },
      { name: "--board-dark", label: "Dark squares" },
      { name: "--board-frame", label: "Frame" },
      { name: "--piece-white", label: "White pieces" },
      { name: "--piece-black", label: "Black pieces" },
      { name: "--move-hint", label: "Move hints" },
      { name: "--selected", label: "Selected square" },
      { name: "--last-move", label: "Last move" },
    ],
  },
];

/** Non-colour tokens a theme must also define (not in the customizer). */
export const THEME_SHAPE_VARS = [
  "--page-bg",
  "--piece-outline",
  "--font-body",
  "--font-display",
  "--font-mono",
  "--radius",
  "--radius-sm",
  "--shadow-sm",
  "--shadow",
  "--shadow-lg",
] as const;

export const ALL_COLOR_VARS: string[] = THEME_VAR_GROUPS.flatMap((g) => g.vars.map((v) => v.name));

/// Read a token's live computed value from the document root.
export function readCssVar(name: string): string {
  return getComputedStyle(document.documentElement).getPropertyValue(name).trim();
}

/// Normalise a CSS colour to `#rrggbb` when it is an opaque hex or rgb()
/// colour, so `<input type="color">` can edit it. Returns null for
/// anything else (rgba with alpha, gradients, named colours), which the
/// customizer then edits as plain text.
export function toHex(value: string): string | null {
  const v = value.trim().toLowerCase();
  const short = /^#([0-9a-f])([0-9a-f])([0-9a-f])$/.exec(v);
  if (short) return `#${short[1]}${short[1]}${short[2]}${short[2]}${short[3]}${short[3]}`;
  if (/^#[0-9a-f]{6}$/.test(v)) return v;
  const rgb = /^rgba?\(\s*(\d+)[\s,]+(\d+)[\s,]+(\d+)\s*(?:[,/]\s*([\d.]+%?)\s*)?\)$/.exec(v);
  if (rgb) {
    if (rgb[4] !== undefined && rgb[4] !== "1" && rgb[4] !== "100%") return null;
    const hex = (n: string) => Math.min(255, Number(n)).toString(16).padStart(2, "0");
    return `#${hex(rgb[1])}${hex(rgb[2])}${hex(rgb[3])}`;
  }
  return null;
}

/// Strip characters that could break out of a `name: value;` declaration.
/// Overrides are the user's own input applied to their own page, so this
/// guards against accidents (a stray `}` disabling every later override),
/// not against an attacker.
export function sanitizeVarValue(value: string): string {
  return value.replace(/[;{}<>]/g, "").trim();
}

/// Only `--kebab-case` names are valid override keys.
export function isValidVarName(name: string): boolean {
  return /^--[a-z0-9-]+$/.test(name);
}

/// Compile an override map into a `:root { ... }` block.
export function generateThemeVarsCss(overrides: Record<string, string>): string {
  const lines = Object.entries(overrides)
    .filter(([name]) => isValidVarName(name))
    .map(([name, value]) => [name, sanitizeVarValue(value)] as const)
    .filter(([, value]) => value.length > 0)
    .map(([name, value]) => `  ${name}: ${value};`);
  return lines.length ? `:root {\n${lines.join("\n")}\n}` : "";
}
