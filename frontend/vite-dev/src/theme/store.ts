// src/theme/store.ts
//
// Appearance state, all persisted to localStorage:
//
//   themeChoice     "system" or a theme id
//   themeOverrides  per-theme colour overrides: overrides[themeId]["--accent"]
//   customCss       hand-written CSS, applied last when enabled
//
// Overrides are keyed by theme so switching themes keeps each theme's own
// tweaks. The applier (`apply.ts`) turns this state into DOM.

import { computed, signal } from "@preact/signals";
import { isBoolean, isRecord, isString, persistedSignal } from "../lib/persisted";
import { DEFAULT_THEME_ID, SYSTEM_THEMES, findTheme, type ThemeManifest } from "./manifest";
import { isValidVarName, sanitizeVarValue } from "./vars";

export const SYSTEM_CHOICE = "system";

export const themeChoice = persistedSignal<string>("chess2.theme", SYSTEM_CHOICE, isString);

type Overrides = Record<string, Record<string, string>>;

const isOverrides = (v: unknown): v is Overrides =>
  isRecord(v) && Object.values(v).every((m) => isRecord(m) && Object.values(m).every(isString));

export const themeOverrides = persistedSignal<Overrides>("chess2.theme-overrides", {}, isOverrides);

export const customCss = persistedSignal<string>("chess2.custom-css", "", isString);
export const customCssEnabled = persistedSignal<boolean>("chess2.custom-css-enabled", true, isBoolean);

const darkQuery =
  typeof window !== "undefined" && typeof window.matchMedia === "function"
    ? window.matchMedia("(prefers-color-scheme: dark)")
    : null;

/** Live OS preference; updated when the user flips their system theme. */
export const systemPrefersDark = signal<boolean>(darkQuery?.matches ?? false);
darkQuery?.addEventListener("change", (e) => {
  systemPrefersDark.value = e.matches;
});

/** The concrete theme in effect after resolving "system" and unknown ids. */
export const activeTheme = computed<ThemeManifest>(() => {
  const choice = themeChoice.value;
  const id =
    choice === SYSTEM_CHOICE ? SYSTEM_THEMES[systemPrefersDark.value ? "dark" : "light"] : choice;
  return findTheme(id) ?? findTheme(DEFAULT_THEME_ID)!;
});

/** Overrides for the active theme only. */
export const activeOverrides = computed<Record<string, string>>(
  () => themeOverrides.value[activeTheme.value.id] ?? {},
);

export function setThemeVar(themeId: string, name: string, value: string): void {
  if (!isValidVarName(name)) return;
  const clean = sanitizeVarValue(value);
  const current = themeOverrides.value[themeId] ?? {};
  themeOverrides.value = { ...themeOverrides.value, [themeId]: { ...current, [name]: clean } };
}

export function resetThemeVar(themeId: string, name: string): void {
  const current = { ...(themeOverrides.value[themeId] ?? {}) };
  delete current[name];
  const next = { ...themeOverrides.value };
  if (Object.keys(current).length === 0) delete next[themeId];
  else next[themeId] = current;
  themeOverrides.value = next;
}

export function resetTheme(themeId: string): void {
  const next = { ...themeOverrides.value };
  delete next[themeId];
  themeOverrides.value = next;
}

export function isThemeModified(themeId: string): boolean {
  return Object.keys(themeOverrides.value[themeId] ?? {}).length > 0;
}
