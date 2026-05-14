# Tessera

> Occupies a 2×2 block. Slides as a block. Pushes whatever's in its path.

## Inspiration

The geometric primitive is **the rigid polyomino**. Sokoban; Stephen's
Sausage Roll's multi-segment sausages; Patrick's Parabox's nested
crates; Into the Breach's tank-pushes; any puzzle game where pushing
chains of objects is the central verb.

The Tessera is the first piece in this set that **occupies more than
one square**. Everything else on this list is a single-square rule
piece; Tessera is a single piece spread across a region. This forces
the engine's representation of "piece at square" to admit a one-to-many
relationship, which is the most consequential structural change in
this directory.

## Mechanic

A Tessera occupies a **2×2 footprint** anchored at its
**south-west square**: if the anchor is `(r, f)`, the four squares
occupied are `(r, f)`, `(r, f+1)`, `(r+1, f)`, `(r+1, f+1)`.

Movement: the Tessera can slide one square in any of the four
cardinal directions (N, S, E, W) per turn, like a 1-step rook.

When the Tessera slides direction `(dr, df)`:

1. The destination 2×2 footprint is `(r+dr, f+df), (r+dr, f+df+1),
   (r+1+dr, f+df), (r+1+dr, f+df+1)`.
2. The **two new leading-edge squares** are checked. (For E motion:
   the two squares `(r, f+2)` and `(r+1, f+2)`. For N: `(r+2, f)` and
   `(r+2, f+1)`. Etc.)
3. If either leading-edge square contains a piece, it is **pushed**
   one square further in the direction of motion. Pushes are
   per-square: each leading-edge square's contents independently
   try to translate by `(dr, df)`.
4. If the pushed square's destination is **also occupied**, the chain
   recurses — push that piece too. Repeat until either:
   - The chain ends with an empty square (everyone shifts cleanly).
   - The chain hits a **wall** (board edge or impassable terrain). In
     that case the **terminal piece is captured** (removed). All
     pieces in the chain still translate one square.
   - The chain hits the Tessera's own footprint (toroidal self-push)
     — illegal, move is rejected.
5. Pieces *inside* the Tessera's leaving footprint that are not on
   the leading edge — i.e. pieces that were riding inside the
   Tessera — **translate with the Tessera** as passengers.

Wait — clarification. The Tessera *occupies* its 2×2 footprint, so
no other piece can be inside the footprint. The "passengers" concept
is for when a Tessera **moves over** a square containing a piece —
but it can't, because the destination must be clear or be pushable.

Re-spec the passenger rule: a Tessera can be **boarded** by friendly
pieces that move into one of its four squares via a normal move — but
the engine forbids two pieces per square. So passengers are out
unless we change the piece-per-square invariant.

**Simpler and shipable: no passengers.** A Tessera is opaque; its
2×2 squares are fully occupied by Tessera; nothing else can be on
them. Pushing is the only interaction. Drop the passenger idea.

(If passengers turn out to be the cool part, revisit with a stacking
mechanic — that's Lens territory.)

**Capturing the Tessera.** The Tessera is captured when **any** of its
four squares is attacked and the opponent makes a capture move
targeting that square. A single capture removes the entire Tessera
(all four squares become empty). The Tessera does not have HP.

## Why it's interesting

It introduces **rigid-body translation with cascading consequences**.
Every Tessera slide is a Sokoban puzzle in microcosm. Combined with
the rest of the board's pieces, a Tessera can be pushed into a
position where its 2×2 footprint covers a critical square set — for
example, both squares the opposing king might flee to.

Capturing the Tessera by any one square gives interesting tactical
asymmetry: the defender wants to protect a 4-square perimeter, while
the attacker only needs one entry point.

## Example scenarios

**Sokoban-style push, 5×5:**

```
. . . . .
. T T . .       T T = Tessera footprint (rows shown SW-anchored)
. T T . .                                 r+1 row
. . p . k       p = enemy pawn, k = enemy king on e1
. . . . .
```

(Top is high rank. Tessera anchor at b3, so footprint b3-c3-b4-c4.)

Tessera slides E one square. New anchor c3, footprint c3-d3-c4-d4.
Leading edge: d3, d4. d3 is the pawn's square — pawn is pushed E one
square to e3. d4 is empty. Pawn at e3 ends turn. King unaffected.

If the same move is repeated three more times, the pawn is pushed
e3→f3→g3→h3 (or its equivalent — bounded by board edges in this
example). If a wall comes before that, the pawn is captured.

**Chain push:**

```
. . . . .
. T T p p
. T T . .
. . . . .
. . . . .
```

Tessera slides E. Leading edge: pawns at d (which becomes the next
square). Wait — Tessera's leading edge after slide is the new
rightmost column. From this position, slide E means new footprint
shifts right by one; new leading edge is column d+1. The pawn on d
gets pushed into e; pawn on e is also there, so e also gets pushed
into f. Chain.

**Wall capture:**

```
. . . . .
. T T p|        | = board edge
. T T .|
. . . .|
. . . .|
```

Tessera slides E. Pawn at column 4 pushed into column 5 — off board.
Wall! Pawn is **captured**. Tessera moves to its new footprint;
column-4 squares are now empty.

This is the **Sokoban-trains-of-shoves** primitive promised in the
brief.

## Where it shines

- **Forced position rearrangement.** Tessera pushes are deterministic;
  composers can craft "push three pawns off the board" puzzles.
- **Material removal.** Capturing pieces by pushing them off-board is
  qualitatively different from capturing-by-attack — pieces blocked
  by their own teammates can be captured without a normal attack.
- **Territorial control.** A 2×2 piece is bulky. It occupies real
  space. Positioning matters.

## Where it's awkward

- **Engine representation.** Every "piece at square" query must
  handle the multi-square case. The Tessera lives at multiple
  coordinates simultaneously. Refactoring required.
- **Move generation cost.** Computing legal slides requires checking
  the leading edge and recursively the chain. Bounded by board size,
  so cheap, but the resolution logic is non-trivial.
- **Pinning / discovery.** Does the Tessera "discover" attacks
  through its body? Probably yes — it's opaque to sliders the same
  way any piece is. Each of its four squares individually blocks
  rays.
- **Promotion.** A Tessera doesn't promote. A pawn pushed onto its
  back rank by a Tessera — does it promote? Yes; pushing is a "move
  for purposes of promotion" event. Document.
- **Castling-through.** If the Tessera occupies a king's castling
  destination square (or any intermediate), castling is illegal.
  Mirror of the normal blocker rule.

## Engine dependencies

- **Variable board dimensions** — present.
- **`is_walkable()`** — present, used at the leading-edge check.

## New features required

- **Multi-square pieces.** The `Square::piece` invariant is one-piece-
  per-square. The Tessera breaks this *interpretation* by claiming
  four squares. Two implementation options:
  1. **Replicated piece-ID.** Each of the four squares has a
     `PieceId` field pointing at the same Tessera entity. The
     Tessera's full state lives in a side table keyed by ID.
  2. **Anchor + sentinel.** The anchor square holds the Tessera; the
     other three squares hold a `Sentinel { tessera_id }` that
     blocks moves but routes interaction back to the anchor.
  Recommend option 1 — cleaner, no sentinel concept needed.
- **Multi-square move generator.** New trait or generator interface.
  Generates one move per cardinal direction; each move records the
  push chain it triggers.
- **Push-chain resolver.** Standalone function: given a piece, a
  direction, and a board, produce the ordered list of pieces that
  translate and which (if any) are captured. Used at apply time.
- **Capture-on-edge primitive.** Pushing a piece off the board
  captures it. Engine probably doesn't have "off-board" as a
  capture trigger today.

## FEN encoding

The Tessera is one logical piece. Encode it at its anchor square;
the other three squares get a sentinel marker pointing back.

```
(P=TS,C=W)              White Tessera anchor (SW corner of 2×2)
(P=ts,C=W)              Sentinel — engine fills the other 3 squares
```

Lowercase `ts` for sentinel, uppercase `TS` for the real piece (mirror
of pawn convention). On FEN emission, only the anchor is decorated;
sentinels are rendered implicitly. On FEN parse, the engine sees the
anchor and writes the sentinels into the three adjacent squares.

Alternative single-anchor encoding: write only the anchor; the parser
fills the other squares. Less robust but more compact.

```
(P=TS,C=W,SZ=2x2)       Future-proof: explicit footprint size
```

Future Tessera variants (3×3, L-shaped polyominos) would extend this.

## Open questions

- **Diagonal motion.** Spec says cardinal only. Should the Tessera
  slide diagonally? Geometrically the leading edge becomes 3 squares
  (the diagonal of a 2×2). Possibly add as a variant; ship cardinal-
  only for v1.
- **Pushing into a Pivot zone.** A pawn pushed into a Pivot zone is
  geometrically inside the zone but isn't "moving" in the sense
  Pivot cares about. Probably fine — Pivot rotates outgoing rays,
  doesn't intercept incoming pushes.
- **Pushing into a Fold's crease.** Push direction reflects? A pawn
  pushed north into a H-Fold rank — does the pawn reflect southward?
  Suggest: pushes don't fold; they're not rays. Fold only acts on
  slider rays. Pushed pawn lands on the crease square (if empty) or
  is captured against the crease.
- **Recursion adjacency.** If the Tessera ends its move adjacent to
  a Recursion (see [recursion.md](recursion.md)) — does the Tessera
  get a free second move? Yes, by the Recursion rule. But the
  Tessera's adjacency footprint is 12 squares (perimeter of 2×2 + 1),
  not 8. Adjust the Recursion check to use any-of-piece's-squares-
  adjacent.
- **Two Tesserae adjacent.** Can two Tesserae touch? Yes. Can one
  push the other? **Yes** — by symmetry with pawn-pushing. A 2×2
  pushing a 2×2 makes a 4-square chain. Strategically wild.
- **Captures of individual squares.** Can a knight capture one square
  of the Tessera, leaving an L-shaped 3-square remnant? **No** — any
  capture destroys the whole Tessera. Cleaner spec, and matches the
  "single rigid body" intuition.
