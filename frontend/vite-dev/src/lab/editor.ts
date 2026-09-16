// src/lab/editor.ts
//
// The Lab's editing model, free of DOM: brushes, what applying a brush to a
// square does, resizing, and the palettes / templates. Ported from the old
// `editor_page.ts`; the rules are unchanged, but every operation now returns
// a new board instead of mutating the one it was given.

import { cloneSquare } from "../core/mirror";
import {
  ALL_TRACK_DIRS,
  getReceiverId,
  isEmitter,
  isReceiver,
  nextSignalId,
  setEmitterTargets,
  setGateId,
  setGateOpen,
  setJunctionBranches,
  setJunctionId,
  setJunctionState,
  setPlateTrigger,
  setTrackDir,
  toggleEmitterTarget,
  type TrackDir,
} from "../core/signal_payload";
import {
  firstOrphanTrainId,
  highestChainIndex,
  highestTrainId,
  parseTrainCart,
  serializeTrainCart,
} from "../core/train_payload";
import type { Coord, Square, SquareType } from "../core/types";

export type Brush =
  | { kind: "piece"; piece: string }
  | { kind: "duck" }
  | { kind: "type"; squareType: SquareType }
  | { kind: "condition"; condition: string }
  | { kind: "erase-piece" }
  | { kind: "erase-conditions" }
  | { kind: "erase-type" }
  | { kind: "erase-all" }
  /// Inspect / wire: clicking a substrate square opens its details; with an
  /// emitter open, clicking a receiver toggles it in the emitter's targets.
  | { kind: "inspect" };

export const STARTING_FEN = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - tr=full p=0";
export const EMPTY_FEN = "8/8/8/8/8/8/8/8 w - - tr=full p=0";

export const WHITE_PIECES = ["K", "Q", "R", "B", "N", "P"];
export const BLACK_PIECES = ["k", "q", "r", "b", "n", "p"];
/// Chess 2 pieces, white then black. Train brushes carry minimal payload —
/// placement allocates ids (see `withSmartTrainPayload`); the inspector
/// tunes the rest.
export const CHESS2_PIECES = ["G", "g", "S", "s", "M", "m", "W", "w", "BUS", "bus"];
export const TRAIN_PIECES = ["LOCO(ID=1,H=F)", "CART(ID=1,I=1)"];

export const SQUARE_TYPES: SquareType[] = ["STANDARD", "VENT", "TURRET", "BLOCK", "SWITCH", "JUNCTION", "GATE", "PLATE", "TRACK"];

export const CONDITIONS: { value: string; label: string }[] = [
  { value: "FROZEN", label: "Frozen" },
  { value: "BRAINROT", label: "Brainrot" },
  { value: "TORNADO:3", label: "Tornado" },
];

export const TEMPLATES: { name: string; fen: string }[] = [
  { name: "Standard chess", fen: STARTING_FEN },
  { name: "Empty board", fen: EMPTY_FEN },
  { name: "Goblin test", fen: "(P=g(H=0-0))nbqkbn(P=g(H=7-0))/pppppppp/8/8/8/8/PPPPPPPP/(P=G(H=0-7))NBQKBN(P=G(H=7-7)) w KQkq -" },
  { name: "Vent test", fen: "(T=VENT)7/8/8/8/8/8/8/8 w - -" },
  { name: "Block test", fen: "(T=BLOCK)7/8/8/8/8/8/8/8 w - -" },
  { name: "Frozen test", fen: "(C=FROZEN)7/8/8/8/8/8/8/8 w - -" },
  {
    name: "Signal substrate test",
    fen: "(T=SWITCH,TARGETS=(3,7))(T=JUNCTION,ID=3,STATE=0,BRANCHES=(N,E))(T=GATE,ID=7,OPEN=0)(T=PLATE,TARGETS=(3),FIRES=B)4/8/8/8/8/8/8/8 w - -",
  },
  {
    // Loco at the western end of a 5-tile east-pointing rail.
    name: "Train test (straight east)",
    fen: "8/8/8/8/(T=TRACK,D=E,P=LOCO(ID=1,H=F))(T=TRACK,D=E)(T=TRACK,D=E)(T=TRACK,D=E)(T=TRACK,D=E)3/8/8/8 w - - tr=ply p=0",
  },
  {
    // 4-tile loop in a 2×2 square; the cart trails one step behind.
    name: "Train test (loop + cart)",
    fen: "8/8/8/3(T=TRACK,D=E,P=LOCO(ID=1,H=F))(T=TRACK,D=E)3/3(T=TRACK,D=E,P=CART(ID=1,I=1))(T=TRACK,D=E)3/8/8/8 w - - tr=ply p=0",
  },
];

/// Accent colour per substrate type, matching the `.type-*` board rules.
export const TYPE_ACCENT: Record<SquareType, string> = {
  STANDARD: "rgba(180, 180, 180, 0.7)",
  TURRET: "rgba(180, 90, 30, 0.95)",
  VENT: "rgba(60, 60, 60, 0.85)",
  BLOCK: "rgba(90, 70, 50, 0.95)",
  SWITCH: "rgba(255, 196, 0, 0.95)",
  JUNCTION: "rgba(120, 200, 255, 0.95)",
  GATE: "rgba(220, 80, 80, 0.95)",
  PLATE: "rgba(170, 130, 255, 0.95)",
  TRACK: "rgba(200, 170, 110, 0.95)",
};

export const TYPE_LABELS: Record<SquareType, string> = {
  STANDARD: "Standard",
  VENT: "Vent",
  TURRET: "Turret",
  BLOCK: "Block",
  SWITCH: "Switch",
  JUNCTION: "Junction",
  GATE: "Gate",
  PLATE: "Plate",
  TRACK: "Track",
};

export function cloneBoard(board: Square[][]): Square[][] {
  return board.map((row) => row.map(cloneSquare));
}

export function describeBrush(b: Brush | null): string {
  if (!b) return "Pick a brush, then tap squares to paint.";
  switch (b.kind) {
    case "piece": return `Placing ${b.piece.split("(")[0]}`;
    case "duck": return "Placing the duck";
    case "type": return `Painting ${TYPE_LABELS[b.squareType]} squares`;
    case "condition": return `Toggling ${b.condition.split(":")[0].toLowerCase()}`;
    case "erase-piece": return "Erasing pieces";
    case "erase-conditions": return "Erasing conditions";
    case "erase-type": return "Resetting square types";
    case "erase-all": return "Erasing everything";
    case "inspect": return "Inspecting: tap a special square or train";
  }
}

export function sameBrush(a: Brush | null, b: Brush): boolean {
  return a !== null && JSON.stringify(a) === JSON.stringify(b);
}

/// Rewrite a LOCO/CART brush so each placement allocates a fresh train id
/// (LOCO) or the next chain index of the latest train (CART). Other pieces
/// pass through unchanged.
export function withSmartTrainPayload(board: Square[][], piece: string): string {
  const cart = parseTrainCart(piece);
  if (!cart) return piece;
  if (cart.kind === "LOCO") {
    // Adopt carts placed before any loco so the chain isn't orphaned.
    const orphan = firstOrphanTrainId(board);
    cart.trainId = orphan !== null ? orphan : highestTrainId(board) + 1;
  } else {
    const target = Math.max(1, highestTrainId(board));
    cart.trainId = target;
    cart.chainIndex = highestChainIndex(board, target) + 1;
  }
  return serializeTrainCart(cart);
}

/// When a locomotive lands on a track tile, point the tile's `D` away from
/// its own carriages so the first tick moves the train out of the chain.
export function pickLocoStartDir(board: Square[][], file: number, rank: number, trainId: number): TrackDir | null {
  const cardinal: { dir: TrackDir; df: number; dr: number }[] = [
    { dir: "N", df: 0, dr: -1 },
    { dir: "E", df: 1, dr: 0 },
    { dir: "S", df: 0, dr: 1 },
    { dir: "W", df: -1, dr: 0 },
  ];
  const behind: TrackDir[] = [];
  const candidates: TrackDir[] = [];
  for (const { dir, df, dr } of cardinal) {
    const sq = board[rank + dr]?.[file + df];
    if (!sq || (sq.squareType !== "TRACK" && sq.squareType !== "JUNCTION")) continue;
    const c = sq.piece ? parseTrainCart(sq.piece) : null;
    if (c && c.trainId === trainId) {
      behind.push(dir);
      continue;
    }
    candidates.push(dir);
  }
  if (candidates.length === 0) return null;
  const opposite: Record<TrackDir, TrackDir> = { N: "S", S: "N", E: "W", W: "E" };
  for (const b of behind) {
    if (candidates.includes(opposite[b])) return opposite[b];
  }
  return candidates[0];
}

const isTornado = (c: string) => c === "TORNADO" || c.startsWith("TORNADO:");

export interface BrushResult {
  board: Square[][];
  inspected: Coord | null;
}

/// Apply `brush` at `at`. Returns the new board and inspected square.
export function applyBrush(board: Square[][], brush: Brush, at: Coord, inspected: Coord | null): BrushResult {
  const next = cloneBoard(board);
  const sq = next[at.rank]?.[at.file];
  if (!sq) return { board, inspected };

  switch (brush.kind) {
    case "piece": {
      sq.piece = withSmartTrainPayload(next, brush.piece);
      delete sq.duck;
      const placed = parseTrainCart(sq.piece);
      if (placed?.kind === "LOCO" && sq.squareType === "TRACK") {
        const dir = pickLocoStartDir(next, at.file, at.rank, placed.trainId);
        if (dir !== null) setTrackDir(sq, dir);
      }
      break;
    }
    case "duck":
      // Only one duck: moving it clears the old square.
      for (const row of next) for (const s of row) delete s.duck;
      sq.piece = null;
      sq.duck = true;
      break;
    case "type":
      sq.squareType = brush.squareType;
      delete sq.extraFields;
      // Freshly painted substrate gets well-formed defaults.
      if (brush.squareType === "JUNCTION") {
        setJunctionId(sq, nextSignalId(next));
        setJunctionState(sq, 0);
        setJunctionBranches(sq, [...ALL_TRACK_DIRS]);
      } else if (brush.squareType === "GATE") {
        setGateId(sq, nextSignalId(next));
        setGateOpen(sq, true);
      } else if (brush.squareType === "SWITCH") {
        setEmitterTargets(sq, []);
      } else if (brush.squareType === "PLATE") {
        setEmitterTargets(sq, []);
        setPlateTrigger(sq, "ANY");
      } else if (brush.squareType === "TRACK") {
        setTrackDir(sq, "E");
      }
      break;
    case "condition": {
      const c = brush.condition;
      const has = isTornado(c) ? sq.conditions.some(isTornado) : sq.conditions.includes(c);
      sq.conditions = has ? sq.conditions.filter((x) => (isTornado(c) ? !isTornado(x) : x !== c)) : [...sq.conditions, c];
      break;
    }
    case "erase-piece":
      sq.piece = null;
      delete sq.duck;
      break;
    case "erase-conditions":
      sq.conditions = [];
      break;
    case "erase-type":
      sq.squareType = "STANDARD";
      delete sq.extraFields;
      break;
    case "erase-all":
      sq.piece = null;
      delete sq.duck;
      sq.conditions = [];
      sq.squareType = "STANDARD";
      delete sq.extraFields;
      break;
    case "inspect": {
      const open = inspected ? next[inspected.rank]?.[inspected.file] ?? null : null;
      if (open && isEmitter(open) && isReceiver(sq)) {
        const rid = getReceiverId(sq);
        if (rid !== null) toggleEmitterTarget(open, rid);
        return { board: next, inspected };
      }
      return { board: next, inspected: { file: at.file, rank: at.rank } };
    }
  }
  return { board: next, inspected };
}

/// Resize to `cols`×`rows`, keeping the top-left overlap.
export function resizeBoard(board: Square[][], cols: number, rows: number): Square[][] {
  const c = Math.max(1, Math.round(cols));
  const r = Math.max(1, Math.round(rows));
  const next: Square[][] = [];
  for (let y = 0; y < r; y++) {
    const row: Square[] = [];
    for (let x = 0; x < c; x++) {
      const existing = board[y]?.[x];
      row.push(existing ? cloneSquare(existing) : { piece: null, squareType: "STANDARD", conditions: [] });
    }
    next.push(row);
  }
  return next;
}

export function inBounds(board: Square[][], c: Coord | null): c is Coord {
  return c !== null && c.rank >= 0 && c.rank < board.length && c.file >= 0 && c.file < (board[0]?.length ?? 0);
}
