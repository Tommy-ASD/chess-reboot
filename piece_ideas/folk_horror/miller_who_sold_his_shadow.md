# The Miller Who Sold His Shadow

> "It was a small thing to give up. I have not missed it. I have not missed it."

## Character

He sold his shadow at the parish fair, for a price he no longer
remembers. The buyer was a stranger in a grey coat who folded the
shadow neatly, twice, and put it in a pocket. The Miller went home and
ate dinner and did not think about it again until the sun came up.

A man without a shadow is half a man. He stands on a square but the
square does not properly contain him; the sunlight falls through him
onto the boards beneath, and so do the bishops, and so do the queens
when they travel diagonally, and so do the pawns when they take
diagonally. They pass through his square as if no one were there. They
do not see him. He does not, exactly, blame them. He is not entirely
there.

What he resents is the slowness. The wheel of the mill turns every
fifth day whether he wants it to or not, and on those days he must
walk into town for the grain, even when the road is bad, even when
his bones are tired. So too on the board: every fifth turn he must
move, must put down whichever foot the stranger left him with, and
move. He cannot decline. He cannot wait. He has obligations he did not
agree to.

## Mechanic

**Movement.** Knight pattern. Standard L-shape jump (any of the eight
knight-leaps). He captures by landing on an enemy piece, like a
knight.

**Shadowlessness.** His square is *transparent* to certain piece
types:
- **Bishops** pass through his square as if it were empty. A bishop
  on a1 may move to h8 across his square at d4 without being blocked.
- **Queens moving diagonally** pass through. A queen moving
  orthogonally does *not* — only the diagonal component is shadowless.
- **Pawns capturing diagonally** treat his square as a transit, not a
  blocker. A pawn capturing diagonally *onto* his square does *not*
  capture him — he is not there, in the relevant sense. The pawn's
  capture lands on an empty square (his square) only if the pawn's
  movement rule allows landing on empty squares diagonally; standard
  pawns do not, so this case rarely fires. If a pawn is *moving
  through* his square via a multi-square push (rare; not standard),
  the pass-through applies.

**Capturable only by orthogonal movers.** He may be captured only by
a piece whose move *as instantiated in that capture* is purely
orthogonal:
- Rooks capture him normally.
- Queens moving orthogonally capture him normally; queens moving
  diagonally cannot (they pass through).
- Pawns may not capture him — pawn captures are diagonal.
- Knights may capture him — knights are neither orthogonal nor
  diagonal, but the rule is *not* "diagonal-movers cannot," it is
  *orthogonal movers can*. Re-spec:

**Re-spec — Capturable only by orthogonal-or-step movers.** He may
be captured by:
- Rooks (orthogonal).
- Queens via orthogonal moves only.
- Kings (orthogonal one-step, or diagonal — diagonal kings? See
  Open questions; recommend: king captures him by orthogonal step
  only).
- Knights (L-jump, treated as neither orthogonal nor diagonal — see
  Open questions; recommend: knights *may* capture him, by character —
  the knight steps cleanly onto his square because the knight does
  not glide).
- Pawns may *not* capture him (their capture is diagonal).
- Bishops may *not* capture him.

The intuition: pieces that glide diagonally pass *through*; pieces
that step or glide orthogonally land *on*.

**The fifth-turn obligation.** Every fifth turn that he is on the
board, he *must* move on his side's turn. Specifically: on his side's
move on turn numbers where `(turn_count - placement_turn) % 5 == 0`
and `turn_count > placement_turn`, the controlling player's set of
legal moves is restricted to moves that move *him*. If he has no
legal moves available, the side passes (forfeits the turn? loses?
see Open questions; recommend: the side must move him if any move
exists, otherwise the obligation is waived but the rule still
triggers next fifth turn).

The obligation does not depend on capture status or check — even in
check, the fifth-turn rule applies, and the player must satisfy both
the obligation *and* check resolution simultaneously, which may be
impossible (and so the side loses).

**State.** Per-Miller: `placement_turn`, the turn-counter value at
which he was first placed on the board.

## Why it's interesting

The Miller introduces *transparency* to the board — a square that is
simultaneously occupied (for some purposes) and empty (for others).
This breaks the most common assumption in chess heuristics: "a piece
on square X blocks the line from A to B through X." Players will
gradually internalise that diagonal lines are *not* the same shape as
orthogonal lines once a Miller is in play.

The fifth-turn forced-move adds tempo pressure that the controlling
player cannot fully plan around. A position where the Miller's only
legal move *loses material* is a real cost, and savvy opponents will
engineer such positions deliberately.

## Example scenarios

1. **The unblocked bishop.** White's bishop on a1, white's Miller on
   d4, black's king on h8. The bishop checkmates — it sees through
   the Miller's square along the a1-h8 diagonal. Black had assumed
   d4 was a block.

2. **Orthogonal hunting.** Black's rook on d1, white's Miller on d5.
   The rook may capture him directly along the d-file. Black plays
   Rxd5; the Miller is gone. He was, finally, somewhere.

3. **Forced ruinous move.** Turn 25, Miller placed on turn 5, so
   `(25-5)%5 == 0`. White *must* move him. His only legal squares
   are knight-jumps into discovered checks. White must walk him into
   one of them; the game collapses. The Miller does not look up.

## Where it shines

- Tactical positions where diagonal lines matter. The transparency
  rule is most visible in opened positions with active bishops and
  diagonal queens.
- Compositions exploring the "transparent square" idea — many
  exchange ideas become possible.
- Puzzles built around the fifth-turn obligation — *zugzwang* on a
  schedule.

## Where it's awkward

- King-side games with closed centres; his transparency rarely
  matters when diagonals are blocked anyway.
- Players forget the rule. Transparency is the kind of mechanic that
  must be reinforced visually in the frontend (greyed-out square,
  perhaps).
- The fifth-turn rule, on long enough games, dominates strategy in
  unfun ways. May need playtesting; consider every *eighth* turn
  instead, or making the cadence configurable.

## Engine dependencies

- Knight move generation (existing).
- Slider move generation hook that consults a *transparency
  predicate* on each square in the slider's path. Diagonal sliders
  (bishops + queens-on-diagonal) treat the Miller's square as
  transparent.
- Capture-eligibility predicate that consults the *capturing piece's
  movement mode* — is this capture orthogonal? Diagonal? Step? The
  existing move-generators produce moves; the predicate inspects the
  move's geometry.
- Turn-counter (existing or trivial).
- Forced-move modifier — restricts the legal move-set to a subset on
  certain turns. Likely needed for several future pieces (the
  Bell-Ringer of Last Parish in this very category needs a
  generalised version).

## New features required

- **Per-square transparency predicate.** A function
  `is_transparent_to(square, move_geometry)` consulted by slider
  pathing. The Miller's square returns true for diagonal geometry.
- **Capture-by-geometry filter.** A piece may declare which move
  geometries can capture it. The Miller declares: orthogonal-step,
  orthogonal-slide, knight-jump. The capture pipeline checks the
  capturing move's geometry against this declaration.
- **Move-geometry tag on each generated move.** Each `GameMove`
  carries a tag (orthogonal-slide, diagonal-slide, knight-jump,
  king-step-orthogonal, king-step-diagonal, pawn-push, pawn-capture,
  etc.). Move generators already implicitly know this; making it
  explicit unlocks more rules of this kind.
- **Forced-move-set modifier.** On certain turns, the legal-move set
  is restricted to moves involving a specific piece.

## FEN encoding

```
(P=M,N=12)     # P=M for Miller, N=12 = placement_turn
```

The placement turn is needed to compute when the fifth-turn obligation
fires. Without it, the rule cannot be replayed correctly from
mid-game FEN.

## Open questions

- **Knight capturing the Miller.** Knights neither glide orthogonally
  nor diagonally — they jump. Should they capture him? Recommend yes:
  the knight's leap is clean, not a glide, and the *folk-horror*
  reading is that the knight does not pass *through* anything, it
  arrives. The mechanic-only reading would also allow yes ("not
  diagonal-glide"), but a stricter reading would say "only purely
  orthogonal." Pick one; default is yes.
- **King step diagonally.** A king moving diagonally — does it
  capture the Miller? By the strict orthogonal-or-step rule, no. By
  the "anything that doesn't glide through" rule, yes. Recommend yes;
  the king is a step-piece in all directions.
- **Other fairy pieces.** Goblin / Skibidi / Bus — each will need to
  declare their move geometry to interact with the Miller's
  capturability rules. Default: any non-glide step or jump captures
  him; any diagonal or orthogonal glide... well, orthogonal glide
  also captures him. Only *diagonal glide* passes through.
- **Two Millers shadowing each other.** Two Millers on adjacent
  diagonals: a bishop passes through both. No interaction beyond that.
- **The fifth turn falling on the opponent's move.** The rule fires
  on *his side's* turn at the matching modulus. If the modulus aligns
  with the opponent's turn, no rule fires; it must align with his
  own.
- **Miller in check on his fifth turn.** Must move *and* resolve
  check. If impossible, the position is lost. This is the
  *zugzwang-on-a-schedule* mentioned above; consider whether the rule
  should yield to check (i.e. check-resolution overrides fifth-turn
  obligation). Recommend: rule does not yield. The Miller has
  obligations he did not agree to.
