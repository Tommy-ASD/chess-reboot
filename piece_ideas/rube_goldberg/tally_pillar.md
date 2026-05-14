# Tally-Pillar

> Counts pieces along its file. Hits its target, freezes the file for a tick.

## Inspiration

The reactor sensor in Spacechem that says "if exactly 4 atoms of
type X are present, fire output." The census-as-trigger. A piece
that reads the board's static state at cascade start and gates the
cascade based on what it saw.

## Mechanic

A Tally-Pillar is a stationary piece (or tile-piece hybrid) with:

- `target: u8` — FEN-fixed. The pillar fires only when its census
  count equals exactly this.
- `direction: Dir::N | Dir::S` — FEN-fixed. The file-direction along
  which the pillar's freeze ray fires. The CENSUS direction is always
  "the rest of this file in BOTH N and S directions," but the freeze
  ray fires in one chosen direction.
- `fired_this_cascade: bool` — resolver scratch.

Census: at the START of each cascade, the pillar counts pieces it has
line-of-sight to in BOTH north and south directions along its file.
Line-of-sight is blocked by walls and by opaque pieces (the count
stops at the first opaque piece, INCLUDING that piece). Transparent
pieces (none in v1, reserved) would let the count continue past them.

- **Trigger.** Cascade start (specifically: the FIRST cascade-step,
  before any motion). The pillar reads its census; if `count ==
  target`, the pillar fires.
- **Effect.** The pillar fires a freeze ray along its `direction`.
  Every tile from `pillar.coord + direction` to the first
  wall-or-opaque-piece becomes Frozen for exactly one cascade-tick.
  The Frozen condition reverts at the END of the cascade.

Bounded propagation: at most one fire per cascade per pillar
(`fired_this_cascade` flag). The Frozen tiles created by the pillar
don't persist past the cascade, so they can't be re-counted on the
next cascade by some "feedback loop" — the board state at the next
cascade's start does not include the temporary Frozen.

## Cascade behavior

Resolution priority: Tally-Pillar fires in a SPECIAL pre-step phase
that runs once at cascade start, before step 1. This is the "census
phase." All pillars census simultaneously, then all firing pillars
emit their freeze rays simultaneously, then step 1 proper begins.

Per-cascade firing: at most once per pillar. The `fired_this_cascade`
flag is cleared in the resolver's post-cascade reset pass.

Signal consumption: Tally-Pillar does NOT consume any substrate
signal. Its census is a board-read, not a signal-receive. The freeze
ray is a direct terrain mutation, not a substrate signal — it doesn't
fire any receivers. (Future extension: pillar could ALSO emit a
substrate signal to a wired target on firing. Out of scope v1.)

The pillar's frozen-tile creation interacts with movement rules
already established for Frozen: Frozen tiles block piece entry. So
the freeze ray erected by the pillar acts as a one-cascade barrier
along its column.

## Why it's interesting

Tally-Pillar is the ONLY piece in this folder that reads the static
board state. All the others react to events (signals, motion,
adjacency). The pillar gives the puzzle designer a way to encode
"the puzzle only resolves correctly if there are exactly N of a
certain piece on this file," which is a fundamentally different
constraint shape than the rest of the system can express.

It also gives Frozen terrain a SECOND creation mechanism. Currently,
Frozen tiles are placed by the puzzle designer at level start. The
pillar lets a Frozen tile appear DURING a cascade — and only during
that cascade — which is useful for "block this enemy piece from
escaping just for this turn" puzzles.

## Example chain

Setup: a Tally-Pillar `T` at `(4, 5)`, `target = 3, direction = S`.
The file is column 4. North of T at column 4: a Knight, a Pawn, an
empty tile, a Rook — that's 3 visible pieces (the rook isn't
visible past the pawn... wait, let me re-spec).

Line-of-sight count: walks NORTH from T's tile, counts each piece
until it hits an opaque piece (counted as the LAST piece seen) or
a wall. Walks SOUTH similarly. Then sums both counts.

Column 4 layout, top to bottom:

```
.   <- row 8
R   <- row 7  (Rook)
.   <- row 6
P   <- row 5  (Pawn)
N   <- row 4  (Knight, NORTH of T... wait, let me restructure)
```

Let me redo this with clearer geometry. Column 4, rows top-to-bottom:

```
row 8: .
row 7: R       (Rook, friendly)
row 6: .
row 5: P       (Pawn, friendly)
row 4: T       (Tally-Pillar, target=3, direction=S)
row 3: .
row 2: k       (enemy king)
row 1: .
row 0: .
```

Census at cascade start:
- Walk NORTH from T (row 4 → 5, 6, 7, 8):
  - row 5: Pawn. Count = 1. Pawn is opaque (assume all pieces are
    opaque in v1). Stop.
- Walk SOUTH from T (row 4 → 3, 2, 1, 0):
  - row 3: empty. No count.
  - row 2: enemy king. Count = 2. Opaque. Stop.
- Total census: 1 + 2 = 3. Target is 3. MATCH.

Pillar fires. Direction = S. Freeze ray walks SOUTH from row 3 until
the first wall/opaque:
- row 3: empty. Freeze. Continue.
- row 2: opaque (the king). DO NOT freeze the king's tile (the king
  itself is the stopper). Stop.

Frozen tiles for this cascade: row 3, column 4.

Player's actual move this turn: some other piece. The cascade rolls.
Meanwhile, the king at row 2 column 4 CANNOT escape north (row 3 is
Frozen for this cascade). The cascade resolves whatever else is
happening, and at cascade end the Frozen is removed.

The puzzle: maybe the cascade ends with a Marble rolling north into
the king's column. The pillar's Frozen-creation was a SETUP — it
prevented the king from sidestepping into row 3 (using whatever
counter-move mechanism the variant has, e.g., a substrate-fired
"king escape" Domino).

## Where it shines

- Census puzzles. "Place pieces such that exactly 3 are visible from
  T's file."
- Conditional barriers. The pillar erects a one-cascade wall only
  if the board state justifies it.
- Multi-pillar gating. Several pillars on the board, all needing to
  match simultaneously for the puzzle to resolve.

## Where it's awkward

- "Line-of-sight" interacts with every other piece in the engine.
  Are Catch-Pans opaque? Catch-Pans are tiles, not pieces, so they
  don't appear in the census even when they hold a piece. (Or do
  they? A held piece IS on the board logically. Decision: the held
  piece counts.) Are Fulcrum tips opaque? They're tiles too, but
  they can hold loads — count the load if present. Mirror-Coils are
  tiles — empty for census purposes. Spec needs a "what counts as
  a piece for census" enumeration.
- Cascade-start timing. The census happens before step 1. But what
  if a previous cascade left the board in a state that the player
  hasn't acted on yet? Spec: census reads the BOARD STATE at the
  start of THIS cascade, which is the player's turn-start state
  PLUS any move the player has just made. Player's move is part of
  the cascade-triggering event, so it's already on the board when
  the census runs. Good.
- Frozen lifetime. "One tick" is ambiguous. Spec: lasts for the
  ENTIRE current cascade, removed at cascade end. So the Frozen
  blocks all motion within this cascade and only this cascade.

## Engine dependencies

- Frozen terrain (existing) — Tally-Pillar is a runtime creator of
  Frozen.
- Line-of-sight infrastructure (existing for sliding pieces) — reuse
  the same ray-walk helper.
- Cascade resolver — needs a pre-step phase for census.

## New features required

- Cascade pre-step phase. Currently the resolver runs steps until
  quiescence; add a phase-0 "census/setup" pass that runs once at
  cascade start. Pillars run there. Future pieces that read board
  state at cascade-start could also live in this phase.
- Temporary terrain conditions. The Frozen system currently treats
  Frozen as a persistent attribute of a tile. Add a `removed_after:
  Option<CascadeId>` field, so terrain conditions can be marked as
  cascade-local. The resolver's post-cascade teardown removes them.
- Pillar's opacity rules for census — formal spec in resolver docs.

## FEN encoding

```
TALLYPILLAR(target=3,dir=S)
TALLYPILLAR(target=1,dir=N)
TALLYPILLAR(target=5,dir=S)
```

`target` domain: 0..=15 (board files are at most 15 tall in v1
variable-board limits). `dir` is N or S only — east-west pillars are
the "Tally-Beam" v2 piece (file-vs-rank generalization).

## Open questions

- What if the pillar's own tile is counted? Spec: no, the pillar is
  its own origin and not counted.
- Multiple Tally-Pillars on the same file, both with target=3. The
  one south fires first? They census INDEPENDENTLY (each runs its
  own walk in both directions from its own coord) and fire
  simultaneously in the pre-step phase. Both can fire if both match.
- Target=0 — fires if the pillar sees NO pieces in either direction.
  Useful for "this file must be cleared" puzzles. Allowed.
- Mid-cascade census re-evaluation: NO. Census is once-per-cascade.
  This keeps termination obvious — pillars cannot loop.
- Pillar interaction with Tracks (the moving-train substrate from
  plan 09). A train on the pillar's file is a piece, counted.
  Multi-tile train counts as one piece (the locomotive position)
  for census purposes.
