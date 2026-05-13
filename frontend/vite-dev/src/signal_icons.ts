// src/signal_icons.ts
//
// SVG sprites for the plan-08 signal substrate + plan-09 train terrain.
// Each icon is a 24×24 viewBox so they scale to whatever cell size the
// board grid uses. Colors come from `currentColor` so the per-type
// box-shadow accent in style.css can drive a unified tint per kind.

import type { Square, SquareType } from "./variables";
import { getGateOpen, getJunctionBranches, type TrackDir } from "./signal_payload";

const SWITCH_SVG = `<svg viewBox="0 0 24 24" preserveAspectRatio="xMidYMid meet" xmlns="http://www.w3.org/2000/svg">
  <rect x="5" y="16" width="14" height="4" rx="1.5" fill="currentColor"/>
  <line x1="12" y1="16" x2="17" y2="6" stroke="currentColor" stroke-width="2.5" stroke-linecap="round"/>
  <circle cx="17" cy="6" r="2.5" fill="currentColor"/>
</svg>`;

const GATE_OPEN_SVG = `<svg viewBox="0 0 24 24" preserveAspectRatio="xMidYMid meet" xmlns="http://www.w3.org/2000/svg">
  <rect x="4" y="5" width="3.5" height="14" rx="1" fill="currentColor"/>
  <rect x="16.5" y="5" width="3.5" height="14" rx="1" fill="currentColor"/>
  <path d="M7.5 6 Q12 10 16.5 6" stroke="currentColor" stroke-width="1.5" fill="none" opacity="0.45" stroke-dasharray="2 2"/>
</svg>`;

const GATE_CLOSED_SVG = `<svg viewBox="0 0 24 24" preserveAspectRatio="xMidYMid meet" xmlns="http://www.w3.org/2000/svg">
  <rect x="4" y="5" width="3.5" height="14" rx="1" fill="currentColor"/>
  <rect x="16.5" y="5" width="3.5" height="14" rx="1" fill="currentColor"/>
  <rect x="4" y="11" width="16" height="2" rx="0.5" fill="currentColor"/>
  <circle cx="12" cy="12" r="2.2" fill="currentColor"/>
  <circle cx="12" cy="12" r="0.9" fill="rgba(0,0,0,0.55)"/>
</svg>`;

const PLATE_SVG = `<svg viewBox="0 0 24 24" preserveAspectRatio="xMidYMid meet" xmlns="http://www.w3.org/2000/svg">
  <circle cx="12" cy="12" r="9" fill="none" stroke="currentColor" stroke-width="2"/>
  <circle cx="12" cy="12" r="5.5" fill="currentColor" opacity="0.35"/>
  <circle cx="12" cy="12" r="2.5" fill="currentColor"/>
</svg>`;

const TURRET_SVG = `<svg viewBox="0 0 24 24" preserveAspectRatio="xMidYMid meet" xmlns="http://www.w3.org/2000/svg">
  <path d="M5 5 L5 7 L7.5 7 L7.5 5 L10 5 L10 7 L12.5 7 L12.5 5 L15 5 L15 7 L17.5 7 L17.5 5 L19 5 L19 19 L5 19 Z" fill="currentColor"/>
  <rect x="10" y="12" width="4" height="7" fill="rgba(0,0,0,0.45)"/>
</svg>`;

const VENT_SVG = `<svg viewBox="0 0 24 24" preserveAspectRatio="xMidYMid meet" xmlns="http://www.w3.org/2000/svg">
  <rect x="5" y="7" width="14" height="1.8" rx="0.5" fill="currentColor"/>
  <rect x="5" y="11" width="14" height="1.8" rx="0.5" fill="currentColor"/>
  <rect x="5" y="15" width="14" height="1.8" rx="0.5" fill="currentColor"/>
</svg>`;

/// Junction is direction-aware: the branches list determines which arrows
/// are drawn out from the center. Empty branches → just the hub.
function junctionSvg(branches: TrackDir[]): string {
  const arms: string[] = [];
  for (const dir of branches) {
    arms.push(armPath(dir));
  }
  return `<svg viewBox="0 0 24 24" preserveAspectRatio="xMidYMid meet" xmlns="http://www.w3.org/2000/svg">
    ${arms.join("\n    ")}
    <circle cx="12" cy="12" r="3.2" fill="currentColor"/>
  </svg>`;
}

function armPath(dir: TrackDir): string {
  // Each arm is a line from (12,12) to the edge plus a small arrow head.
  switch (dir) {
    case "N":
      return `<path d="M12 12 L12 4" stroke="currentColor" stroke-width="2.5" stroke-linecap="round"/>
              <path d="M12 3 L9 7 L15 7 Z" fill="currentColor"/>`;
    case "S":
      return `<path d="M12 12 L12 20" stroke="currentColor" stroke-width="2.5" stroke-linecap="round"/>
              <path d="M12 21 L9 17 L15 17 Z" fill="currentColor"/>`;
    case "E":
      return `<path d="M12 12 L20 12" stroke="currentColor" stroke-width="2.5" stroke-linecap="round"/>
              <path d="M21 12 L17 9 L17 15 Z" fill="currentColor"/>`;
    case "W":
      return `<path d="M12 12 L4 12" stroke="currentColor" stroke-width="2.5" stroke-linecap="round"/>
              <path d="M3 12 L7 9 L7 15 Z" fill="currentColor"/>`;
  }
}

/// Return the SVG markup for a square, or `null` if the type has no
/// icon overlay (Standard squares are just the chequer background).
export function squareIconSvg(sq: Square): string | null {
  switch (sq.squareType) {
    case "STANDARD":
      return null;
    case "TURRET":
      return TURRET_SVG;
    case "VENT":
      return VENT_SVG;
    case "SWITCH":
      return SWITCH_SVG;
    case "JUNCTION":
      return junctionSvg(getJunctionBranches(sq));
    case "GATE":
      return getGateOpen(sq) ? GATE_OPEN_SVG : GATE_CLOSED_SVG;
    case "PLATE":
      return PLATE_SVG;
  }
}

/// Icon by raw type — used by the palette buttons (which don't have a
/// `Square` to consult, just a SquareType brush). Junctions render with
/// all four cardinal branches as a "generic 4-way" preview.
export function squareTypeIconByType(type: SquareType): string | null {
  switch (type) {
    case "STANDARD":
      return null;
    case "TURRET":
      return TURRET_SVG;
    case "VENT":
      return VENT_SVG;
    case "SWITCH":
      return SWITCH_SVG;
    case "JUNCTION":
      return junctionSvg(["N", "E", "S", "W"]);
    case "GATE":
      return GATE_CLOSED_SVG;
    case "PLATE":
      return PLATE_SVG;
  }
}
