//! Property-based regression tests via `proptest`. Each property runs
//! a randomised sequence of legal moves from the standard starting
//! position and asserts an invariant after every step. The shrinker
//! turns any failure into a minimal reproducer (a small `picks` vec).
//!
//! Properties:
//! 1. FEN round-trip — `fen_to_board(board_to_fen(b))` equals `b` at
//!    every step. Brainrot conditions are derived state, but we play
//!    from a standard chess position (no Skibidis) so the raw grid
//!    equality suffices.
//! 2. legal-move applicability — every move returned by `legal_moves`
//!    applies via `make_move` without error. (`make_move` should never
//!    reject a move that `legal_moves` produced.)
//! 3. piece-count delta — after a single `make_move`, the total piece
//!    count on the board changes by 0 (non-capture) or -1 (capture).
//!    Castling moves a rook but doesn't capture, so the delta is 0.
//!    Promotion changes a piece type, not the count.
//! 4. move-changes-state — `make_move` is never a no-op on a legal
//!    move. The board after a move must differ from the board before.
//!    Catches an entire class of "move silently dropped" bugs.
//!
//! Note: `picks` is sized `1..40` so the inner loop runs at least
//! once, and the per-iteration `iters_performed` counter is asserted
//! `> 0` at the end. Without this, a `picks` vec of length 0 would
//! pass the property vacuously — and the shrinker, looking for the
//! smallest failure, would collapse any real bug to the empty case
//! and report it as passing.

use engine::board::{
    Board, GameMove,
    fen::{board_to_fen, fen_to_board},
};
use proptest::prelude::*;

fn standard_start() -> Board {
    fen_to_board("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq -")
}

/// Collects every move legal_moves offers for the side to move.
fn collect_all_legal(board: &Board) -> Vec<GameMove> {
    let mut out = Vec::new();
    for (coord, piece) in board.all_pieces() {
        if piece.get_color() != board.flags.side_to_move {
            continue;
        }
        out.extend(board.legal_moves(&coord));
    }
    out
}

proptest! {
    /// Drive 1-39 random legal moves from the start position and assert
    /// all four properties at each step.
    #[test]
    fn fen_roundtrip_and_invariants_under_random_play(
        picks in prop::collection::vec(any::<u32>(), 1..40)
    ) {
        let mut board = standard_start();
        let mut iters_performed = 0usize;
        for &pick in &picks {
            let legal = collect_all_legal(&board);
            if legal.is_empty() {
                break;
            }
            iters_performed += 1;

            let idx = (pick as usize) % legal.len();
            let chosen = legal[idx].clone();

            let pieces_before = board.all_pieces().len();
            let board_before = board.clone();

            // Property 2: every legal move applies cleanly.
            let mut after = board.clone();
            after.make_move(chosen)
                .expect("legal_moves output must apply via make_move");

            // Property 4: a legal move actually changes the board.
            prop_assert_ne!(
                &after, &board_before,
                "make_move on a legal move must not be a no-op"
            );

            // Property 3: piece-count delta is 0 or -1.
            let pieces_after = after.all_pieces().len();
            let delta = pieces_after as isize - pieces_before as isize;
            prop_assert!(
                delta == 0 || delta == -1,
                "piece count delta must be 0 (move) or -1 (capture), got {delta}"
            );

            // Property 1: FEN round-trip is exact.
            let fen = board_to_fen(&after);
            let recovered = fen_to_board(&fen);
            prop_assert_eq!(&recovered, &after, "FEN round-trip mismatch");

            board = after;
        }
        // Defence against vacuous-pass: ensure at least one step ran.
        // From the standard start, `collect_all_legal` always returns 20
        // moves on the first pass, so reaching zero here would indicate
        // a genuine bug (or a starting-position regression).
        prop_assert!(
            iters_performed > 0,
            "test must execute at least one inner iteration"
        );
    }
}
