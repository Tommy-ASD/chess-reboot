//! Plan 11 (Duck Chess) — movement-stack modifier for the duck blocker.
//!
//! `DuckBlockerFilter` drops any `Candidate` whose piece-landing square
//! holds the duck: no piece may land on the duck (it cannot be captured).
//! Gliders additionally stop *at* the duck — a sliding ray can't pass
//! through it. That ray-truncation lives in `generate_glider_moves`,
//! because a candidate filter can only drop the land-on-duck move, not
//! the slide-past moves a glider would otherwise emit beyond it.
//! Together they make the duck a hard blocker for every mover.
//!
//! Duck *placement* legality and the duck-phase piece-move suppression
//! live in `Board::validate_move` / `Board::get_moves` rather than here:
//! the "place the duck on any empty square" move set isn't anchored to a
//! source piece, so it doesn't fit the per-square stack. See
//! `Board::duck_moves`.

use crate::board::{Board, Coord, GameMove, MoveType};
use crate::movement::stack::{
    EventKindMask, MovementEffect, MovementEvent, MovementModifier,
};

/// Drops `Candidate` events whose piece-landing square holds the duck.
pub struct DuckBlockerFilter;

impl DuckBlockerFilter {
    /// The square a piece would land on, for move types that relocate a
    /// piece onto a board square. Mirrors `WalkabilityFilter::destination`
    /// — non-relocating actions (ThrowSwitch / PhaseShift / PlaceTornado)
    /// and the duck moves themselves surface no piece-landing square.
    fn landing(game_move: &GameMove) -> Option<&Coord> {
        match &game_move.move_type {
            MoveType::MoveTo(c) => Some(c),
            MoveType::Promotion { target, .. } => Some(target),
            MoveType::EnPassant { target, .. } => Some(target),
            MoveType::MoveIntoCarrier(c) => Some(c),
            MoveType::PieceInCarrier { move_type, .. } => match move_type.as_ref() {
                MoveType::MoveTo(c) => Some(c),
                MoveType::MoveIntoCarrier(c) => Some(c),
                _ => None,
            },
            MoveType::Castle { .. }
            | MoveType::PhaseShift
            | MoveType::ThrowSwitch { .. }
            | MoveType::PlaceTornado { .. }
            | MoveType::PlaceDuck { .. }
            | MoveType::MoveDuck { .. } => None,
        }
    }
}

impl MovementModifier for DuckBlockerFilter {
    fn id(&self) -> &'static str {
        "duck.blocker"
    }
    fn priority(&self) -> u32 {
        // Square-filter band, just after walkability (120). Below the 299
        // cap so it runs in both `resolve_moves` and `resolve_legal_moves`
        // — the duck blocks raw move-gen and the legal set alike.
        125
    }
    fn touches(&self) -> EventKindMask {
        EventKindMask::CANDIDATE
    }
    fn apply(&self, board: &Board, event: &MovementEvent) -> MovementEffect {
        let MovementEvent::Candidate { game_move, .. } = event else {
            return MovementEffect::Keep;
        };
        let Some(dest) = Self::landing(game_move) else {
            return MovementEffect::Keep;
        };
        if board.get_square_at(dest).map(|s| s.duck).unwrap_or(false) {
            MovementEffect::Drop
        } else {
            MovementEffect::Keep
        }
    }
}
