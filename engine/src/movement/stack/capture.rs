//! Plan 10 step 10 — the capture pipeline.
//!
//! A sibling registry to `MovementStack`. Where the movement stack
//! answers "what moves exist" and "what threats exist," the capture
//! stack answers "what should happen when piece X is captured at
//! coord Y." Driven by `ResolutionEvent`s emitted from
//! `make_move.rs`'s `relocate_pieces` at every capture point
//! (MoveTo / Promotion / EnPassant / carrier boarding).
//!
//! Handlers return `BoardOp`s — declarative mutations — instead of
//! closures. This keeps them loggable for debug-trace, replayable
//! for any future undo work, and safely commutable across handlers
//! when more than one fires for the same event.
//!
//! **Reserved variants** (commented out, no consumer yet): the
//! `PieceLanded` and `PieceDeparted` events from the plan's
//! reservation list are not declared here yet — uncomment when the
//! first consumer (Sediment, Witness, Plague Doctor miasma) lands.

// Reservation hooks: `len`, `is_empty`, `modifier_ids`, and some
// `BoardOp` variants land in step 10 with the Goblin handler as the
// sole consumer. Drop the allow once Bomb / Antipode / Jackhammer
// land and exercise the rest of the surface.
#![allow(dead_code)]

use std::sync::OnceLock;

use crate::board::{Board, Coord, MoveType};
use crate::board::square::SquareCondition;
use crate::pieces::piecetype::PieceType;

/// Events the capture pipeline operates on. Distinct from
/// `MovementEvent` — they fire during apply, not during generation —
/// and intentionally narrow (`Capture` only, for now).
#[derive(Clone, Debug)]
pub enum ResolutionEvent {
    /// `victim` is being captured by `captor`. The board mutation has
    /// already applied (the captor sits at `captor_coord`, the victim
    /// has been cleared) when handlers run — so handlers can inspect
    /// the post-relocation board and emit `BoardOp`s that mutate it
    /// further before piece post-effects fire.
    ///
    /// Field semantics:
    /// - `captor_coord` is the captor's POST-MOVE position. For
    ///   `MoveTo` / `Promotion` / `EnPassant` it equals the move's
    ///   `target`. For `PieceInCarrier { MoveTo }` it's the
    ///   passenger's exit tile (the inner `MoveTo` target). Use this
    ///   for "captor is here now" effects (Echo compulsion,
    ///   Bomb-finds-captor-as-casualty).
    /// - `captor_origin` is the captor's PRE-MOVE position on the
    ///   outer board, if any. `Some(from)` for direct captures.
    ///   **`None`** in two cases: (a) `PieceInCarrier` — the captor
    ///   emerged from inside a carrier and had no outer-board origin;
    ///   (b) `advance_trains` (R4 audit F1) — the train head's previous
    ///   tile is occupied by carriage 1 post-tick, so there's no
    ///   clean drop site. Use this for "drop something where the
    ///   captor's old square is" effects (Goblin kidnap-drop,
    ///   Jackhammer A1 push-back). Handlers that want origin-tile
    ///   semantics must gracefully handle `None` rather than assuming
    ///   a coord always exists.
    /// - `victim_coord` is where the captured piece was. Equals
    ///   `captor_coord` for MoveTo / Promotion / PIC{MoveTo}; differs
    ///   for EnPassant (captured pawn lives one square away from the
    ///   captor's landing tile).
    /// - `move_type` is the full GameMove's MoveType. Retained for
    ///   handlers that want move-shape distinctions beyond
    ///   captor_origin (e.g. "this was a promotion-with-capture").
    Capture {
        captor_coord: Coord,
        captor_origin: Option<Coord>,
        captor: PieceType,
        victim_coord: Coord,
        victim: PieceType,
        move_type: MoveType,
    },
    // RESERVED for follow-up plans (no consumer yet):
    // PieceLanded { piece: PieceType, at: Coord }   — Sediment, Plague Doctor miasma
    // PieceDeparted { piece: PieceType, from: Coord } — Witness notches, retrograde tooling
}

/// What a capture handler returns.
#[derive(Clone, Debug)]
pub enum ResolutionEffect {
    /// No-op. The standard capture mutation still applies.
    Keep,
    /// Apply this `BoardOp` (or composite) after the standard capture
    /// mutation. Handlers can chain via `BoardOp::Compose`.
    Mutate(BoardOp),
}

/// Declarative mutation primitive. Each variant maps to a single
/// board-state change. `Compose` chains multiple ops; the dispatcher
/// applies them in order.
///
/// Why declarative instead of closures: loggable for debug-trace,
/// replayable for any future undo work, safely commutable across
/// handlers, doesn't break `Send` invariants on the modifier type.
///
/// **R6 audit forward-compat contract:** any new variant that mutates
/// `square.piece` MUST call `board.maybe_clear_castle_on_rook_capture`
/// on both the pre-existing occupant (if any) and any departing
/// piece. The existing `RemovePiece` and `PlacePiece` arms in
/// `BoardOp::apply` do this — mirror the pattern for new variants
/// (e.g. a hypothetical `MovePiece { from, to }` or `SwapPieces`).
/// Without this, a future modifier emitting the new op on a corner
/// rook would silently leave `white_can_castle_*` / `black_can_castle_*`
/// set even after the rook is gone.
#[derive(Clone, Debug)]
pub enum BoardOp {
    RemovePiece {
        at: Coord,
    },
    PlacePiece {
        at: Coord,
        piece: PieceType,
    },
    SetCondition {
        at: Coord,
        condition: SquareCondition,
    },
    ClearCondition {
        at: Coord,
        condition: SquareCondition,
    },
    SetPassengerList {
        at: Coord,
        passengers: Vec<PieceType>,
    },
    Compose(Vec<BoardOp>),
}

impl BoardOp {
    /// Apply this op (or composite) to `board`. Out-of-bounds ops are
    /// dropped with a tracing warn — handlers shouldn't emit them.
    ///
    /// `RemovePiece` and `PlacePiece` run
    /// `maybe_clear_castle_on_rook_capture` on the displaced piece
    /// before overwriting. The revoke is idempotent, so it's safe to
    /// duplicate alongside the inline calls in `make_move.rs` and
    /// `trains.rs` — and required so handlers that emit these ops on
    /// corner-rook squares (e.g. an AOE Bomb design) don't leave
    /// stale castle rights.
    pub fn apply(&self, board: &mut Board) {
        match self {
            BoardOp::RemovePiece { at } => {
                let Some(sq) = board.get_square_mut(at) else {
                    tracing::warn!(?at, "BoardOp::RemovePiece out of bounds");
                    return;
                };
                let removed = sq.piece.take();
                if let Some(p) = &removed {
                    board.maybe_clear_castle_on_rook_capture(at, p);
                }
            }
            BoardOp::PlacePiece { at, piece } => {
                // PlacePiece can overwrite an existing occupant
                // (defacto capture); mirror RemovePiece so corner-rook
                // overwrites revoke castle rights.
                let Some(sq) = board.get_square_mut(at) else {
                    tracing::warn!(?at, "BoardOp::PlacePiece out of bounds");
                    return;
                };
                let overwritten = sq.piece.replace(piece.clone());
                if let Some(p) = &overwritten {
                    board.maybe_clear_castle_on_rook_capture(at, p);
                }
            }
            BoardOp::SetCondition { at, condition } => {
                if let Some(sq) = board.get_square_mut(at) {
                    if !sq.conditions.contains(condition) {
                        sq.conditions.push(condition.clone());
                    }
                } else {
                    tracing::warn!(?at, "BoardOp::SetCondition out of bounds");
                }
            }
            BoardOp::ClearCondition { at, condition } => {
                if let Some(sq) = board.get_square_mut(at) {
                    sq.conditions.retain(|c| c != condition);
                } else {
                    tracing::warn!(?at, "BoardOp::ClearCondition out of bounds");
                }
            }
            BoardOp::SetPassengerList { at, passengers } => {
                // Three failure modes — out-of-bounds, no piece, or
                // piece isn't a carrier — all of which the other
                // BoardOp variants log on. Round-3 audit caught the
                // silent fall-through here; mirror the warn pattern.
                let Some(sq) = board.get_square_mut(at) else {
                    tracing::warn!(?at, "BoardOp::SetPassengerList out of bounds");
                    return;
                };
                let Some(piece) = sq.piece.as_mut() else {
                    tracing::warn!(?at, "BoardOp::SetPassengerList: no piece at square");
                    return;
                };
                let Some(list) = piece.passengers_mut() else {
                    tracing::warn!(
                        ?at,
                        "BoardOp::SetPassengerList: piece at square is not a carrier"
                    );
                    return;
                };
                *list = passengers.clone();
            }
            BoardOp::Compose(ops) => {
                for op in ops {
                    op.apply(board);
                }
            }
        }
    }
}

/// One layer of the capture stack. Same shape as `MovementModifier`
/// but for resolution-time events.
pub trait CaptureModifier: Send + Sync {
    fn id(&self) -> &'static str;
    fn priority(&self) -> u32;
    fn apply(&self, board: &Board, event: &ResolutionEvent) -> ResolutionEffect;
}

/// Registry, mirroring `MovementStack`. Built at engine init via
/// `default_capture_stack()`.
pub struct CaptureStack {
    modifiers: Vec<Box<dyn CaptureModifier>>,
}

impl Default for CaptureStack {
    fn default() -> Self {
        Self::new()
    }
}

impl CaptureStack {
    pub fn new() -> Self {
        Self { modifiers: Vec::new() }
    }

    pub fn register(&mut self, m: Box<dyn CaptureModifier>) {
        self.modifiers.push(m);
        self.modifiers.sort_by_key(|m| m.priority());
    }

    pub fn len(&self) -> usize {
        self.modifiers.len()
    }

    pub fn is_empty(&self) -> bool {
        self.modifiers.is_empty()
    }

    pub fn modifier_ids(&self) -> impl Iterator<Item = &'static str> + '_ {
        self.modifiers.iter().map(|m| m.id())
    }

    /// Fire all registered handlers for a single capture event.
    /// Returns the list of `BoardOp`s the dispatcher should apply
    /// (in priority order).
    pub fn resolve_capture(&self, board: &Board, event: &ResolutionEvent) -> Vec<BoardOp> {
        let mut ops = Vec::new();
        for modifier in &self.modifiers {
            match modifier.apply(board, event) {
                ResolutionEffect::Keep => {}
                ResolutionEffect::Mutate(op) => ops.push(op),
            }
        }
        ops
    }
}

/// Process-wide default capture stack. Currently registers the
/// Goblin drop-victim handler (plan 04).
pub fn default_capture_stack() -> &'static CaptureStack {
    static STACK: OnceLock<CaptureStack> = OnceLock::new();
    STACK.get_or_init(build_default_capture_stack)
}

fn build_default_capture_stack() -> CaptureStack {
    let mut s = CaptureStack::new();
    s.register(Box::new(GoblinDropVictimCapture));
    s
}

/// Plan 04: when a Goblin in `Kidnapping` state is captured by an
/// enemy, the kidnapped piece is dropped onto the goblin's old
/// square (the captor's *origin* square, which is now empty post-
/// relocation in standard MoveTo captures).
///
/// **Where the victim lands** — at `captor_origin`. The captor moved
/// FROM `captor_origin` TO `victim_coord`. So `captor_origin` is now
/// empty after relocation, and the kidnap victim drops there.
///
/// **PIC captures have `captor_origin = None`** — the passenger
/// emerged from inside a carrier, no outer-board origin. There's no
/// clean drop site (the carrier still occupies its tile, and pushing
/// the freed piece into the passenger list would break carrier-color
/// and capacity invariants). We skip the drop in this case; the
/// kidnap victim is silently lost, matching the plan-09 Q7 "silent
/// passenger removal" precedent.
pub struct GoblinDropVictimCapture;

impl CaptureModifier for GoblinDropVictimCapture {
    fn id(&self) -> &'static str {
        "capture.goblin_drop_victim"
    }
    fn priority(&self) -> u32 {
        100
    }
    fn apply(&self, _board: &Board, event: &ResolutionEvent) -> ResolutionEffect {
        let ResolutionEvent::Capture {
            captor_origin,
            victim,
            ..
        } = event;
        // Only fires when the victim is a Goblin in Kidnapping state.
        let PieceType::Goblin(goblin) = victim else {
            return ResolutionEffect::Keep;
        };
        use crate::pieces::fairy::goblin::GoblinState;
        let GoblinState::Kidnapping { piece } = &goblin.state else {
            return ResolutionEffect::Keep;
        };
        // No outer-board origin → no drop site (PIC capture). Silent
        // loss is the documented precedent.
        let Some(origin) = captor_origin else {
            return ResolutionEffect::Keep;
        };
        let kidnapped: PieceType = piece.as_ref().clone();
        ResolutionEffect::Mutate(BoardOp::PlacePiece {
            at: origin.clone(),
            piece: kidnapped,
        })
    }
}
