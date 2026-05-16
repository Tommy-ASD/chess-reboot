//! Plan 13 commit 3 — the tornado destination-compulsion filter.
//!
//! `TornadoCompulsionFilter` sits at priority 305, just after
//! king-safety (300). It is the load-bearing piece of plan 13: it
//! turns `SquareCondition::Tornado` from inert payload into the
//! "you must move *something* *here*" rule.
//!
//! Three behaviours, all on `Candidate` events:
//!
//! 1. **Trap.** A non-king piece standing on a tornado square cannot
//!    leave — all of its candidates are dropped.
//! 2. **Compulsion.** If the side to move can reach *any* tornado
//!    square with a king-safe move, every candidate of that side that
//!    does NOT land on a tornado square is dropped.
//! 3. **King exemption.** A king never triggers, satisfies, is
//!    trapped by, or is restricted by the tornado. See plan 13
//!    Concept (4) for why (the trap, not the compulsion, is the
//!    reason).
//!
//! **Recursion guard.** Behaviour (2) needs to know "can this side
//! reach a tornado square" — which means running move generation. If
//! the probe ran the full stack it would re-enter this filter
//! forever. The probe therefore runs `resolve_moves_capped` with the
//! cap at [`PROBE_CAP`] (= `TORNADO_PRIORITY - 1`): it sees king-safe
//! moves (king-safety is 300) but never this filter (305). King-
//! safety itself only recurses on the `Threat` path, where this
//! filter — `touches() == CANDIDATE` — is skipped. Loop-free.
//!
//! **Perf.** The probe is O(side pieces × moves) and runs once per
//! candidate, gated behind a board-wide "is there any tornado at all"
//! scan so the overwhelmingly common (no-tornado) case is cheap. A
//! pass-scoped memo of the probe result is a known future
//! optimisation (plan 13, "Things to be careful about"); correctness
//! does not depend on it.

use crate::board::square::SquareCondition;
use crate::board::{Board, Coord, GameMove, MoveType};
use crate::movement::stack::{
    EventKindMask, MovementEffect, MovementEvent, MovementModifier,
};
use crate::pieces::piecetype::PieceType;

/// This filter's stack priority. After king-safety (300) so the
/// probe and the final set are both over king-safe moves.
pub const TORNADO_PRIORITY: u32 = 305;

/// Cap for the reachability probe: one below this filter's own
/// priority, so the probe runs king-safety but never re-enters the
/// compulsion. The `-1` relationship to [`TORNADO_PRIORITY`] is the
/// recursion guard — do not break it.
pub const PROBE_CAP: u32 = TORNADO_PRIORITY - 1;

/// Does any square on the board carry a `Tornado` condition? Cheap
/// fast-path gate: when false the whole filter is a no-op `Keep`.
fn any_tornado(board: &Board) -> bool {
    board.grid.iter().flatten().any(|sq| {
        sq.conditions
            .iter()
            .any(|c| matches!(c, SquareCondition::Tornado { .. }))
    })
}

/// Does the square at `coord` carry a `Tornado` condition?
fn is_tornado_square(board: &Board, coord: &Coord) -> bool {
    board
        .get_square_at(coord)
        .map(|sq| {
            sq.conditions
                .iter()
                .any(|c| matches!(c, SquareCondition::Tornado { .. }))
        })
        .unwrap_or(false)
}

/// The square a move would land its piece on, or `None` for moves
/// that don't relocate the piece onto a single board square
/// (`ThrowSwitch`, `PhaseShift`, `Castle`). Mirrors
/// `WalkabilityFilter::destination` — kept in lockstep with it; the
/// exhaustive match means a new piece-relocating `MoveType` arm is a
/// compile error here until classified.
fn move_destination(game_move: &GameMove) -> Option<Coord> {
    match &game_move.move_type {
        MoveType::MoveTo(c) => Some(c.clone()),
        MoveType::Promotion { target, .. } => Some(target.clone()),
        MoveType::EnPassant { target, .. } => Some(target.clone()),
        MoveType::MoveIntoCarrier(c) => Some(c.clone()),
        MoveType::PieceInCarrier { move_type, .. } => match move_type.as_ref() {
            MoveType::MoveTo(c) => Some(c.clone()),
            MoveType::MoveIntoCarrier(c) => Some(c.clone()),
            _ => None,
        },
        MoveType::Castle { .. }
        | MoveType::PhaseShift
        | MoveType::ThrowSwitch { .. }
        // PlaceTornado does not relocate the placer — it can never be
        // the move that "lands on" the tornado for compulsion.
        | MoveType::PlaceTornado { .. } => None,
    }
}

/// Can `side` reach any tornado square with a king-safe move?
///
/// Skips kings (a king never satisfies the compulsion — Concept 4)
/// and skips pieces that are themselves trapped on a tornado square
/// (a trapped piece cannot be the one that fulfils the compulsion;
/// without this skip a multi-tornado position would let a trapped
/// piece's phantom reachability falsely arm the compulsion).
fn side_can_reach_tornado(board: &Board, side: crate::pieces::Color) -> bool {
    let stack = crate::movement::stack::default_stack();
    for (coord, piece) in board.iter_pieces() {
        if piece.get_color() != side {
            continue;
        }
        if matches!(piece, PieceType::King(_)) {
            continue;
        }
        if is_tornado_square(board, &coord) {
            continue;
        }
        for m in stack.resolve_moves_capped(board, &coord, PROBE_CAP) {
            if let Some(dest) = move_destination(&m) {
                if is_tornado_square(board, &dest) {
                    return true;
                }
            }
        }
    }
    false
}

pub struct TornadoCompulsionFilter;

impl MovementModifier for TornadoCompulsionFilter {
    fn id(&self) -> &'static str {
        "square.tornado_compulsion"
    }
    fn priority(&self) -> u32 {
        TORNADO_PRIORITY
    }
    fn touches(&self) -> EventKindMask {
        EventKindMask::CANDIDATE
    }
    fn apply(&self, board: &Board, event: &MovementEvent) -> MovementEffect {
        let MovementEvent::Candidate { mover, game_move } = event else {
            return MovementEffect::Keep;
        };

        // Fast path: no tornado anywhere → inert.
        if !any_tornado(board) {
            return MovementEffect::Keep;
        }

        let Some(piece) = board.get_square_at(mover).and_then(|s| s.piece.as_ref())
        else {
            return MovementEffect::Keep;
        };

        // King is fully exempt: never trapped, never restricted.
        // Returned before the trap and compulsion clauses so neither
        // can touch a king move.
        if matches!(piece, PieceType::King(_)) {
            return MovementEffect::Keep;
        }

        // Trap is intrinsic to the piece, not the turn: a non-king
        // piece on a tornado square simply has no moves while the
        // tornado lives. Checked before the side gate so a trapped
        // enemy is correctly inert when queried.
        if is_tornado_square(board, mover) {
            return MovementEffect::Drop;
        }

        // The compulsion restricts only the side to move. Other-side
        // candidates aren't legal anyway (turn is `validate_move`'s
        // job); leave them untouched.
        let side_to_move = board.flags.side_to_move;
        if piece.get_color() != side_to_move {
            return MovementEffect::Keep;
        }

        if side_can_reach_tornado(board, side_to_move) {
            let lands_on_tornado = move_destination(game_move)
                .map(|d| is_tornado_square(board, &d))
                .unwrap_or(false);
            if !lands_on_tornado {
                return MovementEffect::Drop;
            }
        }

        MovementEffect::Keep
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::square::Square;
    use crate::board::{BoardFlags, TrainTickRate};
    use crate::pieces::Color;

    fn board8() -> Board {
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

    fn c(file: u8, rank: u8) -> Coord {
        Coord { file, rank }
    }

    fn move_to(from: Coord, to: Coord) -> GameMove {
        GameMove {
            from,
            move_type: MoveType::MoveTo(to),
        }
    }

    /// Compulsion: a side that can reach a tornado square must — every
    /// non-tornado-landing move of that side is illegal.
    #[test]
    fn compulsion_restricts_the_set() {
        let mut b = board8();
        // White rook on file 0, rank 7; clear path north to a tornado
        // at file 0, rank 3.
        b.grid[7][0] = Square::new().set_piece(PieceType::new_rook(Color::White));
        b.grid[7][6] = Square::new().set_piece(PieceType::new_knight(Color::White));
        b.grid[3][0] = Square::new()
            .add_square_condition(SquareCondition::Tornado { remaining: 3 });

        // Rook: every move dropped except the one landing on (0,3).
        let rook_moves = b.legal_moves(&c(0, 7));
        assert_eq!(rook_moves, vec![move_to(c(0, 7), c(0, 3))]);

        // Knight cannot reach (0,3) → fully compelled → no moves.
        assert!(b.legal_moves(&c(6, 7)).is_empty());
    }

    /// When no piece of the side can reach a tornado square, normal
    /// rules resume — the tornado is inert for that turn.
    #[test]
    fn no_reachable_mover_is_noop() {
        let mut b = board8();
        b.grid[7][0] = Square::new().set_piece(PieceType::new_rook(Color::White));
        b.grid[7][6] = Square::new().set_piece(PieceType::new_knight(Color::White));
        // Enemy rook blocks the white rook's path to the tornado.
        b.grid[5][0] = Square::new().set_piece(PieceType::new_rook(Color::Black));
        b.grid[3][0] = Square::new()
            .add_square_condition(SquareCondition::Tornado { remaining: 3 });

        // No white piece can land on (0,3): rook is blocked at (0,5),
        // knight can't reach. Compulsion off → knight moves normally.
        assert!(
            !b.legal_moves(&c(6, 7)).is_empty(),
            "compulsion must not fire when the tornado is unreachable"
        );
    }

    /// A non-king piece standing on a tornado square is trapped: zero
    /// legal moves until the tornado dissipates. Unaffected pieces
    /// move freely.
    #[test]
    fn traps_the_occupant() {
        let mut b = board8();
        b.grid[3][0] = Square::new()
            .set_piece(PieceType::new_rook(Color::White))
            .add_square_condition(SquareCondition::Tornado { remaining: 3 });
        b.grid[7][6] = Square::new().set_piece(PieceType::new_knight(Color::White));

        assert!(
            b.legal_moves(&c(0, 3)).is_empty(),
            "piece on a tornado square must be trapped"
        );
        assert!(
            !b.legal_moves(&c(6, 7)).is_empty(),
            "an unrelated piece moves freely (compulsion not armed — \
             the only tornado is occupied by own piece)"
        );
    }

    /// An enemy piece trapped on a tornado square: the side to move,
    /// able to capture into it, is *forced* to.
    #[test]
    fn forces_capture_of_trapped_enemy() {
        let mut b = board8();
        // Black rook trapped on the tornado at (0,3).
        b.grid[3][0] = Square::new()
            .set_piece(PieceType::new_rook(Color::Black))
            .add_square_condition(SquareCondition::Tornado { remaining: 3 });
        // White rook with a clear capture path; white knight that can't.
        b.grid[7][0] = Square::new().set_piece(PieceType::new_rook(Color::White));
        b.grid[7][6] = Square::new().set_piece(PieceType::new_knight(Color::White));

        // White is compelled into the capture and nothing else.
        assert_eq!(
            b.legal_moves(&c(0, 7)),
            vec![move_to(c(0, 7), c(0, 3))],
            "white must capture the trapped enemy on the tornado"
        );
        assert!(b.legal_moves(&c(6, 7)).is_empty());
        // The trapped black rook has no moves (trap is turn-independent).
        assert!(b.legal_moves(&c(0, 3)).is_empty());
    }

    /// King exemption, two faces: a king is neither *restricted* by an
    /// active compulsion nor *trapped* by sitting on a tornado square.
    #[test]
    fn king_is_exempt() {
        // (a) Compulsion armed by a rook; the king's own moves are
        //     NOT restricted to tornado-landing ones.
        let mut b = board8();
        b.grid[7][4] = Square::new().set_piece(PieceType::new_king(Color::White));
        b.grid[7][0] = Square::new().set_piece(PieceType::new_rook(Color::White));
        b.grid[3][0] = Square::new()
            .add_square_condition(SquareCondition::Tornado { remaining: 3 });
        assert!(
            !b.legal_moves(&c(4, 7)).is_empty(),
            "king moves must survive an active compulsion (exempt)"
        );

        // (b) King standing on a tornado square is not trapped.
        let mut b2 = board8();
        b2.grid[3][0] = Square::new()
            .set_piece(PieceType::new_king(Color::White))
            .add_square_condition(SquareCondition::Tornado { remaining: 3 });
        b2.grid[7][6] = Square::new().set_piece(PieceType::new_knight(Color::White));
        assert!(
            !b2.legal_moves(&c(0, 3)).is_empty(),
            "a king on a tornado square must not be trapped"
        );
    }

    /// The probe terminates (no infinite recursion through the capped
    /// resolve). Several reachable pieces exercise the probe loop more
    /// than once; completion is the assertion.
    #[test]
    fn compulsion_terminates_no_recursion() {
        let mut b = board8();
        b.grid[7][0] = Square::new().set_piece(PieceType::new_rook(Color::White));
        b.grid[7][1] = Square::new().set_piece(PieceType::new_rook(Color::White));
        b.grid[7][6] = Square::new().set_piece(PieceType::new_knight(Color::White));
        b.grid[3][0] = Square::new()
            .add_square_condition(SquareCondition::Tornado { remaining: 3 });
        // If the recursion guard were wrong this would hang; reaching
        // the assert at all is the proof.
        let moves = b.legal_moves(&c(0, 7));
        assert_eq!(moves, vec![move_to(c(0, 7), c(0, 3))]);
    }

    /// Compulsion ∩ check, the guarantee that matters: when no
    /// tornado-landing move is king-safe (reaching the tornado does
    /// not resolve the check), the compulsion does NOT fire and normal
    /// check evasion stands. The probe is capped *above* king-safety,
    /// so it too only sees king-safe moves — it cannot arm the
    /// compulsion off an illegal move.
    #[test]
    fn compulsion_intersects_check_no_force_when_unsafe() {
        let mut b = board8();
        b.grid[7][4] = Square::new().set_piece(PieceType::new_king(Color::White));
        // Black rook checks down file 4.
        b.grid[0][4] = Square::new().set_piece(PieceType::new_rook(Color::Black));
        // White rook could slide to a tornado at (0,0) — but that move
        // leaves the king in check, so king-safety drops it (in the
        // real set AND in the probe).
        b.grid[7][0] = Square::new().set_piece(PieceType::new_rook(Color::White));
        b.grid[0][0] = Square::new()
            .add_square_condition(SquareCondition::Tornado { remaining: 3 });

        // King still has its normal escape(s); compulsion never fired.
        let king_moves = b.legal_moves(&c(4, 7));
        assert!(
            !king_moves.is_empty(),
            "king must keep its check-evasion moves; tornado must not \
             force an unreachable-when-safe square"
        );
        // And none of those escapes is the tornado square.
        assert!(king_moves.iter().all(|m| match &m.move_type {
            MoveType::MoveTo(d) => !is_tornado_square(&b, d),
            _ => true,
        }));
    }

    /// Compulsion ∩ check, the other face: a non-king piece whose
    /// *only* king-safe tornado-landing move also resolves the check
    /// is forced to exactly that move (a block through the tornado
    /// square) — and it is legal, because check resolution is baked
    /// into the set before this filter runs.
    ///
    /// The king, being exempt (Concept 4), is NOT compelled: it keeps
    /// its own check escapes even while the compulsion is active. So
    /// the interesting invariant is "non-king pieces are compelled to
    /// the legal check-resolving square; the king stays free" — the
    /// three rules (compulsion, check, king-exemption) coexisting
    /// correctly, not "the block is the only move on the board."
    #[test]
    fn compulsion_intersects_check_forces_legal_block() {
        let mut b = board8();
        b.grid[7][4] = Square::new().set_piece(PieceType::new_king(Color::White));
        b.grid[0][4] = Square::new().set_piece(PieceType::new_rook(Color::Black));
        // White bishop can interpose on file 4 at (4,3); that square
        // carries the tornado.
        b.grid[0][1] = Square::new().set_piece(PieceType::new_bishop(Color::White));
        b.grid[3][4] = Square::new()
            .add_square_condition(SquareCondition::Tornado { remaining: 3 });

        // Bishop's only legal move is the forced, check-resolving,
        // tornado-landing interpose. (Non-king piece → compelled; the
        // forced move is legal because it also resolves the check.)
        assert_eq!(
            b.legal_moves(&c(1, 0)),
            vec![move_to(c(1, 0), c(4, 3))],
            "bishop must be forced to the block-through-tornado"
        );

        // The king is exempt: the compulsion does not strip its check
        // escapes. It still has at least one legal evasion, and none
        // of its moves is forced onto the tornado square.
        let king_moves = b.legal_moves(&c(4, 7));
        assert!(
            !king_moves.is_empty(),
            "king is exempt — compulsion must not remove its check escapes"
        );
        assert!(
            king_moves.iter().all(|m| match &m.move_type {
                MoveType::MoveTo(d) => !is_tornado_square(&b, d),
                _ => true,
            }),
            "king is never compelled toward the tornado"
        );
    }
}
