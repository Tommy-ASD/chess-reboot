// src/signal_icons.ts
//
// SVG sprites for the plan-08 signal substrate + plan-09 train terrain.
// Each icon is a 24×24 viewBox so they scale to whatever cell size the
// board grid uses. Colors come from `currentColor` so the per-type
// box-shadow accent in style.css can drive a unified tint per kind.

import type { Square, SquareType } from "./variables";
import {
  getGateOpen,
  getJunctionBranches,
  getTrackDir,
  isColinear,
  isStaircaseCorner,
  neighborTrackDirs,
  type TrackDir,
} from "./signal_payload";

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

/// Plan 12 Block tile — brick wall pattern. Three rows of bricks with
/// the middle row offset by half a brick so the courses interlock the
/// way real brickwork does. Mortar is implicit (the gaps between
/// rectangles); brick fill is `currentColor` so the tile's accent
/// colour drives the look.
const BLOCK_SVG = `<svg viewBox="0 0 24 24" preserveAspectRatio="none" xmlns="http://www.w3.org/2000/svg">
  <rect x="3" y="4"  width="6.5" height="5" fill="currentColor"/>
  <rect x="10.5" y="4"  width="6.5" height="5" fill="currentColor"/>
  <rect x="18" y="4"  width="3" height="5" fill="currentColor"/>
  <rect x="3" y="10" width="3" height="5" fill="currentColor"/>
  <rect x="7" y="10" width="6.5" height="5" fill="currentColor"/>
  <rect x="14.5" y="10" width="6.5" height="5" fill="currentColor"/>
  <rect x="3" y="16" width="6.5" height="5" fill="currentColor"/>
  <rect x="10.5" y="16" width="6.5" height="5" fill="currentColor"/>
  <rect x="18" y="16" width="3" height="5" fill="currentColor"/>
</svg>`;

/// Plan 09 track tile, drawn minecart-style: the shape (straight,
/// curve, T, or X) is computed from which cardinal neighbors are
/// themselves track / junction tiles. Rails extend all the way to the
/// cell edges so adjacent tiles meet without a visible gap. The stored
/// `D` field is only used as a fallback for isolated tiles (no
/// neighbors).
///
/// Geometry: 24×24 viewBox, cell center at (12, 12). Each rail tile has
/// two parallel rails offset 1.8 from the centerline (centerlines at
/// 10.2 and 13.8). Rail width is 1.6. Curves are quarter-circle arcs
/// centered on the appropriate cell corner with radii 10.2 and 13.8 so
/// they connect smoothly to straight-rail neighbors at the same cell-
/// edge coordinates.
///
/// Diagonal staircases: when a perpendicular corner has a complementary
/// corner neighbor (NE↔SW or NW↔SE), the tile renders as a straight
/// diagonal segment instead of a curve. Chains of such tiles connect
/// into a continuous diagonal line. Pass `context` to enable this — if
/// omitted (palette previews), all corners draw as curves.
function trackSvg(
  connections: TrackDir[],
  fallback: TrackDir,
  context?: { board: Square[][]; file: number; rank: number },
): string {
  const body = railsFor(connections, fallback, context);
  return `<svg viewBox="0 0 24 24" preserveAspectRatio="none" xmlns="http://www.w3.org/2000/svg">
    ${body}
  </svg>`;
}

function railsFor(
  connections: TrackDir[],
  fallback: TrackDir,
  context?: { board: Square[][]; file: number; rank: number },
): string {
  // Snapshot for readability.
  const has = {
    N: connections.includes("N"),
    S: connections.includes("S"),
    E: connections.includes("E"),
    W: connections.includes("W"),
  };
  const n = connections.length;

  if (n === 0) {
    // Isolated track tile — fall back to the stored direction.
    return straightRail(axisOf(fallback));
  }

  if (n === 1) {
    // Single-sided rail: render a full straight rail along that axis.
    // The non-connected end looks like a hanging stub, which matches
    // how Minecraft renders a lone rail next to nothing.
    return straightRail(axisOf(connections[0]));
  }

  if (n === 2) {
    const [a, b] = connections;
    if (isColinear(a, b)) {
      return straightRail(axisOf(a));
    }
    // Diagonal-staircase detection: if a connected neighbor is the
    // opposite corner type (NE↔SW, NW↔SE), render as a straight
    // diagonal segment so chains of corners read as one continuous
    // line. An isolated corner (no staircase neighbor) keeps the
    // smoother quarter-circle curve.
    if (context && isStaircaseCorner(context.board, context.file, context.rank, [a, b])) {
      return cornerDiagonal(a, b);
    }
    return cornerCurve(a, b);
  }

  if (n === 3) {
    // T-shape: pick the colinear pair as a straight rail, render the
    // odd direction as a half-stub from center to the connected edge.
    if (has.N && has.S) {
      const third = (["E", "W"] as TrackDir[]).find(d => has[d])!;
      return straightRail("vertical") + halfStub(third);
    }
    // has.E && has.W must hold (3 connections from {N,S,E,W} always
    // include one colinear pair).
    const third = (["N", "S"] as TrackDir[]).find(d => has[d])!;
    return straightRail("horizontal") + halfStub(third);
  }

  // n === 4: full X — both straight rails overlaid.
  return straightRail("horizontal") + straightRail("vertical");
}

function axisOf(dir: TrackDir): "horizontal" | "vertical" {
  return dir === "N" || dir === "S" ? "vertical" : "horizontal";
}

/// Two parallel rails spanning the full cell along `axis`, with
/// crossties between them. Rails extend from 0 to 24 so adjacent
/// tiles' rails meet exactly at the shared cell edge.
function straightRail(axis: "horizontal" | "vertical"): string {
  if (axis === "horizontal") {
    return `
      <!-- crossties -->
      <rect x="2"  y="7" width="2.2" height="10" rx="0.6" fill="currentColor" opacity="0.55"/>
      <rect x="8"  y="7" width="2.2" height="10" rx="0.6" fill="currentColor" opacity="0.55"/>
      <rect x="14" y="7" width="2.2" height="10" rx="0.6" fill="currentColor" opacity="0.55"/>
      <rect x="20" y="7" width="2.2" height="10" rx="0.6" fill="currentColor" opacity="0.55"/>
      <!-- rails -->
      <rect x="0" y="9.4" width="24" height="1.6" fill="currentColor"/>
      <rect x="0" y="13"  width="24" height="1.6" fill="currentColor"/>
    `;
  }
  return `
    <rect x="7"  y="2"  width="10" height="2.2" rx="0.6" fill="currentColor" opacity="0.55"/>
    <rect x="7"  y="8"  width="10" height="2.2" rx="0.6" fill="currentColor" opacity="0.55"/>
    <rect x="7"  y="14" width="10" height="2.2" rx="0.6" fill="currentColor" opacity="0.55"/>
    <rect x="7"  y="20" width="10" height="2.2" rx="0.6" fill="currentColor" opacity="0.55"/>
    <rect x="9.4" y="0" width="1.6" height="24" fill="currentColor"/>
    <rect x="13"  y="0" width="1.6" height="24" fill="currentColor"/>
  `;
}

/// Half-rail from the cell center to the connected edge, used as the
/// "stub" arm of T-junctions where one direction has no colinear
/// partner. Rails sit at the same offsets as a straight rail so they
/// meet the neighbor's rail at the shared border.
function halfStub(dir: TrackDir): string {
  switch (dir) {
    case "E":
      return `
        <rect x="11" y="9.4" width="13" height="1.6" fill="currentColor"/>
        <rect x="11" y="13"  width="13" height="1.6" fill="currentColor"/>
      `;
    case "W":
      return `
        <rect x="0" y="9.4" width="13" height="1.6" fill="currentColor"/>
        <rect x="0" y="13"  width="13" height="1.6" fill="currentColor"/>
      `;
    case "N":
      return `
        <rect x="9.4" y="0" width="1.6" height="13" fill="currentColor"/>
        <rect x="13"  y="0" width="1.6" height="13" fill="currentColor"/>
      `;
    case "S":
      return `
        <rect x="9.4" y="11" width="1.6" height="13" fill="currentColor"/>
        <rect x="13"  y="11" width="1.6" height="13" fill="currentColor"/>
      `;
  }
}

/// Quarter-circle corner: two parallel arcs centered on the cell corner
/// that lies between the two connection directions. Arc radii (10.2 and
/// 13.8) are picked so the rail endpoints fall at the same coordinates
/// a straight rail would meet the cell edge — adjacent tiles connect
/// seamlessly.
function cornerCurve(a: TrackDir, b: TrackDir): string {
  const dirs = new Set<TrackDir>([a, b]);
  const R_OUTER = 13.8;
  const R_INNER = 10.2;

  // Corner location + arc endpoints + SVG sweep flag (1 = CW). The
  // four corners need different sweep flags because the "short arc
  // through the cell interior" sometimes rotates CW and sometimes CCW
  // depending on where the corner sits.
  let outerStart: [number, number];
  let outerEnd: [number, number];
  let innerStart: [number, number];
  let innerEnd: [number, number];
  let sweep: 0 | 1;

  if (dirs.has("E") && dirs.has("N")) {
    // Corner NE (24, 0). Outer endpoints at (10.2, 0) on N edge and
    // (24, 13.8) on E edge. CCW short arc through cell interior.
    outerStart = [10.2, 0];
    outerEnd = [24, 13.8];
    innerStart = [13.8, 0];
    innerEnd = [24, 10.2];
    sweep = 0;
  } else if (dirs.has("E") && dirs.has("S")) {
    // Corner SE (24, 24). CW short arc.
    outerStart = [10.2, 24];
    outerEnd = [24, 10.2];
    innerStart = [13.8, 24];
    innerEnd = [24, 13.8];
    sweep = 1;
  } else if (dirs.has("W") && dirs.has("N")) {
    // Corner NW (0, 0). CW short arc.
    outerStart = [13.8, 0];
    outerEnd = [0, 13.8];
    innerStart = [10.2, 0];
    innerEnd = [0, 10.2];
    sweep = 1;
  } else {
    // Corner SW (0, 24). CCW short arc.
    outerStart = [13.8, 24];
    outerEnd = [0, 10.2];
    innerStart = [10.2, 24];
    innerEnd = [0, 13.8];
    sweep = 0;
  }

  const path = (
    start: [number, number],
    end: [number, number],
    r: number,
  ): string =>
    `<path d="M ${start[0]} ${start[1]} A ${r} ${r} 0 0 ${sweep} ${end[0]} ${end[1]}" \
            stroke="currentColor" stroke-width="1.6" fill="none"/>`;

  return `
    ${path(outerStart, outerEnd, R_OUTER)}
    ${path(innerStart, innerEnd, R_INNER)}
  `;
}

/// Diagonal version of a perpendicular corner — two parallel straight
/// line segments cutting across the cell from one edge to the
/// adjacent edge. Used when this tile is part of a staircase chain so
/// the chain reads as a continuous diagonal line rather than a series
/// of bumpy curves. Endpoints sit at the same cell-edge coordinates as
/// the curve renderer would use, so a diagonal tile next to a straight
/// or curved tile still connects flush at the shared border.
function cornerDiagonal(a: TrackDir, b: TrackDir): string {
  const dirs = new Set<TrackDir>([a, b]);
  let railA: [[number, number], [number, number]];
  let railB: [[number, number], [number, number]];
  if (dirs.has("E") && dirs.has("N")) {
    railA = [[10.2, 0], [24, 13.8]];
    railB = [[13.8, 0], [24, 10.2]];
  } else if (dirs.has("E") && dirs.has("S")) {
    railA = [[10.2, 24], [24, 10.2]];
    railB = [[13.8, 24], [24, 13.8]];
  } else if (dirs.has("W") && dirs.has("N")) {
    railA = [[13.8, 0], [0, 13.8]];
    railB = [[10.2, 0], [0, 10.2]];
  } else {
    railA = [[13.8, 24], [0, 10.2]];
    railB = [[10.2, 24], [0, 13.8]];
  }
  const seg = (p1: [number, number], p2: [number, number]): string =>
    `<line x1="${p1[0]}" y1="${p1[1]}" x2="${p2[0]}" y2="${p2[1]}" \
           stroke="currentColor" stroke-width="1.6" stroke-linecap="butt"/>`;
  return `
    ${seg(railA[0], railA[1])}
    ${seg(railB[0], railB[1])}
  `;
}

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
/// Render the icon overlay for a square. For Track tiles the visual
/// depends on which cardinal neighbors are themselves tracks (so a tile
/// at the corner of an L-shaped path renders as a curve, not a straight
/// rail). `context` lets the caller supply the board grid + coordinates
/// so neighbor-detection can run; omit it for places that draw a
/// representative icon out of board context (palette buttons), where
/// the stored `D` alone determines the look.
export function squareIconSvg(
  sq: Square,
  context?: { board: Square[][]; file: number; rank: number },
): string | null {
  switch (sq.squareType) {
    case "STANDARD":
      return null;
    case "TURRET":
      return TURRET_SVG;
    case "VENT":
      return VENT_SVG;
    case "BLOCK":
      return BLOCK_SVG;
    case "SWITCH":
      return SWITCH_SVG;
    case "JUNCTION":
      return junctionSvg(getJunctionBranches(sq));
    case "GATE":
      return getGateOpen(sq) ? GATE_OPEN_SVG : GATE_CLOSED_SVG;
    case "PLATE":
      return PLATE_SVG;
    case "TRACK": {
      // Without context we can't run neighbour-detection (this
      // happens for palette previews via `squareTypeIconByType`).
      // Production board renders in both `editor_page.ts` and
      // `main.ts` always pass context, so the empty-connections
      // branch is effectively palette-only.
      const connections = context
        ? neighborTrackDirs(context.board, context.file, context.rank)
        : [];
      return trackSvg(connections, getTrackDir(sq), context);
    }
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
    case "BLOCK":
      return BLOCK_SVG;
    case "SWITCH":
      return SWITCH_SVG;
    case "JUNCTION":
      return junctionSvg(["N", "E", "S", "W"]);
    case "GATE":
      return GATE_CLOSED_SVG;
    case "PLATE":
      return PLATE_SVG;
    case "TRACK":
      // Palette preview — no board context, just show a generic E-W
      // horizontal rail so users see the tile silhouette.
      return trackSvg([], "E");
  }
}
