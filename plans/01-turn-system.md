# Plan 01: Turn system

Everything that has the shape "whose turn is it, and is this move legal for
that player" needs this. Cannot meaningfully implement king-safety,
checkmate, win-by-brainrot, or the goblin-captured-while-kidnapping rule
without it.

## What to change

### `engine/src/board/mod.rs`

Add a `side_to_move: Color` field to `BoardFlags` (or rename `BoardFlags`
to `GameState` if other game-level bookkeeping is going to land here — see
the open question below).

```rust
pub struct BoardFlags {
    pub side_to_move: Color,
    pub white_can_castle_kingside: bool,
    // ... existing fields ...
}
```

Update every `BoardFlags { ... }` literal in the codebase. There are
several in `engine/src/board/tests.rs` (via the `empty_board()` helper
and a few inline constructors) and one in `engine/src/board/fen.rs`'s
`fen_to_board` default.

### `engine/src/board/mod.rs` — `is_valid_move`

Add the turn check. Currently:

```rust
pub fn is_valid_move(&self, game_move: &GameMove) -> bool {
    let possible_moves = self.get_moves(&game_move.from);
    possible_moves.iter().any(|m| m == game_move)
}
```

After the change:

```rust
pub fn is_valid_move(&self, game_move: &GameMove) -> bool {
    let Some(piece) = self.get_square_at(&game_move.from).and_then(|s| s.piece.as_ref()) else {
        return false;
    };
    if piece.get_color() != self.flags.side_to_move {
        return false;
    }
    let possible_moves = self.get_moves(&game_move.from);
    possible_moves.iter().any(|m| m == game_move)
}
```

### `engine/src/board/make_move.rs`

After a successful move, flip `side_to_move`. Place this in
`handle_post_move_effects` after `recalc_brainrot()`, or at the end of
`make_move` itself.

```rust
self.flags.side_to_move = self.flags.side_to_move.opposite();
```

(There's already a `Color::opposite()` in `engine/src/pieces/mod.rs:67`.)

### FEN

Standard FEN encodes the side-to-move as a separate field after the board
(`w` or `b`). The current parser ignores everything after the grid. Two
options:

- **Minimal:** keep the FEN-just-the-grid format and let
  `side_to_move` round-trip via a separate channel. Simpler for now,
  but the API will need a way to send "whose turn" alongside the FEN.
- **Standard:** extend `board_to_fen` and `fen_to_board` to include the
  side-to-move byte. More work, but the FEN strings then capture full
  game state.

Recommended: **standard**. It's a small extension and avoids a parallel
state channel.

## Tests to add

In `engine/src/board/tests.rs`:

- `test_white_cannot_move_on_blacks_turn` — set `side_to_move: Black`,
  try to move a white piece, assert `make_move` returns `Err`.
- `test_make_move_flips_turn` — start White, make a move, assert
  `side_to_move == Black` after.
- `test_fen_roundtrip_with_side_to_move` (if you take the standard
  option) — board with `side_to_move: Black` round-trips correctly.

## Things to be careful about

- The `empty_board()` test helper currently produces a board with no
  side specified. Pick a default (`Color::White` mirrors standard
  chess) and update the helper.
- The `test_skibidi_phase_capped_at_three_without_opponent` test calls
  `make_move(PhaseShift)` directly — if turn checking is added, make
  sure the test's side matches the Skibidi's color, or that test's
  setup needs adjustment.
- The API's `get_new_board_state_handler` will need to receive the
  current side-to-move (either via FEN or as a separate JSON field).
  Out-of-scope here but flag for plan 06.

## Open question

Should `BoardFlags` grow into a `GameState` struct that also tracks
move history, captured pieces, halfmove clock, and full-move number?
The spec doesn't require any of these yet, but plan 02 (king-safety)
and plan 04 (game-over detection) will both want move history eventually.

Recommendation: don't rename or expand the struct now. Add fields as
each plan needs them. Renaming is cheap when there's a real reason.

## Implementation notes (post-landing)

The FEN extension shipped wider than this plan specified. Rather than
adding only the side-to-move byte, `board_to_fen` / `fen_to_board` now
emit and parse all four standard FEN fields after the grid:

```
<grid> <stm> <castling> <ep>
```

- `<stm>` — `w` or `b` (this plan's required scope).
- `<castling>` — `KQkq` subset or `-` (claimed by plan 03; landed here
  because the rest of the FEN extension was already needed and the
  castling field is trivial to add alongside).
- `<ep>` — algebraic coord or `-` (not specified by any plan; folded in
  to keep the FEN single-source-of-truth for board state after
  promotion/castling/EP all landed in plan 03).

A grid-only FEN (`8/8/8/8/8/8/8/8`) still parses with sensible defaults
(`w KQkq -`), preserving compatibility with the frontend's editor.
