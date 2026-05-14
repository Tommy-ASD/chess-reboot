# Tether

> Two paired pieces (Tether-A and Tether-B) that, on either's move,
> stamp the current Chebyshev distance between them into their own
> FEN payload. The stamp persists until one of the two moves again,
> at which point both update.

## Inspiration

Smullyan's puzzles often rely on *spatial alibi* — a piece could not
have been at a certain square at a certain time because of where
some other piece was. The Tether reifies this. The stamp on the pair
is a literal snapshot of past geometry: at the moment of the last
move involving either Tether, the two pieces were exactly *d* squares
apart in king-distance.

## Mechanic

A Tether is a pair: one piece named Tether-A, one Tether-B, of the
same color. They move as kings (the canonical choice; movement
profile is orthogonal to the mechanic). The engine maintains the
pairing — exactly one A and one B per color, linked by identity.

Each Tether carries a payload `D=n`: the Chebyshev distance between
A and B at the moment of the most recent move by *either* member of
the pair. Both A and B carry the same `D` value; the pair updates
atomically.

Update rule:
- A or B moves to a new square. After the move resolves, recompute
  the Chebyshev distance between the new pair of positions. Stamp
  `D` on both pieces to that value. The old `D` is overwritten.
- A or B is captured. The surviving member's `D` is *not* updated —
  it retains whatever value it had at the moment of the most recent
  pre-capture move. A captured Tether takes its `D` to the grave.

The starting `D` value when both Tethers are placed is computed at
placement time (board setup).

The Chebyshev distance (max of |dx|, |dy|) is chosen because it
matches king-distance and is the natural notion for a pair of
king-mobile pieces.

## The deduction it enables

A position has white Tether-A on c3 and white Tether-B on f6. Both
carry `D=2`.

The current Chebyshev distance between c3 and f6 is `max(|f-c|,
|6-3|) = max(3, 3) = 3`. The stamp says `D=2`. The stamp does not
match the current geometry. Therefore, **at least one of the
Tethers has moved since the stamp was set, and the stamp was set
*before* the most recent pair-modifying event.**

Wait — the stamp updates on every Tether move. So the stamp should
reflect the most recent move. If it says `D=2` but the current
distance is `3`, the stamp **cannot** reflect the position as
presented. There is a contradiction.

The resolution: the stamp reflects a moment *just after* the most
recent Tether move — but a *non*-Tether move has happened since,
and the *opponent has captured something on a square between the
Tethers* that does not count. No: opposing moves do not affect the
Tether stamp at all, and capture is the only way for a Tether to
disappear.

The only consistent story: **a Tether on the pair has been captured
and then somehow replaced.** This is not a legal in-game event. The
puzzle composer has placed the position as an illegal state.

Or — more usefully — the deduction runs forward:

The composer presents a position where the stamp matches the current
geometry: white Tether-A on a1 and Tether-B on c3, both showing
`D=2`. Chebyshev a1-c3 = 2. Consistent.

Now the puzzle: prove that one of the Tethers has moved since ply
N. The composer pairs this with a Chainwalker showing `C=3` (or
similar). The argument is: if neither Tether had moved since ply
N, the stamp would still reflect the geometry at ply N. The current
stamp reflects the current geometry. Therefore *at least one
Tether move has happened in the interval*.

A subtler use: **proving a Tether has NOT moved.** Position has
Tether-A on a1, Tether-B on h8, both `D=7`. Chebyshev a1-h8 = 7,
consistent. The composer adds: a third piece (some other obstacle)
on d4 that would block any king-path between a1 and h8 other than
along the long diagonal. The solver argues that A and B must each
have moved at least once to reach their current squares (from some
starting setup) — but each Tether move would have re-stamped the
distance, so the stamp must reflect the geometry *after* the last
such move. The current `D=7` therefore proves that the *last*
Tether move ended with the pair on the long diagonal corners. From
this, the composer derives constraints on what the most recent
non-Tether moves of that color must have been (since they cannot
have occurred *during* the diagonal-corner configuration without
contradicting some other piece of evidence).

## Why it's interesting

The Tether's evidence is **non-monotonic**. Unlike a Witness (which
accumulates) or a Chainwalker (which only increases), the Tether's
stamp can go up *or* down. The current value is the *most recent*
snapshot, nothing else. This makes the Tether well-suited to
"prove something happened *at least once* since the snapshot" rather
than "count how many things happened."

The pair is also the only retrograde piece in this set that *needs*
identity-tracking across moves. A and B are individuated. If the
engine doesn't already track piece identity, the Tether forces it.

## Example puzzle setup

```
. . . . . . . k
. . . . . . . .
. . . . . . . .
. . . . . . . .
. . . . . . . .
. . A . . . . .
. . . . . . . .
K . . . . . B .
```

White Tether-A on c3, Tether-B on f1. Both showing `D=4`.

Chebyshev c3-f1 = max(3, 2) = 3. The stamp says `D=4`. The
mismatch is a problem.

Unless: between the last Tether move and now, an *opponent* move
happened (Black's move), and the position presented is between
moves with Black-to-move. The Tether stamp was set on White's last
Tether move and reflected the geometry at that moment. Since then,
no Tether has moved. The opponent has moved exactly once (to bring
the turn to White, the position presented). The Tether's geometry
*did not change* because no Tether moved.

But the stamp says `D=4` and the current Chebyshev is 3 — so the
geometry *did* change. That's impossible unless a Tether moved. So:
**a Tether did move, and the position presented is post-Tether-move
but pre-stamp-update**, which is also impossible.

The only resolution: the puzzle is in an *illegal* state, or the
stamp was set when one of the Tethers was on a different square.
The latter requires that the captured-Tether case has been invoked
and the surviving Tether retains an old `D`. That requires the pair
to be *down to one member*, contradicting the position showing both
A and B. Contradiction.

So the puzzle's answer is "this position is illegal" — a Smullyan
favorite. The Tether forces it directly via stamp/geometry
mismatch.

## Where it shines

- **Alibi puzzles.** Proving a piece was or wasn't at a square at a
  particular time. The stamp pins the pair's geometry to its most
  recent move.
- **Illegality detection.** Stamp/geometry mismatch is direct
  evidence of an illegal state.
- **Tempo-zugzwang composition.** Where the puzzle's mechanism
  requires exactly one Tether move in a precise ply window.

## Where it's awkward

The pair-identity requirement is heavy. The engine must distinguish
A from B even though they're the same color and movement profile.
This is *one* extra bit per piece if the engine doesn't already
track unique identity, and a real architectural choice if it
doesn't.

The "captured Tether takes the stamp to the grave" rule is the
right rule but invites composition errors — a composer who forgets
this designs puzzles that read as illegal when the answer is "a
Tether was captured."

## Engine dependencies

- Per-piece identity tracking (or at minimum, a "pair-link" pointer
  between A and B).
- Per-piece payload system.
- Chebyshev-distance utility (trivial; max of two abs-diffs).

## New features required

- **Pair linkage.** A and B of the same color are linked by an
  identifier. Engine validates exactly one of each per color at
  game start (or two pairs total — one per color — if both sides
  field Tethers).
- **Post-move stamp hook.** After any move resolves, if the moved
  piece is a Tether, recompute Chebyshev distance to its partner
  and overwrite `D` on both pieces.
- **Capture handling.** When a Tether is captured, do not update
  the survivor's `D`.
- **FEN encoding.** Two pieces share a `D` value; encoding both is
  redundant but symmetric. See below.

## FEN encoding

Piece payload keys: `D` (the Chebyshev stamp), `T` (the pair
designator: `A` or `B`).

Examples:

```
(P=TE,T=A,D=4)          — Tether-A, stamp 4
(P=TE,T=B,D=4)          — Tether-B, stamp 4 (must match its partner)
(P=TE,T=A,D=0,F=COLOR:b)  — black Tether-A, stamp 0 (pair on adjacent squares
                           at last move, or one was just placed onto the other's
                           square — in which case capture happened immediately
                           by the rules of the game)
```

Round-trip rule: a FEN with mismatched `D` values across a pair is
*rejected at parse time* as malformed. The pair invariant is part of
the FEN's validity.

A FEN with one Tether and no partner is acceptable (the partner was
captured); the lone Tether's `D` reflects the moment before the
capture.

## Open questions

- **Can a side field zero Tethers?** Yes — the pair is optional.
- **What if both sides field Tethers?** Each color has its own A/B
  pair. Independent stamps. No cross-color interaction. Confirmed.
- **What if a Tether promotes?** It can't — Tethers move as kings,
  don't reach back rank in a special way. Non-issue.
- **A and B occupy the same square?** Cannot happen by legal play
  (they're the same color, friendly-fire is illegal). Could happen
  via editor placement; reject as malformed.
- **The `D=0` edge case.** Two Tethers on adjacent (Chebyshev=1)
  squares give `D=1`. `D=0` requires same-square, which is rejected.
  So `D >= 1` in any legal stamp post-pair-setup, but `D` could
  arguably be `0` immediately after placement before any move. Decision:
  the placement-time stamp is the Chebyshev distance at placement,
  with `D >= 1` always since same-square placement is rejected.
