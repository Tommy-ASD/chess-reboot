# Fold

> A piece that creases the board along its rank or file — sliders crossing
> the crease re-emerge on the mirror side.

## Inspiration

The geometric primitive is **identification of two halves of the
plane**. Origami; the Möbius cylinder; Portal's portals (though Fold
is non-orientable in 1D rather than 2D); Patrick's Parabox containment
maps. The Fold is the thinnest possible "this is now connected to that"
piece.

Where Pivot rotates the local tangent space, Fold rewrites the global
topology. Fold and Pivot together span the two natural geometric
deformations of a grid: rotation and identification.

## Mechanic

A Fold is placed with an **orientation tag**: `H` (horizontal crease,
along its rank) or `V` (vertical crease, along its file).

The Fold cannot move. It can be captured.

While the Fold lives at coordinate `(r₀, f₀)` with orientation `O`:

- **Horizontal Fold (`H`).** The Fold's rank `r₀` is the crease.
  Every rank `r` with `r ≠ r₀` is identified with its mirror across
  `r₀`: rank `r₀ + k` ↔ rank `r₀ - k`. The board topologically becomes
  two layers glued along rank `r₀`.
- **Vertical Fold (`V`).** Same with files.

**Crossing the crease.** A piece sliding along a ray that would cross
rank `r₀` (or file `f₀`) does the following:

1. The piece reaches the crease square `(r₀, f)` for some file `f`.
2. If that square is the Fold's own square `(r₀, f₀)`, the slide stops
   normally — the Fold blocks like any piece. (Capture the Fold to
   destroy it.)
3. Otherwise, the next step of the ray emerges at `(r₀ - dr, f + df)`
   instead of `(r₀ + dr, f + df)`, where `(dr, df)` was the ray's
   direction vector. The ray continues on the **mirror side**.

A piece can also *land* on the crease without crossing — moving from
rank `r₀ - 1` to `r₀` is a normal move with no fold-effect. The fold
only kicks in when the ray would *pass through* the crease.

**One-directional rays.** A bishop's diagonal coming from below the
crease bends upward at the crease, then continues "north-east" on the
folded geometry — which on the real board is "south-east" on the upper
half. Visually, the bishop attacks a square that looks far away but is
one fold-hop close.

**Multiple Folds.** Each Fold's crease is independent. A ray can cross
multiple Folds in succession; apply the mirror map at each crease in
the order the ray hits them.

```
            f₀
            |
   . . . . .|.X. . .          V-Fold at d4 (crease = d-file)
   . . . . .|. . . .          R at a4 sliding east
   . . . . .|. . . .          Ray: a4-b4-c4 (crease) — emerges
   R . . . .|. . . .          at c4 → b4 → a4 — loops? See
   . . . . .|. . . .          [Open questions] for cycle handling.
```

(The diagram intentionally shows the trap: a ray approaching the crease
parallel to it never crosses. Folds only fold *crossing* rays.)

## Why it's interesting

Tactics suddenly involve **long-distance threats that wrap around the
fold**. A rook on a1 with a V-Fold on d-file can attack h1 by sliding
east — the ray crosses d-file, mirrors to the west half but on the
other side of the fold (the imagery is wrong; see [Open
questions](#open-questions) for the cleaner version), and reaches the
target via geodesic.

It's also strongly **directional asymmetry**. White and black pieces
are not symmetric across a horizontal fold the way they are across the
board's middle — the fold can be placed off-centre, creating attack
asymmetries.

## Example scenarios

**Horizontal Fold on rank 4, rook on a1, target on a7:**

```
. . . . . . . .   rank 8
. . . . . . . .   rank 7   ← target
. . . . . . . .   rank 6
. . . . . . . .   rank 5
=================== crease (Fold somewhere on rank 4)
. . . . . . . .   rank 3
. . . . . . . .   rank 2
R . . . . . . .   rank 1
```

Rook slides north along a-file. Ray reaches rank 3, then rank 4 (the
crease, not the Fold's own square). The ray emerges on rank 3 again,
but flipped — actually, "the mirror of rank 5 about rank 4" is rank 3.
So crossing rank 4 from below should emerge **going south** on the
upper half — i.e. the ray continues "north" on the folded surface,
which corresponds to *descending rank numbers* on the upper half. So:
ray crosses 4, lands on rank "5" but flipped means rank 5 ≡ rank 3 by
identification — wait, that's a self-reference.

The right reading: ranks above the fold are identified with ranks
below. The board becomes effectively half-height. A piece on rank 7
*is also* on rank 1 (with rank 4 fold). A rook on a1 directly attacks
a7 because a7 ≡ a1 — i.e., they're the same square. (Capture-on-self
is unresolved; see [Open questions](#open-questions).)

A more useful framing: the rook on a1 sliding north reaches a2, a3,
then a4 (the fold rank). If the Fold piece is elsewhere on rank 4
(say e4), the a4 square has no Fold but is still on the crease — the
ray passes through it and emerges going *south* on the upper half,
which lands on... a4 again, then a3, a2, a1. So a horizontal fold makes
the rook see itself. That can't be right.

**Cleaner restatement (see [Open questions](#open-questions) for the
formal version):** treat the crease as identifying the two halves so
that ranks > r₀ are renamed to mirror ranks < r₀. The board has
**fewer ranks**. A rook's ray on a1 sliding north reaches the Fold-side
of rank r₀, which is just the new "top edge" — analogous to hitting a
wall.

For the practical mechanic, ship the simpler version: **a ray hitting
the crease square is reflected back along its mirror direction**.
The fold is a *mirror*, not an *identification*.

```
              ↑ ray direction
              |
   . . . . . R . .
   . . . . . . . .
   . . . . . . . .
==================== Fold rank — ray hits and reflects
   . . . . . . . .   (ray emerges going south)
   . . . . . . . .
```

A rook on a3 sliding north with a H-Fold on rank 4 attacks: a3 itself
(start), a4 (the crease), then a3 again, a2, a1 — but a3 is its own
square so the ray's reflected path stops there. Net effect: the rook
attacks rank 4 and is then reflected to attack its own column going
south. The rook on a3 attacks a1, a2 (in addition to whatever was
beyond a4 before, which is now reflected).

This is the **mirror reading**. It's geometrically clean. I'm going
with it.

**Diagonal fold-crossing.** A bishop on c1 sliding NE along c1-d2-e3-f4
with H-Fold on rank 4: ray reaches f4 (crease), reflects, emerges on
g3 going SE. Bishop attacks h2 via fold. This is the
"rook-on-a1-attacks-h1-via-d-Fold" tactic, in diagonal form.

## Where it shines

- **Long-range fold tactics.** A bishop or queen with a sympathetic
  fold can attack squares that look completely safe. The defender
  must constantly check "is there a fold I'm not accounting for."
- **Trapping the king.** A H-Fold placed close to the king's rank
  creates reflected attacks on king-flight squares.
- **Composition.** Two H-Folds at different ranks compose to a
  "double reflection" = a translation. Geometric pedagogy.

## Where it's awkward

- **Pawn moves.** Pawns move one direction. If a pawn's push would
  cross a fold, does it reflect and end up further south? Suggest:
  pawn pushes do **not** fold. Only sliders fold. (Knights and king
  also don't fold — their moves are jumps, not rays.)
- **Castling.** Castling involves king and rook moving by fixed
  offsets. If a fold is between king and rook, castling is illegal.
- **King in check.** Check evaluation must run *all* folds across *all*
  enemy slider rays. Adds complexity to the check function.
- **Self-attack.** Can a piece attack itself through a fold? A bishop's
  ray reflects and may return to its own square. Treat as: ray
  terminates at any square already occupied, including by self.

## Engine dependencies

- **Variable board dimensions** — present.
- **Ray-tracing move generation** for sliders. Must operate
  one-step-at-a-time so each step can check for fold-reflection.

## New features required

- **Fold registry.** Each ply, scan board for Fold pieces; build a
  list `Vec<Fold>` with crease coordinate and orientation. Used by
  the slider ray-trace.
- **Slider ray-tracer mod.** Replace any "compute final square" slider
  code with explicit single-step ray traces that consult the fold
  registry. Each step checks "would this step cross a crease," and if
  so, applies the mirror map to the direction vector before continuing.
- **Fold piece** with no move generator, an orientation field, and a
  board-build-up hook that registers itself.
- **Cycle protection.** If a ray reflects between two facing folds, the
  ray could trace forever. Cap ray length at board diagonal length;
  treat over-cap as "no attack." Make this configurable per piece
  (sliders only).

## FEN encoding

```
(P=FD,C=W,O=H)          White Fold, horizontal crease
(P=FD,C=B,O=V)          Black Fold, vertical crease
```

`FD` is the piece tag. `O=H` or `O=V` mandatory.

A neutral Fold is plausibly interesting: `(P=FD,C=N,O=H)` — a board
feature both sides exploit.

## Open questions

- **Identification vs. mirror.** Two readings of "fold the board":
  identification (the two halves become the same surface, board is
  smaller) and mirror (rays bounce off the crease, board is the same
  size). Identification gives the cooler tactic "rook on a1 attacks
  a7," but creates Klein-bottle ambiguities and self-square issues.
  Mirror is cleaner and ships first. Identification is a future
  variant — call it Identify or Klein.
- **Reflection on the Fold's own square.** Fold occupies `(r₀, f₀)`.
  Does the fold rank exclude that square, or is the Fold itself the
  fold? Suggest: the entire rank `r₀` is the crease; the Fold piece's
  square is the "pivot" of the fold but other squares on rank `r₀`
  also reflect crossing rays. Capturing the Fold removes the entire
  crease.
- **Multiple folds, same rank.** Two H-Folds on the same rank with
  different files — same crease, two redundant pieces. Capturing one
  preserves the crease. Or do they crease at *both* ranks? Choose
  one: same crease, multiple anchors.
- **Folds perpendicular to each other.** An H-Fold at rank 4 and a
  V-Fold at file d. A ray crossing both gets reflected twice — once
  horizontally, once vertically — which is a 180° rotation. Nice
  composition. Confirm with tests.
- **Fold and Pivot interaction.** A piece inside a Pivot zone whose
  ray crosses a Fold: rotate the ray first, then fold-reflect? Or the
  other way? Order matters. Suggest: rotation applies at vector
  emission (per-source), fold-reflection applies at each step. So the
  order is fixed by the resolution order. Document it.
- **Knight + fold.** Knights jump. A knight whose L-shape "crosses"
  a fold — does it land on the reflected square? Cleaner if knights
  ignore folds entirely. Document.
