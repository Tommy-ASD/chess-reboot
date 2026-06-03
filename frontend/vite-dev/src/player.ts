// src/player.ts
//
// Client-side player identity — the frontend half of the account seam.
// Today identity is a trust-the-client UUID persisted in localStorage
// plus a display name; every authenticated request carries them as the
// `X-Player-Id` / `X-Player-Name` headers the server's `AuthPlayer`
// extractor reads. When real accounts land, only this module + that
// extractor change: call sites keep calling `authHeaders()`.

export type Identity = { playerId: string; name: string };

const STORAGE_KEY = "chess.identity";

/// A v4 UUID. Prefers the platform `crypto.randomUUID`; the manual
/// fallback keeps older/insecure-context browsers working (the server
/// only requires a parseable UUID, not cryptographic strength here).
function newUuid(): string {
  if (typeof crypto !== "undefined" && typeof crypto.randomUUID === "function") {
    return crypto.randomUUID();
  }
  return "xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx".replace(/[xy]/g, (c) => {
    const r = (Math.random() * 16) | 0;
    const v = c === "x" ? r : (r & 0x3) | 0x8;
    return v.toString(16);
  });
}

let cached: Identity | null = null;

function persist(): void {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(cached));
  } catch {
    // Private-mode / disabled storage: identity then lives for the tab
    // session only, which is still enough to play.
  }
}

/// The persisted identity, lazily created on first use. The `playerId`
/// is stable across reloads (localStorage); `name` may be blank until the
/// user sets one.
export function getIdentity(): Identity {
  if (cached) return cached;
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (raw) {
      const parsed = JSON.parse(raw) as Partial<Identity>;
      if (parsed && typeof parsed.playerId === "string" && parsed.playerId) {
        cached = { playerId: parsed.playerId, name: parsed.name ?? "" };
        return cached;
      }
    }
  } catch {
    // Fall through to minting a fresh identity.
  }
  cached = { playerId: newUuid(), name: "" };
  persist();
  return cached;
}

export function getName(): string {
  return getIdentity().name;
}

/// Update the display name (trimmed; persisted). Empty is allowed — the
/// server falls back to "Anonymous" when `X-Player-Name` is absent.
export function setName(name: string): void {
  const id = getIdentity();
  id.name = name.trim();
  persist();
}

/// Headers attached to every authenticated request. `X-Player-Id` is
/// always sent (the server 401s without it); `X-Player-Name` only when
/// the user has chosen one.
export function authHeaders(): Record<string, string> {
  const id = getIdentity();
  const headers: Record<string, string> = { "X-Player-Id": id.playerId };
  if (id.name) headers["X-Player-Name"] = id.name;
  return headers;
}
