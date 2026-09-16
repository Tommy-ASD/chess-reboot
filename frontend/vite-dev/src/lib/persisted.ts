// src/lib/persisted.ts
//
// A signal backed by localStorage. Reads once at creation, writes on every
// change. Storage failures (private mode, quota) degrade to an in-memory
// signal for the tab's lifetime.

import { effect, signal, type Signal } from "@preact/signals";

export function persistedSignal<T>(
  key: string,
  fallback: T,
  validate: (raw: unknown) => raw is T = (_raw): _raw is T => true,
): Signal<T> {
  let initial = fallback;
  try {
    const raw = localStorage.getItem(key);
    if (raw !== null) {
      const parsed: unknown = JSON.parse(raw);
      if (validate(parsed)) initial = parsed;
    }
  } catch {
    // Unreadable or corrupt: keep the fallback.
  }
  const s = signal<T>(initial);
  effect(() => {
    const value = s.value;
    try {
      localStorage.setItem(key, JSON.stringify(value));
    } catch {
      // Storage unavailable: the signal still works for this session.
    }
  });
  return s;
}

export const isString = (v: unknown): v is string => typeof v === "string";
export const isBoolean = (v: unknown): v is boolean => typeof v === "boolean";
export const isRecord = (v: unknown): v is Record<string, unknown> =>
  typeof v === "object" && v !== null && !Array.isArray(v);
