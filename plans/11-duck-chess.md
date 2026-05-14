# Plan 11: Duck Chess + variant infrastructure

Add the engine's first true rule-variant — Duck Chess — and the
per-position scaffolding (`variants: Vec<VariantId>`) that future
variants (Atomic, Antichess, King-of-the-Hill, …) will hook into.

Depends on plans 01 (turn system) and 02 (king safety) — both landed.
Plan 10 (movement stack) is *not* a prerequisite, but the two plans
share the long-term home for variant logic — see [§ relationship to
plan 10](#relationship-to-plan-10).

## Why now

Two motivations:

1. Duck Chess is a self-contained, well-known variant that exercises
   every part of the variant surface area: a new piece-like entity, a
   per-turn sub-phase, suppression of king-safety, a different win
   condition, and FEN-state additions. Whatever pattern lands here
   will be the template for every variant that follows.
2. The user has indicated more variants are coming, and that multiple
   variants may compose on a single board (Duck + Atomic, Duck +
   custom-dimensions, …). Designing for one variant in isolation
   builds the wrong abstraction; designing for *one composable variant*
   gets the right shape on the first try.

## Concept

Two tiers, intentionally separate:

- **Per-position activation** — a `Vec<VariantId>` on `BoardFlags`
  declaring which variants are active for this game. Serialized in
  FEN. Empty list = standard chess.
- **Global rule code** — variant-specific behaviour lives in the
  engine binary, gated by reading the active-variants list. After
  plan 10 lands these gates become modifier predicates; before plan
  10 they're conditionals at the existing chokepoints
  (`legal_moves`, `is_attacked_by`, `make_move`, …).

This split is the key design call. Modifier code is global and
versioned with the engine; *which* code fires is per-position state
that round-trips through FEN. Plan 10's "modifiers as code, not
position-state" property is preserved.

## Types

### `engine/src/board/mod.rs`

```rust
#[derive(PartialEq, Eq, Hash, Debug, Clone, Copy, Serialize, Deserialize)]
pub enum VariantId {
    DuckChess,
    // future: Atomic, Antichess, KingOfTheHill, ThreeCheck, …
}

#[derive(PartialEq, Debug, Clone)]
pub struct BoardFlags {
    // ... existing fields ...
    pub variants: Vec<VariantId>,            // empty = standard
    pub duck_phase: DuckPhase,               // ignored unless DuckChess is in `variants`
}

#[derive(PartialEq, Debug, Clone, Copy, Default)]
pub enum DuckPhase {
    #[default]
    PieceMove,        // the mover hasn't moved their piece yet this turn
    DuckPlacement,    // the mover has moved their piece; now must place/move the duck
}
```

`DuckPhase` lives on `BoardFlags`, not inside `VariantId`, because
phase state is per-position and changes every half-turn. The
`VariantId` enum stays pure-discriminator.

Helper:

```rust
impl BoardFlags {
    pub fn has_variant(&self, v: VariantId) -> bool { self.variants.contains(&v) }
}
```

### `engine/src/board/square.rs`

Add a fourth field to `Square`:

```rust
pub struct Square {
    pub piece: Option<PieceType>,
    pub square_type: SquareType,
    pub conditions: Vec<SquareCondition>,
    pub duck: bool,
}
```

Not a new `SquareCondition`: conditions are persistent terrain
effects (Frozen, Brainrot) that don't move. The duck is mobile and
mutually exclusive with `piece.is_some()`. A dedicated bit is honest
about that and reads correctly in pattern matches.

`Square::is_empty` (the existing predicate used by move-gen and duck
placement) now requires `piece.is_none() && !duck`. Both gliders and
the duck-placement modifier consume the same predicate, so this single
change propagates to every "can the duck/piece land here?" call.

### `engine/src/board/mod.rs` — `MoveType`

Add the half-turn variants:

```rust
pub enum MoveType {
    // ... existing variants ...
    /// Place the duck on the first move of a Duck Chess game.
    /// `from` on the wrapping `GameMove` is ignored.
    PlaceDuck { to: Coord },
    /// Move the duck. `from` on the wrapping `GameMove` is the duck's
    /// current square; `to` is the destination.
    MoveDuck { to: Coord },
}
```

Two distinct variants (rather than one `DuckMove { to }`) so first-
placement and relocation are disambiguated in move history without
having to consult prior state. The dispatch in `make_move` is symmetric
either way; the cost is one extra match arm.

## Duck Chess rules

The variant the engine has to enforce:

1. A single colourless "duck" exists on the board. It blocks all piece
   movement: no piece may move *to* a square containing the duck, and
   no glider may move *through* it.
2. The duck does not capture and cannot be captured.
3. Each player's turn is two half-moves: first a normal piece move,
   then placing (first turn of the game) or moving the duck to any
   empty square.
4. Side-to-move flips after the duck half-move, not after the piece
   half-move.
5. `ply_count` increments by 1 per *turn* — bump on the duck half-move,
   not the piece half-move. This keeps the ply counter at parity with
   standard chess (1 ply = 1 player's turn) and preserves train
   tick semantics under `tr=ply`.
6. **No check, no checkmate, no stalemate.** Kings can move into
   "check"; castling through "check" is legal; pinned pieces can move
   freely. Win condition is **king capture** — when a piece moves to a
   square holding the enemy king, the game ends immediately with the
   mover as winner.
7. On the very first move of the game (white's piece half-move +
   duck placement), the duck starts off-board. The first
   `PlaceDuck` is the placement; all subsequent duck half-moves are
   `MoveDuck`.

## Chokepoints to gate

Pre-plan-10, the variant gate lives as conditionals at these existing
sites. Post-plan-10, each conditional becomes a modifier predicate
keyed on `VariantId::DuckChess`.

### `Board::legal_moves` ([engine/src/board/mod.rs](engine/src/board/mod.rs))

Three changes:

- If the variants list includes `DuckChess` and `duck_phase ==
  DuckPlacement`, the only legal moves are `PlaceDuck`/`MoveDuck` to
  every empty square. Piece moves are not legal during the duck phase.
- If the variants list includes `DuckChess` and `duck_phase ==
  PieceMove`, the king-safety filter (clone + `is_in_check` check) is
  skipped. Piece moves are legal purely on geometric reachability +
  duck blocking.
- The duck's square is a blocker for glider path-finding (rook,
  bishop, queen, monkey, etc.) and an illegal destination for all
  piece move-types. Both fall out of `Square::is_empty` being threaded
  through the existing move generators, plus a new "destination is not
  the duck's square" check in the same filter that drops same-color
  captures.

### `Board::is_attacked_by` ([engine/src/board/mod.rs](engine/src/board/mod.rs))

Unchanged in semantics, but callers in Duck Chess never invoke it for
legality. The function stays useful for editor / debug / future
variants. Don't remove it.

### `Board::make_move` / `apply_environment_reactions` ([engine/src/board/make_move.rs](engine/src/board/make_move.rs))

Three changes, all gated on `has_variant(DuckChess)`:

1. **Side-to-move flip** — currently at the tail of
   `apply_environment_reactions`. Gate the flip on
   `duck_phase == DuckPlacement` (i.e. flip only on the duck
   half-move).
2. **Ply increment** — same gate; bump `ply_count` only on the duck
   half-move.
3. **Duck-phase advance** — after a piece move, set
   `duck_phase = DuckPlacement`. After a duck move, set it back to
   `PieceMove`. The piece half-move and duck half-move are submitted as
   *separate* `make_move` calls; the engine doesn't bundle them.

### Win condition

King capture is detected in `make_move` immediately after the move
applies — if the captured piece is the opponent's king, return a
`GameStatus::Win { winner }` (new variant) and short-circuit any
remaining post-effects. This intentionally bypasses
`GameStatus::Checkmate` / `Stalemate`; Duck Chess never returns those.

`Board::status()` ([engine/src/board/mod.rs](engine/src/board/mod.rs))
also branches: when `DuckChess` is active, never return
`Check`/`Checkmate`/`Stalemate`. Return `Ongoing` or `Win`.

### Castling

Gate `is_in_check` and the path-attacked check inside
`Board::castle_moves` ([engine/src/pieces/standard/king.rs](engine/src/pieces/standard/king.rs))
on `!has_variant(DuckChess)`. In Duck Chess the king may castle through
"check" because check doesn't exist. The duck still blocks: the
king's and rook's destinations and traversal squares must all be empty
of pieces *and* the duck.

## FEN encoding

Two new pieces of state to serialize, each going where it fits the
existing convention (see [plans/README.md](plans/README.md) FEN
extensions reference):

### Per-square: `(DUCK)`

The duck's location goes in the grid using the existing extended-
square syntax. New per-square key, no value needed:

```
8/8/8/4(DUCK)3/8/8/8/8 ...
```

Convention: per-square keys are uppercase; `DUCK` joins `P`, `T`, `C`,
`ID`, `STATE`, etc. A square with only a duck has no other keys; a
square with a piece *and* a duck is an invariant violation and parses
as an error.

Update the FEN reference in [plans/README.md](plans/README.md) — add
`DUCK` to the per-square keys table.

### Board-level: `variants=…` and `duck_phase=…`

Both are lowercase, trailing fields, matching the `tr=` / `p=`
pattern at [engine/src/board/fen.rs](engine/src/board/fen.rs):

```
<grid> w KQkq - 0 1 tr=full p=42 variants=duck_chess duck_phase=placing
```

- `variants=<id1>,<id2>,…` — comma-separated, no spaces.
- `duck_phase=piece` | `duck_phase=placing` — only meaningful when
  `duck_chess` is in `variants`; missing field defaults to `piece`
  (i.e. start-of-turn).

The lenient parser (warns on unknown fields, never rejects) keeps
backward compatibility: old FENs without these fields produce
`variants=[]`, which is standard chess.

**Note:** the existing per-square `D=` key is taken (Track exit
direction at [plans/README.md:118](plans/README.md)). Hence the
multi-letter `DUCK` for the duck flag. Don't confuse them.

## Sequencing

Seven commits, each fully working:

1. **`VariantId` enum + `variants` field on `BoardFlags`** — empty
   default; existing tests untouched. `BoardFlags { ... }` literals
   updated wherever they're constructed (`engine/src/board/tests.rs`,
   `engine/src/board/fen.rs`, …; compiler will flag the rest).
2. **`Square.duck` field + `Square::is_empty` update + FEN
   round-trip for `(DUCK)`** — no behaviour gated on it yet; just
   parses and serializes. Test: `(DUCK)` on a square, round-trip
   asserts equality.
3. **`MoveType::PlaceDuck` and `MoveType::MoveDuck` + match
   exhaustiveness in `make_move`'s dispatch** — they apply the duck
   to the grid and clear the prior duck square. No legality gate yet;
   reachable only from tests.
4. **`DuckPhase` field + FEN round-trip + side-flip / ply gate** —
   side-to-move and `ply_count` advance only on duck half-moves when
   `DuckChess` is active.
5. **Duck as blocker + `legal_moves` gate** — duck blocks glider
   paths; duck phase emits only duck-move candidates; king-safety
   filter skipped in Duck Chess.
6. **King-capture win condition + `GameStatus::Win`** — short-circuit
   in `make_move`. Update `Board::status` to branch on `DuckChess`.
7. **Castling-through-check allowance** — gate the in-check + path-
   attacked checks in `Board::castle_moves`.

Each commit ships passing tests + a new Duck-Chess scenario test
exercising the new behaviour.

## Tests to add

In `engine/tests/`:

- `test_duck_chess_fen_roundtrip` — full FEN with `(DUCK)` and
  `variants=duck_chess duck_phase=placing` round-trips.
- `test_duck_blocks_glider` — rook on a1, duck on a3, black piece on
  a5: rook cannot move to a5 or a4. With duck removed, a5 capture and
  a4 move are legal.
- `test_duck_phase_alternation` — make a piece move, assert
  `duck_phase == DuckPlacement`. Make a duck move, assert
  `duck_phase == PieceMove` and side flipped.
- `test_piece_move_does_not_flip_side` — same as above, but assert
  side did *not* flip after the piece move.
- `test_ply_increments_only_on_duck_move` — `ply_count` before / after
  each half-move.
- `test_king_walks_into_attack_in_duck_chess` — white king moves to a
  square attacked by a black rook; legal in Duck Chess.
- `test_castle_through_check_in_duck_chess` — white castles kingside
  with black bishop attacking f1; legal in Duck Chess.
- `test_king_capture_wins` — white queen captures black king; result
  is `GameStatus::Win { winner: White }`. No checkmate.
- `test_first_move_places_duck` — fresh Duck Chess board: white's
  first piece move, then `PlaceDuck { to: e3 }`. Subsequent duck
  half-moves are `MoveDuck`.
- `test_duck_cannot_share_square_with_piece` — `PlaceDuck` /
  `MoveDuck` to an occupied square is rejected.
- `test_duck_does_not_capture_or_die` — pawn capture geometry pointed
  at the duck is illegal; the duck's square never appears in any
  piece's threat set.

Perft is *very* different under Duck Chess (move count multiplies by
≈ empty-square count per turn). Don't reuse standard-chess perft
counts — generate new ones from a reference implementation or document
the first few plies by hand and freeze them.

## Things to be careful about

- **First-turn off-board duck.** On a fresh Duck Chess board, the
  duck's coordinate doesn't exist yet. `Square.duck` is false on every
  square. The first `PlaceDuck` is legal; subsequent moves must be
  `MoveDuck`. Distinguish by scanning for a square with `duck = true`:
  if none, this is the first turn and `PlaceDuck` is expected.
- **En-passant target lifetime.** Standard logic clears `en_passant_
  target` at the start of the opponent's turn. In Duck Chess that's
  after the duck half-move, not the piece half-move. Make sure the
  clear is gated on the side-to-move flip, not on every `make_move`.
- **Castling rights.** A piece move that moves a rook/king still
  updates castle rights even though no ply elapses. Gate the *clear*
  on the move semantics (piece moved), not on the ply increment.
- **Train interaction.** Trains tick by `train_tick_rate`. With
  `tr=ply`, trains tick on duck half-moves only (the half-move that
  increments `ply_count`). With `tr=full` (full turn), trains tick
  once per Duck Chess turn — same as standard chess semantics. Both
  behaviours fall out of the existing gate if the ply-increment hook
  is the only thing that advances trains; no extra work needed.
- **Phantom king-capture during validation.** `legal_moves` clones
  the board and applies the move under
  `apply_move_for_validation` to check king safety. In Duck Chess
  the filter is *off*, so this clone-and-apply happens only for the
  win-condition check. Make sure `apply_move_for_validation` doesn't
  itself terminate the game on a king-capture during validation — only
  the real `make_move` should produce `GameStatus::Win`.
- **API error surface.** The API ([api/src/main.rs](api/src/main.rs))
  currently returns 400 on illegal moves. Add a "must place duck"
  error variant so the frontend can render "you have to move the
  duck now" rather than a generic "illegal move."
- **Frontend duck sprite.** Out of scope here, but the editor /
  rendering surface at [frontend/vite-dev/src/main.ts](frontend/vite-dev/src/main.ts)
  needs a duck glyph. Re-use the per-square FEN parser at
  [frontend/vite-dev/src/fen.ts](frontend/vite-dev/src/fen.ts) — same
  `(DUCK)` syntax; render as an overlay on the square.
- **Multi-variant composition.** `variants: Vec<VariantId>` allows
  two variants to be active at once (e.g. Duck + future Atomic). v1
  validates the list at FEN-parse time and rejects known-conflicting
  combinations (none yet — the list is `[DuckChess]`). Each new
  variant adds itself to the conflict matrix as it lands.
- **Empty-square count blows up move-gen.** During the duck half-
  move, the legal-move list is "every empty square," which on an 8×8
  with ~30 pieces is ~34 candidate duck destinations per half-turn,
  not counting the piece move. Doubles the branching factor on the
  duck half-move. Confirm `legal_moves` isn't called in a hot loop
  with large boards before optimizing.

## Open questions

1. **Conflict semantics for composed variants.** When two active
   variants disagree on something (e.g. both want to override
   `GameStatus`, or one wants king-safety on and another off),
   what's the rule? v1 punts: the list has one entry. When variant
   #2 lands the matrix needs explicit pairs. Don't pre-build a
   conflict resolver.
2. **Duck colour in 3+ player variants.** The duck is colourless in
   2-player Duck Chess. If a 3-player variant ever lands, "the
   opponent's duck" is ambiguous. Out of scope.
3. **Pre-placed duck in custom positions.** Should the FEN allow a
   board with `(DUCK)` already on the grid but `duck_phase=placing`?
   Yes, treat the existing duck as the starting state and the next
   duck half-move as a `MoveDuck`. Document explicitly.
4. **Multiple ducks.** Out of scope, but the architecture allows it:
   `Square.duck` is per-square, so N ducks would just be N true bits.
   The duck-phase logic would need rework (one phase per duck?
   placement chooses which duck?). Defer until requested.

## Relationship to plan 10

Plan 10's movement stack is the natural long-term home for the
chokepoint conditionals in this plan. The mapping:

| This plan's conditional | Plan 10 modifier (eventual) |
|--|--|
| Duck blocks glider paths | Square-walkability modifier (100–199 band) |
| Duck-phase emits placement candidates | Move-generation modifier (0–99 band, variant-gated) |
| King-safety filter skipped in DuckChess | King-safety modifier (200+ band) reads `has_variant` and is `Keep` on DuckChess |
| King-capture wins | New game-end modifier (300+ band) |
| Castle-through-attack allowed | Same king-safety modifier — when off, castle path-check is also off |

Plan 10 step 8 (king-safety as modifier) is the most direct
beneficiary: once landed, the `legal_moves` conditional in this plan
collapses into "the king-safety modifier reads `has_variant(DuckChess)`
and short-circuits to `Keep`." The `variants` flag on `BoardFlags`
stays exactly as-is; what changes is *who* reads it.

Two ordering options once both plans are in flight:

- **Plan 11 first, plan 10 absorbs later.** Faster to ship the
  variant; the conditionals migrate to modifiers when plan 10 reaches
  the relevant step. Net rework: ~5 conditional sites become ~5
  modifier predicates.
- **Plan 10 step 8 pulled forward, then plan 11.** Cleaner but
  blocks Duck Chess on plan 10's prerequisites (type scaffolding +
  threat-resolution shim). Net work is the same; just reordered.

Recommend the former. The conditional → modifier migration is
mechanical and well-localized; gating on plan 10's full scaffolding
delays Duck Chess for no permanent gain.
