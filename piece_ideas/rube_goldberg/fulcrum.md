# Fulcrum

> A 1x3 seesaw. Land on one tip, the other tip rises and flicks whatever sat on it upward.

## Inspiration

The seesaw in the playground Goldberg cartoon: ball drops on the left
end, kid on the right end is launched skyward. In Zachtronics terms,
the "level gauge" arm in Spacechem — a single piece that occupies
three tiles, has rotational state, and translates weight in one place
into ejection in another.

## Mechanic

A Fulcrum is a piece that occupies THREE tiles: a center pivot and
two tip tiles, horizontally aligned. It is the first piece in this
folder that occupies more than one square. The Locomotive + Carriage
trains already established multi-tile pieces in plan 09; Fulcrum
adopts that pattern but is stationary.

State:

- `center: Coord` — the pivot tile.
- `axis: Axis::Horizontal | Axis::Vertical` — FEN-fixed. v1 only
  Horizontal; Vertical reserved for v2.
- `tip_loads: (Option<PieceData>, Option<PieceData>)` — the (left,
  right) tip-held pieces. Conceptually "sitting on the seesaw."
- `triggered_this_cascade: bool` — resolver scratch.

For v1, the Fulcrum's three tiles are:
- left tip: `(center.x - 1, center.y)`
- center: `center`
- right tip: `(center.x + 1, center.y)`

- **Trigger.** Any piece's cascade-step motion ends on a tip tile.
  The center tile is non-interactive (pieces can sit on it; the
  Fulcrum's body doesn't care).
- **Effect.** When a piece lands on the left tip (and the Fulcrum
  hasn't triggered this cascade):
  - Set `triggered_this_cascade = true`.
  - Look at the RIGHT tip:
    - If the right tip currently holds a piece (in `tip_loads.1`),
      that piece is ejected one tile UPWARD (north) from the right
      tip. Standard slide rules: if blocked, the piece stays. If
      the destination has a piece, capture.
    - If the right tip holds a `Frozen` terrain condition (no piece,
      just the frozen tile), the seesaw motion REMOVES the Frozen
      condition. (Symbolic "the seesaw cracks the ice.")
    - If the right tip is empty terrain, nothing happens.
- Symmetric for right-tip trigger ejecting upward from the left tip.

Pieces sitting on a tip without being the trigger sit there harmlessly
until something lands on the opposite tip. The `tip_loads` slots
record what's there.

The "land on a tip" trigger fires the seesaw, and the ARRIVING piece
is now sitting on that tip (and recorded in the appropriate
`tip_loads` slot for future seesaw firings — across cascades).

## Cascade behavior

Resolution priority: Fulcrum eject runs at the END of each
cascade-step, but BEFORE the Catch-Pan tip-check. So if a piece
lands on a Fulcrum tip and is then ejected upward and lands in a
Catch-Pan, the Catch-Pan registers the arrival in the same
cascade-step.

Per-cascade firing: at most once. The `triggered_this_cascade` flag
guards. This prevents the obvious bouncing cycle (eject → piece
lands on the other tip → other tip ejects → ... ).

Specifically: even if a piece lands on the left tip, gets the right
tip ejected upward, and then a DIFFERENT piece lands on the right
tip in the same cascade, the Fulcrum does NOT fire again. The
designer must use TWO Fulcrums for double-seesaw puzzles.

Signal consumption: none. Fulcrum is mechanical.

## Why it's interesting

Fulcrum is the only piece in this folder that turns horizontal motion
into vertical motion. Marble rolls and Domino slides are
direction-preserving — a chain on a row stays on that row. Fulcrum
takes a piece arriving from the east on the right tip and ejects the
left-tip occupant north (or removes a Frozen condition north of the
left tip).

This makes Fulcrum the canonical "lift" — a way to inject a piece
into a row it couldn't reach by sliding. Combined with Catch-Pans
(which eject in a fixed direction), Fulcrum is the matching
"redirect-and-lift" partner.

The Frozen-removal rule is unique to this piece. Currently no other
mechanism in v1 removes a Frozen terrain condition. Fulcrum is the
"thawing" mechanism, which means Frozen tiles become a puzzle element
specifically gated by Fulcrum geometry.

## Example chain

Setup: a Frozen tile at `(4, 3)`. A Fulcrum with center at `(5, 4)`,
so left tip at `(4, 4)` and right tip at `(6, 4)`. The Frozen tile is
one row north of the left tip — exactly the eject destination for a
"land on right tip" trigger. A friendly Pawn at `(4, 1)` that the
player wants to advance, but it can't enter the Frozen tile at
`(4, 3)`.

```
. . . . . . . . .
. . . . . . . . .
. . . . X . . . .   <- (4,3) Frozen (X)
. . . . . . . . .
. . . . L C R . .   <- Fulcrum: L=(4,4), C=(5,4), R=(6,4)
. . . . . . . . .
. . . . . . . . .
. . . . P . . . .   <- (4,1) Pawn
```

Plus: a Domino chain bringing a Marble in from the east, such that
the Marble rolls west into the Fulcrum's right tip `(6, 4)`.

Trigger: player throws a Switch that fires the eastern chain.

- **Step 1.** Switch fires. Domino chain begins propagating west.
- **Step 2.** Last Domino slides west, bumping a Marble that was
  sitting east of it. (Marble's facing must be west; pre-staged.)
- **Step 3.** Marble rolls west, one tile per step. Eventually
  reaches `(6, 4)` — the Fulcrum's right tip.
- **Step N.** Marble enters right tip. Fulcrum triggers.
  - Right tip ejection target: look at LEFT tip's contents and eject
    upward. Wait — re-read the rule. "Land on right tip" triggers
    ejection from the LEFT tip's upward.

  Looking at the left tip `(4, 4)`: empty. No piece to eject. But
  one tile north of the LEFT tip is `(4, 3)`, the Frozen tile. The
  seesaw rule says "or removes a Frozen condition above" — and
  "above" means one tile in the eject-upward direction from the
  opposite tip.
  - Frozen at `(4, 3)` is removed.
  - The Marble is now sitting on the right tip `(6, 4)`. Recorded
    in `tip_loads.1`.
- **Step N+1.** Cascade quiesces.

Next turn: the Pawn at `(4, 1)` can now advance north — `(4, 3)` is
no longer Frozen. The seesaw mechanic was the unlock.

## Where it shines

- Frozen-tile unlock puzzles. The only thaw mechanism.
- Vertical redirect — a piece sliding east is converted to a
  northbound ejection.
- Conservation puzzles: the piece on the opposite tip is what gets
  flung, not the arriving piece. So the designer can pre-stage a
  "payload" on one tip and let the player's input land on the other
  tip as the "trigger."

## Where it's awkward

- Multi-tile pieces have FEN headaches. The Fulcrum's three tiles
  need to either be three FEN entries with shared identity, or a
  single FEN entry that occupies multiple squares. The Locomotive +
  Carriage pattern (plan 09) used the latter via a special render
  rule. Fulcrum should follow that pattern.
- The "land on a tip" trigger doesn't distinguish gentle approach
  from forceful arrival. A Domino sliding onto a tip and a Marble
  rolling onto a tip trigger identically. Maybe later add
  weight-sensitive Fulcrums for nuance.
- Only one Fulcrum-firing per cascade is restrictive. Designers will
  occasionally want a Fulcrum that bounces back. Workaround:
  multiple Fulcrums.

## Engine dependencies

- Multi-tile piece infrastructure (plan 09 Locomotive + Carriage) —
  Fulcrum is the second multi-tile piece. Generalize the rendering
  and click-target logic that plan 09 introduced for trains.
- Frozen terrain condition (existing) — Fulcrum is its consumer.
- Cascade resolver — end-of-step Fulcrum check, before Catch-Pan
  check.

## New features required

- Multi-tile piece movement is not needed (Fulcrum is stationary)
  but multi-tile occupancy reading is — the cascade resolver, when
  checking "what's on this tile", must recognize that two of the
  three Fulcrum tiles are tip slots, not regular occupied space (in
  the sense that pieces CAN co-exist on them via `tip_loads`).
  Actually: simplest rule is "tip tiles are passable terrain that
  records sitters." Treat tip tiles like soft Tracks.
- A new `SquareType::FulcrumTip { which: Left | Right, pivot: Coord,
  load: Option<PieceData> }`, with the center as
  `SquareType::FulcrumPivot { left: Coord, right: Coord }`. The
  three squares form a logical unit identified by the center
  coordinate.

## FEN encoding

A Fulcrum is three adjacent SquareType entries:

```
FULCRUMTIP(side=L,pivot=(5,4))     # left tip at (4,4)
FULCRUMPIVOT(left=(4,4),right=(6,4))  # center at (5,4)
FULCRUMTIP(side=R,pivot=(5,4))     # right tip at (6,4)
```

Pieces sitting on the tips embed as `load=`:

```
FULCRUMTIP(side=L,pivot=(5,4),load=PAWN-white)
```

If `load` is absent the tip is empty.

## Open questions

- What happens if a piece lands on a tip whose load slot already
  contains a piece? Two pieces on the same tile is normally
  disallowed. Spec: the arrival captures the load (replaces it).
  The captured piece is removed. The arriving piece becomes the new
  load. The Fulcrum still fires its opposite-tip ejection — the
  arriving piece's "landing" is the trigger; the captured-by-replace
  doesn't cancel that.
- Vertical Fulcrums (axis=Vertical) — left/right tips become
  top/bottom, eject direction becomes east instead of north?
  Reserve for v2 to keep v1 spec small.
- Two Fulcrums sharing a tip (geometry quirk) — disallow. The editor
  must enforce non-overlapping multi-tile pieces.
