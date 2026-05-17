# Plan 05: FEN parser hardening

## Implementation notes (post-landing)

All three steps landed.

- **Doctests** — both `fen.rs` doctests un-`ignore`d with `use` lines;
  the wrong `Some(15)` corrected to `Some(16)`. They now run as
  regression guards.
- **FenError** — `FenError` introduced (the proposed enum).
  `fen_to_board` / `fen_to_square` / `fen_row_to_squares` are now
  fallible. Hard structural errors (empty input, ragged board, unknown
  glyph, unbalanced parens) abort the parse; field-level slips (bad
  `OPEN=`, `tr=`, `C=TORNADO:` suffix, over-capacity Bus) stay
  deliberately lenient (`warn!` + default). The fallible result is
  cascaded through every call site. `api/src/main.rs` returns a
  structured **400** (`FenErrorBody { code, message, fen }`) on
  malformed input.
- **Latent bug** — `symbol_to_piece` had no `M`/`m` arm, so the Monkey
  silently round-tripped to an empty square (the exact silent-garbage
  class this plan targets). Arm added; the fairy perft constants, pinned
  against the monkey-less board, were re-pinned.

The original analysis below is kept for rationale.

---

The FEN parser currently swallows everything. Bad input produces wrong
boards instead of clear errors. Two doctests are `ignore`d because their
assertions are wrong. Several "by coincidence" panic paths exist on
malformed input.

## The two broken doctests

In `engine/src/board/fen.rs`:

- `find_matching_paren` doctest at line ~33. Missing `use` statement.
  One assertion has the wrong expected value (`Some(15)` but the
  function correctly returns `Some(16)` for `"foo(bar(baz),qux)"`'s
  `(` at index 3 — closing `)` is at byte 16, not 15).
- `split_top_level` doctest at line ~260. Missing `use` statement.

### Fix

Either:

1. Remove the ` ```ignore ` and add the missing `use` lines + correct
   the expected value:

   ```rust
   /// ```
   /// use engine::board::fen::find_matching_paren;
   ///
   /// let s = "foo(bar(baz),qux)";
   /// assert_eq!(find_matching_paren(s, 3), Some(16));
   /// ```
   ```

2. Or convert to non-runnable ` ```text ` blocks. Loses test value but
   keeps the rendered example.

Option 1 is the right call — the assertions become real regressions
guards.

## Silent-garbage error paths

### `fen_row_to_squares` accepts ill-formed rows

`engine/src/board/fen.rs` around line 78 onward parses a row character
by character. There's no validation that the resulting row is 8 squares
wide. `"9"` produces 9 empty squares; `"PPPPPPPPPP"` produces 10
squares.

**Recommended fix**: at end of `fen_row_to_squares`, assert the
expected width (default 8) or return `Result<Vec<Square>, FenError>`.

But: the function is currently infallible (`fn fen_row_to_squares(row:
&str) -> Vec<Square>`). Making it return `Result` would cascade through
`fen_to_board` (currently also infallible). That's the right
restructure but it's a bigger change.

Minimum fix for now: accept ragged rows but log a warning.

### `fen_to_square` ignores unknown piece symbols silently

`PieceType::symbol_to_piece(value)` returns `None` for unknown symbols,
and `fen_to_square` then constructs a `Square { piece: None }`. Result:
`"Z"` in the FEN becomes an empty square. No signal that the input was
bogus.

**Recommended fix**: same as above — change `fen_to_square` to return
`Result<Square, FenError>` and propagate. Or at minimum, `warn!` when
a piece symbol fails to parse.

### `fen_row_to_squares` on a stray `)` underflows depth

The inner `while let Some(c) = chars.next()` matches `(` (depth++) and
`)` (depth--, with no check that depth > 0). A stray `)` outside any
`(` triggers `usize` underflow → debug-mode panic.

**Fix**: clamp to 0 or short-circuit:

```rust
')' => {
    if depth == 0 {
        warn!("stray ')' in FEN extended block");
        break;
    }
    depth -= 1;
    if depth == 0 { break; }
}
```

### `fen_to_board` always sets castle rights to `true`

Currently:

```rust
let flags = BoardFlags {
    white_can_castle_kingside: true,
    white_can_castle_queenside: true,
    black_can_castle_kingside: true,
    black_can_castle_queenside: true,
    en_passant_target: None,
};
```

This is fine for a freshly-set-up board but lossy on any board that's
already mid-game. Plan 01 will likely extend `fen_to_board` to parse a
side-to-move byte; this is the place to also parse the castle-rights
field and en-passant target.

## Proposed `FenError` type

```rust
#[derive(Debug, PartialEq)]
pub enum FenError {
    EmptyInput,
    BadRowCount { expected: usize, found: usize },
    BadRowWidth { row: usize, expected: usize, found: usize },
    UnknownPieceSymbol(String),
    UnbalancedParen { in_row: usize },
    BadExtendedSquare { content: String, reason: &'static str },
    BadFlagsField(String),
}
```

Then `pub fn fen_to_board(fen: &str) -> Result<Board, FenError>`. All
the API handlers in `api/src/main.rs` would then propagate a 400 with
the error message instead of silently producing garbage.

## Sequencing

1. Fix the two doctests (small, low-risk).
2. Add `warn!`s on the silent-garbage paths. Keep the infallible
   signatures. Low-risk transitional state — still produces wrong
   boards on bad input but at least logs them.
3. Introduce `FenError` and convert `fen_to_board` /
   `fen_to_square` / `fen_row_to_squares` to return `Result`. Propagate
   through `api/src/main.rs`. Bigger change.

Steps 1-2 can happen any time. Step 3 should land before plan 06
(API evolution) so the API can use real error responses.

## Tests to add

- `test_fen_unknown_symbol_returns_err` — `"Z7/8/8/8/8/8/8/8"` returns
  `FenError::UnknownPieceSymbol("Z")`.
- `test_fen_too_many_in_row_returns_err` — `"PPPPPPPPP/..."` returns
  `BadRowWidth { found: 9, .. }`.
- `test_fen_stray_close_paren_does_not_panic` —
  `"(P=R))7/8/8/8/8/8/8/8 w - -"` returns `UnbalancedParen` rather than
  panicking.
- `test_fen_roundtrip_preserves_castle_rights` (after plan 01 / step 3) —
  board with non-default castle rights round-trips correctly.
