# The Hollow Bride

> "I will know him when I see him. I have been told this is so."

## Character

She wears white because she was told to. The veil is long and the
veil is heavy, and she does not remember when it was put on her or by
whom. She moves in long diagonal lines because that is what brides do,
in some country, in some old story she might have read or might have
been. She is looking for her groom.

She has been told she will know him. She has not been told how. So she
passes among the pieces and looks at each one as she goes by, and
because she is hollow inside — the bridal dress is full of nothing —
each piece she looks at gives a little of itself to her. Not enough to
notice at first. A queen who has been looked at by the Hollow Bride
still moves like a queen, almost. But a queen looked at twice moves
like a rook. And a rook looked at moves like a bishop. And a bishop
looked at moves like a pawn. And a pawn looked at is nothing at all,
because it had so little to give.

When eight pieces have given her what they had, she stops. She has
collected enough of him, or enough not-him, to be certain. On the next
turn, whatever piece is adjacent to her is her groom, and she takes it
by the hand, and they both leave the board. The other pieces never see
either of them again.

She is not cruel. She is only mistaken about what brides do.

## Mechanic

**Movement.** Bishop. Any distance along an unblocked diagonal. Cannot
jump over pieces. She *passes over* every empty square along her path
— note that her path is the squares between her origin and her
destination, inclusive of the destination but exclusive of the origin.

**Wait.** When she passes *over* a square containing a piece, this
is impossible — bishops are blocked by occupancy. The Bride is *not*
a slider that ignores occupancy. So how does she pass over pieces?

She doesn't. Re-spec:

**Re-spec — Movement.** Bishop-pattern, but she *may* pass through
occupied squares. Pieces in her path do not block her. Each piece in
her path (origin to destination, exclusive of origin and destination
themselves) is *passed over*. She does not capture pieces she passes
over. She may capture a piece at her destination only by normal
bishop-capture rules.

**The hollowing.** Each piece passed over (not captured — only passed
over) loses one *rank of movement* permanently:

| Current rank | After being passed over |
|--------------|--------------------------|
| Queen | Rook |
| Rook | Bishop |
| Bishop | Pawn |
| Pawn | (removed from the board) |
| Knight | Pawn (a step-piece becomes a step-pawn) |
| King | King (the king is exempt — the Bride does not see him) |
| Other / fairy pieces | One rank reduction per piece-specific table |

The rank reduction is permanent and stored on the piece. A piece that
has been hollowed once and then hollowed again continues down the
table.

**The count.** The Bride maintains a counter — *hollowed*, an integer
0–8 — incremented by one each time she passes over a piece (king
excepted; the king does not count). When the counter reaches 8, she
*stops moving*. She may no longer be commanded; she is finished
walking.

**The wedding.** On the turn *after* she stops (i.e. the next time her
controller would normally move her), instead of a move: every piece
adjacent to her (the eight king-pattern neighbours) is *examined*. If
exactly one piece is adjacent — that is the groom. Both the Bride and
the groom are removed from the board. If zero pieces are adjacent —
nothing happens; she waits. She remains immobile, and the check
re-fires every turn until exactly one piece is adjacent.

If *more than one* piece is adjacent — she chooses the one with the
*highest current rank* (queen > rook > bishop > knight > pawn). Ties
broken by lowest file, then lowest rank. The chosen piece is the
groom. Both leave.

**Colour.** Pieces of any colour, including her own side's pieces,
can be hollowed and can be the groom. She does not distinguish. Her
own queen passed over is hollowed; her own pawn passed over is removed
from the board. *This is the point.* She is not a tactical piece. She
is a haunting in white.

**Capture.** The Bride may be captured normally (she has no defence).
If captured before reaching 8 hollowings, her state is gone and the
hollowed pieces remain hollowed forever. If captured after stopping
but before the wedding, same: she dies on the threshold and the groom
is never named.

## Why it's interesting

The Bride is the only piece in the design space that *damages* the
pieces in her path without capturing them. This produces an entirely
new positional consideration: the *cost of letting the Bride pass*.
Players will route their own valuable pieces away from her diagonals,
and route their *opponent's* valuable pieces *into* her diagonals when
they can. She bends the geometry of the whole board around her
trajectory.

Because she counts to 8, she has a clock. The game's pacing acquires a
soft deadline: at some point she stops, and the next adjacency
determines who leaves with her. This invites tactics around her
*final position* — placing a low-value piece adjacent to her at the
right moment to be the "groom" instead of a queen.

## Example scenarios

1. **The careful diagonal.** White's Bride starts on c1 and moves to
   h6. Her path: d2, e3, f4, g5, h6. Black has placed a queen on f4
   and a rook on g5. After the move, the queen on f4 becomes a rook;
   the rook on g5 becomes a bishop. The Bride captured nothing at h6
   (h6 was empty). Black has lost a queen and a rook to *one bishop
   move*. Hollowed count: 2.

2. **Sacrificing the wrong groom.** The Bride has hollowed 7 pieces.
   On her next move, she walks one diagonal step and the hollowing
   count becomes 8. White, controlling her, is alarmed: the only
   piece adjacent to her current square is black's queen. White wants
   to position a black *pawn* adjacent to her instead, so the wedding
   takes the pawn. But it is black's turn; black moves their queen
   away, and a black pawn moves into adjacency. The wedding fires —
   the pawn is the groom. Both leave. White is annoyed.

3. **The hollowed army.** Mid-endgame: the Bride has passed over
   black's entire piece structure on two long diagonals. Black is
   left with a king, a hollowed-pawn-formerly-rook, and a regular
   pawn. White has a queen and a Bride at 5/8. Black resigns. The
   Bride has not captured a single piece directly.

## Where it shines

- Long open boards where diagonals run free.
- Asymmetric setups where one side controls the Bride and uses her
  as a constant pressure.
- Narrative puzzle scenarios — the "stop at 8 and find the groom"
  endgame is rich.
- Positions where the Bride passes over *both* sides' pieces; the
  collateral makes her a true wild card.

## Where it's awkward

- Closed positions with many pawn chains; her diagonals are short.
- Short games — she may never reach 8.
- The "kings exempt" rule needs to be solid; if she ever passes over
  a king on a diagonal, that is a check-like situation that the
  engine must handle without hollowing.

## Engine dependencies

- Bishop move generation, modified to allow passing through occupied
  squares (pieces in her path do *not* block her movement).
- Per-piece rank-of-movement state, stored as an integer or move-set
  override. The reduction table is data-driven.
- Per-piece move-set lookup that consults the override.
- Adjacency check (king-pattern neighbours).
- Counter state stored on the Bride.
- End-of-move hook that fires the wedding when she is in the "stopped"
  state.

## New features required

- **Slider-through-occupancy variant.** Bishop generator with an
  "ignores occupancy" flag. May generalise to other ghost-like
  pieces.
- **Movement-rank tracking.** Each piece carries a `rank_reduction`
  integer. A piece's effective move-set is `base_move_set rank-shifted
  by rank_reduction`. The rank table is engine-wide data.
- **Permanent piece state via FEN payload.** The Bride's counter and
  each piece's `rank_reduction` are both persistent state.
- **Stopped-piece behaviour.** When the Bride's counter is 8, her
  move-set becomes empty and the wedding-check fires at end-of-turn.

## FEN encoding

The Bride:
```
(P=H,K=N)      # P=H for Hollow Bride, K=N for hollowed-count (0..=8)
```

Pieces with reduced rank:
```
(P=Q,R=1)      # a queen reduced by 1 rank — currently moves as a rook
(P=R,R=2)      # a rook reduced by 2 ranks — currently moves as a pawn
```

A piece whose `R` would reduce it to "nothing" is removed from the
board at hollow-time; no FEN encoding is needed for that case.

## Open questions

- **Promotion of a hollowed pawn.** A pawn-shaped piece that was
  originally a queen (hollowed twice → bishop → pawn-rank-formerly-
  queen) reaches the last rank. Does it promote? Recommend: yes,
  promotion restores it to a queen — and the hollowing was for
  nothing. This is fairy-tale logic: the queen-pawn was always going
  to remember herself.
- **The Bride hollowing herself.** Impossible by construction — her
  path excludes origin and destination.
- **King in her path.** The king is exempt from hollowing. She passes
  over him without effect. This means she does not give *check* by
  passing over the king either; the king is invisible to her. Confirm
  with the check-resolution path.
- **What if no piece is ever adjacent.** She waits forever. Fine — it
  is in character. Possibly forbid via a fifty-move-rule analogue
  ("if she has been stopped for fifty turns, the game is a draw").
- **Same-coloured groom.** She may take her own piece as the groom.
  This is intentional. It is sad.
- **Multiple Brides.** Permitted in principle. Each carries its own
  counter. They do not interact except by occupying the board.
