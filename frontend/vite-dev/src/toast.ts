// src/toast.ts
//
// Minimal non-blocking toast notifications. Replaces the blocking
// `alert()` for errors and powers the online event cues (opponent joined /
// resigned, your turn, connection lost/restored). The container is created
// lazily on first use, so no markup is required.

export type ToastKind = "info" | "success" | "warn" | "error";

function container(): HTMLElement {
  let el = document.getElementById("toast-container");
  if (!el) {
    el = document.createElement("div");
    el.id = "toast-container";
    el.className = "toast-container";
    document.body.appendChild(el);
  }
  return el;
}

/// Show a toast. Auto-dismisses after `ms`; click to dismiss early. The
/// entrance is a CSS animation (plays on insert) rather than a class toggle,
/// so it can't get stuck invisible when `requestAnimationFrame` is throttled
/// in a background tab.
export function toast(message: string, kind: ToastKind = "info", ms = 3500): void {
  const el = document.createElement("div");
  el.className = `toast toast-${kind}`;
  el.setAttribute("role", "status");
  el.textContent = message;
  container().appendChild(el);

  let removed = false;
  const remove = () => {
    if (removed) return;
    removed = true;
    el.classList.add("toast-hide");
    setTimeout(() => el.remove(), 220);
  };
  const timer = setTimeout(remove, ms);
  el.addEventListener("click", () => {
    clearTimeout(timer);
    remove();
  });
}
