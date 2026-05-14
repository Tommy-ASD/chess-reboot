//! Plan 10 step 8 — piece-intrinsic move generation.
//!
//! One modifier in the 0..99 band that converts a `MoveQuery` seed
//! into `Candidate` events by reading the piece at the query's source
//! square and calling `PieceType::get_moves` on it. The carrier-
//! rewrite filter (the 140-line block in `PieceType::get_moves`)
//! stays where it is — calling `get_moves` rather than `initial_moves`
//! gets us the rewritten moves for free.
//!
//! The square-driven filters at priority 100–199
//! (`SquareConditionFilter`, `WalkabilityFilter`,
//! `SwitchTileAugment`) sit downstream and post-process the
//! `Candidate` set this modifier emits.

use crate::board::Board;
use crate::board::square::SquareCondition;
use crate::movement::stack::{
    EventKindMask, MovementEffect, MovementEvent, MovementModifier,
};

/// Piece-intrinsic move emitter at priority 30 (just below the
/// piece-attack band at 40). For a `MoveQuery { from }` event, reads
/// the piece at `from` and emits one `Candidate` per legal move
/// `PieceType::get_moves` returns.
///
/// Early-exits on `Brainrot` / `Frozen` source conditions for
/// performance — `SquareConditionFilter` at 110 would catch the same
/// case downstream, but skipping `piece.get_moves` avoids running a
/// full move generator for a square that produces nothing.
pub struct PieceMovesModifier;

impl MovementModifier for PieceMovesModifier {
    fn id(&self) -> &'static str {
        "piece_intrinsic.moves"
    }
    fn priority(&self) -> u32 {
        30
    }
    fn touches(&self) -> EventKindMask {
        EventKindMask::MOVE_QUERY
    }
    fn apply(&self, board: &Board, event: &MovementEvent) -> MovementEffect {
        let MovementEvent::MoveQuery { from } = event else {
            return MovementEffect::Keep;
        };
        let Some(sq) = board.get_square_at(from) else {
            return MovementEffect::Keep;
        };
        if sq.conditions.contains(&SquareCondition::Brainrot)
            || sq.conditions.contains(&SquareCondition::Frozen)
        {
            // Brainrot/Frozen sources produce NO moves — including any
            // square-driven augments. We `Replace` the seed with an
            // empty set rather than `Keep`-ing it, so downstream
            // MOVE_QUERY-touching modifiers (`SwitchTileAugment` at
            // priority 130) don't fire on a query the source square's
            // conditions should suppress entirely. `SquareConditionFilter`
            // (priority 110) only touches `CANDIDATE` events, so it
            // can't catch a post-augment ThrowSwitch Candidate; the
            // MoveQuery itself must die here.
            return MovementEffect::Replace(Vec::new());
        }
        let Some(piece) = sq.piece.as_ref() else {
            return MovementEffect::Keep;
        };
        let moves = piece.get_moves(board, from);
        let candidates: Vec<MovementEvent> = moves
            .into_iter()
            .map(|m| MovementEvent::Candidate {
                mover: from.clone(),
                game_move: m,
            })
            .collect();
        MovementEffect::Augment(candidates)
    }
}
