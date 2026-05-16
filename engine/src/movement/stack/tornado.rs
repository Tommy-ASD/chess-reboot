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

use std::cell::RefCell;

use crate::board::square::SquareCondition;
use crate::board::{Board, Coord, GameMove, MoveType};
use crate::movement::stack::{
    EventKindMask, MovementEffect, MovementEvent, MovementModifier,
    resolve_legal_epoch,
};
use crate::pieces::Color;
use crate::pieces::piecetype::PieceType;

thread_local! {
    /// Audit Round-A/A-DoS — pass-scoped memo of the two
    /// board-invariant facts the compulsion needs: whether any tornado
    /// exists, and whether the side to move can reach one with a
    /// king-safe move. Both depend only on `board`, which is immutable
    /// for the duration of one `resolve_legal_moves` call; that call
    /// bumps `RESOLVE_LEGAL_EPOCH`, so a matching epoch means "same
    /// board, same query" and the cached result is sound to reuse for
    /// every candidate. A different query (different board / different
    /// `status()` piece / different request) has a different epoch and
    /// recomputes — no stale reuse. Thread-local ⇒ no cross-request
    /// sharing. This turns the O(pieces×moves×king-safety) probe from
    /// once-per-candidate into once-per-legal-move-query, closing the
    /// crafted-FEN super-linear DoS amplification.
    static PROBE_MEMO: RefCell<Option<ProbeMemo>> = const { RefCell::new(None) };
}

#[derive(Clone, Copy)]
struct ProbeMemo {
    epoch: u64,
    side: Color,
    any_tornado: bool,
    side_can_reach: bool,
}

/// `(any_tornado(board), side_can_reach_tornado(board, side))`,
/// memoized for the current `resolve_legal_moves` epoch. `side_can_
/// reach` is only probed when a tornado actually exists (mirrors the
/// filter's fast-path), and only the boolean result is cached — never
/// a board reference — so there is no lifetime/aliasing hazard.
fn compelled_facts(board: &Board, side: Color) -> (bool, bool) {
    let epoch = resolve_legal_epoch();
    if let Some(m) = PROBE_MEMO.with(|c| *c.borrow()) {
        if m.epoch == epoch && m.side == side {
            return (m.any_tornado, m.side_can_reach);
        }
    }
    let any = any_tornado(board);
    let can_reach = any && side_can_reach_tornado(board, side);
    PROBE_MEMO.with(|c| {
        *c.borrow_mut() = Some(ProbeMemo {
            epoch,
            side,
            any_tornado: any,
            side_can_reach: can_reach,
        });
    });
    (any, can_reach)
}

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
/// `pub(crate)` so `validate_move` (the make_move enforcement gate)
/// and `TornadoTickHandler` share one definition rather than three
/// drifting copies of the same scan.
pub(crate) fn any_tornado(board: &Board) -> bool {
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
/// Iterates top-level pieces and colour-gates on the top-level piece
/// (line below). A *same-colour* carrier (a Bus — same-colour
/// passengers by invariant) passes the gate, and its `get_moves`
/// surfaces its passengers' exits as `PieceInCarrier` candidates that
/// `move_destination` resolves, so a Bus passenger's exit onto a
/// tornado DOES arm the compulsion (correct per Concept 1). A
/// `Neutral` train cart is skipped by the colour gate, so its own
/// passenger's exit does NOT arm a compulsion — deliberately,
/// matching the trains-immune precedent (plan 13 R1/E-4a; R1/E-4b as
/// corrected in R4 and scoped in R5). Such a passenger is still
/// *subject to* a compulsion armed by the side's other top-level
/// same-colour pieces.
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

        // Board-invariant facts, computed once per legal-move query
        // (pass-scoped memo) rather than once per candidate.
        let side_to_move = board.flags.side_to_move;
        let (has_tornado, side_can_reach) = compelled_facts(board, side_to_move);

        // Fast path: no tornado anywhere → inert.
        if !has_tornado {
            return MovementEffect::Keep;
        }

        let Some(piece) = board.get_square_at(mover).and_then(|s| s.piece.as_ref())
        else {
            return MovementEffect::Keep;
        };

        // Audit R2/finding-1: `mover` is the *carrier's* square for a
        // PieceInCarrier candidate, and `piece` the carrier — not the
        // passenger that is actually moving. Resolve the EFFECTIVE
        // moving piece (mirrors `Board::effective_mover_color` /
        // `find_king`'s carrier descent) so the king-exemption and the
        // side gate see the passenger, not the Bus/cart. Without this a
        // King riding a carrier is neither exempted nor protected from
        // the trap — a Concept-4 violation that yields a false
        // stalemate.
        let is_pic = matches!(game_move.move_type, MoveType::PieceInCarrier { .. });
        let effective_piece: &PieceType = match &game_move.move_type {
            MoveType::PieceInCarrier { piece_index, .. } => piece
                .passengers()
                .and_then(|ps| ps.get(*piece_index as usize))
                .unwrap_or(piece),
            _ => piece,
        };

        // King is fully exempt: never trapped, never restricted.
        // Keyed on the *effective* piece so a king passenger is exempt
        // too. Returned before the trap and compulsion clauses so
        // neither can touch a king move.
        if matches!(effective_piece, PieceType::King(_)) {
            return MovementEffect::Keep;
        }

        // Trap is intrinsic to the piece *standing on* the tornado, not
        // the turn: a non-king piece on a tornado square has no moves
        // while the tornado lives. A PieceInCarrier candidate is a
        // *passenger exiting a carrier* — the passenger is NOT on the
        // tornado; the carrier is, and the carrier's own top-level
        // MoveTo candidates are trapped by this same path (non-PIC),
        // while trains relocate via the env tick and are immune by
        // precedent (R1/E-4a). So the trap applies only to non-PIC
        // candidates. Checked before the side gate so a trapped enemy
        // is correctly inert when queried.
        if !is_pic && is_tornado_square(board, mover) {
            return MovementEffect::Drop;
        }

        // The compulsion restricts only the side to move. Use the
        // EFFECTIVE colour (the passenger's, for a PieceInCarrier
        // candidate out of a Neutral cart) so it stays consistent with
        // `effective_mover_color`/`validate_move`; otherwise a Neutral
        // cart's passenger reads as Neutral and is never compelled.
        // Other-side candidates aren't legal anyway (turn is
        // `validate_move`'s job); leave them untouched. (`side_to_move`
        // was bound above for the memo key.)
        if effective_piece.get_color() != side_to_move {
            return MovementEffect::Keep;
        }

        // Concept 1's "never produces a self-checking move" guarantee
        // depends on KingSafetyFilter (300) having filtered the set this
        // runs over (305) and the probe's capped set (PROBE_CAP=304 still
        // includes 300). A variant that disables king-safety
        // (king_safety.rs has a documented unshipped Duck-Chess
        // short-circuit) voids that guarantee for the compulsion too —
        // such a variant must independently disable/redefine the
        // compulsion. See plan 13 "Check interaction is free" bullet.
        if side_can_reach {
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
    /// resolve). `side_can_reach_tornado` short-circuits on the first
    /// reachable piece, so the position just needs one; completion of
    /// `legal_moves` (rather than a hang) is the assertion.
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

    /// Audit R1/C1: `make_move` (not just `legal_moves`) must enforce
    /// the compulsion. A non-tornado move while compelled is rejected
    /// with `CompelledByTornado`; the forced tornado move is accepted.
    #[test]
    fn make_move_enforces_compulsion() {
        use crate::board::MoveError;

        // Compelled-away move is rejected by make_move itself.
        let mut b = board8();
        b.grid[7][0] = Square::new().set_piece(PieceType::new_rook(Color::White));
        b.grid[7][6] = Square::new().set_piece(PieceType::new_knight(Color::White));
        b.grid[3][0] = Square::new()
            .add_square_condition(SquareCondition::Tornado { remaining: 3 });
        // A geometrically-valid, king-safe knight move that does NOT
        // land on the tornado — legal pre-tornado, illegal under it.
        let knight_move = b.legal_moves(&c(6, 7)); // empty (compelled)
        assert!(knight_move.is_empty());
        let err = b
            .make_move(move_to(c(6, 7), c(5, 5)))
            .expect_err("compelled side must not be able to play a non-tornado move");
        assert!(
            matches!(err, MoveError::CompelledByTornado { .. }),
            "expected CompelledByTornado, got {err:?}"
        );

        // The forced tornado-landing move is accepted by make_move.
        let mut b2 = board8();
        b2.grid[7][0] = Square::new().set_piece(PieceType::new_rook(Color::White));
        b2.grid[7][6] =
            Square::new().set_piece(PieceType::new_knight(Color::White));
        b2.grid[3][0] = Square::new()
            .add_square_condition(SquareCondition::Tornado { remaining: 3 });
        b2.make_move(move_to(c(0, 7), c(0, 3)))
            .expect("the compelled tornado-landing move must be accepted");
    }

    /// Audit R1/C1: a piece trapped on a tornado square cannot be moved
    /// by `make_move` either (the trap is enforced on the execution
    /// path, not only in `legal_moves`).
    #[test]
    fn make_move_rejects_moving_trapped_piece() {
        use crate::board::MoveError;
        let mut b = board8();
        b.grid[3][0] = Square::new()
            .set_piece(PieceType::new_rook(Color::White))
            .add_square_condition(SquareCondition::Tornado { remaining: 3 });
        b.grid[7][6] = Square::new().set_piece(PieceType::new_knight(Color::White));
        let err = b
            .make_move(move_to(c(0, 3), c(1, 3)))
            .expect_err("a trapped piece must not be movable via make_move");
        assert!(
            matches!(err, MoveError::CompelledByTornado { .. }),
            "expected CompelledByTornado, got {err:?}"
        );
        // The unrelated knight is not compelled (only tornado is the
        // occupied (0,3)) — its move executes fine.
        b.make_move(move_to(c(6, 7), c(5, 5)))
            .expect("unrelated piece moves normally; compulsion not armed");
    }

    /// Audit R1/E2: strengthen the recursion guard beyond
    /// termination-as-proof. Proves *structurally* that the probe does
    /// not re-enter the compulsion filter: the const invariant
    /// `PROBE_CAP < TORNADO_PRIORITY` holds, AND the probe's capped
    /// resolve still returns non-tornado moves (i.e. the 305 filter
    /// demonstrably did NOT run inside it) while the full `legal_moves`
    /// (which DOES run 305) collapses to the single forced move. If the
    /// probe re-entered the filter, the capped set would also be
    /// collapsed and this test would fail.
    #[test]
    fn compulsion_probe_does_not_re_enter_filter() {
        // Recursion-guard invariant, asserted (not just commented).
        assert!(PROBE_CAP < TORNADO_PRIORITY);
        assert_eq!(PROBE_CAP, TORNADO_PRIORITY - 1);

        let mut b = board8();
        b.grid[7][0] = Square::new().set_piece(PieceType::new_rook(Color::White));
        b.grid[7][6] = Square::new().set_piece(PieceType::new_knight(Color::White));
        b.grid[3][0] = Square::new()
            .add_square_condition(SquareCondition::Tornado { remaining: 3 });

        let stack = crate::movement::stack::default_stack();
        // Probe-equivalent resolve, capped just below the filter.
        let capped = stack.resolve_moves_capped(&b, &c(0, 7), PROBE_CAP);
        // It reaches the tornado square...
        assert!(
            capped.iter().any(|m| *m == move_to(c(0, 7), c(0, 3))),
            "capped probe must see the tornado-landing move"
        );
        // ...AND it still contains non-tornado moves — proof the 305
        // compulsion filter did NOT run inside the capped probe (no
        // re-entry). If it had, these would be dropped.
        assert!(
            capped.iter().any(|m| match &m.move_type {
                MoveType::MoveTo(d) => !is_tornado_square(&b, d),
                _ => true,
            }),
            "capped probe must retain non-tornado moves (filter excluded)"
        );
        assert!(capped.len() > 1);

        // The full legal set DOES run the 305 filter → collapses to the
        // single forced move. Filter ran outside, not inside the probe;
        // the whole resolution terminated.
        assert_eq!(b.legal_moves(&c(0, 7)), vec![move_to(c(0, 7), c(0, 3))]);
    }

    /// Audit R1/B1: end-to-end — a tornado's countdown is wired into
    /// the real `make_move` pipeline (not just the direct handler) and
    /// dissipates after the expected number of plies. Pieces are
    /// positioned so NOTHING can reach the tornado square (compulsion
    /// never arms), isolating the countdown-through-make_move path.
    #[test]
    fn tornado_dissipates_through_real_make_move() {
        let mut b = board8();
        // Kings present (king-safety meaningful), far from each other
        // and from the tornado; rooks that cannot reach (4,4) in one
        // move and don't check either king.
        b.grid[7][0] = Square::new().set_piece(PieceType::new_king(Color::White));
        b.grid[0][7] = Square::new().set_piece(PieceType::new_king(Color::Black));
        b.grid[7][2] = Square::new().set_piece(PieceType::new_rook(Color::White));
        b.grid[0][5] = Square::new().set_piece(PieceType::new_rook(Color::Black));
        b.grid[4][4] = Square::new()
            .add_square_condition(SquareCondition::Tornado { remaining: 2 });

        assert_eq!(
            b.grid[4][4].conditions,
            vec![SquareCondition::Tornado { remaining: 2 }]
        );

        // Ply 1 (White rook shuffles): tick 2 → 1.
        b.make_move(move_to(c(2, 7), c(3, 7)))
            .expect("white rook move is legal (compulsion not armed)");
        assert_eq!(
            b.grid[4][4].conditions,
            vec![SquareCondition::Tornado { remaining: 1 }],
            "one real make_move ply must tick the tornado once"
        );

        // Ply 2 (Black rook shuffles): tick 1 → 0 → removed.
        b.make_move(move_to(c(5, 0), c(4, 0)))
            .expect("black rook move is legal");
        assert!(
            b.grid[4][4].conditions.is_empty(),
            "tornado must dissipate through the real make_move pipeline"
        );
    }

    /// Audit R2/finding-1: a King riding a carrier on a tornado square
    /// must NOT be trapped (Concept 4 is unconditional). Before the fix
    /// the filter saw the Bus (not the King passenger) and dropped
    /// every exit → false Stalemate. `find_king` descends into the Bus,
    /// so the king is "found" but had no moves.
    #[test]
    fn king_passenger_in_carrier_on_tornado_not_trapped() {
        use crate::board::GameStatus;
        // White Bus carrying the White King on a tornado square at
        // (0,3); lone Black king far away; White to move.
        let b = crate::board::fen::fen_to_board(
            "7k/8/8/(P=BUS(P=(K)),C=TORNADO:3)7/8/8/8/8 w - -",
        );
        let bus = c(0, 3);
        assert!(
            !b.legal_moves(&bus).is_empty(),
            "king-passenger exits must survive — a king in a carrier on \
             a tornado is NOT trapped (Concept 4)"
        );
        assert_eq!(
            b.status(),
            GameStatus::Ongoing,
            "must not be a false Stalemate"
        );
    }

    /// Audit R2/finding-1 (C1 path): the round-1 `validate_move`
    /// enforcement calls `legal_moves` → same filter, so `make_move`
    /// must also let the king-passenger escape (not `CompelledByTornado`).
    #[test]
    fn make_move_lets_king_passenger_escape_carrier_on_tornado() {
        let mut b = crate::board::fen::fen_to_board(
            "7k/8/8/(P=BUS(P=(K)),C=TORNADO:3)7/8/8/8/8 w - -",
        );
        let escape = b
            .legal_moves(&c(0, 3))
            .into_iter()
            .next()
            .expect("king passenger must have an escape move");
        b.make_move(escape)
            .expect("make_move must accept the king-passenger's escape");
    }

    /// Audit R2/finding-1 + R3/A6: the non-king counterpart of
    /// `king_passenger_in_carrier_on_tornado_not_trapped`. A carrier on
    /// a tornado must NOT trap its (non-king) passenger — the passenger
    /// is not the piece standing on the tornado; the carrier is. (R3/A6:
    /// the *carrier's own* / a normal piece's non-PIC trap is robustly
    /// covered by `traps_the_occupant` and
    /// `make_move_rejects_moving_trapped_piece`; the earlier
    /// "no top-level MoveTo" assertion here was vacuous — a lone Bus
    /// emits no top-level move anyway — so it's replaced by the
    /// discriminating "every move is a passenger exit" check below.)
    #[test]
    fn non_king_passenger_of_carrier_on_tornado_not_trapped() {
        let b = crate::board::fen::fen_to_board(
            "7k/8/8/(P=BUS(P=(R)),C=TORNADO:3)7/8/8/8/8 w - -",
        );
        let moves = b.legal_moves(&c(0, 3));
        // The rook passenger genuinely has exits (not vacuously empty),
        // and every legal move from the carrier square is a passenger
        // exit: the non-king passenger is NOT trapped, and the compulsion
        // is not spuriously armed (the only piece is the on-tornado Bus,
        // skipped by the probe's on-tornado skip — so non-tornado exits
        // are kept rather than dropped).
        assert!(
            !moves.is_empty()
                && moves
                    .iter()
                    .all(|m| matches!(m.move_type, MoveType::PieceInCarrier { .. })),
            "non-king passenger of a carrier-on-tornado must keep its \
             exits (got {moves:?})"
        );
    }

    /// Audit R3/B1: a rook trapped on a tornado square must NOT be
    /// rescued by castling (plan 13 Concept 2) — same rationale as the
    /// existing closed-Gate "stranded rook rescued by castling" guard
    /// in `king::castle_moves`. The king itself stays tornado-exempt
    /// (Concept 4): only the rook's tornado blocks the castle.
    #[test]
    fn castle_blocked_when_rook_trapped_on_tornado() {
        // Control: White can castle kingside (K e1, R h1, rights, clear).
        let mut b = board8();
        b.flags.white_can_castle_kingside = true;
        b.grid[7][4] = Square::new().set_piece(PieceType::new_king(Color::White));
        b.grid[7][7] = Square::new().set_piece(PieceType::new_rook(Color::White));
        assert!(
            b.legal_moves(&c(4, 7))
                .iter()
                .any(|m| matches!(m.move_type, MoveType::Castle { .. })),
            "control: castling must be legal without the tornado"
        );

        // Tornado on the rook's home square → rook trapped → no castle.
        let mut b2 = board8();
        b2.flags.white_can_castle_kingside = true;
        b2.grid[7][4] = Square::new().set_piece(PieceType::new_king(Color::White));
        b2.grid[7][7] = Square::new()
            .set_piece(PieceType::new_rook(Color::White))
            .add_square_condition(SquareCondition::Tornado { remaining: 3 });
        let king_moves = b2.legal_moves(&c(4, 7));
        assert!(
            !king_moves
                .iter()
                .any(|m| matches!(m.move_type, MoveType::Castle { .. })),
            "a rook trapped on a tornado must not be rescued by castling"
        );
        // The king is NOT over-restricted: Concept 4 keeps its normal
        // steps (it is not on a tornado and the compulsion is unarmed —
        // the only other white piece, the rook, is on the tornado and
        // is skipped by the probe).
        assert!(
            king_moves
                .iter()
                .any(|m| matches!(m.move_type, MoveType::MoveTo(_))),
            "king keeps its ordinary moves (only the castle is blocked)"
        );
    }

    /// Audit R3/B4: discriminating companion to
    /// `compulsion_intersects_check_no_force_when_unsafe` (which only
    /// asserted on the always-exempt king). A NON-king piece with a
    /// legal, non-tornado-landing check evasion must keep it: the
    /// compulsion must not arm off a move that isn't king-safe (the
    /// probe is capped *above* king-safety, so an only-reachable-while-
    /// in-check tornado cannot arm it).
    #[test]
    fn compulsion_intersects_check_non_king_evasion_survives() {
        let mut b = board8();
        b.grid[7][4] = Square::new().set_piece(PieceType::new_king(Color::White));
        // Black rook checks down file 4.
        b.grid[0][4] = Square::new().set_piece(PieceType::new_rook(Color::Black));
        // White rook on rank 0 can capture the checker at (4,0) — a
        // king-safe, check-resolving, NON-tornado move.
        b.grid[0][7] = Square::new().set_piece(PieceType::new_rook(Color::White));
        // Tornado in the corner: reachable only by a move that does NOT
        // resolve the check (so king-safety drops it, in the real set
        // AND in the capped probe) → compulsion must stay unarmed.
        b.grid[0][0] = Square::new()
            .add_square_condition(SquareCondition::Tornado { remaining: 3 });

        let rook_moves = b.legal_moves(&c(7, 0));
        assert!(
            !rook_moves.is_empty()
                && rook_moves.iter().any(|m| *m == move_to(c(7, 0), c(4, 0))),
            "the non-king rook must keep its check-resolving capture; \
             the tornado must not arm the compulsion off a non-king-safe \
             move (got {rook_moves:?})"
        );
    }

    /// Audit Round-A/A-DoS regression: the pass-scoped probe memo must
    /// NOT leak across distinct legal-move queries on the same thread.
    /// Each `legal_moves` call bumps the epoch, so a compelled board
    /// and a non-compelled board evaluated back-to-back on one thread
    /// must each get their own correct result (no stale reuse).
    #[test]
    fn probe_memo_does_not_leak_across_queries() {
        // Compelled position: rook forced onto the tornado, knight gets
        // nothing.
        let mut compelled = board8();
        compelled.grid[7][0] =
            Square::new().set_piece(PieceType::new_rook(Color::White));
        compelled.grid[7][6] =
            Square::new().set_piece(PieceType::new_knight(Color::White));
        compelled.grid[3][0] = Square::new()
            .add_square_condition(SquareCondition::Tornado { remaining: 3 });

        // A completely separate, tornado-free position.
        let mut free = board8();
        free.grid[7][6] =
            Square::new().set_piece(PieceType::new_knight(Color::White));

        // Interleave on ONE thread. If the memo keyed on anything
        // weaker than the per-query epoch, the second/third results
        // would be the first's stale answer.
        assert_eq!(
            compelled.legal_moves(&c(0, 7)),
            vec![move_to(c(0, 7), c(0, 3))],
            "compelled query #1"
        );
        assert!(
            !free.legal_moves(&c(6, 7)).is_empty(),
            "non-compelled query must NOT reuse the compelled memo"
        );
        assert!(
            compelled.legal_moves(&c(6, 7)).is_empty(),
            "compelled query #2 (knight) must recompute, not reuse free's"
        );
        assert!(
            !free.legal_moves(&c(6, 7)).is_empty(),
            "non-compelled query #2 still correct after interleaving"
        );
    }
}
