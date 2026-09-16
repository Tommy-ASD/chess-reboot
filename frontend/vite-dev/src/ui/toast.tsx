// src/ui/toast.tsx
//
// Non-blocking toasts. `toast()` can be called from anywhere (components,
// stores, transport callbacks); `<Toasts />` renders the stack once in the
// app shell. Click a toast to dismiss it early.

import { signal } from "@preact/signals";

export type ToastKind = "info" | "success" | "warn" | "error";

interface ToastItem {
  id: number;
  message: string;
  kind: ToastKind;
  leaving: boolean;
}

const toasts = signal<ToastItem[]>([]);
let nextId = 1;

function dismiss(id: number) {
  const item = toasts.value.find((t) => t.id === id);
  if (!item || item.leaving) return;
  toasts.value = toasts.value.map((t) => (t.id === id ? { ...t, leaving: true } : t));
  setTimeout(() => {
    toasts.value = toasts.value.filter((t) => t.id !== id);
  }, 200);
}

export function toast(message: string, kind: ToastKind = "info", ms = 3500): void {
  const id = nextId++;
  toasts.value = [...toasts.value, { id, message, kind, leaving: false }];
  setTimeout(() => dismiss(id), ms);
}

/// Surface an unknown error as an error toast (and log it).
export function showError(err: unknown): void {
  const msg = err instanceof Error ? err.message : String(err);
  console.error("Surfaced to user:", err);
  toast(msg, "error", 5500);
}

export function Toasts() {
  return (
    <div class="toast-stack" aria-live="polite">
      {toasts.value.map((t) => (
        <div
          key={t.id}
          class={`toast toast-${t.kind}${t.leaving ? " is-leaving" : ""}`}
          role="status"
          onClick={() => dismiss(t.id)}
        >
          {t.message}
        </div>
      ))}
    </div>
  );
}
