//! Plan 10 step 5 — square-driven `Candidate` filters and augments.
//!
//! Three modifiers in the 100..199 priority band:
//!
//! - `SquareConditionFilter` (110) — drops candidates whose source
//!   square has `Brainrot` or `Frozen`. Mirrors the existing inline
//!   short-circuit at the top of `Board::get_moves`.
//! - `WalkabilityFilter` (120) — drops candidates whose destination
//!   square is not walkable (closed Gate, Turret, Vent, Block).
//! - `SwitchTileAugment` (130) — emits a `ThrowSwitch` candidate
//!   alongside a piece's own moves when the source square is a
//!   `Switch` tile and the piece passes `can_throw_switch()`.
//!
//! All three are dormant until step 8 wires `Board::get_moves` to
//! `resolve_moves` (no `MoveQuery` ever fires until then). Landing
//! them now means step 8 is purely a callsite migration — the
//! filtering logic is already in place and unit-tested.

use crate::board::{Board, Coord, GameMove, MoveType};
use crate::board::square::{SquareCondition, SquareType};
use crate::movement::stack::{
    EventKindMask, MovementEffect, MovementEvent, MovementModifier,
};

/// Drops `Candidate` events whose source square has `Brainrot` or
/// `Frozen`. Both conditions short-circuit move generation in the
/// pre-refactor `Board::get_moves`; this modifier preserves that
/// semantics inside the stack.
pub struct SquareConditionFilter;

impl MovementModifier for SquareConditionFilter {
    fn id(&self) -> &'static str {
        "square.condition_filter"
    }
    fn priority(&self) -> u32 {
        110
    }
    fn touches(&self) -> EventKindMask {
        EventKindMask::CANDIDATE
    }
    fn apply(&self, board: &Board, event: &MovementEvent) -> MovementEffect {
        let MovementEvent::Candidate { mover, .. } = event else {
            return MovementEffect::Keep;
        };
        let Some(sq) = board.get_square_at(mover) else {
            return MovementEffect::Drop;
        };
        if sq.conditions.contains(&SquareCondition::Brainrot)
            || sq.conditions.contains(&SquareCondition::Frozen)
        {
            return MovementEffect::Drop;
        }
        MovementEffect::Keep
    }
}

/// Drops `Candidate` events whose source OR target square is not
/// walkable. Catches piece-intrinsic move modifiers that proposed a
/// landing on a closed Gate / Turret / Vent / Block — defence-in-
/// depth against new piece types that forget to consult
/// `is_walkable()`.
///
/// **Source check (R7 audit):** also drops candidates whose SOURCE
/// is unwalkable. A piece stranded on a closed Gate / Block (placed
/// by FEN, or marooned when a signal closes a gate under it) is
/// genuinely inert — without this check it remains mobile but
/// uncapturable (target-walkability blocks captors from reaching it),
/// which is an asymmetric and confusing game state. The check
/// applies uniformly to all MoveType arms — including PhaseShift,
/// ThrowSwitch, and PIC — so a stranded piece can do nothing until
/// its square becomes walkable again.
///
/// `ThrowSwitch`, `PhaseShift`, and the internal-to-carrier moves
/// don't land on an outer board square, so the target check applies
/// only to move types that surface a destination coord. Castle is
/// excluded — its path safety is checked at move-gen time per
/// `king.rs::castle_moves`.
pub struct WalkabilityFilter;

impl WalkabilityFilter {
    fn destination(game_move: &GameMove) -> Option<&Coord> {
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
            // Plan 13: placing a tornado doesn't relocate the placer,
            // so there's no landing square to walkability-check.
            | MoveType::PlaceTornado { .. } => None,
        }
    }
}

impl MovementModifier for WalkabilityFilter {
    fn id(&self) -> &'static str {
        "square.walkability"
    }
    fn priority(&self) -> u32 {
        120
    }
    fn touches(&self) -> EventKindMask {
        EventKindMask::CANDIDATE
    }
    fn apply(&self, board: &Board, event: &MovementEvent) -> MovementEffect {
        let MovementEvent::Candidate { mover, game_move } = event else {
            return MovementEffect::Keep;
        };
        // Source-walkability check. Applies uniformly across all
        // MoveType arms — a stranded piece can't even ThrowSwitch /
        // PhaseShift / PIC. Castle's source is the king's tile (back
        // rank Standard), which is always walkable in well-formed
        // setups; the path-safety check in `king.rs::castle_moves`
        // handles the in-between squares.
        if !board.is_walkable_at(mover) {
            return MovementEffect::Drop;
        }
        let Some(dest) = Self::destination(game_move) else {
            return MovementEffect::Keep;
        };
        // Out-of-bounds dests also fall through to Drop via the
        // helper's `unwrap_or(false)`; the piece-intrinsic move
        // modifier shouldn't propose them, but the filter is the
        // safety net.
        if board.is_walkable_at(dest) {
            MovementEffect::Keep
        } else {
            MovementEffect::Drop
        }
    }
}

/// Augments `Candidate` events from a `MoveQuery` source square that
/// is a `Switch` tile: emits an additional `ThrowSwitch` candidate so
/// any piece standing on a Switch can throw it.
///
/// Touches `MOVE_QUERY` rather than `CANDIDATE` — the augment fires
/// once per query, not once per produced candidate. Gated on the
/// piece's `can_throw_switch()` trait method (default `true`).
pub struct SwitchTileAugment;

impl MovementModifier for SwitchTileAugment {
    fn id(&self) -> &'static str {
        "square.switch_augment"
    }
    fn priority(&self) -> u32 {
        130
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
        if !matches!(sq.square_type, SquareType::Switch { .. }) {
            return MovementEffect::Keep;
        }
        let Some(piece) = &sq.piece else {
            return MovementEffect::Keep;
        };
        if !piece.can_throw_switch() {
            return MovementEffect::Keep;
        }
        MovementEffect::Augment(vec![MovementEvent::Candidate {
            mover: from.clone(),
            game_move: GameMove {
                from: from.clone(),
                move_type: MoveType::ThrowSwitch {
                    switch: from.clone(),
                },
            },
        }])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::{BoardFlags, Coord, TrainTickRate};
    use crate::board::square::Square;
    use crate::movement::stack::MovementStack;
    use crate::pieces::Color;
    use crate::pieces::piecetype::PieceType;

    fn empty_board() -> Board {
        let grid = (0..8)
            .map(|_| (0..8).map(|_| Square::new()).collect())
            .collect();
        Board {
            grid,
            flags: BoardFlags {
                side_to_move: Color::White,
                white_can_castle_kingside: false,
                white_can_castle_queenside: false,
                black_can_castle_kingside: false,
                black_can_castle_queenside: false,
                en_passant_target: None,
                train_tick_rate: TrainTickRate::EveryFullTurn,
                ply_count: 0,
                last_move: None,
            },
        }
    }

    /// A `Candidate` whose source square has `Frozen` is dropped.
    #[test]
    fn frozen_source_drops_candidates() {
        let mut board = empty_board();
        board.grid[0][0] = Square::new()
            .set_piece(PieceType::new_rook(Color::White))
            .add_square_condition(SquareCondition::Frozen);
        let filter = SquareConditionFilter;
        let candidate = MovementEvent::Candidate {
            mover: Coord { file: 0, rank: 0 },
            game_move: GameMove {
                from: Coord { file: 0, rank: 0 },
                move_type: MoveType::MoveTo(Coord { file: 0, rank: 1 }),
            },
        };
        assert!(matches!(
            filter.apply(&board, &candidate),
            MovementEffect::Drop
        ));
    }

    /// Walkability filter drops a candidate that targets a Vent.
    #[test]
    fn walkability_filter_drops_unwalkable_target() {
        let mut board = empty_board();
        board.grid[0][0] = Square::new().set_piece(PieceType::new_rook(Color::White));
        board.grid[0][3] = Square::new().set_square_type(SquareType::Vent);
        let filter = WalkabilityFilter;
        let candidate = MovementEvent::Candidate {
            mover: Coord { file: 0, rank: 0 },
            game_move: GameMove {
                from: Coord { file: 0, rank: 0 },
                move_type: MoveType::MoveTo(Coord { file: 3, rank: 0 }),
            },
        };
        assert!(matches!(
            filter.apply(&board, &candidate),
            MovementEffect::Drop
        ));
    }

    /// Switch tile augments a MoveQuery with a ThrowSwitch candidate.
    #[test]
    fn switch_tile_augments_with_throw_switch() {
        let mut board = empty_board();
        board.grid[3][3] = Square::new()
            .set_piece(PieceType::new_rook(Color::White))
            .set_square_type(SquareType::Switch { targets: vec![1] });
        let aug = SwitchTileAugment;
        let query = MovementEvent::MoveQuery {
            from: Coord { file: 3, rank: 3 },
        };
        match aug.apply(&board, &query) {
            MovementEffect::Augment(events) => {
                assert_eq!(events.len(), 1);
                assert!(matches!(
                    &events[0],
                    MovementEvent::Candidate {
                        game_move: GameMove {
                            move_type: MoveType::ThrowSwitch { .. },
                            ..
                        },
                        ..
                    }
                ));
            }
            other => panic!("expected Augment; got {other:?}"),
        }
    }

    /// The augment and the condition filter compose without looping:
    /// register both, fire a query on a Switch square, the augment
    /// fires once and the filter never sees a CANDIDATE-kind event
    /// it could re-feed.
    #[test]
    fn augment_and_filter_compose_without_loop() {
        let mut board = empty_board();
        board.grid[3][3] = Square::new()
            .set_piece(PieceType::new_rook(Color::White))
            .set_square_type(SquareType::Switch { targets: vec![1] });

        let mut stack = MovementStack::new();
        stack.register(Box::new(SwitchTileAugment));
        stack.register(Box::new(SquareConditionFilter));

        let moves = stack.resolve_moves(&board, &Coord { file: 3, rank: 3 });
        // SquareConditionFilter doesn't drop the ThrowSwitch (the
        // source square isn't Frozen). Output: one ThrowSwitch.
        assert_eq!(moves.len(), 1);
        assert!(matches!(
            &moves[0],
            GameMove {
                move_type: MoveType::ThrowSwitch { .. },
                ..
            }
        ));
    }
}
