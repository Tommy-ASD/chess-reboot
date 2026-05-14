# Plan 12: Block square type

Add a new `SquareType::Block` — impassable terrain that pieces cannot
land on and sliders cannot pass through. No payload, no signals, no
hooks. The simplest possible square type.

No dependencies. Touches the same surface that plan 08 already
established for `Turret` / `Vent` / closed `Gate`.

## Why now

The engine already has non-walkable squares (`Turret`, `Vent`, closed
`Gate`), but each of those carries implied future semantics — a Turret
suggests "shoots," a Vent suggests "emits," a Gate is "toggleable." A
generic, semantics-free **wall tile** is missing.

Use cases that motivate it: hand-crafted puzzle positions, variant
boards with carved-out playable regions, future map-style content
that needs interior walls without the visual/semantic baggage of a
Turret. The frontend editor's brush palette has a slot waiting for it.

## Concept

A square that:
1. Cannot be a move destination (no piece can land there).
2. Blocks slider paths (rook/bishop/queen/monkey rays stop at it).
3. Cannot hold a piece — `piece` is always `None` on a Block square.
4. Has no payload, no ID, no state. The FEN encoding is just
   `(T=BLOCK)`.

Functionally identical to `Turret` and `Vent` today. The point of a
*separate* variant is naming honesty: `Block` will never grow extra
behaviour. Turret/Vent are placeholders for unimplemented behaviour;
Block is "this is the final form."

## Naming

Going with **`Block`**. Alternatives considered:

- `Wall` — also good, slightly more evocative. Either reads fine.
- `Obstacle` — verbose.
- `Impassable` — describes the property, not the thing.

`Block` matches the user's framing and the existing single-word
convention (`Turret`, `Vent`, `Standard`). FEN tag: `BLOCK`.

If the user prefers `Wall`, this plan applies verbatim with a rename
— the work is mechanical.

## Types

### `engine/src/board/square.rs`

Add one enum variant:

```rust
pub enum SquareType {
    Standard,
    Turret,
    Vent,
    Block,                          // NEW — impassable, payload-free
    Switch { targets: Vec<SignalId> },
    Junction { /* ... */ },
    Gate { id: SignalId, open: bool },
    PressurePlate { /* ... */ },
    Track { direction: TrackDir },
}
```

Two trivial method updates:

```rust
impl SquareType {
    pub fn type_tag(&self) -> &'static str {
        match self {
            // ... existing arms ...
            SquareType::Block => "BLOCK",
        }
    }

    pub fn is_walkable(&self) -> bool {
        match self {
            // ... existing walkable arms ...
            SquareType::Turret | SquareType::Vent | SquareType::Block => false,
            // ... existing gate arm ...
        }
    }
}
```

That's the entire engine type-system change. Because every existing
non-walkability site already routes through `is_walkable()` (see
[chokepoints](#chokepoints)), no other engine source file needs to be
touched for the core behaviour. The compiler exhaustive-match check
will flag the few sites that match on `SquareType` directly — the only
known one is the no-payload arm in [engine/src/board/fen.rs:470](engine/src/board/fen.rs).

## Chokepoints

These already do the right thing the moment `is_walkable()` returns
`false` for `Block`. Listed for the reviewer's confidence, not because
they need code changes:

- [`Board::square_is_empty`](engine/src/board/mod.rs) — destination check used by Goblin, etc.
- [`generate_glider_moves`](engine/src/movement/glider.rs) — sliders stop at non-walkable.
- [`Knight::moves`](engine/src/pieces/standard/knight.rs) — jump landings filtered.
- [`King::moves`](engine/src/pieces/standard/king.rs) — adjacency filtered.
- [`Pawn::moves`](engine/src/pieces/standard/pawn.rs) — single/double push + diagonal capture all gated.
- [`Monkey::find_jump_moves`](engine/src/pieces/chess2/monkey.rs) — jump landings filtered.
- [`Skibidi::moves`](engine/src/pieces/fairy/skibidi.rs) — destinations + path filtered.
- [`Bus::moves`](engine/src/pieces/fairy/bus.rs) — slider path filtered.
- [`Board::relocate_pieces`](engine/src/board/make_move.rs) — safety net at apply time.

If any of these regressions show up under the new variant, the bug is
in *that* generator's path, not in this plan.

## FEN encoding

`Block` is payload-free. The FEN form is exactly:

```
(T=BLOCK)
```

Two edits in [engine/src/board/fen.rs](engine/src/board/fen.rs):

1. **Encoder** — add `Block` to the no-payload arm next to `Turret` /
   `Vent`:
   ```rust
   SquareType::Standard | SquareType::Turret | SquareType::Vent | SquareType::Block => {}
   ```
2. **Decoder** — add a tag match next to `TURRET` / `VENT`:
   ```rust
   Some("BLOCK") => SquareType::Block,
   ```

The lenient-parse behaviour falls out for free: an old FEN never
produces `T=BLOCK`, and an unknown tag already warns + falls back to
`Standard` (no behavioural regression). Round-trip is symmetric.

### Reference update

Add `BLOCK` to the `T=` value list in
[plans/README.md](plans/README.md):

```
| `T`  | Square type (default `STANDARD`) | `T=SWITCH`, `T=PLATE`, `T=GATE`, `T=JUNCTION`, `T=TRACK`, `T=VENT`, `T=TURRET`, `T=BLOCK`, `T=STANDARD` |
```

(One-line change. No new key, no new payload row.)

## Frontend (out of scope but enumerated)

The frontend mirrors the engine's square-type vocabulary. After the
engine commit lands, the frontend needs a corresponding three-line
update for the editor palette + FEN parser:

- [frontend/vite-dev/src/variables.ts](frontend/vite-dev/src/variables.ts) — add `"BLOCK"` to the `SquareType` union.
- [frontend/vite-dev/src/fen.ts](frontend/vite-dev/src/fen.ts) — add `"BLOCK"` to `KNOWN_SQUARE_TYPES`.
- [frontend/vite-dev/src/editor_page.ts](frontend/vite-dev/src/editor_page.ts) — add to `SQUARE_TYPES` brush list, `TYPE_CLASS`, and `TYPE_ACCENT`. CSS class `block` needs a colour (solid grey is fine).
- [frontend/vite-dev/src/signal_icons.ts](frontend/vite-dev/src/signal_icons.ts) — optional `BLOCK_SVG` icon and a `case "BLOCK":` arm in both `squareTypeIcon` and `squareTypeIconByType`. If skipped, the brush still works; the tile just renders bare.

Ship this as a follow-up commit after the engine commit, or fold it
into the same PR. Frontend is decoupled enough that either is fine.

## Sequencing

Two commits, both fully working:

1. **Engine** — enum variant + `is_walkable` + `type_tag` + FEN
   encoder/decoder + tests. Plans/README.md row updated.
2. **Frontend** — palette wiring + FEN parser entry + CSS + optional
   SVG icon.

Splitting is optional; one combined commit is also fine given the
total LoC is small.

## Tests to add

In [engine/src/board/tests.rs](engine/src/board/tests.rs):

- Extend `test_square_type_is_walkable` — add `assert!(!SquareType::Block.is_walkable());`.
- `test_block_blocks_glider_path` — rook on a1, Block on a3, black
  piece on a5. Rook's legal moves include a2 but not a3, a4, or a5.
  Removing the Block restores a5 as a capture.
- `test_block_rejects_knight_landing` — knight on b1, Block on c3.
  Knight's legal moves omit c3.
- `test_block_rejects_pawn_push` — pawn on a2, Block on a3. Pawn has
  no legal forward move. (Bonus: pawn on a2, Block on a4, with a3
  clear: single-push to a3 legal, double-push to a4 illegal.)
- `test_block_fen_roundtrip` — board with a Block square round-trips
  through `board_to_fen` → `fen_to_board` → `board_to_fen` and equals
  the canonical form. Confirms `(T=BLOCK)` is the emitted form.
- `test_relocate_pieces_rejects_block_destination` — synthesize a
  `GameMove` with destination = Block square, call `make_move`,
  assert `Err`. Guards against new generators forgetting the filter.

## Things to be careful about

- **`Square.piece` invariant.** A `Block` square should never hold a
  piece. `Square::new()` returns `Standard` (unchanged); a brush /
  hand-crafted FEN that places `(P=K,T=BLOCK)` is technically
  parseable today (the parser doesn't cross-validate piece + type).
  Two options: (a) reject at FEN-parse time with a warn + drop the
  piece, (b) leave it tolerant and rely on `relocate_pieces` to
  ensure no piece *moves* there. Recommend (b): the existing
  parser is lenient elsewhere, and the relocate safety net plus the
  test above cover the practical case. Document the constraint in
  the `SquareType::Block` doc comment.
- **Editor brush UX.** When the user paints `Block` over an occupied
  square, the frontend should clear the piece (mirror of how painting
  a piece over a Turret should be impossible). Out of engine scope;
  flagged here so the frontend follow-up doesn't miss it.
- **Train / track interaction.** Trains move along Track tiles and
  test `is_walkable()` when entering the next tile. A Block in a
  train's path will stop the train — same behaviour as a closed
  Gate. No code change needed; just confirm in a test if you ship a
  train + block scenario (optional; the unit tests above cover the
  primitive).
- **No SquareCondition implications.** `Frozen` / `Brainrot` on a
  Block square is meaningless but not harmful — no piece is ever
  there to be frozen. Don't bother filtering; the conditions list
  just sits inert.
- **Plan 10 (movement stack).** When plan 10 reaches the
  square-walkability modifier band (100–199), `Block` collapses into
  the same modifier that handles closed `Gate` / `Turret` / `Vent`
  today. No new modifier; just one more case the predicate returns
  `Reject` for. Mechanical migration.

## Relationship to plan 10

Trivial. The square-walkability modifier predicted in plan 10 step ~3
(the 100–199 band) covers all non-walkable square types uniformly.
When plan 10 lands, `is_walkable()` becomes the predicate body of that
modifier, and `Block` is one match arm among several. No design choice
needed in advance.
