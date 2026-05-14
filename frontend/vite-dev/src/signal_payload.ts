// src/signal_payload.ts
//
// Typed accessors over `Square.extraFields` for the plan-08 signal
// substrate. The wire format keeps payload as strings (parens-wrapped
// lists, numbers, single chars) — this module is the only place that
// knows how to parse and serialize them. Anywhere else in the editor
// should call these helpers rather than touching `extraFields` directly.

import type { Square } from "./variables";

export type TrackDir = "N" | "S" | "E" | "W";
export const ALL_TRACK_DIRS: readonly TrackDir[] = ["N", "S", "E", "W"];

/// PressurePlate's `FIRES` field. "ANY" matches any piece; "W" / "B"
/// only fire for the corresponding color. "N" fires only for Neutral
/// pieces — i.e. train carts (plan 09). Mirrors the engine's
/// `PressureTrigger::OnlyColor(Color::Neutral)` round-trip in
/// `engine/src/board/fen.rs::format_pressure_trigger`.
export type PressureTrigger = "ANY" | "W" | "B" | "N";
export const ALL_PRESSURE_TRIGGERS: readonly PressureTrigger[] = ["ANY", "W", "B", "N"];

/// Square types that *emit* signals when activated. Switch is player-
/// triggered (via ThrowSwitch); PressurePlate fires automatically when
/// a piece settles on it.
export type EmitterKind = "SWITCH" | "PLATE";

/// Square types that *receive* signals. Junctions cycle through branches;
/// gates toggle open/closed.
export type ReceiverKind = "JUNCTION" | "GATE";

export function isEmitter(sq: Square): sq is Square & { squareType: EmitterKind } {
  return sq.squareType === "SWITCH" || sq.squareType === "PLATE";
}

export function isReceiver(sq: Square): sq is Square & { squareType: ReceiverKind } {
  return sq.squareType === "JUNCTION" || sq.squareType === "GATE";
}

// ----------------------------------------------------------------
// Parse / format primitives
// ----------------------------------------------------------------

function parseIdList(raw: string | undefined): number[] {
  if (!raw) return [];
  const inner = raw.replace(/^\(|\)$/g, "").trim();
  if (inner === "") return [];
  return inner
    .split(",")
    .map(x => Number.parseInt(x.trim(), 10))
    .filter(n => Number.isFinite(n));
}

function formatIdList(ids: number[]): string {
  return `(${ids.join(",")})`;
}

function parseDirList(raw: string | undefined): TrackDir[] {
  if (!raw) return [];
  const inner = raw.replace(/^\(|\)$/g, "").trim();
  if (inner === "") return [];
  return inner
    .split(",")
    .map(x => x.trim() as TrackDir)
    .filter((d): d is TrackDir => ALL_TRACK_DIRS.includes(d));
}

function formatDirList(dirs: TrackDir[]): string {
  return `(${dirs.join(",")})`;
}

function ensureFields(sq: Square): Record<string, string> {
  if (!sq.extraFields) sq.extraFields = {};
  return sq.extraFields;
}

// ----------------------------------------------------------------
// Junction
// ----------------------------------------------------------------

export function getJunctionId(sq: Square): number {
  const n = Number.parseInt(sq.extraFields?.ID ?? "", 10);
  return Number.isFinite(n) ? n : 0;
}

export function setJunctionId(sq: Square, id: number): void {
  ensureFields(sq).ID = String(id);
}

export function getJunctionState(sq: Square): number {
  const n = Number.parseInt(sq.extraFields?.STATE ?? "", 10);
  return Number.isFinite(n) ? n : 0;
}

export function setJunctionState(sq: Square, state: number): void {
  ensureFields(sq).STATE = String(state);
}

export function getJunctionBranches(sq: Square): TrackDir[] {
  return parseDirList(sq.extraFields?.BRANCHES);
}

export function setJunctionBranches(sq: Square, dirs: TrackDir[]): void {
  ensureFields(sq).BRANCHES = formatDirList(dirs);
}

// ----------------------------------------------------------------
// Gate
// ----------------------------------------------------------------

export function getGateId(sq: Square): number {
  const n = Number.parseInt(sq.extraFields?.ID ?? "", 10);
  return Number.isFinite(n) ? n : 0;
}

export function setGateId(sq: Square, id: number): void {
  ensureFields(sq).ID = String(id);
}

export function getGateOpen(sq: Square): boolean {
  // Match engine semantics (fen.rs Gate parsing exactly):
  //  - missing field → open (engine: `open.unwrap_or(true)`)
  //  - "1"           → open
  //  - "0"           → closed
  //  - anything else → closed (engine warns and defaults to closed)
  // The "anything else" arm matters: a hand-typed `OPEN=garbage` must
  // render closed in the editor so the editor preview matches what the
  // engine actually executes.
  const raw = sq.extraFields?.OPEN;
  if (raw === undefined) return true;
  return raw === "1";
}

export function setGateOpen(sq: Square, open: boolean): void {
  ensureFields(sq).OPEN = open ? "1" : "0";
}

// ----------------------------------------------------------------
// Track (plan 09)
// ----------------------------------------------------------------

/// Track tiles carry a single `D=...` direction field. The engine
/// defaults to `E` for tracks parsed without an explicit direction;
/// match that here so a freshly-painted Track tile renders sensibly
/// before the user picks a direction.
export function getTrackDir(sq: Square): TrackDir {
  const raw = sq.extraFields?.D;
  if (raw === "N" || raw === "S" || raw === "E" || raw === "W") return raw;
  return "E";
}

export function setTrackDir(sq: Square, dir: TrackDir): void {
  ensureFields(sq).D = dir;
}

/// Cardinal directions from (file, rank) that point at a Track or
/// Junction tile. Mirrors `Board::neighbor_track_dirs` in the engine —
/// drives the editor's minecart-style rail rendering: a tile's shape
/// (straight / curve / T / X) is computed from which sides have rails.
export function neighborTrackDirs(
  board: Square[][],
  file: number,
  rank: number,
): TrackDir[] {
  const out: TrackDir[] = [];
  const checks: { dir: TrackDir; df: number; dr: number }[] = [
    { dir: "N", df: 0, dr: -1 },
    { dir: "S", df: 0, dr: 1 },
    { dir: "E", df: 1, dr: 0 },
    { dir: "W", df: -1, dr: 0 },
  ];
  for (const { dir, df, dr } of checks) {
    const nf = file + df;
    const nr = rank + dr;
    const row = board[nr];
    if (!row) continue;
    const sq = row[nf];
    if (!sq) continue;
    if (sq.squareType === "TRACK" || sq.squareType === "JUNCTION") {
      out.push(dir);
    }
  }
  return out;
}

/// Two cardinals form a straight (colinear) pair.
export function isColinear(a: TrackDir, b: TrackDir): boolean {
  return (a === "N" && b === "S")
    || (a === "S" && b === "N")
    || (a === "E" && b === "W")
    || (a === "W" && b === "E");
}

export type CornerType = "NE" | "NW" | "SE" | "SW";

/// Classify a 2-connection tile's perpendicular pair as a corner.
/// Returns null for colinear pairs or non-2-connection sets — those
/// aren't corners, so they don't take part in the diagonal-staircase
/// detection.
export function cornerTypeOf(connections: TrackDir[]): CornerType | null {
  if (connections.length !== 2) return null;
  if (isColinear(connections[0], connections[1])) return null;
  const has = (d: TrackDir): boolean => connections.includes(d);
  if (has("N") && has("E")) return "NE";
  if (has("N") && has("W")) return "NW";
  if (has("S") && has("E")) return "SE";
  return "SW";
}

export function oppositeCornerType(c: CornerType): CornerType {
  switch (c) {
    case "NE": return "SW";
    case "NW": return "SE";
    case "SE": return "NW";
    case "SW": return "NE";
  }
}

/// Does this corner tile sit on a diagonal staircase? True iff one of
/// its connected neighbors is the *opposite* corner type — that's the
/// signature of a NE↔SW or NW↔SE alternating chain. Drives the track-
/// render path (corner draws as a straight diagonal line instead of
/// the default quarter-circle curve) so a chain of corner tiles reads
/// as one smooth diagonal. The cart-sprite rotation path rotates 45°
/// on *any* perpendicular corner regardless of staircase status —
/// see `train_payload.ts::trainCartRotationDegrees`.
export function isStaircaseCorner(
  board: Square[][],
  file: number,
  rank: number,
  connections: TrackDir[],
): boolean {
  const myType = cornerTypeOf(connections);
  if (myType === null) return false;
  const opposite = oppositeCornerType(myType);
  const offsets: Record<TrackDir, [number, number]> = {
    N: [0, -1],
    E: [1, 0],
    S: [0, 1],
    W: [-1, 0],
  };
  for (const dir of connections) {
    const [df, dr] = offsets[dir];
    const nFile = file + df;
    const nRank = rank + dr;
    const sq = board[nRank]?.[nFile];
    if (!sq) continue;
    if (sq.squareType !== "TRACK" && sq.squareType !== "JUNCTION") continue;
    const nConns = neighborTrackDirs(board, nFile, nRank);
    if (cornerTypeOf(nConns) === opposite) return true;
  }
  return false;
}

// ----------------------------------------------------------------
// PressurePlate
// ----------------------------------------------------------------

// NB: `getPlateTargets` / `setPlateTargets` were removed; the editor
// uses the unified `getEmitterTargets` / `setEmitterTargets` for both
// Switch and Plate, since they share the `TARGETS` wire-format.

/// Read this plate's `FIRES` trigger. Falls back to `"ANY"` for any
/// unrecognised value — defensive so a future engine-side trigger
/// kind doesn't crash the editor. Known values: `"ANY"`, `"W"`,
/// `"B"`, `"N"` (Neutral — plan 09 trains).
export function getPlateTrigger(sq: Square): PressureTrigger {
  const raw = sq.extraFields?.FIRES;
  if (raw === "W" || raw === "B" || raw === "N") return raw;
  return "ANY";
}

/// Write this plate's `FIRES` trigger. Engine-side parser accepts
/// `"ANY"`, `"W"`, `"B"`, `"N"`; anything else round-trips as
/// `"ANY"` per `getPlateTrigger`.
export function setPlateTrigger(sq: Square, trigger: PressureTrigger): void {
  ensureFields(sq).FIRES = trigger;
}

// ----------------------------------------------------------------
// Unified emitter accessor (Switch and Plate share `TARGETS` keys)
// ----------------------------------------------------------------

export function getEmitterTargets(sq: Square): number[] {
  return parseIdList(sq.extraFields?.TARGETS);
}

export function setEmitterTargets(sq: Square, ids: number[]): void {
  ensureFields(sq).TARGETS = formatIdList(ids);
}

/// Toggle a target ID in/out of an emitter's `TARGETS` list. Used by the
/// editor's click-to-wire mode: clicking a receiver while an emitter is
/// selected adds or removes its ID from the targets.
export function toggleEmitterTarget(sq: Square, id: number): void {
  const targets = getEmitterTargets(sq);
  const idx = targets.indexOf(id);
  if (idx === -1) targets.push(id);
  else targets.splice(idx, 1);
  setEmitterTargets(sq, targets);
}

// ----------------------------------------------------------------
// Receiver ID accessor (works for both Junction and Gate)
// ----------------------------------------------------------------

export function getReceiverId(sq: Square): number | null {
  if (!isReceiver(sq)) return null;
  const n = Number.parseInt(sq.extraFields?.ID ?? "", 10);
  return Number.isFinite(n) ? n : null;
}

// ----------------------------------------------------------------
// Auto-ID allocation
// ----------------------------------------------------------------

/// Lowest positive SignalId not in use by any receiver on `board`.
/// Editor calls this when placing a new Junction or Gate so the user
/// doesn't have to assign IDs manually. Skips 0 — that's the engine's
/// fallback for malformed FENs, so reserving it makes "id=0 receivers"
/// recognizable as un-configured.
export function nextSignalId(board: Square[][]): number {
  const used = new Set<number>();
  for (const row of board) {
    for (const sq of row) {
      if (isReceiver(sq)) {
        const id = getReceiverId(sq);
        if (id !== null) used.add(id);
      }
    }
  }
  for (let i = 1; ; i++) {
    if (!used.has(i)) return i;
  }
}
