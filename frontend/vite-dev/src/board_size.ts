// src/board_size.ts
//
// Shared board-resize controller. Owns the `--board-size` CSS variable
// (drives `#board { width: var(--board-size); aspect-ratio: 1; }`) and
// keeps the slider, the user's drag-resize handle on `#board`, and
// localStorage in sync.

const STORAGE_KEY = "chess-board-size";
/// Bounds expressed in *border-box* px so they match `#board`'s actual
/// rendered size (slider value === computed border-box width).
const MIN_PX = 276;   // 8*32 cells + 20 frame
const MAX_PX = 980;   // 8*120 cells + 20 frame
const DEFAULT_PX = 532;

function clamp(n: number): number {
  if (!Number.isFinite(n)) return DEFAULT_PX;
  return Math.max(MIN_PX, Math.min(MAX_PX, Math.round(n)));
}

function loadStored(): number {
  const raw = localStorage.getItem(STORAGE_KEY);
  return clamp(raw === null ? DEFAULT_PX : Number(raw));
}

let currentSize = DEFAULT_PX;
let suppressObserver = false;

/// Single source of truth: when this runs, CSS, slider value, label,
/// and storage all match `px`. ResizeObserver entries that already
/// match `currentSize` are ignored to avoid feedback loops.
function setSize(
  px: number,
  slider: HTMLInputElement | null,
  label: HTMLElement | null,
) {
  const next = clamp(px);
  if (next === currentSize) {
    if (slider && Number(slider.value) !== next) slider.value = String(next);
    if (label) label.textContent = `${next}px`;
    return;
  }
  currentSize = next;
  document.documentElement.style.setProperty("--board-size", `${next}px`);
  localStorage.setItem(STORAGE_KEY, String(next));
  if (slider && Number(slider.value) !== next) slider.value = String(next);
  if (label) label.textContent = `${next}px`;
}

/// Push the board's dimensions into the CSS so `grid-template-columns`,
/// `grid-template-rows`, and the board's `aspect-ratio` all match. Call
/// this whenever the rendered grid's shape changes (any FEN load or
/// in-editor dimension change).
export function setBoardDimensions(cols: number, rows: number) {
  document.documentElement.style.setProperty("--cols", String(cols));
  document.documentElement.style.setProperty("--rows", String(rows));
}

export function initBoardResize(opts: {
  /// CSS selector of the `<input type="range">` slider control.
  sliderSelector: string;
  /// CSS selector of the board container (defaults to `#board`).
  boardSelector?: string;
  /// Optional element that displays the current size as text.
  valueLabelSelector?: string;
}) {
  const slider = document.querySelector<HTMLInputElement>(opts.sliderSelector);
  const board = document.querySelector<HTMLElement>(opts.boardSelector ?? "#board");
  const label = opts.valueLabelSelector
    ? document.querySelector<HTMLElement>(opts.valueLabelSelector)
    : null;

  if (slider) {
    slider.min = String(MIN_PX);
    slider.max = String(MAX_PX);
    slider.step = "4";
    slider.addEventListener("input", () => {
      suppressObserver = true;
      setSize(Number(slider.value), slider, label);
      // The board element animates to the new width; let the observer
      // fire once for that change without bouncing it back.
      requestAnimationFrame(() => { suppressObserver = false; });
    });
  }

  setSize(loadStored(), slider, label);

  // Drag-resize handle on `#board` → slider/storage.
  if (board && typeof ResizeObserver !== "undefined") {
    const ro = new ResizeObserver(entries => {
      if (suppressObserver) return;
      for (const entry of entries) {
        // borderBoxSize matches what we store (border-box px).
        const borderBox = entry.borderBoxSize?.[0]?.inlineSize;
        const px = borderBox ?? entry.contentRect.width + 20;
        if (Math.abs(px - currentSize) < 1) continue;
        setSize(px, slider, label);
      }
    });
    ro.observe(board);
  }
}
