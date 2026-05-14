# The Lantern-Eyed Widow

> "I am only looking. When I find what was taken, I will know it."

## Character

She was a queen once, perhaps, or perhaps a woman who lived above a
forge — the records do not agree and she does not correct them. What
she remembers is that something was taken from her, and that she will
know it when her lantern catches it. So she walks the back rank with
the lantern held at shoulder height, and she does not hurry, because
hurrying never found anything.

The lantern is the colour of old butter. It throws no shadow she can
see. When its light falls on a piece down a straight line of squares,
that piece becomes — to her, and only to her — *marked*. Marked
pieces are the ones she is still considering. She has not decided
which is the thing she lost. She may never decide.

The pieces of her own colour do not address her. They have learned not
to. The pieces of the other colour, if they could speak, would tell
each other that being marked is not the worst that can happen, but it
is a kind of being seen that the board is not built for. They cannot
castle while marked. They cannot promote. They cannot be captured —
not by knight, not by queen — except by her. She wants to be the one
who finds it.

## Mechanic

**Movement.** One square per turn, in any of the eight directions
(king's move, but no castling, no privileges). She cannot capture by
moving onto a piece; she can only capture pieces she has *marked*.

**Sight.** At the end of every turn (her side's or the opponent's),
the engine computes the Widow's sight-set: every square reachable from
her current position by a straight orthogonal or diagonal ray, stopping
at the first occupied square (inclusive) or board edge. Enemy pieces
on those squares are *marked*. The mark persists; it is only cleared
when the piece moves to a square not currently in her sight-set on the
turn it arrives. (Pieces that move and re-enter sight on the same turn
remain marked. Pieces that move out of sight clear their mark, and
must be re-seen to be re-marked.)

**Marked-piece restrictions.**
1. A marked piece cannot castle. (If the king is marked, no side may
   castle that turn until the mark clears.)
2. A marked piece cannot promote. A marked pawn reaching the last rank
   is stuck — it stays a pawn on the last rank, immovable forward,
   capturable normally by the Widow only.
3. A marked piece cannot be captured by any piece other than the
   Widow. Other pieces may still attack the square (attack maps are
   unchanged); their captures are *rejected* at move-application.

**Her capture.** She captures a marked piece by moving onto its
square. Standard one-square king-step. She gains nothing from the
capture beyond removing the piece — no promotion, no signal payload.

**State.** None on the piece itself beyond colour and square. The
mark-set is *derived* from her position each turn — no FEN payload
needed for the marks.

## Why it's interesting

The Widow inverts the normal threat-evaluation flow. Usually a piece
under attack is in danger; a piece marked by the Widow is *protected*
from every threat except her, which means a player can sometimes
welcome marking — parking a vulnerable queen in the Widow's sight-line
makes it untouchable by anything but a slow back-rank shuffler. The
Widow forces both players to think about her line of sight not as a
threat radius but as a kind of inviolable cage that only she has the
key to.

This produces unusual tactical patterns: a marked king cannot castle,
so the Widow's sight-line is also a *delay* on king safety; a marked
promoting pawn is a stalled piece on the eighth rank, which can
deadlock entire endgames.

## Example scenarios

1. **Cage gambit.** White's queen on d4 is forked by a black knight.
   White moves the Widow to a1; her sight along the a1-h8 diagonal
   passes through d4. The knight's capture is rejected. Black has
   bought nothing with the fork.

2. **Promotion deadlock.** A black pawn races to a1. It is marked
   (the Widow stands on f1; her sight along the first rank reaches
   a1). The pawn arrives, cannot promote, is stuck. The Widow shuffles
   toward it across nine turns. If the Widow is ever blocked from
   sight before she arrives — by a piece moving between them — the
   pawn promotes immediately on its next move.

3. **Castling denial.** Black castles long. The Widow stands on h1;
   her sight along the first rank marks the black king (passing
   through nothing — no pieces between, because black has just cleared
   the rank for castling). The castling move is rejected at apply
   time. Black must find another tempo.

## Where it shines

- Slow positional puzzles where one piece's invulnerability changes
  everything.
- Endgame studies — the marked-pawn promotion lock is a rich
  composition vein.
- Narrative scenarios where the Widow is the protagonist; her
  back-rank walk gives the game a pace separate from the players'.

## Where it's awkward

- Fast tactical games. The Widow's one-square pace makes her useless
  in any position where the action ends before she crosses the file
  she started on.
- King-side rush openings. If both sides ignore her she contributes
  almost nothing for the first ten moves.
- Stalemate edge cases. A position where the only piece that *could*
  capture a marked piece is the Widow, and she is blockaded, may dead-
  lock. The rule that an opponent's mark *cannot* be cleared by their
  own capture creates positions that the existing stalemate detection
  must handle (see Open questions).

## Engine dependencies

- Standard move generation (king-pattern).
- Sight-ray computation (already implicit in rook/bishop generators).
- Move-application filter: reject captures whose target carries an
  active mark from an opposing Widow, unless the capturing piece is
  that Widow.
- Castling-eligibility predicate (existing).
- Promotion-eligibility predicate (must become extensible).

## New features required

- **Marked-piece predicate.** A function `is_marked(square, side)`
  that recomputes from the Widow's position each turn. Cheap; no
  storage.
- **Capture-eligibility hook.** The move-legality pipeline needs a
  late-stage filter that rejects captures based on the target's marked
  state. Likely lives near the same site as the en-passant /
  castling-through-check checks.
- **Promotion-eligibility hook.** Currently promotion is unconditional
  on reaching the last rank; needs to consult `is_marked`.
- **Castling-eligibility hook.** Same — consult `is_marked` on the
  king.

## FEN encoding

The Widow is payload-free; her marks are derived, not stored.

```
(P=W)   # W for Widow, colour from rank conventions or explicit (P=W,C=W)
```

Multiple Widows on the board: their sight-sets union. A piece marked
by either Widow of a colour counts as marked for that colour. (Two
Widows of opposite colour both marking the same piece: the piece is
marked-by-white AND marked-by-black, both restrictions apply, and
either Widow may capture it.)

## Open questions

- **Mark persistence across blocking.** If a Widow's sight to a piece
  is interrupted by an intervening piece moving in, does the mark
  clear immediately or persist until the marked piece itself moves?
  Recommend: clear immediately at end-of-turn recomputation. Simpler.
- **The Widow seeing her own side.** Allies in her sight-line are
  *not* marked. Marks only apply to enemy pieces. Confirmed by the
  character (she is looking for what was taken from her, and friends
  are not enemies), but worth making explicit in code comments.
- **Two Widows of the same colour.** Likely fine — the mark-sets just
  union. No special rule needed.
- **Stalemate with locked marks.** If the only legal capture is one
  that the marking-rule rejects, and no other moves exist, the side to
  move is stalemated. Acceptable — same as any other movement lock.
- **King in check from a piece protecting a marked target.** Standard
  check resolution. Marks don't interact with check-state. (A marked
  piece can still *give* check.)
