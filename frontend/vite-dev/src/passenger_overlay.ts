// src/passenger_overlay.ts
//
// Render a small "passenger" sprite over a carrier piece (Bus,
// Locomotive, Carriage) so the user can see who's riding inside. The
// overlay is positioned absolutely on top of the cell so it sits
// above the cart/bus sprite without being subject to its rotation.

import { pieceToImage, pieceToSymbol } from "./fen";
import { getCarrierPassengers } from "./train_payload";

/// Append a passenger-overlay child to `cell` for any carrier piece
/// with at least one passenger. No-op for non-carriers and empty
/// carriers. The number of passengers is exposed via a `data-count`
/// attribute on the overlay container so CSS can scale the sprites
/// per-count without inline styling.
export function renderCarrierPassengerOverlay(
  cell: HTMLElement,
  piece: string,
): void {
  const passengers = getCarrierPassengers(piece);
  if (passengers.length === 0) return;

  const overlay = document.createElement("div");
  overlay.className = "carrier-passengers";
  // Cap at 6 — beyond that the per-passenger size shrinks past the
  // point of legibility and we just show the first six.
  overlay.dataset.count = String(Math.min(passengers.length, 6));

  for (let i = 0; i < Math.min(passengers.length, 6); i++) {
    overlay.appendChild(makePassengerSprite(passengers[i]));
  }

  cell.appendChild(overlay);
}

function makePassengerSprite(piece: string): HTMLElement {
  const imgPath = pieceToImage(piece);
  if (imgPath !== undefined) {
    const img = document.createElement("img");
    img.src = imgPath;
    img.alt = piece;
    img.className = "carrier-passenger";
    return img;
  }
  // Fallback for standard chess pieces (K, Q, R, B, N, P) which have
  // no PNG/SVG sprite — render the Unicode chess glyph instead.
  const span = document.createElement("span");
  span.textContent = pieceToSymbol(piece);
  span.className = "carrier-passenger carrier-passenger-glyph";
  return span;
}
