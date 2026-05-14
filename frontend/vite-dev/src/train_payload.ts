// src/train_payload.ts
//
// Plan 09 piece-payload helpers — Locomotive and Carriage. Mirrors the
// shape of `signal_payload.ts` but operates on the piece-string format
// the engine uses (e.g. "LOCO(ID=1,H=F,P=(K,R))"). Parsing happens by
// scanning the parenthesised body for `KEY=VALUE` fields with
// nested-paren awareness so `P=(K,R)` works alongside ID/H/I.

import { getBusPassengers } from "./fen";
import {
  getTrackDir,
  neighborTrackDirs,
  type TrackDir,
} from "./signal_payload";
import type { Square } from "./variables";

export type TrainHeading = "F" | "R";
export const ALL_TRAIN_HEADINGS: readonly TrainHeading[] = ["F", "R"];

export type TrainCart = {
  /// "LOCO" or "CART" — the leading symbol determines which fields apply.
  kind: "LOCO" | "CART";
  /// `ID=...` — the train this cart belongs to. Carts sharing an ID move
  /// together. Defaults to 0 when the FEN omits the field.
  trainId: number;
  /// Locomotive only. Direction this train follows the underlying
  /// tracks. Defaults to "F".
  heading: TrainHeading;
  /// Carriage only. Position in the chain (1..255; 0 is the loco).
  /// Defaults to 1.
  chainIndex: number;
  /// Passenger piece symbols, in declaration order. The engine treats
  /// a cart as a carrier (max-cap-free for trains); the editor passes
  /// these through verbatim.
  passengers: string[];
  /// Locomotive only. The cardinal direction the loco entered its
  /// current tile through. `null` ("not set") means "loco hasn't ticked
  /// yet" — the engine then falls back to the tile's `D` field on the
  /// first tick. Once set, subsequent ticks use minecart-style
  /// neighbor detection (exit through the side that isn't `lastDir`).
  lastDir: TrackDir | null;
};

/// Is this piece a train cart (LOCO or CART)? Cheap prefix check.
/// Case-insensitive (the engine accepts either casing — see
/// `parseTrainCart`).
export function isTrainCart(piece: string | null | undefined): boolean {
  if (!piece) return false;
  const head = piece.split("(")[0].toUpperCase();
  return head === "LOCO" || head === "CART";
}

/// Passenger symbols carried by any carrier piece (Bus, Locomotive,
/// Carriage). The wire format differs slightly between Bus (no other
/// payload fields, so the `P=(...)` is the only thing inside) and
/// train carts (P=... lives alongside ID/H/I/L), but the *content* is
/// a comma-separated piece-symbol list either way. Pieces that aren't
/// carriers return an empty array.
export function getCarrierPassengers(piece: string): string[] {
  const cart = parseTrainCart(piece);
  if (cart !== null) return cart.passengers;
  const base = piece.split("(")[0];
  // Match engine strict-prefix rejection (Bus::from_symbol requires
  // exactly "BUS" or "bus" — mixed case is invalid).
  if (base === "BUS" || base === "bus") return getBusPassengers(piece);
  return [];
}

/// Parse a LOCO or CART symbol into structured form. Returns `null` for
/// non-train symbols (a fast escape hatch for the editor's render path).
///
/// Matches case-insensitively because the engine's `symbol_to_piece`
/// lowercases the prefix before dispatch — a hand-typed FEN may use
/// `loco(...)` / `cart(...)` and still parse engine-side. Normalize
/// to uppercase here so the rest of the frontend doesn't have to
/// branch on casing.
export function parseTrainCart(piece: string): TrainCart | null {
  const headRaw = piece.split("(")[0];
  const headUpper = headRaw.toUpperCase();
  if (headUpper !== "LOCO" && headUpper !== "CART") return null;
  const head: "LOCO" | "CART" = headUpper;

  const cart: TrainCart = {
    kind: head,
    trainId: 0,
    heading: "F",
    chainIndex: 1,
    passengers: [],
    lastDir: null,
  };

  const open = piece.indexOf("(");
  if (open === -1) return cart;
  const close = findMatchingParen(piece, open);
  if (close === -1) return cart;

  const inside = piece.slice(open + 1, close);
  for (const field of splitTopLevel(inside)) {
    const eq = field.indexOf("=");
    if (eq === -1) continue;
    const key = field.slice(0, eq).trim();
    const value = field.slice(eq + 1).trim();

    switch (key) {
      case "ID": {
        const n = Number.parseInt(value, 10);
        if (Number.isFinite(n)) cart.trainId = n;
        break;
      }
      case "H":
        if (value === "F" || value === "R") cart.heading = value;
        break;
      case "L":
        if (value === "N" || value === "S" || value === "E" || value === "W") {
          cart.lastDir = value;
        }
        break;
      case "I": {
        const n = Number.parseInt(value, 10);
        if (Number.isFinite(n)) cart.chainIndex = n;
        break;
      }
      case "P": {
        if (value.startsWith("(") && value.endsWith(")")) {
          // Mirror the engine's `from_symbol` rejection of nested
          // carriers (bus / locomotive / carriage as passengers).
          // The engine's move filter never produces such a state and
          // its FEN parsers refuse to accept it; if the frontend
          // accepted it, the editor would display a nested carrier
          // that the engine then silently drops on round-trip.
          const raw = splitTopLevel(value.slice(1, -1));
          cart.passengers = raw.filter(sym => {
            const prefix = sym.split("(")[0].toLowerCase();
            const isCarrier =
              prefix === "bus" || prefix === "loco" || prefix === "cart";
            if (isCarrier) {
              console.warn(
                `train_payload: dropping nested carrier passenger '${sym}' ` +
                  `(engine rejects this on FEN parse)`,
              );
            }
            return !isCarrier;
          });
        }
        break;
      }
      default:
        // unknown field — drop silently; FEN round-trips through the
        // structured form, so unrecognised payload would be lost. The
        // plan doesn't define any other fields today.
        break;
    }
  }
  return cart;
}

/// Serialize a train cart back to engine symbol form. The engine's
/// canonical ordering is ID → H/I → P, which we mirror so the FEN
/// round-trips byte-identical.
export function serializeTrainCart(cart: TrainCart): string {
  const parts: string[] = [`ID=${cart.trainId}`];
  if (cart.kind === "LOCO") {
    parts.push(`H=${cart.heading}`);
    if (cart.lastDir !== null) {
      parts.push(`L=${cart.lastDir}`);
    }
  } else {
    parts.push(`I=${cart.chainIndex}`);
  }
  if (cart.passengers.length > 0) {
    parts.push(`P=(${cart.passengers.join(",")})`);
  }
  return `${cart.kind}(${parts.join(",")})`;
}

/// Highest train ID already in use across all carts on the board. The
/// editor uses `+1` of this when placing a new LOCO so each fresh
/// locomotive starts its own train. Returns 0 for a train-free board so
/// the first placed loco gets ID 1.
export function highestTrainId(board: Square[][]): number {
  let max = 0;
  for (const row of board) {
    for (const sq of row) {
      if (!sq.piece) continue;
      const cart = parseTrainCart(sq.piece);
      if (cart && cart.trainId > max) max = cart.trainId;
    }
  }
  return max;
}

/// Smallest train_id whose chain has carts but no locomotive, or
/// `null` if every train on the board has a head. Used by the editor's
/// LOCO-placement smart-payload helper so that placing a LOCO after a
/// CART adopts the orphan's train_id instead of allocating a fresh ID
/// and leaving the carriage permanently disconnected.
///
/// Skips `train_id === 0` — the engine treats `0` as the default
/// missing-id sentinel (`CART()` with no ID parses to trainId=0), so
/// adopting it for a fresh LOCO would silently produce a `LOCO(ID=0)`
/// chain that collides with the next default placement.
export function firstOrphanTrainId(board: Square[][]): number | null {
  const haveLoco = new Set<number>();
  const haveCart = new Set<number>();
  for (const row of board) {
    for (const sq of row) {
      if (!sq.piece) continue;
      const cart = parseTrainCart(sq.piece);
      if (!cart) continue;
      if (cart.kind === "LOCO") haveLoco.add(cart.trainId);
      else haveCart.add(cart.trainId);
    }
  }
  const orphans = [...haveCart].filter(id => id !== 0 && !haveLoco.has(id));
  if (orphans.length === 0) return null;
  return Math.min(...orphans);
}

/// Visual rotation in degrees for a train cart's sprite. The locomotive
/// and carriage SVGs are drawn top-down facing east (positive x); this
/// helper returns the CSS rotation needed to point the sprite at the
/// cart's direction of motion.
///
/// The function determines two things for the cart's current tile: the
/// cardinal *entry* direction (where the cart came from / will be
/// followed-into) and the *exit* direction (where it's headed). For a
/// straight rail or T/X tile the cart rotates to the exit's cardinal
/// (0/90/180/270). For *any* perpendicular corner — staircase or
/// isolated U-bend — the cart rotates to the entry→exit diagonal
/// (45/135/225/315) so it reads as mid-turn regardless of whether
/// the underlying rail draws as a curve or a diagonal line.
///
/// Returns 0° (east) for non-cart pieces or when no direction can be
/// resolved (e.g. an orphaned carriage with no matching loco).
export function trainCartRotationDegrees(
  piece: string,
  board: Square[][],
  file: number,
  rank: number,
): number {
  const cart = parseTrainCart(piece);
  if (!cart) return 0;

  const connections = neighborTrackDirs(board, file, rank);
  const ee = cartEntryExit(cart, board, file, rank, connections);
  if (ee === null) {
    // No directional info — try the simpler facing fallback.
    const fallback = simpleCardinalFacing(cart, board, file, rank);
    return fallback === null ? 0 : cardinalToDegrees(fallback);
  }

  // Any perpendicular corner — staircase pair, single U-bend, or end-
  // of-curve transition — gets the 45° diagonal rotation. The cart's
  // motion through the tile bends from entry to exit, so the sprite
  // sits "mid-turn" rather than snapping to one of the two cardinals.
  // Straight or colinear entry/exit pairs return null from
  // `entryExitDiagDegrees`, in which case we fall through to the
  // cardinal exit direction.
  const diagDeg = entryExitDiagDegrees(ee.entry, ee.exit);
  if (diagDeg !== null) return diagDeg;

  return cardinalToDegrees(ee.exit);
}

interface EntryExit {
  entry: TrackDir;
  exit: TrackDir;
}

/// Determine the cardinal directions through which the cart entered
/// and will exit its current tile. Requires the tile's connection set;
/// returns null when the cart's state doesn't pin down a unique
/// entry/exit pair (e.g. a loco placed off-track, or a carriage with
/// no preceding cart found).
function cartEntryExit(
  cart: TrainCart,
  board: Square[][],
  file: number,
  rank: number,
  connections: TrackDir[],
): EntryExit | null {
  if (cart.kind === "LOCO") {
    return locoEntryExit(cart, board, file, rank, connections);
  }
  // Carriage: exit = direction toward the preceding cart; entry =
  // the *other* connection. Requires a 2-connection tile to be
  // unambiguous — straights and corners qualify.
  const exit = directionToPrecedingCart(cart, board, file, rank);
  if (exit === null) return null;
  if (connections.length === 2) {
    const entry = connections.find(d => d !== exit);
    if (entry === undefined) return null;
    return { entry, exit };
  }
  // Non-2-connection tile (T, X, straight isolated). Best-effort: use
  // the cart's motion direction as the exit and treat the opposite as
  // entry. This keeps the rotation cardinal for non-corner tiles.
  return { entry: oppositeTrackDir(exit), exit };
}

function locoEntryExit(
  cart: TrainCart,
  board: Square[][],
  file: number,
  rank: number,
  connections: TrackDir[],
): EntryExit | null {
  // Loco has ticked at least once → last_dir is the cardinal it
  // entered this tile through. Exit through the connection that
  // isn't last_dir, mirroring the engine's neighbor-detection.
  if (cart.lastDir !== null) {
    if (connections.length === 2) {
      const entry = cart.lastDir;
      const exit = connections.find(d => d !== entry);
      if (exit === undefined) return null;
      return { entry, exit };
    }
    if (connections.length >= 1) {
      const candidates = connections.filter(d => d !== cart.lastDir);
      if (candidates.length === 1) {
        return { entry: cart.lastDir, exit: candidates[0] };
      }
    }
    // Fall back: assume straight-rail-style "opposite of last_dir".
    return { entry: cart.lastDir, exit: oppositeTrackDir(cart.lastDir) };
  }
  // First tick (last_dir unset). Use the tile's stored D rotated by
  // heading as the *preferred* exit. If that direction isn't a real
  // connection of this tile, mirror the engine's first-tick fallback
  // (engine/src/board/trains.rs::next_train_step): pick the unique
  // non-cart neighbor track. Otherwise the sprite would render
  // facing the bogus D while the engine actually exits via the
  // valid neighbor — visible as a 90° snap on the first real tick.
  const tile = board[rank]?.[file];
  const tileD: TrackDir = tile ? getTrackDir(tile) : "E";
  const preferred: TrackDir =
    cart.heading === "F" ? tileD : oppositeTrackDir(tileD);
  let exit: TrackDir = preferred;
  if (!connections.includes(preferred)) {
    // Fallback: find the unique non-cart neighbor (engine excludes
    // any cart-occupied neighbor on the first tick).
    const candidates = connections.filter(d => {
      const offset: Record<TrackDir, [number, number]> = {
        N: [0, -1],
        E: [1, 0],
        S: [0, 1],
        W: [-1, 0],
      };
      const [df, dr] = offset[d];
      const sq = board[rank + dr]?.[file + df];
      if (!sq || !sq.piece) return true;
      // Skip neighbors that hold any train cart.
      const c = parseTrainCart(sq.piece);
      return c === null;
    });
    if (candidates.length === 1) {
      exit = candidates[0];
    }
    // 0 or 2+ candidates → keep `preferred` and let the resulting
    // entry/exit pair drive a (possibly nonsensical) rotation. Same
    // policy as the engine's "ambiguity falls back to preferred".
  }
  if (connections.length === 2) {
    const entry = connections.find(d => d !== exit);
    if (entry !== undefined) return { entry, exit };
  }
  return { entry: oppositeTrackDir(exit), exit };
}

/// Cardinal direction from this carriage's tile to the preceding cart
/// (chain_index − 1, same train_id). Locomotive counts as chain_index 0.
function directionToPrecedingCart(
  cart: TrainCart,
  board: Square[][],
  file: number,
  rank: number,
): TrackDir | null {
  if (cart.kind !== "CART") return null;
  const prevIdx = cart.chainIndex - 1;
  for (let r = 0; r < board.length; r++) {
    const row = board[r];
    for (let f = 0; f < row.length; f++) {
      const sq = row[f];
      if (!sq.piece) continue;
      const other = parseTrainCart(sq.piece);
      if (other === null || other.trainId !== cart.trainId) continue;
      const otherIdx = other.kind === "LOCO" ? 0 : other.chainIndex;
      if (otherIdx !== prevIdx) continue;
      const dx = f - file;
      const dy = r - rank;
      if (dx > 0 && dy === 0) return "E";
      if (dx < 0 && dy === 0) return "W";
      if (dx === 0 && dy > 0) return "S";
      if (dx === 0 && dy < 0) return "N";
      return null; // chain broken — non-adjacent preceding cart
    }
  }
  return null;
}

/// Simple cardinal facing used when entry/exit can't be pinned down
/// (e.g. orphan tile, no connections). Same as the previous behavior:
/// loco uses heading + D, carriage uses preceding-cart direction.
function simpleCardinalFacing(
  cart: TrainCart,
  board: Square[][],
  file: number,
  rank: number,
): TrackDir | null {
  if (cart.kind === "LOCO") {
    const tile = board[rank]?.[file];
    const tileD: TrackDir = tile ? getTrackDir(tile) : "E";
    if (cart.lastDir !== null) return oppositeTrackDir(cart.lastDir);
    return cart.heading === "F" ? tileD : oppositeTrackDir(tileD);
  }
  return directionToPrecedingCart(cart, board, file, rank);
}

function oppositeTrackDir(d: TrackDir): TrackDir {
  switch (d) {
    case "N": return "S";
    case "S": return "N";
    case "E": return "W";
    case "W": return "E";
  }
}

function cardinalToDegrees(d: TrackDir): number {
  switch (d) {
    case "E": return 0;
    case "S": return 90;
    case "W": return 180;
    case "N": return 270;
  }
}

/// Diagonal direction in degrees for entry→exit motion through a
/// corner tile. The cart moves from the entry edge midpoint to the
/// exit edge midpoint; the rotation matches that vector. Returns null
/// for entry/exit pairs that aren't a perpendicular corner.
function entryExitDiagDegrees(entry: TrackDir, exit: TrackDir): number | null {
  const key = `${entry}${exit}`;
  switch (key) {
    case "NE": return 45;  // motion SE
    case "EN": return 225; // motion NW
    case "NW": return 135; // motion SW
    case "WN": return 315; // motion NE
    case "SE": return 315; // motion NE
    case "ES": return 135; // motion SW
    case "SW": return 225; // motion NW
    case "WS": return 45;  // motion SE
    default:
      return null; // straight or invalid pair
  }
}

/// Highest `chain_index` for a given `train_id` already on the board.
/// Used to suggest the next index when the editor places a CART.
/// Returns 0 if no carriages of that train exist yet (so the first
/// carriage placed gets index 1).
export function highestChainIndex(board: Square[][], trainId: number): number {
  let max = 0;
  for (const row of board) {
    for (const sq of row) {
      if (!sq.piece) continue;
      const cart = parseTrainCart(sq.piece);
      if (cart && cart.kind === "CART" && cart.trainId === trainId && cart.chainIndex > max) {
        max = cart.chainIndex;
      }
    }
  }
  return max;
}

// ----------------------------------------------------------------
// Private parser primitives. `findMatchingParen` / `splitTopLevel`
// here intentionally replicate the helpers used inline by `fen.ts`
// rather than depending on them — this module is consumed by the
// piece-rotation render path that runs once per frame, and we want a
// stable, self-contained parser independent of any future fen.ts
// refactor. Bounds-guarded against unbalanced parens (same shape as
// the engine's defensive parsers).
// ----------------------------------------------------------------

function findMatchingParen(s: string, openIdx: number): number {
  let depth = 0;
  for (let i = openIdx; i < s.length; i++) {
    const ch = s[i];
    if (ch === "(") depth++;
    else if (ch === ")") {
      depth--;
      if (depth < 0) return -1;
      if (depth === 0) return i;
    }
  }
  return -1;
}

function splitTopLevel(input: string): string[] {
  const parts: string[] = [];
  let buf = "";
  let depth = 0;
  for (const ch of input) {
    if (ch === "(") {
      depth++;
      buf += ch;
    } else if (ch === ")") {
      depth = Math.max(0, depth - 1);
      buf += ch;
    } else if (ch === "," && depth === 0) {
      parts.push(buf.trim());
      buf = "";
    } else {
      buf += ch;
    }
  }
  if (buf.length > 0) parts.push(buf.trim());
  return parts;
}
