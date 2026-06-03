// src/config.ts
//
// Single source of truth for the backend origin. Override at dev/build
// time with `VITE_API_URL` (e.g. a deployed API); otherwise defaults to
// the local dev server the README documents. `WS_BASE` is derived by
// swapping the http(s) scheme for ws(s) so a single env var configures
// both transports.

export const API_BASE: string =
  import.meta.env.VITE_API_URL ?? "http://localhost:8080";

/// `http://host` -> `ws://host`, `https://host` -> `wss://host`.
export const WS_BASE: string = API_BASE.replace(/^http/, "ws");
