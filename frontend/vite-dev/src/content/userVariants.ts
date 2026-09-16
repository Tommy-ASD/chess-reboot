// src/content/userVariants.ts
//
// Community variants without a backend (yet):
//
//   * Saved variants — positions saved from the Lab, kept in localStorage.
//   * Shared variants — a variant travels entirely in a URL
//     (`/play?fen=…&name=…&by=…`), so a link is enough to share one.
//
// When a server-side variant store exists, `saveVariant` / `savedVariants`
// are the seam to swap out.

import { computed } from "@preact/signals";
import { isRecord, persistedSignal } from "../lib/persisted";
import { officialVariant, type Variant } from "./variants";

export interface SavedVariant {
  id: string;
  name: string;
  description: string;
  fen: string;
  author: string;
  createdAt: number;
}

const isSavedVariant = (v: unknown): v is SavedVariant =>
  isRecord(v) &&
  typeof v.id === "string" &&
  typeof v.name === "string" &&
  typeof v.description === "string" &&
  typeof v.fen === "string" &&
  typeof v.author === "string" &&
  typeof v.createdAt === "number";

const store = persistedSignal<SavedVariant[]>(
  "chess2.my-variants",
  [],
  (v): v is SavedVariant[] => Array.isArray(v) && v.every(isSavedVariant),
);

export const savedVariants = computed(() => store.value);

function newId(): string {
  return `v-${Date.now().toString(36)}-${Math.random().toString(36).slice(2, 7)}`;
}

export function saveVariant(input: Omit<SavedVariant, "id" | "createdAt">): SavedVariant {
  const saved: SavedVariant = { ...input, id: newId(), createdAt: Date.now() };
  store.value = [saved, ...store.value];
  return saved;
}

export function deleteVariant(id: string): void {
  store.value = store.value.filter((v) => v.id !== id);
}

export function toVariant(v: { id: string; name: string; description: string; fen: string; author: string }): Variant {
  return {
    id: v.id,
    name: v.name || "Untitled variant",
    tagline: v.description || "A custom position.",
    description: v.description,
    fen: v.fen,
    pieces: [],
    rules: [],
    status: "experimental",
    source: "community",
    author: v.author || "Anonymous",
  };
}

/** Query string that fully describes a variant — the share format. */
export function variantQuery(v: { name: string; fen: string; author?: string; description?: string }): string {
  const q = new URLSearchParams({ fen: v.fen });
  if (v.name) q.set("name", v.name);
  if (v.author) q.set("by", v.author);
  if (v.description) q.set("about", v.description);
  return q.toString();
}

export function shareUrl(v: { name: string; fen: string; author?: string; description?: string }): string {
  return `${window.location.origin}/play?${variantQuery(v)}`;
}

/// Resolve the variant a play / local route is about: an official id in
/// the path, a saved variant id, or a variant carried in the query string.
export function resolveVariant(id: string | undefined, query: Record<string, string>): Variant | null {
  if (id) {
    const official = officialVariant(id);
    if (official) return official;
    const saved = store.value.find((v) => v.id === id);
    return saved ? toVariant(saved) : null;
  }
  if (query.fen) {
    return toVariant({
      id: "shared",
      name: query.name ?? "",
      description: query.about ?? "",
      fen: query.fen,
      author: query.by ?? "",
    });
  }
  return null;
}
