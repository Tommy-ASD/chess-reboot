# Ohio

> A single tile, fixed at game start, that permutes the moveset of
> any piece — friend, foe, or neutral — that ends a turn standing
> on it.

## Inspiration

The "only in Ohio" meme — Ohio as a vortex of inexplicable cursed
weirdness, where normal rules of reality bend. The state-as-shibboleth
joke has been refined for years; the chess version takes the joke
literally and makes the tile a *rule mutator.*

Strip the paint: this is a **tile that rewrites its occupant.** No
existing engine system does this. Square conditions can stun
(Frozen) or transform behaviour temporarily (Brainrot), but
nothing permanently mutates the *moveset* of a piece. The mechanic
is interesting because it decouples "what a piece can do" from
"what its type says it can do." A bishop that has been through
Ohio twice is genuinely a different piece, encoded entirely in
piece-local state.

## Mechanic

### Tile

`Ohio` is a `SquareType`, not a piece. It is set once at game
start (encoded in FEN) and never moves. There is exactly one Ohio
tile per board, by convention; the engine should not enforce
uniqueness (a multi-Ohio variant is the obvious follow-up).

Ohio looks and acts like a `Standard` square in every respect
*except* for the end-of-turn hook.

### End-of-turn hook

After both halves of a turn resolve (move + any reactive signal
fires), check: did any piece *end* its movement on the Ohio tile?
If yes, apply a **90° clockwise rotation** to that piece's
movement directions. The rotation is permanent and persists with
the piece, even after it leaves Ohio.

The piece carries a per-piece state `R` — an integer in `{0, 1,
2, 3}` representing the number of 90° clockwise quarter-turns
currently applied. Default `R=0`. Each visit increments `R`,
wrapping at 4.

Stackable: a piece that ends three consecutive turns on Ohio (by
being shuffled on and back on, or by some weirder pathway) ends
with `R=3` — a 270° rotation, equivalent to a 90° counter-
clockwise rotation. A fourth visit returns it to `R=0`.

### Permutation semantics

The rotation applies to the piece's **direction vectors**, not its
destination set. Concretely:

- Each piece's moveset is a set of `(dx, dy)` offsets and ray
  directions.
- Applying `R=1`: each `(dx, dy)` becomes `(dy, -dx)`.
- Applying `R=2`: `(dx, dy)` → `(-dx, -dy)`.
- Applying `R=3`: `(dx, dy)` → `(-dy, dx)`.

A bishop at `R=1` becomes a piece that moves on *orthogonals*
(the bishop's `(1,1)`, `(1,-1)`, `(-1,1)`, `(-1,-1)` rotate into
`(1,-1)`, `(-1,-1)`, `(1,1)`, `(-1,1)` — wait, the bishop is
symmetric under 90°, so it's still a bishop). Choose your example
carefully:

- **Knight at `R=1`.** `(2,1)` → `(1,-2)`. `(1,2)` → `(2,-1)`.
  Under the full 4-element symmetry the knight is also invariant
  — knight moves are 90°-symmetric.
- **Pawn at `R=1`.** A white pawn's `(0,1)` push becomes `(1,0)`
  — it now pushes east, not north. Its diagonal captures
  `(1,1)`, `(-1,1)` become `(1,-1)`, `(1,1)`. The pawn's
  asymmetry is the whole point: pawns are the pieces where
  rotation visibly matters.
- **King at `R=2`.** Invariant (kings are 90°-symmetric).
- **Custom piece (Sigma) at `R=1`.** All 8 queen-style rays
  rotate to 4 new rays; queen is also symmetric. Range cap
  preserved.

In short: symmetric pieces are unaffected by `R`. Asymmetric
pieces (pawns, the future NPC piece, the Italian Brainrot zigzag
hop) are the interesting case.

### Other interactions

- **Castling on Ohio.** If a king or rook involved in castling
  ends a castling move on Ohio, the rotation applies after the
  rook+king resolve. A rotated castled king has rotated moves
  going forward but castling rights are still expended.
- **Promotion on Ohio.** A pawn promoting on Ohio rotates *after*
  promotion. The new piece (queen, knight, etc.) carries `R=1`
  forward. This matters for asymmetric promotion targets like
  Bus.
- **Goblin kidnap from Ohio.** Goblin captures a piece on Ohio.
  The captive carries its `R` to wherever it gets returned. (It
  did not "end" the turn on Ohio — it was captured *from* Ohio.
  Skip the rotation increment in that turn; the captive's `R`
  freezes at its prior value.)
- **Italian Brainrot tralala pulse.** Toggles Switches, not
  square types — Ohio is unaffected.

## Why it's interesting

1. **Strategic black hole.** The tile sits permanent on the board.
   It has no immediate effect — pieces are perfectly safe to
   walk *through* it on the way to somewhere else. The risk is
   *ending a turn* on it. This creates a route-planning problem
   distinct from anything else in chess: every move-generator
   has to consider "does this destination cost me my moveset?"
2. **Weaponizable cursed tile.** A player can deliberately push
   an enemy piece onto Ohio (with Gooner, with a forced trade, or
   with Costco Guy's transport) to ruin it. A knight at `R=1`
   is still a perfectly fine knight; a pawn at `R=1` is now an
   orthogonal pusher and a powerful weapon if the controller
   knows how to use it.
3. **Asymmetry surfacer.** Every piece in chess has some
   directional asymmetry, but most are 90°-symmetric on
   movement. Pawns are the obvious exception. With custom pieces
   (Gooner, NPC, Mewing) that are asymmetric, Ohio becomes a
   first-class strategic location. Variant-design lever.
4. **Permanent state, no decay.** Unlike Brainrot or Frozen, the
   `R` counter stays on the piece. The board's history is
   readable from the FEN.

## Example scenarios

1. **Black pushes a white pawn onto Ohio.** White's pawn was on
   f5, Ohio is on f6. White advances; pawn ends on f6.
   `R=1`. White's pawn now pushes east. It used to threaten g6
   and e6; now it threatens g5 and g7 (its capture squares
   rotated). The pawn's promotion path is gone — it cannot reach
   the 8th rank without further rotations.
2. **Knight tourist.** Black knight visits Ohio at `R=0`, then
   later returns at `R=1` (still functionally a knight), then
   later at `R=2`, etc. Useless for symmetric pieces. The cost
   is just the tempo to get there and back, which may still be
   worth it if it denies Ohio to a more critical asymmetric
   piece — a one-tempo lockout move.
3. **Deliberate promotion catastrophe.** White's pawn is about to
   promote on e8. Black engineers a forced-move sequence where
   the pawn must instead promote on the diagonally adjacent
   Ohio. After promotion to queen and rotation, the new queen
   is still a queen (symmetric). No harm done. White picked the
   right promotion target.
4. **Bus through Ohio.** A Bus carrying 3 passengers ends a
   journey on Ohio. The Bus rotates `R=1`. The Bus's rook moves
   become rotated-rook moves (still rook movement — rook is
   symmetric). Passengers do not rotate; they're inside the
   Bus, not on Ohio.

## Where it shines

- **Variants with multiple asymmetric pieces** — Gooner, NPC,
  Mewing, pawns — get genuine variety.
- **Tactical puzzles** — the tile is a fixed feature; puzzle
  composers can build positions where the *only* solution
  involves a rotation.
- **Pawn endgames** — radically changes how passed pawns work
  if Ohio sits on a key file.

## Where it's awkward

- **Symmetric piece spam.** A board of only kings, knights,
  queens, and rooks is barely affected by Ohio. The tile is
  visual noise. Mitigated by: this is a variant feature, not a
  classical board feature.
- **Move-display UX.** The frontend needs to render pieces with
  rotation indicators (an arrow, a tinted halo). Without UX
  affordance, players will forget which pieces are rotated and
  blunder.
- **`R` and promotion choice.** A pawn-becomes-rook at `R=1` is
  equivalent to a non-rotated rook. The rotation is "wasted" in
  a sense. Acceptable: it's a meaningful choice, not a bug.
- **Multiple Ohio tiles.** If the variant designer places two
  Ohios, a piece can pinball, accumulating `R` quickly. Probably
  fine — the engine should handle it. Worth a stress test.

## Engine dependencies

- **`SquareType` extension** — same surface as `Block`/`Turret`/
  `Vent`. Plan 12 is the blueprint.
- **End-of-turn hook on square type** — Track tiles already have
  a "piece entered" hook; Ohio needs a "piece is on at end of
  turn" hook. Different timing, similar dispatch.
- **Per-piece state field `R`** — a `u8` modulo 4. Identical
  serialization to Skibidi phase.
- **Movement vector rotation primitive** — pure function over
  `(dx, dy)`. Should be one helper plus a unit test grid.
- **Move-generator rewrite** — every piece's `moves()` function
  needs to consult `R` and rotate its direction set before
  generating destinations. This is a cross-cutting change.

## New features required

- **Rotation-aware move generators.** Add `apply_rotation(dirs:
  &[(i8,i8)], r: u8) -> Vec<(i8,i8)>` and call it in every
  piece's move function as the first step. Plan 10's movement
  stack absorbs this naturally.
- **Direction-asymmetric piece marker.** Some pieces don't care
  about `R` (they're 90°-symmetric). Skipping the rotation step
  for those is a perf optimization; not required for
  correctness.
- **Frontend rotation indicator.** Render piece with a small
  arrow or rotation overlay. Out of engine scope but flagged.

## FEN encoding

Square-type tag: `T=OHIO`. No payload (the tile is stateless).

```
(T=OHIO)
```

Per-piece state: `R=<0|1|2|3>`, default `0`, omitted when zero.

```
(P=p,R=1)       # rotated black pawn
(P=N,R=3)       # white knight, 270° rotated (no effect — symmetric)
```

Round-trip: `R=0` omitted by encoder, parser tolerates explicit
`R=0`.

## Open questions

- **Counter-clockwise variant.** Should the rotation direction
  be a tile property (`T=OHIO,DIR=CW` vs `T=OHIO,DIR=CCW`)? Adds
  one bit of variety with negligible code cost. Probably yes.
- **Mirror-mode tile.** A sibling "Florida" tile that mirrors
  horizontally instead of rotating. Same infrastructure, one
  more transformation. Out of scope here but lined up.
- **Stacking with Brainrot.** A Brainrot'd piece on Ohio at end
  of turn — does the rotation apply? Yes; the conditions are
  orthogonal. Worth a test.
- **`R` reset condition.** Should anything *un-rotate* a piece?
  Probably not — permanence is the point. Promotion is the only
  natural reset (the piece type changes; could argue `R` should
  reset). Recommend: `R` does *not* reset on promotion — the
  piece carries its history.
- **Sigma's `G` counter and `R` together.** Coexist with no
  interaction. Just two independent state fields on the piece.
