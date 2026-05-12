# Plan 03: Standard chess completeness

The three standard rules that aren't implemented despite the FEN-level
plumbing being half-there: **promotion**, **castling**, **en passant**.

Doable independently of each other. Castling needs plan 02 (king-safety)
to check "is any square on the king's path attacked." Promotion and en
passant don't strictly need king-safety, but probably want plan 01
(turns) at minimum.

## Promotion

### What's missing

A white pawn reaching rank 0 (or black at rank 7) just emits a normal
`MoveType::MoveTo` and stays a pawn after make_move. There's no
`MoveType::Promotion` variant.

The `movement/mod.rs` file had a `MoveKind::Promotion(PieceType)` enum
variant — that file's been deleted as dead code. It needs to come back,
on the live `MoveType` enum in `board/mod.rs`.

### Changes

1. Add to `MoveType` in `engine/src/board/mod.rs`:

   ```rust
   pub enum MoveType {
       MoveTo(Coord),
       MoveIntoCarrier(Coord),
       PieceInCarrier { piece_index: u8, move_type: Arc<MoveType> },
       PhaseShift,
       Promotion { target: Coord, into: PromotionTarget },
   }

   #[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
   pub enum PromotionTarget {
       Queen, Rook, Bishop, Knight,
       // Add fairy promotions if the spec wants them
   }
   ```

   Restricting `into` to a small enum (rather than `PieceType` directly)
   avoids letting the API promote into a King.

2. `Pawn::initial_moves` (engine/src/pieces/standard/pawn.rs): when the
   destination rank would be 0 (white) or 7 (black), replace the single
   `MoveType::MoveTo(coord)` with four `Promotion` moves, one per
   `PromotionTarget`. Same for diagonal capture targets.

3. `Board::make_move` (engine/src/board/make_move.rs): handle the new
   variant. Clear the source square; place a new piece of the chosen
   type and the pawn's color at the target square.

4. `get_moves` filter in `piecetype.rs`: route `Promotion { target, .. }`
   through the same target-square checks as `MoveTo`. The promotion
   choice is irrelevant to filtering — the move is legal iff the
   underlying advance/capture is legal.

5. FEN: promotion doesn't need a FEN field (the resulting piece is
   already encoded). But the API request format for "play this move"
   needs to express the promotion choice.

### Tests

- White pawn at rank 1 → moves to rank 0 generate four `Promotion`
  moves, not one `MoveTo`.
- `make_move` with `Promotion { into: Queen }` replaces the pawn with a
  Queen of the same color.
- Capture-promotion: pawn at rank 1 with diagonal enemy at rank 0,
  generates promotion-by-capture moves.

## Castling

### What's missing

`BoardFlags.white_can_castle_kingside`, `..._queenside`, and the black
counterparts exist but are written-only — they're not read in the
move-generation path. King doesn't emit castle moves.

### Changes

1. Either reuse `MoveType::MoveTo` (king moves two squares; engine
   detects "this is a castle" from the distance) or add a dedicated
   `MoveType::Castle { side: CastleSide }` variant. The dedicated
   variant is clearer; do that.

2. `King::initial_moves`: emit `Castle` candidates when the
   corresponding flag is set, the king and rook haven't moved, the
   path is empty, and the king's path squares aren't attacked.

   The "haven't moved" check is currently encoded only in the flags —
   any king or rook move should clear the relevant flag. Implement
   that in `King::post_move_effects` and `Rook::post_move_effects`.

3. `make_move`: handle `Castle` — move king two squares toward the
   rook, rook to the king's old neighbor. Clear both castle flags
   for that color.

4. FEN: standard FEN encodes castle rights in a separate field
   (`KQkq` after the side-to-move byte). Either extend the FEN parser
   to read this (plan 01 already mentions extending FEN), or treat the
   FEN-flag default-true as a known limitation.

### Tests

- King in starting position with rook and clear path generates a
  castle move.
- King in check cannot castle.
- King moving through an attacked square cannot castle.
- King or rook that has moved cannot castle (flag cleared).
- After castling, both pieces are in correct positions.

## En passant

### What's missing

`BoardFlags.en_passant_target` is `Option<Coord>` but never read. Pawn
double-push doesn't set it; subsequent pawn captures don't check it.

### Changes

1. `Pawn::post_move_effects`: if this was a double push, set
   `board_after.flags.en_passant_target = Some(square_passed)`. Clear
   it otherwise.

2. `Pawn::initial_moves`: if `board.flags.en_passant_target` is `Some(c)`
   and `c` is one of the pawn's diagonal capture squares, emit a
   `MoveType::EnPassant { target: c, captured: behind_target }` move
   (or just `MoveTo` if you'd rather not add a variant — see "design
   choice" below).

3. `make_move`: handle the en-passant capture — clear the captured
   pawn's square (not the target square, which is empty).

4. Make sure every other piece's move generation does *not* clear
   `en_passant_target` — only `make_move`'s post-effect for a pawn
   double-push should set it, and the next move should clear it back
   to `None`.

### Design choice

En passant is *almost* a regular `MoveTo` — the difference is that the
captured piece isn't at the destination. Two encodings:

- **Encode as `MoveTo(target)` and detect en passant in `make_move`** by
  checking "is target == flags.en_passant_target && piece is a pawn".
  Simpler API, but `make_move` has to do the cleanup of the captured
  pawn from a *different* square. Adds special-case logic.
- **Dedicated `MoveType::EnPassant { target, captured }`** variant. More
  variants but cleaner per-variant semantics.

Recommendation: dedicated variant. It's already the pattern (MoveTo,
MoveIntoCarrier, PhaseShift, PieceInCarrier) and the filter logic is
identical to MoveTo.

### Tests

- Pawn double-push sets `en_passant_target` on the board after.
- Adjacent enemy pawn can en-passant-capture; the captured pawn is
  removed from its square.
- After any non-double-push move by either color,
  `en_passant_target` reverts to `None`.

## Sequencing

Cheapest order: **promotion → en passant → castling**. Castling depends on
plan 02 (attack detection) for the "no path attacked" rule. Promotion and
en passant don't, so they can land first.
