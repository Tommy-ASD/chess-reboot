# The Bargainer at Crossroads

> "I have a bag. In the bag are several things. Would you like to see them?"

## Character

He arrives before the game begins and chooses his square — always a
crossroads, which on a standard board means one of the four
quarter-points, the squares from which the board's diagonals and
midlines all meet. On irregular boards he finds whichever square is
closest to the intersection of medians; he has a sense for these
things. Once placed, he does not move. He has never moved. The bag is
heavy and he is tired.

He is not malicious. He is, perhaps, exactly as malicious as a
shopkeeper. The trade he offers is fair by his own measure: the
passing piece gains everything, but the passing piece's player gives
up one of their own. Most pieces refuse. Some accept. He does not
remember which is which, only that the bag has been a little lighter
some games than others.

He hums to himself. The other pieces find this unsettling and have
learned to take the long way around.

## Mechanic

**Placement.** At setup, the Bargainer occupies a *crossroads square*
— defined as the square nearest to the board's geometric centre that
is also a quarter-point of the playable region. On a standard 8x8,
candidate squares are d4, d5, e4, e5. On variable boards, the engine
computes the four quarter-points (rounded to the nearest playable
square; non-walkable squares are excluded). Setup chooses one; only
one Bargainer per board. He may not be placed mid-game.

**Movement.** None. He never moves. He cannot be commanded to move.
He can be captured normally (he has no special defence), at which
point all outstanding offers expire.

**The offer.** When an enemy piece *passes through* a square
orthogonally adjacent to the Bargainer — i.e. the piece's move
trajectory enters one of the four squares N/E/S/W of him, even briefly
— that piece is offered the bargain. "Passes through" means the
piece's path geometrically crosses the adjacent square; for sliders
this is straightforward, for knights this means the knight's
destination is adjacent to him, for pawns this means the pawn moved
into adjacency.

**The trade.** When offered, the opponent (the player whose piece is
*not* passing — i.e. the Bargainer's controller) selects, in the same
move-input, one of their own non-king pieces to sacrifice. That piece
is removed from the board. The passing piece is then permanently
*queen-marked* — it retains its original move-set in *addition to*
gaining queen-movement (rook + bishop). A queen-marked knight moves as
both knight and queen. A queen-marked pawn moves as a queen and may
still promote on the last rank.

**Refusal.** The Bargainer's controller may *refuse*: select no
sacrifice. Nothing happens. The passing piece continues its move
unchanged. The Bargainer remembers nothing. The offer is *not*
re-presented if the same piece passes through again next turn — yes,
it is. He has no memory.

**Forced refusal.** If the Bargainer's controller has no non-king
pieces, they must refuse. (The trade cannot remove the king.)

**State.** Per-piece: queen-marked-or-not. This is a single bit added
to each piece's FEN payload.

## Why it's interesting

The Bargainer is the rare mechanic where the player *not* moving gets
to make a decision in the middle of the opponent's move. This changes
the rhythm of the game — every piece passing near the centre opens a
window the opponent must consciously close or accept. The cost is
explicit (one of *your* pieces) and the benefit is large (permanent
queen-movement on the opponent's piece), so the trade is rarely
trivial.

He creates positional gravity: pieces that *want* the queen-mark will
route near him; pieces that fear the trade will route around. The
adjacency squares become contested zones independent of the usual
chess priorities, and the centre takes on a different texture — not
just powerful but *transactional*.

## Example scenarios

1. **Knight upgrade.** Black's knight passes through e5 (Bargainer on
   d5, adjacency at e5). White is asked. White sacrifices a backward
   pawn on a2. The knight gains queen-movement; it is now a *knight-
   queen* and threatens forks and back-rank mates simultaneously.
   White paid one pawn; black gained a near-game-winning piece. White
   should have refused.

2. **The refused march.** Black's rook slides past d4 four turns in a
   row, each time offering the trade. White refuses each time. Black
   knows the rook gets nothing, but black has gained four free tempi
   of central pressure. The Bargainer's mere presence has bent the
   game.

3. **The strategic sacrifice.** White's bishop on c1 has been bad for
   the whole game. Black's queen passes through e4. White accepts —
   sacrifices the bad bishop — and black's queen gains nothing
   meaningful (it already moved like a queen). White has spent a bad
   piece to *deny* black's bishop the same upgrade later in the game.
   The Bargainer accepts the bishop with a small nod.

## Where it shines

- Positions with heavy central traffic. The Bargainer is most
  influential in the middlegame.
- Multi-piece strategic puzzles where the player must weigh long-term
  piece quality against short-term sacrifice.
- Asymmetric setups where one side has a deep piece reserve and the
  other does not — the trade dynamic becomes lopsided in interesting
  ways.

## Where it's awkward

- Empty boards / endgames. With few pieces in motion, the Bargainer
  offers nothing.
- Speed chess. The mid-move decision adds a clock-stop that is
  awkward in fast play.
- King-and-pawn endgames where the controller has no pieces to
  sacrifice — the forced-refusal rule keeps the game functional but
  the Bargainer becomes scenery.

## Engine dependencies

- Move-path computation (already exists for slider attack-rays).
- Adjacency predicate (trivial).
- Mid-move decision pause — likely already needed for promotion
  choice, en-passant, and castling-side selection.
- Per-piece extensible move-set (the queen-mark adds queen-movement
  to *any* piece without replacing its base move-set).

## New features required

- **Pass-through detection.** A trajectory predicate that fires for
  every square a move passes through (not just its endpoints).
  Knights pass through their destination only; sliders pass through
  every intermediate square.
- **Queen-mark piece flag.** A per-piece boolean; movement generators
  union the queen's move-set with the piece's base move-set when the
  flag is set.
- **Mid-move opponent input.** The move-application pipeline needs a
  callback to the *non-moving* player for the sacrifice choice. This
  may already exist for analogous cases; if not, generalize it.
- **Crossroads square computation.** A board-geometry helper that
  identifies quarter-points. Cached at board creation.

## FEN encoding

The Bargainer himself:
```
(P=B,T=BARGAINER)    # or (P=B) if the type is unambiguous
```

Queen-marked pieces carry a `Q=1` flag in their payload:
```
(P=N,Q=1)            # a knight-queen
(P=P,Q=1)            # a pawn-queen
```

The queen-mark persists across the rest of the game and round-trips
through FEN.

## Open questions

- **Multiple Bargainers.** Probably forbidden — the rule reads as
  singular. If allowed, do two Bargainers' adjacency zones overlap,
  and does a piece passing through the overlap trigger two trades?
  Recommend: one per board.
- **Bargainer captured mid-offer.** If the Bargainer is captured on
  the same turn a piece would pass through his adjacency, does the
  trade still fire? Recommend: no — captures happen at move-apply,
  and an offer requires a living Bargainer at end-of-move.
- **Adjacency diagonals.** Spec says orthogonal adjacency. Should
  diagonal adjacency also trigger? Recommend: no — keeps the trigger
  zone to four squares and matches the "crossroads" framing (cardinal
  directions).
- **Stacking the queen-mark.** A piece queen-marked twice is no
  different from once (the move-set union is idempotent). The flag is
  a single bit, not a counter.
- **Queen-marked king.** A queen-marked king is significantly
  stronger. Is the king sacrificable? No — already handled by the
  forced-refusal rule. Can the king itself be queen-marked? Yes, if
  the king passes through adjacency. This is intentional and
  flavourful — the king has dealt with him directly.
