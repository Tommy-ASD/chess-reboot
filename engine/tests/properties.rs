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
    square::{Square, SquareCondition},
};
use proptest::prelude::*;

fn standard_start() -> Board {
    fen_to_board("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq -")
}

/// Small board with a closed-loop train on the right side and idle
/// kings in opposite corners. Each player has only the king (plus a
/// rook for castling sanity) — most "picks" will be king shuffles
/// while the train ticks autonomously every full turn.
///
/// Layout (rank, file):
///   rank 7: K . . . . . . k    (white K at (0,7), black k at (7,7))
///   ranks 0-3: empty
///   ranks 4-5, files 4-5: 2×2 Track loop with a Locomotive at (4,4)
///                          facing E and a Carriage at (5,4) chain_idx=1
///
/// The property test then exercises Property 1 (FEN round-trip) +
/// Property 4 (move-changes-state) across the train tick. Without
/// the round-3 through round-6 train work, this random play would
/// surface a bug — corrupted FEN, lost cart, phantom check, etc.
fn train_start() -> Board {
    // 8×8 board. Kings on opposite top-row corners; 2×2 closed loop
    // tiles slightly below. The engine indexes `grid[rank][file]`
    // with `grid[0]` corresponding to the FEN's *first* (topmost) row,
    // so the FEN below places:
    //   grid[0] = K6k        — white K at (file=0, rank=0), black k at (file=7, rank=0)
    //   grid[1..=2] = 8/8    — empty
    //   grid[3] = track row  — loco tile at (4,3) NW, plain at (5,3) NE
    //   grid[4] = track row  — cart at (4,4) SW, plain at (5,4) SE
    //   grid[5..=7] = 8/8/8  — empty
    //
    // Loco at (4,3) facing east with last_dir=S (came from SW). Cart at
    // (4,4) chain_index 1. neighbor_track_dirs forms the 2×2 closed
    // loop; the train circulates NW→NE→SE→SW→NW indefinitely. Kings
    // start far enough away that legal_moves seldom routes them into
    // the loop, but the random walk can find the train so the
    // property test allows a -1 piece-count delta.
    fen_to_board(
        "K6k/8/8/4(T=TRACK,D=E,P=LOCO(ID=1,H=F,L=S))(T=TRACK,D=E)2/4(T=TRACK,D=E,P=CART(ID=1,I=1))(T=TRACK,D=E)2/8/8/8 w - - tr=full p=0",
    )
}

/// Collects every move legal_moves offers for the side to move.
/// Mirrors `Board::status()`'s descent: a Neutral cart carrying a
/// `side_to_move`-colour passenger contributes legal `PieceInCarrier`
/// exit moves and must be included, otherwise the property net is
/// strictly weaker than the engine's `status()` invariant.
fn collect_all_legal(board: &Board) -> Vec<GameMove> {
    use engine::pieces::{Color, Piece};
    let mut out = Vec::new();
    for (coord, piece) in board.all_pieces() {
        let counts = piece.get_color() == board.flags.side_to_move
            || (piece.get_color() == Color::Neutral
                && piece.passengers().is_some_and(|ps| {
                    ps.iter().any(|q| q.get_color() == board.flags.side_to_move)
                }));
        if !counts {
            continue;
        }
        out.extend(board.legal_moves(&coord));
    }
    out
}

/// Tornado lifecycle under random play. A White Stormcaller (`W` at
/// (4,4)) plus a Black rook it can trap; kings apart, not in check.
/// Random `picks` will play `PlaceTornado` (the only way a tornado
/// enters mid-game), then the compulsion/trap/tick/dissipation run
/// through real `make_move` — exactly the dynamic coverage the
/// random-play harness lacked before this pass (Round A only added a
/// *static* FEN-idempotence proptest for `remaining`).
fn tornado_start() -> Board {
    fen_to_board("k5r1/8/8/8/4W3/8/8/7K w - -")
}

/// A token-soup / arbitrary-bytes FEN fuzz strategy biased toward the
/// tornado parse arms (parens, `C=TORNADO:`, colons, digits) plus
/// pure lossy-UTF8 byte noise for breadth.
fn fuzz_fen() -> impl Strategy<Value = String> {
    // DIGITS ARE DELIBERATELY EXCLUDED. `fen_row_to_squares` reads a
    // run of digits as one decimal count and pushes that many empty
    // squares *before* the per-row 255 truncate, so a ~10-char digit
    // run (`"9999999999"`) saturates the u32 count and allocates ~4.29e9
    // squares → OOM/SIGKILL. That is a *pre-existing baseline* memory
    // blowup (Round A finding #2 — NOT tornado; the digit path touches
    // zero tornado code) and is out of plan-13 scope. Re-triggering it
    // here would only OOM-kill the test runner without surfacing
    // anything new, so this fuzz uses a digit-free alphabet to
    // exercise the in-scope parser logic (parens, `C=…`, conditions,
    // piece tokens, `split_once`, the tornado arms) for *panics*
    // without re-finding the documented baseline OOM.
    let toks = prop::sample::select(vec![
        "(", ")", "/", "C=TORNADO:", "C=TORNADO", "C=FROZEN", "C=BRAINROT",
        "P=BUS(P=(K))", "P=W", "P=K", "T=BLOCK", ":", ",", "=", "k", "K",
        "Q", "w", "b", " w - -", "  ", "", "(C=TORNADO:", "TORNADO",
    ]);
    prop_oneof![
        prop::collection::vec(toks, 0..40).prop_map(|v| v.concat()),
        // Arbitrary bytes, digits stripped (see above), lossy-decoded.
        prop::collection::vec(any::<u8>(), 0..220).prop_map(|v| {
            let no_digits: Vec<u8> =
                v.into_iter().filter(|b| !b.is_ascii_digit()).collect();
            String::from_utf8_lossy(&no_digits).into_owned()
        }),
    ]
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

    /// Same four properties, but driving a board with an active train
    /// loop. Each player has only a king on a corner; almost every
    /// step ticks the train. Without the round-3–round-6 train work
    /// this property would surface train-related regressions (lost
    /// carts, phantom check, head-swap, etc.) within a few picks.
    ///
    /// Property 1 (FEN round-trip) is the most informative here —
    /// last_dir, ply_count, train_tick_rate, all the cart payloads
    /// must serialise+parse cleanly after every tick.
    ///
    /// Property 3 (piece-count delta) is relaxed to allow {0, -1}:
    /// a train tick can capture a non-cart piece (a king that walked
    /// into the train's path), making the per-step delta -1 even on
    /// a king-shuffle move. Note king-safety blocks moves *into*
    /// `would_capture_at` next-tick tiles, but a multi-tick approach
    /// can still result in a future train hit — the random play
    /// occasionally finds these scenarios, and we want the FEN
    /// round-trip to still hold.
    #[test]
    fn fen_roundtrip_and_invariants_under_random_play_with_train(
        picks in prop::collection::vec(any::<u32>(), 1..40)
    ) {
        let mut board = train_start();
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

            let mut after = board.clone();
            after.make_move(chosen)
                .expect("legal_moves output must apply via make_move");

            prop_assert_ne!(
                &after, &board_before,
                "make_move on a legal move must not be a no-op"
            );

            // Piece-count delta accounting:
            //   0: ordinary move (king-shuffle, no train capture).
            //  -1: chosen move captures a piece (regular take), OR a
            //      train tick captured a piece that wandered into
            //      its path. Exactly one capture, either source.
            //  +1: passenger-exit via `PieceInCarrier{MoveTo}` —
            //      the passenger becomes a top-level piece while the
            //      cart stays. `all_pieces()` counts only top-level
            //      entries, so the count rises by one.
            //  -2: chosen move captures *and* the subsequent train
            //      tick captures another piece on its new head tile.
            //      Reachable in train fixtures only — `train_start`
            //      has one train so two captures per move is the
            //      worst case; an N-train fixture would need a wider
            //      lower bound.
            let pieces_after = after.all_pieces().len();
            let delta = pieces_after as isize - pieces_before as isize;
            prop_assert!(
                (-2..=1).contains(&delta),
                "piece count delta must be in [-2, 1]; got {delta}"
            );

            // Property 1: FEN round-trip is exact after train tick.
            let fen = board_to_fen(&after);
            let recovered = fen_to_board(&fen);
            prop_assert_eq!(&recovered, &after, "FEN round-trip mismatch after train tick");

            board = after;
        }
        prop_assert!(
            iters_performed > 0,
            "train property test must execute at least one inner iteration"
        );
    }

    /// Audit Round-A/A-PropCov: the random-play generators never seed a
    /// tornado (no Stormcaller in the start positions ⇒ `PlaceTornado`
    /// is never reached), so the FEN round-trip safety net above gave
    /// the `C=TORNADO:<n>` payload ZERO coverage. This property fuzzes
    /// `remaining` across the full `u8` range (incl. the documented
    /// `0`→1 clamp and `255` boundary) and asserts the board is a
    /// fixed point under serialize→parse after the first normalization
    /// round — i.e. the tornado FEN is idempotent (no value parses to
    /// something that re-serializes differently). Mirrors a Frozen
    /// sibling condition to also exercise multi-condition ordering.
    #[test]
    fn tornado_fen_roundtrip_idempotent(remaining in 0u8..=255, with_frozen in any::<bool>()) {
        let mut b0 = fen_to_board("k7/8/8/8/8/8/8/7K w - -");
        let mut sq = Square::new()
            .add_square_condition(SquareCondition::Tornado { remaining });
        if with_frozen {
            sq = sq.add_square_condition(SquareCondition::Frozen);
        }
        b0.grid[4][4] = sq;

        // First round normalizes (e.g. remaining 0 → 1, garbage paths
        // are unreachable here since we construct the value directly).
        let b1 = fen_to_board(&board_to_fen(&b0));
        // Second round must be a no-op: b1 is a fixed point.
        let b2 = fen_to_board(&board_to_fen(&b1));
        prop_assert_eq!(&b1, &b2, "tornado FEN not idempotent for remaining={}", remaining);

        // And the normalized value is the documented clamp: 0→1, else
        // unchanged; the Tornado condition survives round-trip.
        let conds = &b1.grid[4][4].conditions;
        let expected = if remaining == 0 { 1 } else { remaining };
        prop_assert!(
            conds.iter().any(|c| matches!(
                c, SquareCondition::Tornado { remaining: r } if *r == expected
            )),
            "expected Tornado{{remaining:{}}} after round-trip; got {:?}",
            expected, conds
        );
    }

    /// Empirical TOTALITY fuzz (cargo-fuzz/nightly unavailable here, so
    /// proptest-driven, not libFuzzer). Throws adversarial token-soup
    /// AND arbitrary lossy-UTF8 byte strings at `fen_to_board` (then
    /// `board_to_fen`) and asserts only that neither panics — a panic
    /// fails + shrinks the case. This empirically validates the
    /// Round-A-*reasoned*-but-never-run claim that `fen_to_board` is a
    /// total function over any input (no reachable panic/unwrap from
    /// hostile FEN, incl. all the tornado parse arms).
    ///
    /// NOTE on scope: idempotence is deliberately NOT asserted here.
    /// The general parser is intentionally lenient and a degenerate
    /// input (e.g. `""` → a 0×0 board) does NOT round-trip to a fixed
    /// point — a *pre-existing baseline* property, NOT introduced by
    /// plan-13 (the empty/garbage path touches no tornado code), in the
    /// same out-of-scope family as Round A's other baseline FEN-leniency
    /// findings. In-scope idempotence (the tornado `C=TORNADO:n`
    /// subset) is soundly covered by `tornado_fen_roundtrip_idempotent`.
    #[test]
    fn fen_to_board_is_total(s in fuzz_fen()) {
        let b = fen_to_board(&s);
        let f = board_to_fen(&b);
        // Re-parsing the engine's own output must also not panic.
        let _ = fen_to_board(&f);
        // Survival is the assertion — no panic for any fuzz_fen() input.
    }

    /// The four random-play invariants (Property 1–4) driven from a
    /// Stormcaller position so `PlaceTornado` actually fires and the
    /// tornado compulsion/trap/tick/dissipation run through real
    /// `make_move`. Closes the Round-A gap that the random-play
    /// harness never seeded a tornado (it only had a static
    /// FEN-idempotence proptest).
    #[test]
    fn invariants_under_random_play_with_tornado(
        picks in prop::collection::vec(any::<u32>(), 1..40)
    ) {
        let mut board = tornado_start();
        let mut iters_performed = 0usize;
        for &pick in &picks {
            let legal = collect_all_legal(&board);
            if legal.is_empty() {
                break;
            }
            iters_performed += 1;
            let chosen = legal[(pick as usize) % legal.len()].clone();

            let pieces_before = board.all_pieces().len();
            let board_before = board.clone();

            let mut after = board.clone();
            after.make_move(chosen)
                .expect("a legal_moves move must apply via make_move (incl. PlaceTornado / compelled / trapped positions)");

            prop_assert_ne!(
                &after, &board_before,
                "make_move on a legal move must not be a no-op (PlaceTornado flips side + adds the condition)"
            );

            let delta = after.all_pieces().len() as isize - pieces_before as isize;
            prop_assert!(
                delta == 0 || delta == -1,
                "piece-count delta must be 0 or -1 (PlaceTornado/step = 0), got {delta}"
            );

            // Property 1: FEN round-trip exact even with a live
            // `C=TORNADO:n` mid-game.
            let recovered = fen_to_board(&board_to_fen(&after));
            prop_assert_eq!(&recovered, &after, "FEN round-trip mismatch under tornado play");

            board = after;
        }
        prop_assert!(
            iters_performed > 0,
            "tornado random-play test must execute at least one inner iteration"
        );
    }
}
