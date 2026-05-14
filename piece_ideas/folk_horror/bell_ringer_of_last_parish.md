# The Bell-Ringer of Last Parish

> "One. Two. Three. Four. Five. Six. Seven. Eight."

## Character

The parish has been gone for some time. The houses fell first, then
the orchard, then the road that led there, and finally the name. What
remains is the bell-tower, which was built well, and the man inside
it, who was built less well but who has not yet stopped. He is older
than anyone who could remember him. He has been counting since before
the counting meant anything.

Every eighth turn he tolls. The bell sounds across the board and the
pieces nearby — for three squares in every direction — must answer.
They must move on their next turn, every one of them, because the bell
has been rung and the bell is older than the rules of the game. Pieces
that cannot move are removed; the bell does not wait for those who
cannot answer.

He himself does not move. He has not moved in a long time. He does not
know who is winning. He does not know which side he is on, and neither
does anyone else; the records were lost with the parish. He is
counting because counting is what bell-ringers do, and he is, despite
everything, still that.

## Mechanic

**Placement.** Once at setup, on any square. Once placed, he does not
move and cannot be commanded. He occupies a single square and counts
as a piece for board-state purposes (he can be captured).

**Colour.** He is `Color::Neutral`. He belongs to no side. He does
not threaten captures; he does not block them. His square is occupied
by a non-aligned piece, which other pieces may capture by normal
rules (Neutral pieces can be captured by any side).

**Movement.** None ever.

**Capture.** Normal — any piece that can reach his square captures
him. When captured, he is gone and the bell never rings again.

**The toll.** The engine maintains a *toll counter*, incrementing by
one per *full turn* (both sides moved). When the counter reaches a
multiple of 8 (i.e. counter % 8 == 0 and counter > 0), the bell tolls.

When the bell tolls:
1. The engine identifies every piece (any side, including Neutral)
   within a *Chebyshev distance of 3* from the Bell-Ringer's square.
   This is the set of *called-by-bell* pieces. The Bell-Ringer
   himself is excluded (he is not called by his own bell).
2. Each called-by-bell piece is *bell-marked* for exactly the next
   one turn of their own side.
3. On a bell-marked piece's side's next turn, the player *must* move
   that piece, if it has any legal move. If the side has multiple
   bell-marked pieces, the player must move *all of them*, one per
   turn, in any order they choose — but each must move before any
   non-bell-marked piece of that side moves. (See Open questions for
   the simpler "one bell-marked move per turn" alternative.)
4. If a bell-marked piece has *no legal move*, it is *removed* at
   the start of its side's turn, before any move is made.
5. Once a bell-marked piece has moved, its mark clears.

**Multiple Bell-Ringers.** Permitted. Each carries its own toll
counter (or rather, the global counter modulo 8 fires *all* of them
on the same turn — this is the simpler interpretation). Their
3-radius circles may overlap; pieces in the overlap are marked once
(the mark is binary, not stacking).

**Counter persistence.** The toll counter is global to the game and
ticks regardless of which Bell-Ringer fired it. If the Bell-Ringer is
captured before turn 8, the bell never rings — there is no one to
ring it. If a second Bell-Ringer is placed mid-game (some fairy
mechanics may spawn pieces), the toll counter continues from its
current value.

**State.** A single global integer: `bell_toll_counter`. Pieces
marked-by-bell carry a transient flag in their FEN payload, cleared
on their next move.

## Why it's interesting

The Bell-Ringer is a *clock*. He turns the otherwise-unbounded chess
game into a sequence of crises spaced eight turns apart. Players will
plan around the toll — clustering pieces away from him before the
toll, or *into* him before the toll if they have a reason (perhaps
to force an opponent's piece to move). The 3-square radius makes him
a *zone-of-disruption* whose footprint is roughly the size of a small
army.

Because he is Neutral, *both* sides' pieces are affected. This makes
him a shared problem — the player who placed him near their own army
suffers as much as the opponent. He punishes the *board*, not a side.

The "must move or be removed" rule is brutal but rarely-firing — only
once every eight turns. The pacing is deliberate. The player feels
the dread accumulate across the seven quiet turns between tolls.

## Example scenarios

1. **The first toll.** Turn 8. Bell-Ringer on e4. Pieces within
   3-radius: white queen on d3, black knight on f5, black pawn on
   e6, white rook on b1 (Chebyshev to e4 is 3 — counts). Bell tolls.
   All four are bell-marked.
   - On white's next turn, white must move the queen and rook.
     White moves the queen first to a legal square; the rook second.
     Both marks clear.
   - On black's next turn, black must move the knight and pawn.
     Same.

2. **The trapped pawn.** Turn 16. Bell-Ringer on h1. White pawn on
   h2, blocked by white piece on h3. Pawn has no diagonal capture
   available. Pawn has no legal move. Toll fires; pawn is bell-marked;
   start of white's next turn, pawn is removed. White curses the
   bell.

3. **The sacrificial cluster.** Black has been forced to congregate
   near the Bell-Ringer to defend a key square. Turn 24 is in two
   turns. Black knows that on turn 24, *all* nearby pieces will be
   forced to move, and most of them have only one legal move each,
   and several of those legal moves walk into white's attacks. The
   toll fires; black loses three pieces in the cascading forced
   moves. The Bell-Ringer counts on, indifferent.

## Where it shines

- Long strategic games where the eight-turn cadence creates rhythm.
- Cluttered midgames where forced movement creates cascades.
- Compositions: puzzles designed around a specific toll-turn.
- Asymmetric setups where one side has more pieces near him.

## Where it's awkward

- Open positions with few pieces near him; the toll fires and
  nothing happens.
- Short games. If the game ends before turn 8, he never tolls.
- Forced-move rules cascade with check resolution in complex ways;
  see Open questions.

## Engine dependencies

- Global turn counter (existing).
- Chebyshev-distance computation (trivial).
- `Color::Neutral` (existing).
- Forced-move modifier (probably shared with the Miller's
  fifth-turn rule and the Wife's call rule).
- End-of-turn hook for the toll.

## New features required

- **Global toll counter** in board state. FEN-serializable as a
  single integer.
- **Forced-move marking.** Pieces carrying a `bell_mark` flag in
  their FEN payload. Cleared at move-apply when the marked piece
  moves; held across turns otherwise.
- **Multi-piece forced-move ordering.** When a side has multiple
  bell-marked pieces, all must move before any non-marked piece.
  This is a *move-set restriction*, not a sequence-forcing — at each
  step of the side's turn, only marked pieces are legal movers, until
  all marks are clear; thereafter the side may move any piece.
  Note: this requires multiple sub-turns within a "turn," which is
  unusual. The simpler version: only ONE bell-marked piece must move
  per turn, and the marks expire only when each piece individually
  acts. See Open questions.
- **Remove-on-no-legal-move.** Pieces with `bell_mark` and no legal
  move are removed at the start of their side's turn.
- **Neutral piece interaction.** He is `Color::Neutral`. Already
  supported (per the engine context).

## FEN encoding

The Bell-Ringer:
```
(P=B,T=RINGER,C=N)    # P=B for Bell, T=RINGER, C=N for Neutral
```

The global toll counter, on the board's state line:
```
... w KQkq - 0 1 [TOLL=12]
```

Pieces with active bell-marks:
```
(P=N,M=1)             # bell-marked
```

## Open questions

- **One-marked-per-turn versus all-marked.** Spec above leans toward
  *all marked pieces must move before any non-marked piece* on the
  same turn. This is unusual chess pacing; chess turns are
  one-move-per-side. The alternative is: only *one* bell-marked piece
  must move per turn, and the mark persists across turns until that
  piece moves. This is simpler and more chess-like. Recommend the
  simpler version; it matches the bell-ringer's slow patience.
- **Bell-mark on the king.** The king is bell-marked. The king must
  move. If the king has no legal move (and is not in check), the
  king is *removed* — game over. This is harsh but in character.
  Alternative: the king is exempt. Recommend exempt; the king is
  exempt from most folk-horror removals.
- **Bell-mark on a pinned piece.** The piece is pinned and the
  bell-mark says it must move. Pin restrictions still apply. If the
  pin makes movement illegal, the piece is removed. This is the
  intended *zugzwang* of the bell.
- **Bell-mark on a Bell-Ringer.** A second Bell-Ringer of any colour
  is in the radius of the first. The toll fires; the second is
  bell-marked. But the second never moves. Recommend: the
  Bell-Ringer is exempt from his own and other tolls — he is
  motionless by character. Mark fires, removal triggers next turn,
  he is removed. *Bells consume bell-ringers.* This is fine
  actually. Pick the dramatic version.
- **Toll on turn 0.** The counter increments per *full turn*. Turn
  0 is setup. The first toll fires when the counter ticks to 8 — i.e.
  after 8 full turns of play. Confirm.
- **Counter reset on Bell-Ringer death.** If the Bell-Ringer is
  captured and then re-spawned (via some fairy mechanic), does the
  counter reset? Recommend no — the counter is global and persistent.
  The bell tolls when it tolls; whether anyone is there to ring it
  is incidental. If no Bell-Ringer is on the board on a toll turn,
  the toll fires but has no effect (no centre, no radius). The
  counter still ticks.
- **Multiple Bell-Ringers and overlap.** Two Bell-Ringers within 6
  squares: their radii overlap. A piece in the overlap is marked
  once. Multiple Bell-Ringers do *not* multiply the toll's
  frequency — the toll counter is global and fires for all of them
  simultaneously.
- **Bell-mark and check.** A bell-marked king's side is in check on
  the next turn. The king must resolve check *and* the bell-mark.
  If the king has a check-resolving move, that move clears the
  bell-mark naturally. If the king has *no* legal move (checkmate),
  the game ends by checkmate before the bell-mark removal fires.
  Order: check-resolution takes precedence; the bell-mark is
  secondary.
