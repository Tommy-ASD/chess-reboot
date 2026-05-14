# The Boy Who Followed the Geese

> "The geese went and so did I. I do not remember the way home."

## Character

There were geese. They flew low over the village one autumn morning
and the boy, who was perhaps six, perhaps seven, followed them out of
the garden and through the orchard and across the river and into the
forest, and by evening the geese had stopped being geese and the
forest had stopped being a forest and the boy had stopped being a boy
who knew where his mother was.

He walks now. He walks toward whatever moves. He cannot help it. When
something moves on the board he turns his head and his feet follow,
and he takes one step in the direction of the thing. If the thing is
near, he reaches it, and when he reaches it he *becomes* it, because
that is what happens to boys who follow things long enough.

He has no will. He has no memory. He is sweet about it. The pieces of
his own side avoid making sudden movements.

## Mechanic

**Movement.** Zero squares per turn *by player command* — the player
does not move him. He moves *automatically* at the end of every
turn (either side's), exactly one square, along the shortest path
toward whichever enemy piece moved most recently.

**The target.** At the end of any turn, the *target piece* is the
enemy piece that *moved this turn* (or the most recent enemy piece to
have moved, if neither enemy moved this turn). If no enemy piece has
ever moved (he is on the board but the enemy has been entirely
passive — unusual), he does not move.

If multiple enemy pieces moved this turn (rare — only possible with
multi-piece move mechanics like castling or signal-cascades), the
target is the *primary* mover of the move, where primary is defined
by the move-application order in that mechanic.

**The step.** He moves one square in the eight-direction king-pattern
toward the target's *current* square. The path is computed as: among
his eight neighbouring squares, choose the one that *minimises
Chebyshev distance* to the target. Ties broken by:
1. Lowest absolute change to the file (prefers vertical movement when
   tied).
2. Then lowest absolute change to the rank.
3. Then alphabetical: N, NE, E, SE, S, SW, W, NW.

If the chosen square is *occupied* by:
- His own side's piece: he does not move this turn. The piece blocks
  him. (He is not aggressive enough to push past family.)
- An enemy piece *that is not the target*: he does not move. He is
  only following the target.
- A non-walkable square (Block, closed Gate, etc.): he does not move.
- The target: he *enters* the target's square (see below).

**The arrival.** If his step would land him on the target's square,
he reaches the target. The target piece is *replaced* by the Boy: the
square now holds a piece of *his side* but of the *target's type*. The
original target piece is removed. The Boy ceases to exist as a Boy;
the piece on his old square is gone (he was there); the piece on the
target's square is a member of his side with the target's identity.

Example: Boy on d4 (white), black knight on d5. End of turn, the Boy
steps to d5. The black knight is removed. There is now a *white
knight* on d5. The white knight has no special state — it is a
normal white knight. It carries no placement-turn-trickery (unless the
engine globally tracks `placement_turn`; see Judge), and it is
otherwise indistinguishable from any other knight.

The Boy is, at this point, gone. He has become.

**Capture of the Boy.** The Boy himself is capturable by normal rules
— he occupies a square and can be taken like any other piece. He has
no defence.

**State.** None beyond position and colour. His behaviour is fully
derived from the current board state plus the *last enemy mover*,
which must be tracked.

## Why it's interesting

The Boy is a *reactive* piece — every enemy move pulls him one step
closer to that piece. This creates a tactical layer where every move
must be weighed against the question: *do I want the Boy walking
toward this piece?* If the Boy is far away, the answer is "I don't
care." If the Boy is two squares away, the answer changes the move.

He is also an *opportunity*: a slow enemy piece (a Bargainer, a
Bell-Ringer, a hollowed-pawn) can be transformed into a Boy-version of
itself, recolouring the piece and effectively converting an enemy
asset into a friendly one. This is rare in chess design — most
"conversion" mechanics are explicit one-off moves; the Boy converts
through patience.

The piece teaches players that *movement is dangerous*. Even moving a
piece you don't care about can summon the Boy.

## Example scenarios

1. **The slow chase.** White's Boy on e1, black's queen on h8. Black
   moves the queen to a8. Boy steps to e2 (one square toward a8 via
   d2 — wait, e2 is closer? Compute: Chebyshev from e2 to a8 is
   max(4, 6) = 6; from d2 to a8 is max(3, 6) = 6. Tied. Tiebreak by
   file change: d2 reduces file by 1, e2 reduces by 0. Lowest file
   change is e2 — but the spec says *lowest absolute change to file
   prefers vertical movement when tied* which means we prefer the
   one with *zero* file change. So Boy moves to e2.) The chase
   continues. Twelve turns later, the Boy reaches a8 — now empty
   because the queen has moved again. The Boy stands on a8 and looks
   around.

2. **The reluctant move.** Black's pawn on a2, one step from
   promotion. Black does not want to move it because the Boy is on
   b2 and would step to a2 and replace the pawn, demoting it. Black
   moves a different piece. The Boy steps toward *that* piece
   instead. The pawn lives. The Boy walks elsewhere.

3. **Conversion of a piece.** White's Boy on f7, black's bishop on
   g8. Black moves the bishop nowhere (it's pinned, or whatever). End
   of turn, the Boy steps to g8. The black bishop is removed; a white
   bishop appears on g8. White has gained a bishop for one Boy.

## Where it shines

- Tense positions where every move matters and the Boy adds a
  reactive layer.
- Long games — he eventually reaches something.
- Puzzles built around forcing the Boy to convert a specific piece.

## Where it's awkward

- The "last enemy mover" tracking adds engine state.
- Multi-piece moves (castling, signals) require disambiguation.
- A Boy with no path (surrounded by his own pieces) sits idle for
  many turns; this is fine in character but visually inert.
- The conversion is *strong* — losing a queen to a Boy is brutal —
  and the Boy is not very dangerous in any other way. Balance is
  one-shot-spike.

## Engine dependencies

- King-pattern adjacency (existing).
- Chebyshev distance (trivial).
- Last-enemy-mover tracking (new engine state).
- End-of-turn hook for the Boy's automatic step.
- Piece-replacement at apply time (the Boy *becomes* the target).

## New features required

- **Last-enemy-mover tracking.** The engine must track, per side,
  the most recent enemy piece to move (square + identity). Updated at
  the end of every move-apply. FEN-serializable: a single field on
  the side-to-move state, like en-passant square is today.
- **Auto-move pieces.** A category of piece that is *not* commanded
  by the player but moves automatically at end-of-turn. Likely
  generalises (the Drowned Miller's Wife's "called" pieces also
  involve automatic movement on the next turn).
- **Piece-conversion at apply.** When the Boy steps onto the target,
  the target's *type* (piece-class) is read, and a new piece of the
  Boy's side and the target's type is placed on that square. The
  Boy's original square is cleared. This is a structurally unusual
  move (one piece *replaces* another and changes type), and the
  apply pipeline needs an arm for it.

## FEN encoding

The Boy:
```
(P=Y,T=BOY)        # P=Y, T=BOY — to disambiguate from other Y-pieces
```

The last-enemy-mover state at the board level:
```
... w KQkq - 0 1 [LASTENEMY=e4,N]
```

Where `LASTENEMY=<square>,<piece-letter>` carries the square and
piece-class of the most recent enemy mover. (Format-bikeshed in Open
questions.)

## Open questions

- **Boy on the board but target unmoved.** If the Boy is placed
  during setup and no enemy has yet moved, he sits still. Recommend:
  yes, exactly. He waits.
- **Multiple Boys.** Each tracks the last enemy mover from their own
  side's perspective. Multiple Boys of the same colour all step toward
  the same target. Multiple Boys of opposite colour step toward
  different targets (each side's enemy).
- **The Boy stepping onto the target's old square.** If the target
  moves out of his path, the Boy continues toward the target's *new*
  square, not the old one. Confirmed by the spec — the path is
  recomputed every turn against the *current* target square.
- **Auto-move ordering with other auto-move pieces.** Whose
  automatic step fires first if multiple auto-move pieces (Boy +
  Wife's called pieces, e.g.) move at end-of-turn? Recommend: process
  all auto-moves simultaneously based on the *state before* any of
  them fire; ties broken by FEN-position order.
- **Multi-piece moves and the "primary" mover.** Castling moves two
  pieces — the spec says the king is the primary. Signal cascades
  may move several pieces at once — recommend the *first* in
  apply-order is the primary.
- **The Boy converting a king.** Impossible — the king cannot be
  the target of a Boy's step? Or can it? If the king moves and the
  Boy is adjacent, does the king get replaced? Recommend: yes —
  but this is effectively checkmate for the king's side, and is
  thematically appropriate (the boy reached the king; the boy is now
  the king). Treat as game-ending. (Alternative: the king is
  exempt from the conversion, mirroring other piece protections.
  This is more conservative. Pick the conservative version unless
  playtests favour the dramatic one.)
- **Promoted pawns and conversion.** A pawn converted by the Boy
  becomes a pawn on the Boy's side. Standard pawn rules apply
  (including future promotion).
