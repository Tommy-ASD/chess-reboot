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
/// only fire for the corresponding color.
export type PressureTrigger = "ANY" | "W" | "B";
export const ALL_PRESSURE_TRIGGERS: readonly PressureTrigger[] = ["ANY", "W", "B"];

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
// Switch
// ----------------------------------------------------------------

export function getSwitchTargets(sq: Square): number[] {
  return parseIdList(sq.extraFields?.TARGETS);
}

export function setSwitchTargets(sq: Square, ids: number[]): void {
  ensureFields(sq).TARGETS = formatIdList(ids);
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
  // Engine emits "1" / "0"; treat missing as "open" (matches the engine's
  // `open.unwrap_or(true)` for fresh-placed gates with no FEN OPEN field).
  return sq.extraFields?.OPEN !== "0";
}

export function setGateOpen(sq: Square, open: boolean): void {
  ensureFields(sq).OPEN = open ? "1" : "0";
}

// ----------------------------------------------------------------
// PressurePlate
// ----------------------------------------------------------------

export function getPlateTargets(sq: Square): number[] {
  return parseIdList(sq.extraFields?.TARGETS);
}

export function setPlateTargets(sq: Square, ids: number[]): void {
  ensureFields(sq).TARGETS = formatIdList(ids);
}

export function getPlateTrigger(sq: Square): PressureTrigger {
  const raw = sq.extraFields?.FIRES;
  if (raw === "W" || raw === "B") return raw;
  return "ANY";
}

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
