// src/theme/apply.ts
//
// Turns appearance state into DOM. Injection order in <head>, each layer
// overriding only what it sets:
//
//   1. src/styles/base.css            structure; reads tokens only
//   2. <link id="chess2-theme">       the active theme's CSS file
//   3. <style id="chess2-theme-vars"> the user's overrides for that theme
//   4. <style id="chess2-custom-css"> the user's hand-written CSS
//
// Runs as signal effects, outside the component tree, so the theme is in
// place before the first render and follows every later change.

import { effect } from "@preact/signals";
import { DEFAULT_THEME_ID } from "./manifest";
import { activeOverrides, activeTheme, customCss, customCssEnabled } from "./store";
import { generateThemeVarsCss } from "./vars";

// Eagerly resolve every theme file to a URL so Vite emits each as its own
// hashed asset; only the active one is ever linked.
const themeUrls = import.meta.glob<string>("../themes/*.css", {
  query: "?url",
  import: "default",
  eager: true,
});

function headElement<K extends "link" | "style">(tag: K, id: string): HTMLElementTagNameMap[K] {
  let el = document.getElementById(id) as HTMLElementTagNameMap[K] | null;
  if (!el) {
    el = document.createElement(tag);
    el.id = id;
    document.head.appendChild(el);
  }
  return el;
}

function themeUrl(id: string): string | undefined {
  const url = themeUrls[`../themes/${id}.css`];
  if (url === undefined) {
    console.warn(`[Chess 2] No CSS file for theme "${id}". Falling back to ${DEFAULT_THEME_ID}.`);
    return themeUrls[`../themes/${DEFAULT_THEME_ID}.css`];
  }
  return url;
}

/// Install the appearance effects. Resolves once the first theme stylesheet
/// has loaded (or after a short timeout), so the caller can hold the first
/// paint and avoid a flash of unthemed content.
export function installTheme(): Promise<void> {
  // Create the layers in cascade order up front; later effects only fill them.
  const link = headElement("link", "chess2-theme");
  link.rel = "stylesheet";
  const vars = headElement("style", "chess2-theme-vars");
  const custom = headElement("style", "chess2-custom-css");

  const firstLoad = new Promise<void>((resolve) => {
    const done = () => resolve();
    link.addEventListener("load", done, { once: true });
    link.addEventListener("error", done, { once: true });
    setTimeout(done, 1500);
  });

  effect(() => {
    const theme = activeTheme.value;
    const href = themeUrl(theme.id);
    if (href && link.getAttribute("href") !== href) link.setAttribute("href", href);
    document.documentElement.dataset.theme = theme.id;
    document.documentElement.style.colorScheme = theme.scheme;
  });

  effect(() => {
    vars.textContent = generateThemeVarsCss(activeOverrides.value);
  });

  effect(() => {
    custom.textContent = customCssEnabled.value ? customCss.value : "";
  });

  return firstLoad;
}
