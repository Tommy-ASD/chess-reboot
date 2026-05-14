# Witness

> A pawn-like piece that scratches a directional notch onto every
> square it leaves; the notches are permanent and the path is
> reconstructable from the board alone.

## Inspiration

Smullyan's puzzles repeatedly hinge on knowing *which pawn came from
which file*. Pawn structure is the only piece of in-game evidence
about historical motion that classical chess preserves — and even
that evidence is fragile (captures erase the file record). The
Witness extends pawn-structure logic to a *complete* path record. The
piece leaves footprints. The solver reads them.

## Mechanic

A Witness moves like a pawn (forward one, capture diagonal, no
en-passant). It has no double-step and does not promote.

When a Witness *leaves* a square, that square acquires a **notch**: a
directional mark recording which of the eight rays the Witness
departed along. The notch belongs to the square, not the piece. The
piece does not carry it forward.

Notches are permanent. They stack: a square can hold multiple notches
(one per Witness departure that ever happened there). They are
FEN-encoded as a square-level payload, not a piece payload.

A notch reset is impossible by gameplay. Only an editor stroke removes
one.

A Witness terminating its life on a capture square leaves no
*outgoing* notch (it did not leave). A Witness capturing on a square
does, however, deposit its arrival into the previous square's outgoing
notch — the trail ends precisely at the kill.

## The deduction it enables

Consider a 4x4 corner of a board after some sequence of moves. The
solver sees:

```
. . . W
. . N E
. N . .
. W . .
```

A Witness `W` sits on d4. Another stationary Witness `W` sits on a1.
Notches `N` mark b2 (pointing NE) and c3 (pointing NE).

The solver reasons:

1. The d4 Witness arrived at d4. The square c3 has an NE notch — a
   Witness departed c3 toward d4. That Witness is the one on d4.
2. So before its last move, the d4 Witness stood on c3. But Witnesses
   move like pawns; reaching d4 from c3 is a diagonal step, meaning
   the d4 Witness *captured* something on d4.
3. The c3 notch must therefore precede a capture. There is no piece
   currently on d4 other than W itself, so whatever was captured is
   gone — but the c3 notch *proves* the capture happened, and pins
   the ply on which it happened to be the last move of the side
   owning W.
4. The b2 notch (also NE) traces back one more step: a Witness
   departed b2 toward c3. The only Witness that could have done this
   is the same one. Working backward through b2 → c3 → d4, the
   solver reconstructs the last three plies of W's life: a quiet step,
   then a capture, in exactly that order.
5. The Witness on a1 has *no* outgoing notch on any square. So a1 has
   not moved this game.

The five facts together establish: (a) something died on d4, (b) it
died on the most recent W-side ply, (c) it died to a piece that began
the game-tail subsequence on b2, (d) the a1 Witness is irrelevant to
this deduction and stayed home. None of these conclusions are
available in classical chess without external annotation.

## Why it's interesting

The notch is the simplest possible record: one bit per ray per square
(plus a count if multiple Witness departures stack). And yet the
record is enough to reconstruct an entire Witness *trajectory*,
including the precise square of every capture along the way. The
piece converts the implicit history of pawn motion into a literal,
readable trail.

A Witness whose path crosses an enemy piece's path produces a
*conflicting* notch profile that the solver must disentangle. The
genre includes "two Witnesses, one trail" puzzles where the solver
must decide which notch belongs to which piece, on grounds of timing
and reachability alone.

## Example puzzle setup

```
. . . . W .
. . . n . .
. n . . . .
. . . . . .
. . . . . .
W . . . . .
```

Two white Witnesses, two square-notches (both NE). The puzzle
question: prove that the e6 Witness has moved at least twice this
game and the a1 Witness has not moved at all.

Solution: b3 and d5 carry NE notches. The Witness on e6 is reachable
backward via d5 → e6, and d5 is reachable backward via c4 (uncached)
or b3 → c4 (one notch upstream). The chain of notches uniquely fits
the e6 Witness. The a1 Witness has no associated outgoing notch.
Therefore e6 moved at least twice (the two notches), a1 moved zero
times.

## Where it shines

- **Path-of-the-pawn puzzles.** Classical retrograde puzzles asking
  *which file did this pawn start on*. The Witness answers the
  question on its face.
- **Capture-reconstruction puzzles.** Where a notch terminates at an
  empty square, *something* died there. The Witness pins down both
  the square and the ply.
- **Two-trail puzzles.** Two Witnesses, overlapping notches, ambiguity
  resolved only by reachability + capture-square constraints.

## Where it's awkward

Notches accumulate forever. A long game with several Witnesses leaves
a board cluttered with directional marks. For *composition*, this is
fine — puzzles start from a curated position, not a long history.
For analysis of an actual game, the visual density is a real cost.
The intended use is puzzle composition only.

A subtler awkwardness: the Witness's pawn-like movement is the right
*restriction* (it makes the trail simple and forward), but the
inability to promote is jarring. A Witness reaching the back rank
just sits there. We accept this — the piece is a forensic instrument,
not a candidate for promotion mechanics.

## Engine dependencies

- Existing square-payload system (the FEN already supports
  parenthesized per-square payloads — see plan 12's `(T=BLOCK)`).
- Existing per-color piece registry.

## New features required

- **Square-level notch tracking.** A new payload on the square data
  structure: a multiset of (ray, color) pairs, where ray is one of
  the eight directions and color is the owner of the Witness that
  departed. Order of insertion is not preserved (a multiset is
  enough).
- **Notch hook in move application.** When a Witness move is applied,
  the from-square's notch set gains the appropriate (ray, color) on
  the way out. No symmetric hook on the destination square (notches
  mark *departures*, not arrivals).
- **FEN encoding.** New square-payload tag, see below.
- **Editor support.** A brush for placing/removing notches by hand.
  Required for puzzle composition.

## FEN encoding

Square payload key: `NOTCH`. Value: a comma-separated list of
`<ray><color>` pairs. Rays use the standard 8-direction shorthand
(`N`, `NE`, `E`, `SE`, `S`, `SW`, `W`, `NW`). Color is `w` or `b`.

Examples:

```
(NOTCH=NEw)              — one white-Witness NE departure from this square
(NOTCH=NEw,NEw)          — two white departures, same ray (one square left twice)
(NOTCH=Nw,SEb)           — one white northward, one black southeast
```

The piece itself encodes as `W` (white Witness) / `w` (black Witness)
with no piece-payload — all of the evidence is on the squares.

Round-trip: emit notches in canonical order (ray, then color) so that
two boards differing only in encoding order serialize identically.

## Open questions

- **Does a Witness capture mark the *arrival* square at all?** Current
  design says no — notches are departure-only. But an arrival-mark
  on capture would resolve some otherwise-ambiguous "where did it
  die" cases. The cost is a second payload form on the square. Lean
  toward keeping it simple (departures only); the puzzle composer
  can place the kill marker by hand if needed.
- **Should the engine ever auto-fade old notches?** No — that breaks
  determinism and would require a hidden ply counter on each notch.
  Notches are permanent until the editor removes them.
- **Interaction with `Block` squares.** A Witness can never land on
  one, but could it *depart* from one (if hand-placed)? Decision:
  no — placement-time validation rejects a Witness on a non-walkable
  square, same as a Goblin would be rejected.
