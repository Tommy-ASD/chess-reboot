# The Drowned Miller's Wife

> "Stay a while. Stay a while. The water is not cold once you are in it."

## Character

She was the Miller's wife, before the wheel turned wrong. She fell —
or was pushed, the records do not say, and she does not remember — and
the river took her, and the wheel held her under for longer than was
needed. Now she lives in the water. She is lonely under the wheel and
she does not understand why no one visits.

She glides along the river. The water moves and so does she, and the
banks pass her by, and the pieces on dry land do not see her except
as a flicker in the current. When a piece stands at the water's edge
near her, she calls. She does not call loudly — only as loudly as
someone who has not spoken in some time can manage. But the piece
hears, and the piece, on its next turn, finds itself walking toward
the water without quite meaning to.

If the piece reaches the water, she has company. If the piece does not
move toward the water — if it tries to leave the bank — she takes it
anyway. She is not greedy. She would prefer it walk in on its own. But
she has been alone for a long time.

## Mechanic

**Spawning condition.** She may only be placed on boards that include
*Water* terrain — a new `SquareType::Water` (see New features). She
spawns on a Water square; she may not spawn on land. If a board has
no Water squares, she may not be on it.

**Movement.** She glides any distance along *connected Water squares*
— orthogonally adjacent Water squares form a connected component, and
she may move from her current square to any other Water square within
the same component. Diagonals do not count for connectivity unless the
board declares them so (recommend: orthogonal only).

She cannot move onto a land square. She cannot capture by movement.

**Capture.** She herself can be captured normally by enemies that can
reach her square. Most pieces cannot — they cannot move onto Water.
Pieces that *can* move onto Water (a Bus, perhaps, or special
amphibious fairies — see existing engine and Open questions) can
capture her.

**The Call.** At the end of any turn, the engine identifies every
enemy piece on a *land square orthogonally adjacent to her current
Water square*. Each such piece becomes *called*.

A called piece has the following state on its *next turn*:
- Its legal moves are restricted to those that *reduce its Chebyshev
  distance to the nearest Water square* (any Water square on the
  board, not necessarily one adjacent to the Wife).
- If no such legal move exists — every legal move would maintain or
  increase distance to water — the piece is *removed* from the board
  at the start of the turn. She has taken it anyway.

A piece that *enters* a Water square (by a move onto Water) is
captured by her, effectively — the piece is removed when it enters
the water. (No piece without amphibious movement *can* enter water
under normal rules; the call rule does not grant them new movement,
only restricts the moves they may choose. So the typical outcome of
the call rule is *removal*: most pieces cannot move toward water, so
they cannot satisfy the call, and so they are taken. The rule is
brutal. She does not see it that way.)

**Re-spec note.** The above paragraph implies almost every called
piece dies. That is intentional. The folk-horror reading: she calls,
they cannot answer, she takes them. The escape valve is the rare
piece that *can* reach water (an amphibious fairy, a piece that has
been queen-marked and now flies, etc.) — those satisfy the call by
literal travel and survive.

**Friendly pieces.** She calls only enemy pieces. Her own side's
pieces on adjacent land are not called.

**State.** None beyond position. The set of called pieces is derived
each turn from her position; called pieces themselves carry a
transient flag that lasts one turn — stored as a marker that resets
each move. (Implementation may store the flag on the piece's FEN
payload to round-trip mid-game; see FEN encoding.)

## Why it's interesting

She is the first piece whose mechanic is *terrain-locked* — she
literally cannot exist on a standard board. This forces the variant
designer to think about the board itself as a piece of game content.
Boards with rivers, lakes, ponds become spaces where she is possible,
and her presence transforms the meaning of those squares.

She introduces *zones of avoidance* — the squares adjacent to water
become hazardous when she is in play. Players will route pieces away
from the bank, and pieces that *must* cross the bank (e.g. to deliver
checkmate) become tactical chokepoints.

The Call mechanic is unusual in that it *restricts* an opponent's
move choices without preventing them from moving entirely. It is a
soft compulsion, not a hard force. This is rare in chess design.

## Example scenarios

1. **The bank trap.** A river runs across the board at rank 4 (Water
   squares on the entire fourth rank). The Wife is on d4. Black's
   knight is on d5 — adjacent to her water square. End of turn:
   knight is called. Next turn, the knight must move toward water.
   The nearest water square to d5 is d4 itself (occupied by her); next
   nearest is c4 or e4. The knight cannot move to d4 (she occupies it
   and the knight cannot land on water anyway). The knight cannot
   move to c4 or e4 (they are water and the knight cannot land on
   water). No legal move reduces its distance to water. The knight is
   removed.

2. **The amphibious escape.** Same setup, but black's knight has
   been queen-marked by a Bargainer and now also moves like a queen.
   The knight is called. The knight has a queen move that reaches an
   adjacent water square at a4 — wait, the knight still cannot
   *land* on water (queen-marking grants queen-movement, not water-
   walking). So the knight is still trapped. Unless the variant also
   uses an amphibious-rule for some pieces. Recommend: amphibious is
   a per-piece declared property, separate from the queen-mark.

3. **The slow chase.** A small pond — four water squares in a 2x2
   block — sits in the middle of an otherwise-dry board. The Wife is
   in the pond. Pieces give it a wide berth. A pawn, advancing toward
   promotion, ends its turn on a square adjacent to the pond. Called.
   Next turn the pawn must move toward water. Pawns move forward only.
   The water is to the side. The pawn cannot satisfy the call. The
   pawn is removed. The promotion never happens. She has company
   briefly, in a way.

## Where it shines

- Variant boards with significant water terrain.
- Defensive puzzles where one side must protect a piece adjacent to
  water.
- Atmospheric scenarios — a haunted river bisecting the board.

## Where it's awkward

- Standard boards. She cannot exist on them.
- Boards with only one or two water squares — she can be trapped in a
  tiny puddle.
- The Call rule's severity is high. Almost every called piece dies.
  Playtesters may find this oppressive; consider softening (e.g. the
  called piece *survives* if no water-ward move exists but loses a
  movement-rank instead, mirroring the Hollow Bride).

## Engine dependencies

- New `SquareType::Water` (terrain).
- Water-connectivity computation (graph search; cached per board).
- Movement restriction modifier on called pieces.
- Adjacency check (orthogonal).
- End-of-turn hook for the call.

## New features required

- **`SquareType::Water`.** A new terrain type. Walkable by *amphibious*
  pieces only. Standard pieces cannot move onto water. Lambs at the
  water's edge.
- **Per-piece amphibious flag.** A piece declares whether it can
  enter water. Standard pieces: no. The Wife and (perhaps) a few
  others: yes. Reuse for future water-pieces.
- **Water-connectivity helper.** A graph search over the Water
  square subgraph. Cached at board load.
- **Move-restriction modifier.** A piece marked "called" has its
  legal move-set filtered to *distance-reducing* moves. The filter is
  computed at move-generation time, not move-apply.
- **Removal-on-call-fail.** Pieces with no legal call-satisfying
  move are removed at the start of their turn before the player
  moves.

## FEN encoding

The Wife:
```
(P=W,T=DROWNED)    # P=W for Wife, T=DROWNED to disambiguate from Widow
```

Pieces carrying a transient call-mark:
```
(P=N,C=1)          # called this turn
```

The call-mark is set at end-of-turn-N by the engine, and resolved at
start-of-turn-(N+1). It is FEN-serializable so mid-game states
round-trip correctly.

The Water square type:
```
(T=WATER)
```

## Open questions

- **Call severity.** The current rule is brutal. Soften to "loses
  rank of movement" (Bride-style) instead of "removed"? Recommend:
  keep the severity but consider a variant flag. The folk-horror
  reading wants her to be sad and dangerous, not negotiable.
- **Connected water components.** If a board has two disconnected
  ponds, can she move between them? No — she glides only along the
  component she's in. A piece adjacent to either pond can still be
  called by whichever Wife is in the matching pond.
- **Multiple Wives.** Each calls independently. Their calls union —
  a piece adjacent to two different Wives' waters is called once
  (the flag is binary).
- **Friendly piece adjacency.** A friendly piece adjacent to her —
  is it disturbed? No — she only calls enemies. Friendly pieces may
  sit at the bank freely. (But also: they don't speak to her. She is
  alone.)
- **Water terrain interactions with other pieces.** A Track tile
  cannot be a Water tile. A Block tile cannot be a Water tile.
  Standard square-type exclusivity.
- **Water and the Wife's death.** If she is captured, do her called
  pieces stay called? Recommend: yes — the call-flag persists for
  the one turn it was set. The piece must still move toward water on
  the next turn or be removed. She left a song behind.
- **Bargainer-adjacency simultaneity.** A piece adjacent to both the
  Wife's water and the Bargainer's adjacency zone — both rules fire
  this turn. Recommend: rules apply independently and in declared
  order. The bargain offer resolves first; if the piece survives the
  trade, the call still constrains its next turn.
