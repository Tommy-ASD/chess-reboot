# Pivot

> A stationary piece that rotates the move directions of every piece within
> 3 squares by 90° around a chosen orthogonal axis.

## Inspiration

The geometric primitive is **local rotation of the tangent space**.
Patrick's Parabox uses orientation flips inside containers; Antichamber
uses rotated rooms that re-orient the player; Into the Breach's wind
mechanic rotates pushes. The Pivot generalizes "this region's idea of
north is rotated" into a piece that sits on one square and broadcasts
that twist outward.

Mechanically it's the simplest possible local geometric transformation:
swap *forward* with *sideways* in a neighbourhood. Everything downstream
is consequence.

## Mechanic

A Pivot is placed with a fixed **axis tag**: either `NS` (north-south)
or `EW` (east-west). The axis is set at placement and never changes.

The Pivot itself cannot move. It can be captured by any piece that could
legally reach its square *under the rotated geometry* — i.e. captures
have to respect the rule too. (Some Pivots are uncapturable from current
position; you must approach by rearranging the surrounding pieces.)

While the Pivot lives, for every square `S` within **Chebyshev distance
≤ 3** of the Pivot (a 7×7 zone centred on the Pivot), any piece on `S`
has its move-direction vectors rotated 90° around the Pivot's axis:

- `NS` axis: each direction vector `(dr, df)` becomes `(df, -dr)`.
  Equivalent to: N↔E, E↔S, S↔W, W↔N. Knights' L-shapes rotate
  identically. Diagonals rotate to the other diagonal family.
- `EW` axis: each direction vector `(dr, df)` becomes `(-df, dr)`.
  Equivalent to: N↔W, W↔S, S↔E, E↔N. Mirror of NS.

The rotation is **applied per source square**: a slider's ray rotates
from its origin, but each step along the ray re-checks its own square
for a Pivot zone. Practically: rotation is a property of the moving
piece's current square, not a global field. A rook leaving a Pivot zone
straightens out the moment it steps outside the 7×7. (See [Open
questions](#open-questions) for an alternative reading.)

**Composition.**

- One Pivot in zone → 90° rotation.
- Two overlapping Pivots, same axis → 180° rotation.
- Two overlapping Pivots, perpendicular axes → cancel (rotations compose
  to identity in the 2D rotation group of order 4 only when net angle
  is 0 mod 360 — perpendicular composition is 0° on this lattice).
- Three overlapping Pivots → 270° rotation, equivalent to 90° the other
  way.

Composition uses integer rotation count modulo 4. Each Pivot whose zone
covers a square contributes `+1` (NS) or `-1` (EW); the net rotation is
that sum mod 4, applied to move vectors before generation.

## Why it's interesting

It turns the board into an orientation puzzle. The familiar question
"can my rook attack that pawn" requires you to mentally project the
rook's rays through every Pivot zone they cross. A pawn pushed into a
Pivot zone can suddenly move sideways. Castling under a Pivot becomes a
spatial gymnastic. Knights, whose moves are already non-intuitive,
become genuinely puzzle-worthy.

The 7×7 footprint is large enough to dominate a corner of an 8×8 board
but small enough that placement matters. Two Pivots are not twice as
strong — they cancel or stack.

## Example scenarios

**Single Pivot, NS axis, white rook on a1:**

```
. . . . . . . .
. . . . . . . .
. P . . . . . .       Pivot zone covers a1-d4 corner
. . . . . . . .       Rook's "north" along a-file becomes "east"
. . . . . . . .       along the 1st rank
R . . . . . . .
```

The rook on a1 inside the Pivot zone: its rays north and south rotate
to east and west. Its east-west rays rotate to north-south. Net effect:
it still moves on rank 1 and file a — but a friendly piece on a4 no
longer blocks it, while a piece on d1 does. (The Pivot itself doesn't
change which squares the rook attacks, but it changes *which squares
block* the rays — because the ray's direction depends on the rook's
square, and the blockers' squares are evaluated independently.)

This is the kind of subtlety puzzle composers will love.

**Two Pivots, perpendicular axes, overlapping zones:**

```
. . . . . . . .
. . P . . . . .       NS Pivot at c5
. . . . . . . .       EW Pivot at e5
. . . . P . . .       Zones overlap on d4-f6 ish
. . . . . . . .       Net rotation in overlap: 0° (cancel)
```

The overlap region is **a Pivot-free island** geometrically, but it's
still surrounded by single-Pivot rotated regions. A bishop crossing the
island moves diagonally inside it, then diagonally-rotated outside.

**180° zone (two same-axis Pivots):**

A king in a 180° zone has its move directions flipped. "Castle kingside"
becomes "castle queenside" geometrically, though the legality
infrastructure for castling probably refuses to deal with this — see
[Open questions](#open-questions).

## Where it shines

- **Hand-crafted puzzles.** "Mate in 3, but you must first place a Pivot
  to redirect the attack." Composer's dream.
- **Asymmetric defensive structures.** A Pivot in front of your king
  forces attackers to approach via the rotated geometry, which often
  makes the king's natural escape squares unreachable.
- **Knight repositioning.** Knights inside a Pivot zone reach a rotated
  L-pattern. Combined with normal-geometry squares outside the zone,
  knights can tour an unreachable square in three half-moves instead of
  the usual minimum.

## Where it's awkward

- **Pawn promotion direction.** A pawn whose forward direction is
  rotated 90° will reach the edge of the board sideways — does it
  promote on the a-file or the 8th rank? Probably it doesn't promote
  inside a Pivot zone, and the player has to manoeuvre it out first.
  Documented as a clarification, not a bug.
- **Castling.** The king's two-square hop is geometric. Inside a Pivot
  zone the hop direction rotates. Suggest: castling requires both king
  and rook to be outside any Pivot zone, otherwise the move is illegal.
- **Check evaluation.** "Is the king in check" must be answered under
  the rotated geometry of every attacker. Performance-wise this is
  fine — the rotation is a per-square table — but the bug surface is
  large. Tests need to cover several Pivot-zone-check positions.
- **En passant.** The capturing pawn's diagonal is rotated. Probably
  illegal inside a Pivot zone; same reasoning as castling.

## Engine dependencies

- **Variable board dimensions** — already there.
- **Per-piece move-direction tables** — every piece exposes its
  direction vectors via a uniform interface. The engine has this for
  sliders; knight/king/pawn need a direction-table view (see new
  features).
- **A rule-evaluation phase** that runs *during* move generation, not
  after. The rotation has to apply at vector-emission time, otherwise
  rays trace through the wrong squares.

## New features required

- **`SquareTransform` table.** For each square, the engine computes a
  net rotation count (mod 4) at the start of each ply by scanning
  Pivots and summing. Cached per ply.
- **Direction-rotation helper.** `rotate_dir(dir: (i8, i8), k: u8) ->
  (i8, i8)` that applies a 90°·k rotation. Used by every move generator
  that emits direction vectors.
- **Move generator refactor: emit direction vectors, not destination
  squares.** Standard pieces probably already do this. Fairy pieces
  with hardcoded destination lists (Goblin, Skibidi phases) need their
  destinations expressed as vectors from the source. Some may not be
  rotatable — flag those as "Pivot-immune" in the piece's metadata.
- **Pivot piece** — a `Piece` with no move generator, an axis field
  (`Axis::NS` / `Axis::EW`), and a hook in board build-up to populate
  the `SquareTransform` table.

## FEN encoding

Standalone piece in a square's `P=` slot, with axis in the payload:

```
(P=PV,C=W,A=NS)         White Pivot, NS axis
(P=PV,C=B,A=EW)         Black Pivot, EW axis
```

`PV` is unique. `A=` is mandatory for Pivots (no default — the axis is
the whole identity of the piece).

If we ever introduce neutral-coloured immobile rule-pieces (compare
`Color::Neutral` in the engine), a Pivot could also be neutral:
`(P=PV,C=N,A=NS)` — captured by either side.

## Open questions

- **Per-square vs. per-ray rotation.** The current spec says rotation
  is evaluated at the source square. Alternative: rotation is
  evaluated *per ray step*, so a rook whose ray enters a Pivot zone
  mid-flight bends into the rotated direction at that point.
  Trade-off: per-step is more puzzle-rich but harder to compute and
  visualize; per-source is cleaner. Recommend per-source for v1;
  per-step is a future variant.
- **Knights and rotation.** Knights inside a Pivot zone should rotate
  their L-pattern. Trivial: rotate each of the 8 direction vectors.
  Confirmed mechanics, but worth one explicit test.
- **Promotion under rotation.** Suggested rule: promotion is suspended
  inside Pivot zones; player must manoeuvre out. Alternative: promote
  on whichever rank/file the rotated forward direction terminates at.
- **Captures of Pivots from outside the zone.** A rook 4 squares away
  attacks the Pivot square along an un-rotated ray. Should that capture
  succeed? Yes — the rook is outside the zone, its geometry is normal,
  the Pivot dies. The zone vanishes the instant the Pivot is captured.
- **Stacked Pivots in same square.** Allowed? Probably not — one piece
  per square, standard chess. Composition only comes from overlapping
  *zones*, not overlapping *Pivots*.
