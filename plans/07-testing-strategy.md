# Plan 07: Testing strategy

37 tests is a good start. This plan covers the test categories that
aren't yet present and would catch real regressions as the engine
grows.

## Perft

The chess world's standard correctness test. Count the number of legal
move sequences at depth N from a starting position. Compare against
known-good numbers.

Standard chess from the starting position:

| Depth | Nodes      |
|-------|------------|
| 1     | 20         |
| 2     | 400        |
| 3     | 8902       |
| 4     | 197281     |
| 5     | 4865609    |

Implementation:

```rust
fn perft(board: &Board, depth: u32) -> u64 {
    if depth == 0 { return 1; }
    let mut count = 0;
    for (coord, piece) in board.all_pieces() {
        if piece.get_color() != board.flags.side_to_move { continue; }
        for m in board.legal_moves(&coord) {
            let mut next = board.clone();
            next.make_move(m).unwrap();
            count += perft(&next, depth - 1);
        }
    }
    count
}
```

Needs plan 01 (side_to_move) and plan 02 (legal_moves) before it can
work. Once those are in:

- `test_perft_starting_position_depth_3` — asserts 8902.
- `test_perft_kiwipete_depth_2` — Kiwipete is a classic perft position
  with castling, promotion, en passant. Catches a *lot* of bugs at once.
- `test_perft_custom_piece_smoke` — set up a board with each fairy piece
  and run perft to depth 2. No reference number, but locks in current
  behavior as a regression target.

Run perft tests with `#[ignore]` by default at higher depths so they
don't slow down `cargo test` — invoke with `cargo test -- --ignored`.

## Property-based tests

Use `proptest` or `quickcheck`. Useful properties:

- **FEN round-trip**: for any board produced by a sequence of `make_move`
  calls from the starting position, `fen_to_board(board_to_fen(b)) == b`.
- **Move-then-undo equivalence**: well, no undo yet, but cloning before
  a move and comparing the original to a "we never moved" version works.
- **`make_move` never panics on `legal_moves` output**: for every move
  in `legal_moves(from)`, `make_move(m)` returns `Ok`.
- **Color invariant**: after `make_move`, exactly one piece changed
  color/position; `all_pieces()` size delta is in {-1, 0} (depending
  on capture).

Property tests catch the corner cases unit tests miss. Spend
proportional effort — one well-crafted property test can replace ten
unit tests.

## Integration tests via `engine/tests/`

Rust convention: `engine/tests/*.rs` files compile as separate crates
that only see the public API. Useful for full-game scenario tests:

- `engine/tests/standard_game.rs` — play the fool's mate, scholar's
  mate, etc., move by move; assert each move succeeds and final
  status is `Checkmate`.
- `engine/tests/fairy_scenarios.rs` — set up a Skibidi 4-phase
  brainrot wall; have the opponent run out of legal moves; assert
  `BrainrotWin`.

Integration tests exercise the actual public API the API layer uses,
catching regressions that unit tests internal to modules miss.

## What to skip / deprioritize

- **Per-piece "all N moves from center"** unit tests — perft subsumes
  these. Keep edge cases (corner, blocked) as unit tests; skip the
  center-of-empty-board counts.
- **`Piece::name()` returns the literal string** — implementation
  trivia. Catches no bugs.
- **Constructor-equality tests** — `derive(PartialEq)` is not
  worth testing.
- **Trivially-passing scenarios** — "new Skibidi has phase=1" is
  testing the constructor literal, not engine behavior.

## Existing test debt

From the round-2 audit, these existing tests should be strengthened:

- `test_pawn_diagonal_capture_only_when_enemy_present` — add a
  friendly-piece-on-diagonal case (must not capture).
- `test_rook_captures_enemy_blocker` — assert the intermediate
  squares `(4,3)`, `(5,3)` are present, not just the capture target.
- `test_make_move_rejects_illegal` — match the exact piece variant
  (`Rook`) at the source, not just `piece.is_some()`.
- `test_skibidi_neutralization` — also assert the non-radiating
  Skibidi's phase is unchanged.
- `test_bus_at_capacity_blocks_entry` — assert the knight retains
  its other normal L-moves.

These are 5-minute touch-ups; bundle into a single tightening commit.

## Test layout going forward

```
engine/src/board/tests.rs        # inline unit tests (current 37)
engine/tests/standard_game.rs    # integration scenarios
engine/tests/fairy_scenarios.rs  # custom-piece scenarios
engine/tests/perft.rs            # perft suite (some #[ignore])
```

`engine/src/board/tests.rs` is already getting long. Consider
splitting it by topic if it crosses ~1000 lines. Don't preemptively.

## Sequencing

1. **Tightening pass** — fix the weak assertions noted above. Free.
2. **Perft** — blocks on plans 01 + 02. Most valuable single test.
3. **Property tests** — proportional effort, high signal. Can land
   any time but most valuable after plans 02 + 03 expand the legal-
   move logic.
4. **Integration tests** — fold in as new features land; each plan
   above should add at least one integration scenario test.

## CI thought

There's no CI configured. As tests grow, lock the build with a GitHub
Actions workflow:

```yaml
- run: cargo test --workspace
- run: cargo clippy --workspace -- -D warnings
- run: cargo fmt --check
```

Out of scope for the engine plans but worth flagging — without CI,
the test suite drifts silently.
