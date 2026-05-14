//! Plan 10 steps 6–7 — train geometry modifiers.
//!
//! Train threats and their filters live here, separated from piece-
//! intrinsic logic. Three modifiers:
//!
//! - `TrainHeadCrushModifier` (210) — for each Locomotive, emits a
//!   `Threat` event for its next-tick tile. Same threat the legacy
//!   `Locomotive::attacks` returns; emitting from a board-state
//!   reading modifier lets train rules evolve without per-piece
//!   override fan-out.
//! - `TrainCartCaptureFilter` (211, step 7) — drops `Threat` events
//!   whose target square holds any train cart. Implements the
//!   "chain-following isn't a capture" rule. Subsumes the
//!   `Locomotive::would_capture_at` override.
//! - `TwoTrainCollisionFilter` (212, step 7) — drops `Threat` events
//!   where two locomotives target the same tile this tick (mutual
//!   stop).

use crate::board::Board;
use crate::movement::stack::{
    EventKindMask, MovementEffect, MovementEvent, MovementModifier,
};
use crate::pieces::piecetype::PieceType;

/// Step 6: emit next-tick crush threats for every Locomotive on the
/// board. Augments — never drops — the threat working set. Sits at
/// priority 210; step 7's filters at 211/212 then sanitise the
/// threats this modifier emits along with the ones piece_attacks
/// already produced.
///
/// **Redundancy note:** as of step 6 the existing
/// `piece_attacks.locomotive` modifier ALSO emits threats for the
/// loco's next-tile (via `Locomotive::attacks` returning that tile).
/// `is_attacked_by` only cares about non-empty, so duplicate threats
/// for the same target are harmless. Step 7's filters will drop both
/// equally. A future cleanup can empty `Locomotive::attacks` once
/// every threat path flows through here.
pub struct TrainHeadCrushModifier;

impl MovementModifier for TrainHeadCrushModifier {
    fn id(&self) -> &'static str {
        "train.head_crush"
    }
    fn priority(&self) -> u32 {
        210
    }
    fn touches(&self) -> EventKindMask {
        EventKindMask::THREAT_QUERY
    }
    fn apply(&self, board: &Board, event: &MovementEvent) -> MovementEffect {
        let MovementEvent::ThreatQuery { target, .. } = event else {
            return MovementEffect::Keep;
        };
        let mut threats: Vec<MovementEvent> = Vec::new();
        for (coord, piece) in board.iter_pieces() {
            let PieceType::Locomotive(loco) = piece else {
                continue;
            };
            // Stranded loco projects no head-crush threat.
            // `next_train_step` already returns None for non-
            // Track/Junction sources, so most stranded cases
            // short-circuit downstream — this guard catches a
            // hand-crafted FEN or any future relaxation.
            if !board.is_walkable_at(&coord) {
                continue;
            }
            // Connection-aware traversal: pass `last_dir` so the
            // neighbour-detection branch sees the same exit-side as
            // `Locomotive::attacks` and `advance_trains`. Using the
            // cold-start `next_train_tile` wrapper (which discards
            // `last_dir`) diverges at curves and dead-ends after the
            // first tick — a king parked on the phantom-cold-start
            // tile would read as in-check.
            let Some(next) = board
                .next_train_step(&coord, loco.heading, loco.last_dir)
                .map(|(c, _)| c)
            else {
                continue;
            };
            if &next != target {
                continue;
            }
            // Inline cart-target filter: the loco can't crush its own
            // chain-follower, so a same-train cart on the next tile
            // means no movement, no threat. Step 7's
            // `TrainCartCaptureFilter` owns this conceptually; here
            // it's inlined so this modifier produces the right
            // threat set in step 6 standalone.
            let blocked_by_cart = board
                .get_square_at(&next)
                .and_then(|sq| sq.piece.as_ref())
                .map(|p| p.is_train_cart())
                .unwrap_or(false);
            if blocked_by_cart {
                continue;
            }
            threats.push(MovementEvent::Threat {
                attacker: coord.clone(),
                attacker_piece: piece.clone(),
                target: target.clone(),
            });
        }
        MovementEffect::Augment(threats)
    }
}

/// Step 7a: drop `Threat` events whose target square holds any train
/// cart (Locomotive or Carriage). Captures are not possible against
/// invincible cart bodies — chain-following onto a same-train cart's
/// tile is a stop, not a capture; ramming a foreign cart is also a
/// stop (foreign-cart filter in `advance_trains`).
///
/// This filter is the modifier-form of the
/// `Locomotive::would_capture_at` override. The override stays on
/// the piece for now (it's still consulted by
/// `piece_attacks.locomotive` at priority 40), but new train rules
/// should land here rather than in the piece file.
pub struct TrainCartCaptureFilter;

impl MovementModifier for TrainCartCaptureFilter {
    fn id(&self) -> &'static str {
        "train.cart_capture_filter"
    }
    fn priority(&self) -> u32 {
        211
    }
    fn touches(&self) -> EventKindMask {
        EventKindMask::THREAT
    }
    fn apply(&self, board: &Board, event: &MovementEvent) -> MovementEffect {
        let MovementEvent::Threat {
            target,
            attacker_piece,
            ..
        } = event
        else {
            return MovementEffect::Keep;
        };
        // Only filter when a train cart is the attacker — chain-
        // following / cart-vs-cart is the only case where landing on
        // another cart isn't a capture. Non-train pieces (Monkey,
        // Skibidi, etc.) can absolutely capture passengers by
        // landing on a cart, so their threats must pass through.
        if !attacker_piece.is_train_cart() {
            return MovementEffect::Keep;
        }
        let target_is_cart = board
            .get_square_at(target)
            .and_then(|sq| sq.piece.as_ref())
            .map(|p| p.is_train_cart())
            .unwrap_or(false);
        if target_is_cart {
            MovementEffect::Drop
        } else {
            MovementEffect::Keep
        }
    }
}

/// Step 7b: when two locomotives would both step onto the same tile
/// this tick, neither moves (mutual stop) — and so neither generates
/// a crush threat for that tile. This filter drops `Threat` events
/// from a Locomotive whose computed next-tile is shared with another
/// Locomotive's computed next-tile.
///
/// Subtle: this is a *threat-set* filter, not the apply-time
/// collision resolver in `trains.rs::advance_trains`. The latter
/// handles the actual movement; this one prevents the king-safety
/// query from seeing a threat that won't materialise.
pub struct TwoTrainCollisionFilter;

impl MovementModifier for TwoTrainCollisionFilter {
    fn id(&self) -> &'static str {
        "train.two_train_collision_filter"
    }
    fn priority(&self) -> u32 {
        212
    }
    fn touches(&self) -> EventKindMask {
        EventKindMask::THREAT
    }
    fn apply(&self, board: &Board, event: &MovementEvent) -> MovementEffect {
        let MovementEvent::Threat {
            attacker,
            attacker_piece,
            target,
        } = event
        else {
            return MovementEffect::Keep;
        };
        // Only locomotives produce two-train collisions.
        let PieceType::Locomotive(this_loco) = attacker_piece else {
            return MovementEffect::Keep;
        };
        // Confirm this attacker IS the train head computing the
        // crush threat — otherwise we'd accidentally drop unrelated
        // threats that happen to share a Locomotive attacker.
        // `last_dir`-aware so this matches the head-crush emission.
        let our_next = board
            .next_train_step(attacker, this_loco.heading, this_loco.last_dir)
            .map(|(c, _)| c);
        if our_next.as_ref() != Some(target) {
            return MovementEffect::Keep;
        }
        // Scan for another Locomotive whose next-tile is the same.
        for (other_coord, other_piece) in board.iter_pieces() {
            if &other_coord == attacker {
                continue;
            }
            let PieceType::Locomotive(other_loco) = other_piece else {
                continue;
            };
            let Some(other_next) = board
                .next_train_step(&other_coord, other_loco.heading, other_loco.last_dir)
                .map(|(c, _)| c)
            else {
                continue;
            };
            if &other_next == target {
                // Both locomotives would land here. Mutual stop —
                // no crush threat.
                return MovementEffect::Drop;
            }
        }
        MovementEffect::Keep
    }
}
