//! Plan 10 step 9 — king-safety as a modifier.
//!
//! `KingSafetyFilter` sits at priority 300, so `resolve_moves` (capped
//! at 299) doesn't run it but `resolve_legal_moves` (uncapped) does.
//! `Board::legal_moves` uses the latter; `Board::get_moves` and the
//! geometric-check step of `Board::validate_move` use the former.
//!
//! **Plan 11 hook:** the filter reads
//! `BoardFlags.has_variant(VariantId::DuckChess)` (when plan 11
//! lands) and short-circuits to `Keep` — Duck Chess has no concept
//! of check. The variant infrastructure isn't in the engine yet,
//! so today the check is a TODO. Once plan 11 ships, that's a
//! single-line read.
//!
//! **Critical invariant:** the hypothetical board's threat resolution
//! must not recurse into `resolve_moves`, or we'd infinite-loop.
//!
//! The implementation route: clone-and-apply via
//! `apply_move_for_validation`, then `is_in_check` → `is_attacked_by`
//! → `resolve_threats` on the **full default stack**. That stack
//! includes this filter, but recursion is prevented by the
//! `touches() = EventKindMask::CANDIDATE` mask below — the dispatcher
//! skips this modifier for `Threat` events. Loop-freedom rests on
//! that mask. Do NOT remove the `touches` impl without rewiring the
//! hypothetical to a stripped-down threat path.

use crate::board::Board;
use crate::movement::stack::{
    EventKindMask, MovementEffect, MovementEvent, MovementModifier,
};

pub struct KingSafetyFilter;

impl MovementModifier for KingSafetyFilter {
    fn id(&self) -> &'static str {
        "king_safety"
    }
    fn priority(&self) -> u32 {
        300
    }
    fn touches(&self) -> EventKindMask {
        EventKindMask::CANDIDATE
    }
    fn apply(&self, board: &Board, event: &MovementEvent) -> MovementEffect {
        let MovementEvent::Candidate { mover, game_move } = event else {
            return MovementEffect::Keep;
        };

        // Plan 11 (Duck Chess) hook. When `BoardFlags.variants`
        // contains `VariantId::DuckChess`, king-safety is disabled
        // entirely (Duck Chess has no concept of check). The
        // variants infrastructure isn't shipped yet; once it lands,
        // replace this TODO with:
        //   if board.flags.has_variant(VariantId::DuckChess) {
        //       return MovementEffect::Keep;
        //   }
        // For now this is a no-op.

        let Some(source_piece) = board.get_square_at(mover).and_then(|s| s.piece.as_ref())
        else {
            // No piece at source — the move couldn't apply anyway.
            // Let the downstream apply-time error surface naturally.
            return MovementEffect::Keep;
        };

        let (mover_color, _) = board.effective_mover_color(source_piece, game_move);

        let mut hypothetical = board.clone();
        match hypothetical.apply_move_for_validation(game_move.clone()) {
            Ok(()) => {
                if hypothetical.is_in_check(mover_color) {
                    MovementEffect::Drop
                } else {
                    MovementEffect::Keep
                }
            }
            // The hypothetical apply failed — the move can't legally
            // execute. Drop so `legal_moves` doesn't emit a move
            // `make_move` would reject. (This mirrors the legacy
            // `legal_moves` filter's `Err(_) => false`.)
            Err(_) => MovementEffect::Drop,
        }
    }
}
