//! Perft (performance test) suite — counts the leaf positions reachable
//! in a given depth of legal play from a starting board. Compared
//! against canonical reference numbers, perft is the strongest
//! single-test correctness signal a move-generator can have: a mismatch
//! at depth N pinpoints a wrong move (or set of moves) somewhere in
//! that tree.
//!
//! We run the cheap depths in the default test pass and gate the
//! deeper / known-slow ones behind `#[ignore]`. Invoke them with:
//!
//! ```text
//! cargo test --test perft -- --ignored
//! ```

use engine::board::{Board, fen::fen_to_board};

/// Standard perft: recursively counts leaf positions reachable in
/// `depth` plies of legal play from `board`. Returns 1 at depth 0 so
/// it functions as a counting unit at the leaves.
fn perft(board: &Board, depth: u32) -> u64 {
    if depth == 0 {
        return 1;
    }
    let mut count = 0u64;
    for (coord, piece) in board.all_pieces() {
        if piece.get_color() != board.flags.side_to_move {
            continue;
        }
        for m in board.legal_moves(&coord) {
            let mut next = board.clone();
            next.make_move(m)
                .expect("legal_moves output must apply cleanly via make_move");
            count += perft(&next, depth - 1);
        }
    }
    count
}

/// Standard chess starting position. The classic perft anchor —
/// reference counts are tabulated to ridiculous depths and any small
/// move-gen bug eventually breaks them.
fn standard_start() -> Board {
    fen_to_board("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq -")
}

#[test]
fn perft_start_depth_1() {
    assert_eq!(perft(&standard_start(), 1), 20);
}

#[test]
fn perft_start_depth_2() {
    assert_eq!(perft(&standard_start(), 2), 400);
}

#[test]
fn perft_start_depth_3() {
    assert_eq!(perft(&standard_start(), 3), 8902);
}

/// Depth 4 = 197,281. Slow under the cloning legal_moves
/// implementation; runs under `--ignored`.
#[test]
#[ignore]
fn perft_start_depth_4() {
    assert_eq!(perft(&standard_start(), 4), 197_281);
}

/// Kiwipete — the classic perft test position, packed with castling,
/// en passant, promotion candidates, and check-blocking captures all
/// reachable within a few plies. A clean Kiwipete number is a very
/// strong signal that plans 01–03 are wired correctly.
fn kiwipete() -> Board {
    fen_to_board(
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq -",
    )
}

#[test]
fn perft_kiwipete_depth_1() {
    assert_eq!(perft(&kiwipete(), 1), 48);
}

#[test]
#[ignore]
fn perft_kiwipete_depth_2() {
    assert_eq!(perft(&kiwipete(), 2), 2039);
}

/// "Position 3" — endgame-y perft anchor designed to exercise pawn
/// promotion and en passant heavily.
fn position_three() -> Board {
    fen_to_board("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - -")
}

#[test]
fn perft_position_three_depth_1() {
    assert_eq!(perft(&position_three(), 1), 14);
}

#[test]
fn perft_position_three_depth_2() {
    assert_eq!(perft(&position_three(), 2), 191);
}

#[test]
#[ignore]
fn perft_position_three_depth_3() {
    assert_eq!(perft(&position_three(), 3), 2812);
}

/// Fairy-piece smoke perft. No canonical reference number exists for
/// custom-piece positions; instead this records the current legal-move
/// tree size at depth 2 as a regression target. Any change to fairy
/// move-gen (Goblin / Skibidi / Bus / Monkey) that alters the count
/// will trip this test, forcing a deliberate update.
///
/// The position sits one of each fairy piece on the board alongside a
/// minimal standard-piece scaffold (kings so check-detection has
/// something to chew on, plus a couple of pawns for the Goblin to
/// kidnap and the Monkey to jump).
fn fairy_setup() -> Board {
    // Layout (file/rank, rank 0 at top):
    //   . . . . k . . .      ← black king on e8 = (4,0)
    //   . . . . . . . .
    //   . . . s . . . .      ← black Skibidi on d6 = (3,2)
    //   . . p . . . . .      ← black pawn on c5 = (2,3)
    //   . . . . . P . .      ← white pawn on f4 = (5,4)
    //   . . . . S . . .      ← white Skibidi on e3 = (4,5)
    //   . M . . . . . .      ← white Monkey on b2 = (1,6)
    //   B . . . K . . G      ← white Bus a1 = (0,7), king e1 = (4,7),
    //                          Goblin h1 = (7,7)
    //
    // Bus and Goblin are multi-character / stateful so they use the
    // extended `(P=...)` square syntax. Goblin requires a `H=file-rank`
    // home-square clause.
    fen_to_board(
        "4k3/8/3s4/2p5/5P2/4S3/1M6/(P=BUS)3K2(P=G(H=7-7)) w - -",
    )
}

#[test]
fn perft_fairy_setup_depth_1_smoke() {
    // Locks in the current depth-1 count for the fairy setup. A bare
    // smoke test — if move-gen for any piece changes, this trips.
    // Depth 1 captures move-gen breadth; depth 2 captures composition
    // (see the `#[ignore]`d depth-2 variant below).
    let board = fairy_setup();
    let count = perft(&board, 1);
    assert_eq!(
        count, FAIRY_PERFT_DEPTH_1,
        "fairy-setup depth-1 perft drifted — investigate which piece changed"
    );
}

#[test]
#[ignore]
fn perft_fairy_setup_depth_2_smoke() {
    // Depth 2 — the recommended fairy-piece regression target. Slow
    // enough to gate behind `--ignored`. Locks in compositional
    // behaviour (move-gen → make_move → next-side move-gen).
    let board = fairy_setup();
    let count = perft(&board, 2);
    assert_eq!(
        count, FAIRY_PERFT_DEPTH_2,
        "fairy-setup depth-2 perft drifted — investigate which piece changed"
    );
}

// Constants pinned by the first run of these tests; future drift trips
// the regression. Treat changes here as load-bearing: each requires
// understanding which piece's move-gen changed and why.
const FAIRY_PERFT_DEPTH_1: u64 = 40;
const FAIRY_PERFT_DEPTH_2: u64 = 501;
