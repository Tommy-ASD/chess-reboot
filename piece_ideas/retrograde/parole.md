# Parole

> A piece that may only return to a square it has previously occupied
> after an enemy piece has visited that square in between. The set of
> squares currently barred to the Parole is FEN-listed on the piece.

## Inspiration

A retrograde puzzle often hinges on proving that an enemy piece
*must* have visited a particular square at some point during the
game. Direct evidence of such a visit is hard to come by — the
classical solver infers it from pawn-capture geometry, missing-piece
counts, and similar second-order signals. The Parole is direct
evidence. Its bar-list is the set of squares it has touched, and
every entry on the bar-list is *also* a guarantee that no enemy has
visited that square since the Parole left.

## Mechanic

A Parole moves as a knight (canonical; any movement profile is
admissible). It maintains a per-piece **bar-list**: a set of squares
currently barred to it as legal destinations.

The rules of the bar-list:

1. **On move:** the square the Parole *leaves* enters the bar-list.
2. **On enemy visit:** if any enemy piece (of any color other than
   the Parole's own — practically meaning the opposing color, plus
   `Color::Neutral` if relevant) occupies a barred square (whether
   by moving onto it or being placed there), that square *exits*
   the bar-list. The visit "clears" the bar.
3. **On Parole capture:** the bar-list dies with the piece.
4. **Initial state:** when a Parole is placed (at game start or by
   editor), the bar-list is empty.

A move that would land the Parole on a square in its bar-list is
**illegal**. The Parole simply cannot go there.

"Enemy visit" is defined as the enemy piece *occupying* the barred
square at any moment, not merely *passing through* it. A slider's
intermediate path does not clear the bar — the enemy must actually
land on (or be placed on) the barred square at some point during
its existence on that square.

## The deduction it enables

A position has a black Parole on f6 with bar-list `B=[a1, c3, e5]`.

The solver reasons:

1. The Parole has occupied a1, c3, and e5 at some point in the
   game. (These are its move-history "exit squares.")
2. The Parole has *not* had any of those three squares visited by a
   white piece since the Parole left them. If any of them had been
   visited, that square would have exited the bar-list.
3. From observation 2: throughout the rest of the game, no white
   piece has occupied a1, c3, or e5 at any time after the Parole's
   most recent departure from each.

This is a strong constraint. It rules out entire classes of white
piece histories.

Consider a refinement: the white king is currently on b2. The bar-
list contains a1. A white king on b2 *can* attack a1, but the bar-
list proves the white king has not *landed* on a1 since the Parole
left it. If the puzzle's other evidence demonstrates that the white
king walked from e1 to b2 by a king-path, the path cannot have gone
through a1 (or that bar would be cleared). Therefore the white king
must have reached b2 by a path that avoids a1 — say e1→d1→c1→b2 or
e1→d2→c2→b2 — and this constraint may interact with other bar
entries.

The composer may stack the bar-list with squares forming a *wall*:
`B=[c1, c2, c3]` would prove the white king cannot have crossed the
c-file at any point since the Parole left, which combined with the
white king's current position west of the c-file would be an
impossibility — proving the bar-list cannot be `[c1, c2, c3]` if the
white king is currently on a1, *or* proving the white king has not
moved.

## Why it's interesting

The bar-list is the only retrograde piece-state in this set that
encodes a **negative** fact: not "this happened" but "this did
*not* happen." Every barred square is a proof that no enemy visited
since departure.

A negative fact is unusually powerful in retrograde reasoning,
because positive facts (Witness notches, Sediment tokens, Scar
checks) accumulate from things that *did* happen. The bar-list lets
the composer constrain the gaps.

The piece also has an unusual "self-restricting" feel — the longer
it lives, the more squares it bars from itself, until it eventually
runs out of legal moves and is effectively trapped. This is itself
a composition device: a Parole that has barred itself into a corner
proves that the enemy has been *unable* to clear any of its bars.

## Example puzzle setup

```
. . . . . . . k
. . . . . . . .
. . . . . . . .
. . . . . . . .
. . . . P . . .
. . . . . . . .
. . . . . . . .
K . . . . . . .
```

Black Parole `P` on e4 with `B=[a8, h1, d6, f6]`.

Two kings, one Parole, four bars. The puzzle: prove that neither
white king move nor any other white piece (had any existed) has
visited a8, h1, d6, or f6 since the Parole departed.

The white king is on a1. It has not visited a8 (otherwise that bar
would clear). It has not visited h1 directly — well, h1 is a square
the white king could plausibly have walked to from e1. But the bar
proves it didn't. Combined with the black king on h8 (whose recent
location we are *not* given evidence for), we can deduce the white
king's path was constrained to avoid h1.

For the four bars to all stand, the white king's entire path since
e1 must avoid a8, h1, d6, f6 (the first three are trivial; the
fourth, f6, is suggestive — it implies the white king has not
walked through the kingside center).

The bar-list is a *path obstacle* for white's history.

## Where it shines

- **Path-exclusion puzzles.** Prove the enemy cannot have walked
  through this region.
- **Castling-history puzzles.** A bar on f1 or g1 + a white king
  currently on c1 + castling rights pins the castling event to
  queenside.
- **Negative-evidence proofs.** Particularly effective in
  combination with positive-evidence pieces (Witness, Sediment) —
  positive evidence says what happened, the bar-list says what
  didn't.

## Where it's awkward

The bar-list grows during play and can become long. A Parole that
has made many moves with few enemy visits to its old squares will
carry a list of a dozen entries. The FEN remains parseable but
visually dense.

The clearance rule ("enemy visits the barred square") requires the
engine to monitor *every* piece-on-square event for whether it
matches any barred square in any Parole's list. This is an
O(active-Paroles × moves) check at minimum — manageable, but a real
addition to the move-application pipeline.

The "enemy occupies" definition is the right granularity but
permits oddities: if an enemy piece *passes through* a barred
square via a slider move (the square is on the slider's path but
not the destination), the bar is NOT cleared. Composers must
understand this.

## Engine dependencies

- Per-piece list-valued payload system.
- A move-application hook that fires *after* every move resolves and
  scans all opposing Paroles' bar-lists for matches against the
  move's destination square. (Plus a similar hook for editor
  placements on Parole-barred squares.)

## New features required

- **Bar-list payload.** Per-piece, set-of-squares value.
- **Departure hook.** When a Parole moves, add its from-square to
  its bar-list.
- **Arrival hook (opposing).** When any enemy piece lands on a
  square, check each Parole owned by the *other* color; if the
  square is in that Parole's bar-list, remove it.
- **Move-legality filter.** Parole's move generator must filter out
  destinations in its own bar-list.
- **FEN encoding.** Set-valued payload. See below.

## FEN encoding

Piece payload key: `B`. Value: a comma-separated list of square
names. Empty list is encoded as `B=`.

Examples:

```
(P=PA,B=)               — fresh Parole, no bars
(P=PA,B=a1)             — one bar on a1
(P=PA,B=a1,c3,e5)       — three bars
(P=PA,B=a8,h1,d6,f6,F=COLOR:b)  — black Parole, four bars
```

Round-trip rule: emit bars in canonical order (file ascending,
rank ascending) so two boards differing only in bar-list ordering
serialize identically. Duplicate bars in input are deduplicated;
re-emission has no duplicates.

## Open questions

- **Does a square become barred if the Parole *captures* on
  another square and the destination becomes empty?** No — the
  from-square is barred (the square the Parole leaves), regardless
  of whether it captured or moved quietly.
- **What about Color::Neutral pieces visiting a barred square?**
  The engine has `Color::Neutral`; design call. Recommendation:
  treat Neutral as "any color, including enemy from the Parole's
  perspective." A neutral piece visiting clears the bar. This is
  the simpler rule.
- **Pawn promotion landing on a barred square.** Same as any other
  arrival; clears the bar if the pawn was enemy.
- **Editor placement of a piece directly onto a barred square.**
  Treat as an arrival; clears the bar. The bar-list is a running
  fact, not a frozen one.
- **Self-bar interactions across Parole pieces of the same color.**
  Each Parole has its own bar-list. They do not share. Two Paroles
  of the same color can each visit different squares with no cross-
  contamination of bars.
