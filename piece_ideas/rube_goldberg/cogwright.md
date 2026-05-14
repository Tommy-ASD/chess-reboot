# Cogwright

> Watches its four neighbors. Two distinct types touching it rotate it 90° clockwise. After rotating, ejects a pulse-stone along its new knight-vector.

## Inspiration

The rotor in Opus Magnum that turns its working arm to a new angle
each time the input changes. The clockwork escapement — a piece that
moves in discrete clicks driven by external conditions, not by direct
signals. The Cogwright is the only piece in this folder with a
ROTATIONAL state that the cascade itself manipulates.

## Mechanic

A Cogwright has:

- `facing: u8` — current rotation, 0..=3 (0=N, 1=E, 2=S, 3=W). FEN
  field is `facing=N/E/S/W`. The Cogwright's "knight-vector" is the
  knight-move offset rotated by `facing * 90°` clockwise. Base
  knight-vector at facing=N is `(+1, +2)` (one east, two north — a
  standard knight L). Rotating clockwise:
  - facing=N: knight-vector = (+1, +2)   (NNE)
  - facing=E: knight-vector = (+2, -1)   (ENE)
  - facing=S: knight-vector = (-1, -2)   (SSW)
  - facing=W: knight-vector = (-2, +1)   (WNW)
- `pending_rotation: bool` — resolver scratch. True if this
  cascade-step's adjacency-check determined the piece should rotate
  at end-of-step.
- `pending_eject: bool` — resolver scratch. True if the rotation
  happened LAST cascade-step and the piece should eject this step.

## Cascade behavior

This piece runs in a TWO-PHASE rhythm across cascade-steps.

**Phase A: adjacency observation.** At the end of every cascade-step,
the Cogwright looks at its four orthogonal neighbors. It counts the
distinct PIECE TYPES present:

- "Piece type" means the `PieceType` enum variant — Pawn, Knight,
  Domino, Marble, Hour-Petal, etc. Same type in two neighbors counts
  once.
- Empty tiles and terrain-only tiles (Frozen, Brainrot, Track,
  Mirror-Coil) do NOT count — only piece-types.
- The Cogwright itself does not count (it's not its own neighbor).

If the distinct count is EXACTLY 2, set `pending_rotation = true`.
Otherwise (count = 0, 1, 3, or 4), do nothing.

**Phase B: rotation.** At the START of the NEXT cascade-step, if
`pending_rotation` is true:
- Rotate `facing` by 90° clockwise (`facing = (facing + 1) % 4`).
- Clear `pending_rotation`. Set `pending_eject = true`.

**Phase C: pulse-stone ejection.** At the START of the
cascade-step AFTER rotation (one more step later), if `pending_eject`
is true:
- Compute the destination: `cogwright.coord + knight_vector(facing)`.
- If the destination is on-board and empty, place an inert
  "pulse-stone" piece there (a Color::Neutral piece that does
  nothing — see "Pulse-stone" below).
- If the destination is off-board or occupied, no stone is placed
  (the ejection fizzles). The Cogwright still consumed its
  pending_eject state.
- Clear `pending_eject`.

This gives a three-cascade-step rhythm: observe → rotate → fire.
Within ONE cascade, a Cogwright can complete at most one full cycle
(unless the cascade runs for many steps), so it acts as a phase
counter measured in cascade-steps.

**Pulse-stone.** An inert piece. `Color::Neutral`. No motion, no
signals, no captures. Occupies its tile and counts toward Catch-Pan
weight and Tally-Pillar census (weight 1, opaque to LOS). Removed
only by capture. FEN: `PULSESTONE`.

## Per-cascade firing

A Cogwright can rotate AT MOST ONCE per cascade — by construction.
The observation in step N triggers rotation in step N+1 and
ejection in step N+2. The `pending_*` flags are mutex: you can't
have a second rotation queued until the first ejection has fired
(or fizzled).

This is the bounded-propagation guarantee. Cogwright is a slow
piece — it does not produce runaway cascades.

## Why it's interesting

Cogwright is the phase-counter primitive. It makes the LENGTH of a
cascade matter to the result. Other pieces in this folder don't
care if a cascade takes 3 steps or 30; their outputs are the same
either way. A Cogwright's output depends on WHICH step it fires —
specifically, what its neighbors looked like the step before it
rotated.

It's also the piece that synthesizes the others. The "distinct type
count = 2" rule is naturally satisfied by puzzles that mix
Cogwright neighbors of different types — a Domino + a Marble + an
empty tile + a Pawn would be 3 distinct types, no rotation. A
Domino + a Marble + two empty tiles is 2 distinct types, rotates.
The puzzle designer arranges the cascade so that the Cogwright's
neighbors transition through the right sequence of distinct-type
states over consecutive cascade-steps, advancing it through a
specific number of rotations.

The pulse-stone is intentionally inert. It is a MARKER — placed by
the cascade to show that something happened, and useful only because
it then participates as ballast in Catch-Pans or as a sightline
blocker for Tally-Pillars. The Cogwright is a writer; the rest of
the system reads.

## Example chain

Setup: a Cogwright `Cw` at `(5, 5)`, facing N. Three Dominoes
arriving at three of its neighbors over the course of a cascade,
each from a different direction, each at a different step.

Initial neighbors of `Cw`:
- N (5,6): empty
- E (6,5): empty
- S (5,4): a Pawn (stationary, friendly)
- W (4,5): empty

Distinct piece types adjacent: just Pawn. Count = 1. No rotation
queue.

Player throws a switch. Three Dominoes (`D1, D2, D3`) start
arriving at the Cogwright's neighbors.

- **Step 1.** `D1` slides into (4,5) — Cogwright's west neighbor.
  Now neighbors are: empty, empty, Pawn, Domino. Distinct types: 2
  (Pawn, Domino). Set `pending_rotation = true`.
- **Step 2 start.** Apply pending rotation: `facing = N → E`. Set
  `pending_eject = true`. End of step 2 observation: neighbors are
  same as before (Pawn south, Domino west, empties N and E),
  distinct count still 2. Set `pending_rotation = true` AGAIN.
  (Note: the flag wasn't cleared mid-step; it was cleared at the
  start of step 2 when the rotation applied. The end-of-step
  observation re-sets it.)
- **Step 3 start.** Two pending flags. Process in order:
  - `pending_eject` (from step 2's rotation): Cogwright at (5,5),
    now facing E, knight-vector (+2, -1). Destination (7, 4). If
    empty, drop a pulse-stone. Clear `pending_eject`.
  - `pending_rotation` (from step 2's observation): rotate
    `facing = E → S`. Set `pending_eject = true`.
- **Step 3 mid.** Meanwhile, `D2` slides into (6, 5) — Cogwright's
  east neighbor.
- **Step 3 end-of-step observation.** Neighbors: empty (N), Domino
  (E), Pawn (S), Domino (W). Distinct types: 2 (Pawn, Domino).
  `pending_rotation` already set; no change.
- **Step 4 start.** Process:
  - `pending_eject`: Cogwright facing S, knight-vector (-1, -2).
    Destination (4, 3). If empty, drop a pulse-stone. Clear.
  - `pending_rotation`: rotate `facing = S → W`. Set
    `pending_eject = true`.
- **Step 4 mid.** `D3` slides into (5, 6) — Cogwright's north
  neighbor.
- **Step 4 end-of-step.** Neighbors: Domino (N), Domino (E), Pawn
  (S), Domino (W). Distinct types: 2 (Pawn, Domino) — still 2,
  because three Dominoes is one type. Set `pending_rotation`.
- **Step 5.** Eject (facing W, knight-vector (-2, +1), destination
  (3, 6)). Rotate W → N. Set pending_eject. Observe; if neighbors
  haven't changed, distinct count is still 2; queue another rotation.

This loop continues as long as the neighbor configuration keeps
producing distinct-count = 2. The cascade will only terminate when
either the neighbors change (e.g., a Pawn captures, a Domino slides
away) or the cascade is otherwise quiescent and the Cogwright is the
only active piece — at which point, hmm, the cascade has work to do
each step, so it doesn't quiesce.

This is a problem. The Cogwright as specified can spin indefinitely.

**Bounded-propagation fix.** Add an explicit per-cascade rotation
cap: a Cogwright may rotate AT MOST ONCE per cascade. Use a
`rotated_this_cascade: bool` resolver-scratch flag. After the first
rotation, subsequent `pending_rotation` observations are ignored.

Re-spec example (corrected):

- **Step 1.** D1 arrives W. distinct=2. queue rotation.
- **Step 2.** rotate N→E. queue eject. observation: distinct=2, but
  `rotated_this_cascade=true`, so do NOT queue another rotation.
- **Step 3.** eject toward (7,4). pulse-stone placed if empty.
  cogwright is now done for this cascade.
- **Step 4+.** Cogwright observes but does not act. D2 and D3 still
  propagate; cascade ends when their motion does.

End state: pulse-stone at (7,4), Cogwright facing E.

Next turn, the cascade resets `rotated_this_cascade = false` and
the Cogwright is eligible to rotate again.

## Where it shines

- Phase-counter puzzles. Over multiple turns, the Cogwright rotates
  cumulatively. The pulse-stone deposits act as a record of which
  step the rotation happened on.
- Pulse-stone-as-payload puzzles. The Cogwright is the only piece
  that creates pulse-stones, and pulse-stones are useful for
  Catch-Pan ballast and Tally-Pillar census. So the Cogwright is
  the "ballast factory."
- Sightline-blocker puzzles. A pulse-stone at (7,4) blocks
  line-of-sight along its file/rank. A Tally-Pillar's census
  changes once the Cogwright has fired enough to deposit a stone in
  its file.

## Where it's awkward

- The two-step rhythm (observe → rotate → fire) is hard to teach.
  Players reading the FEN see `facing=E` but don't know that's the
  state AFTER a rotation; they have to mentally rewind.
- The "exactly 2 distinct types" rule is sensitive to which pieces
  the designer places. A neighbor that's an empty tile doesn't
  count, but adding a single Marble can flip the count from 1 → 2
  or 2 → 3. Brittle to small edits.
- The once-per-cascade rotation cap (added above) means the
  Cogwright is a one-shot in any given cascade. Over a long puzzle
  with many turns, it accumulates rotations. This is fine but the
  rhythm is asymmetric — a "fast" puzzle with a long cascade only
  gets one rotation, the same as a "slow" puzzle with a one-step
  cascade.

## Engine dependencies

- Cascade resolver — needs end-of-step observation hooks, multi-step
  scheduling for pending_eject and pending_rotation.
- Existing piece-type enum — for distinct-type counting.
- New piece: pulse-stone. Inert; the simplest new piece.

## New features required

- `PieceType::Cogwright { facing }` and `PieceType::PulseStone`
  variants. (Or as `SquareType`s; Cogwright is more piece-like.)
- Resolver-scratch fields for the Cogwright's flags. Three booleans
  per Cogwright per cascade. Free.
- Distinct-piece-type-counter helper for adjacency. Walks the four
  neighbors, collects unique PieceType discriminants. Simple.
- The once-per-cascade rotation cap is also resolver state.

## FEN encoding

```
COGWRIGHT(facing=N)
COGWRIGHT(facing=E)
PULSESTONE
```

Cogwright's `facing` is the static state between turns. Pulse-stones
have no payload.

## Open questions

- Should the once-per-cascade cap be relaxed to once-per-N-steps
  with a designer-tunable N? Probably not — keep v1 simple.
- Should the knight-vector be configurable per Cogwright? E.g., one
  Cogwright with the standard knight (+1, +2) base, another with
  (+2, +1) base. v2 feature; v1 uses the single fixed base vector.
- What happens when a Cogwright rotates but its eject destination is
  occupied? Spec above says fizzle. Alternative: capture the
  occupant (turning the Cogwright into a knight-range capturer).
  Decision: fizzle for v1 — keeps the Cogwright purely about phase
  state, not about combat.
- Are pulse-stones capturable by normal pieces? Yes — they're
  pieces, no special protection. A Pawn moving onto a pulse-stone
  captures it. The stone is gone.
- Adjacency through walls — if a wall separates Cogwright from a
  neighbor tile, does the neighbor count? Spec: no, walls block
  adjacency. Use existing wall-adjacency rules.
