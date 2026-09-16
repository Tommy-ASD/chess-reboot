// src/app/Shell.tsx
//
// Site chrome: the top bar (brand, primary nav, settings) and the footer.

import { useLocation } from "preact-iso";
import { displayName } from "./prefs";

const NAV = [
  { href: "/", label: "Play", match: (p: string) => p === "/" || p.startsWith("/play") || p.startsWith("/game") || p.startsWith("/local") },
  { href: "/pieces", label: "Pieces", match: (p: string) => p.startsWith("/pieces") },
  { href: "/lab", label: "Lab", match: (p: string) => p.startsWith("/lab") },
];

export function BrandMark({ size = 30 }: { size?: number }) {
  return (
    <svg class="brand-mark" width={size} height={size} viewBox="0 0 64 64" aria-hidden="true">
      <rect width="64" height="64" rx="16" fill="currentColor" />
      <path d="M8 40h24v16H24a16 16 0 0 1-16-16z M32 24h24v16H32z" class="brand-mark-checks" />
      <text x="32" y="45" text-anchor="middle" class="brand-mark-two">
        2
      </text>
    </svg>
  );
}

export function TopBar() {
  const { path } = useLocation();
  const name = displayName.value.trim();
  return (
    <header class="topbar">
      <div class="topbar-inner">
        <a class="brand" href="/" aria-label="Chess 2 home">
          <BrandMark />
          <span class="brand-name">Chess 2</span>
        </a>
        <nav class="topnav" aria-label="Primary">
          {NAV.map((item) => {
            const active = item.match(path);
            return (
              <a key={item.href} href={item.href} class={`topnav-link${active ? " is-active" : ""}`} aria-current={active ? "page" : undefined}>
                {item.label}
              </a>
            );
          })}
        </nav>
        <a
          href="/settings"
          class={`topbar-profile${path.startsWith("/settings") ? " is-active" : ""}`}
          aria-label="Settings"
          title="Settings: name, themes, board"
        >
          <span class="avatar" aria-hidden="true">
            {(name || "?").slice(0, 1).toUpperCase()}
          </span>
          <span class="topbar-profile-name">{name || "Set your name"}</span>
          <GearIcon />
        </a>
      </div>
    </header>
  );
}

function GearIcon() {
  return (
    <svg class="icon" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
      <circle cx="12" cy="12" r="3" />
      <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 1 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 1 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 1 1-2.83-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 1 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 1 1 2.83-2.83l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 1 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 1 1 2.83 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 1 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z" />
    </svg>
  );
}

export function Footer() {
  return (
    <footer class="footer">
      <div class="footer-inner">
        <span class="brand-name brand-name-sm">Chess 2</span>
        <nav class="footer-links" aria-label="Footer">
          <a href="/">Variants</a>
          <a href="/pieces">Piece guide</a>
          <a href="/lab">Lab</a>
          <a href="/settings">Themes & settings</a>
        </nav>
      </div>
    </footer>
  );
}
