# Tithe

> A king-mover that permanently donates one square of its own movement
> reach to an adjacent enemy every time it moves; selfmates write themselves.

## Inspiration

The **selfmate** — a problem where White moves first and forces Black to
deliver mate against White's will. Selfmates are perverse, beautiful, and
notoriously hard to construct, because the composer must engineer a
position where Black has *exactly one* legal move at every juncture and
that move mates the white king.

The Tithe is built from the ground up to make selfmates obvious. Every
Tithe move *strengthens the enemy*. The composer who places a Tithe in a
selfmate problem is handing themselves a tempo-feeding machine.

## Mechanic

A Tithe moves like a king — one square in any of eight directions — but
with a **movement-power counter** that begins at 8 and decreases.

On every successful Tithe move, *one square of the Tithe's own movement
power is transferred to a single adjacent enemy piece, permanently*. The
mover chooses the target enemy piece and which directional power to
transfer. The counter on the Tithe drops by 1; the recipient piece gains
a permanent **extra move-direction** in the chosen compass direction.

The directional powers form an 8-element bitset: N, NE, E, SE, S, SW, W, NW.
The Tithe starts with all 8 bits set. The recipient piece's
"extra-direction" bitset starts at 0 and grows by one bit per gift.

When the Tithe's counter reaches 0, it is a **stone** — present on the
board, blocks lines, can be captured, has no movement at all.

The recipient's gained directions act as **king-style one-square moves**
in those directions, added to whatever the recipient piece's native
movement is. Example: a pawn that has received the NW gift can move one
square NW (a pawn-king hybrid in that direction only). The gifts stack:
a pawn that has received N + NW + W from successive Tithe donations gains
three additional one-square moves.

Captures by gifted directions follow normal capture rules (capture by
displacement; one piece taken). En passant and pawn-promotion are
unaffected by gifts.

State in FEN:

- Tithe carries `M=N` where N is its remaining counter (0..=8) and a
  bitset `D=NESW...` for the *remaining* directions it can still donate
  (subset of the original 8).
- Every piece on the board may carry an optional `G=...` bitset of
  *gifted* directions.

## Why it's interesting (compositionally)

This piece *is* the selfmate. White uses the Tithe to feed Black the
exact movement powers Black needs to deliver mate, and only the *right
sequence* of donations produces a forced mate-by-Black in N moves.

Variants of the motif:

- **Forced-feed selfmate.** White's only legal move is a Tithe gift; the
  gift transforms a Black piece into the exact mating piece needed; Black
  has only that piece able to move; mate.
- **Wrong-gift try.** A tempting first move gives Black the wrong
  direction; Black's "free" move with that direction fails to mate and
  loses the tempo race.
- **Zugzwang from depletion.** The Tithe's M counter eventually hits 0;
  the composer engineers a position where the Tithe must make exactly N
  moves and on move N+1 it is a stone, which removes a defender from the
  white king's escape squares.

## A worked problem

White to play, **selfmate in 2**.

```
8 . . . . . . . .
7 . . . . . . . .
6 . . . . . . . .
5 . . . . . . . .
4 . . . . . . . .
3 . . . . . . . .
2 . . . . . p . .
1 . . . . . T K .
  a b c d e f g h
```

Pieces:
- White king on h1.
- White Tithe on g1, M=8, all 8 directions available to donate.
- Black pawn on f2 (one square from promotion if it moves to f1).
- Black king on … let's place it far away on a8 so it isn't relevant.

Black king on a8. White's king is on h1 with two escape squares: h2 and
g2 (both empty). The black pawn on f2 cannot reach the white king
without a capture, and the white king is not in check.

White wants to *force Black to mate White*. Black has only one moveable
piece other than its king — the pawn on f2. The pawn's natural move is
f2-f1 (promotion). Pawn promotion to a Black queen on f1 attacks g1 (the
Tithe), e1, h1 (the white king) — that *is* mate if g2 and h2 are also
covered.

The pawn alone cannot cover both h2 and g2. So White must engineer a
Black piece that, when it makes its forced move, covers both escapes
or delivers double-check.

Solution attempt:

1. **Tg1–f2, donating direction "NE" to the black pawn.** White captures
   the black pawn? No — the pawn is on f2, the Tithe moves to f2,
   capturing the pawn. The pawn is removed. That doesn't work — we want
   to *gift* the pawn, not capture it. The donation requires the
   recipient to be alive after the move.

   Reconsider. The Tithe must move *adjacent* to the recipient. Tithe on
   g1, pawn on f2 — they are diagonally adjacent. The Tithe can move to
   *any* king-square as long as that square is empty (g2, h2, f1, f2 if
   capturing) and donate to an adjacent enemy on completion. To donate
   to the pawn without capturing it, the Tithe must end on a square
   adjacent to the pawn — meaning g1 (its current square, can't stay),
   g2, g3, f1, e1, e2, e3, or f3.

   The Tithe moves only one square at a time (king-move). From g1 the
   reachable squares adjacent to the pawn-on-f2 are: g2, f1.

   **1.Tg1–g2, donating direction "NW" to the pawn.** Tithe ends on g2,
   adjacent to f2 (diagonally NW). White king on h1 now has only h2 as
   an escape (g1 was vacated, g2 now occupied by white Tithe — a friendly
   piece, blocks king escape to g2). Wait — the white king on h1 needs
   to *not* be in check after White's move; let's verify. Black pawn on
   f2 attacks e1 and g1. Tithe was on g1, white king on h1. After Tithe
   to g2: king on h1 is attacked by the pawn from f2? Pawn on f2 attacks
   e1 and g1 (one square forward-diagonal, but Black pawns capture
   *toward white*, which from f2's perspective is toward rank 1, so
   yes — attacks e1 and g1). The king on h1 is not attacked. The Tithe
   on g2 is not attacked (pawn on f2 attacks g1, not g2 — pawn captures
   forward-diagonal). Good: legal move.

   Black's response: Black has only the pawn (and the far-away king) to
   move. The pawn moves f2→f1 with promotion. The pawn has been gifted
   direction NW, which adds a one-square NW attack from wherever the
   pawn sits — but pawn-promotion replaces the pawn with a new piece. Does
   the gift transfer to the promoted piece? **Compositional decision: yes.**
   The gift is on the piece-slot, not the piece-type; promotion preserves
   it. Black promotes f2→f1=Q with NW gift. The queen on f1 attacks the
   entire f-file, the 1st rank (h1 ✓ — check on white king), and the
   a6-f1 diagonal. With the NW gift, it also attacks e2 — irrelevant.

   Is this mate? White king on h1 is in check from Qf1. Escape squares:
   g1 (empty, attacked by Qf1 along the rank — no, Q on f1 attacks g1
   along the rank ✓), g2 (occupied by friendly Tithe — blocked), h2
   (empty, attacked? Qf1 to h2 is two squares diagonally — not a queen
   attack; Qf1 to h2 is along the f1-h3 diagonal, h3 not h2. Not attacked
   by Q). So h2 is an escape. **Not mate.**

   We need to also cover h2. The Tithe gift to the pawn should add a
   direction that, when the pawn promotes, also threatens h2.

   A queen on f1 attacks h3 along the f1-h3 diagonal but not h2 (h2 is
   not on a queen ray from f1). The only direction from f1 to h2 is a
   knight's L (f1-h2 is two-right-one-up — a knight move). A pawn-with-N
   gift on f1 attacks f2 (forward). A pawn-with-NE gift on f1 attacks g2.

   So if the gift is the **N direction**, the promoted queen on f1 also
   has the king-style N attack (already covered by the queen, redundant).
   No help for h2.

   What if we donate **two** directions over **two Tithe moves**?
   Selfmate in 2 means: White move 1 (Tithe gift), Black move 1, White
   move 2 (Tithe gift again? or some other move), Black move 2 (forced
   mate). White has two move-pairs to set up the gift package.

   Selfmate in 2 = white moves 2 times, black moves 2 times, black's
   second move mates. So:

   1.Tg1–g2 (gift NW to pawn) — Black must respond.
   1...f2–f1=Q (promotion). The queen now has gift NW.
   2.Tg2–h2 (gift … wait, the Tithe is no longer adjacent to the queen.
   Tithe on g2, queen on f1, they are diagonally adjacent SW. Tithe can
   gift to any adjacent enemy. Tithe gifts S to queen-on-f1.
   The queen now has gifts NW and S. The queen on f1 attacks the entire
   1st rank, f-file, diagonals. The S-gift on f1 attacks f0 (off-board)
   — useless.

This problem is too constrained for a clean mate-in-2 sketch by hand,
but the *shape* is now visible. The genuine selfmate problem with this
piece looks like:

**Problem template.** A black pawn one square from promotion; a white
king with one escape square; a Tithe positioned to deliver enough gifts
in N moves to ensure the promoted Black queen covers every escape AND
delivers check. The composer's job is to find a starting position where
the *exact* directional gifts required happen to be in the Tithe's
remaining donation set, and *no other gift combination* mates.

Tries fail because:
- Donating direction X gives Black a queen that attacks an extra
  square — but the wrong square, missing one escape.
- Donating direction Y captures the recipient (Tithe ends on the
  recipient's square) — wrong, the donation requires the recipient to
  live.
- Skipping a donation (Tithe moves to a non-adjacent square) wastes a
  tempo; the pawn promotes before the gifts complete.

The mechanic guarantees that *only one* white move-sequence forces the
black promotion into a mating geometry.

## Compositional notes

- **Direction bitsets matter.** Track which directions the Tithe has
  already donated; each is one-shot. A selfmate-in-5 problem may
  require all 8 directions delivered in order — be ruthless about
  ordering.
- **The Tithe's path is constrained.** It must be adjacent to the
  recipient at the *end* of each move. Plan the Tithe's tour to
  shadow the recipient.
- **Stone endings.** When the Tithe hits M=0 it is a stone. Use this
  for endgame tries where the stone-Tithe blocks the white king's
  escape, completing the selfmate geometry.
- **Promotion preserves gifts.** Confirm this with the engine
  (compositional decision: yes). Without it, half the selfmates die at
  the promotion barrier.
- **Adjacency for donation.** A Tithe that ends its move *non-adjacent*
  to every enemy donates nothing — the move is legal but the counter
  does not decrement. Use this for tempo moves where the Tithe must
  reposition without spending a donation.

## Where it shines

- Selfmates in 2–6 moves featuring a single black pawn near promotion.
- Selfmates where the white army's only legal moves are Tithe moves.
- Zugzwang endgames where the Tithe's depletion to a stone removes a
  white piece-effect and forces Black's hand.

## Where it's awkward

- The bookkeeping is intense. A board with one Tithe and seven gifted
  black pieces has 8 + 8×8 = 72 bits of donation state. The FEN gets
  long.
- Solvers must track which Black pieces have which gifts. A diagram
  with gift annotations alongside each gifted piece is essential.
- Two Tithes per side is a nightmare. One per problem, ideally one per
  board.

## Engine dependencies

- Per-piece state (counter on Tithe, bitset on every piece).
- A donation move-hook that fires on Tithe-move completion.
- Movement function that consults gift-bitsets to extend any piece's
  legal-move set by king-style adjacent moves in gifted directions.
- Promotion preserves gift-bitset.
- Capture removes gift state with the piece.

## New features required

- New per-piece field `gifts: Bitset8` (default 0) on all pieces.
- New per-piece field on Tithe: `power: u8` (0..=8) and `donations_remaining: Bitset8`.
- Movement augmentation: `legal_moves(piece) = native_moves(piece) ∪ gifted_adjacents(piece)`.
- Move-completion hook for Tithe that:
  1. Identifies an adjacent enemy (user-selected at move-construction time, or auto-select rule).
  2. Selects a direction from `donations_remaining` (user-selected).
  3. Sets the bit on the recipient's `gifts`; clears it from the Tithe's `donations_remaining`; decrements `power`.
  4. If no adjacent enemy exists, the move is still legal; no donation occurs and the counter does not decrement.
- FEN encoder/decoder for `M=`, `D=`, `G=`.

## FEN encoding

Tithe piece payload:

```
(P=T,C=W,M=8,D=NESWnews)        # full power, all directions available
(P=T,C=W,M=3,D=Nes)             # 3 donations remaining, can give N, E, S
(P=T,C=W,M=0)                   # stone; D and M=0 redundant
```

The `D=` field uses NESWnesw notation (uppercase cardinals, lowercase
intercardinals) for the eight directions. `M=` is the integer counter.

Any piece's gifts:

```
(P=P,C=B,G=NE)                  # black pawn with N + E gifts
(P=Q,C=B,G=W)                   # black queen with W gift
```

The `G=` field uses the same NESWnesw notation.

## Open questions

- **Auto-select vs explicit donation choice.** Does the engine require
  the player to choose the recipient and direction at move-construction
  time, or does it auto-pick (e.g. lowest-rank adjacent enemy, leftmost
  available direction)? Composers will want explicit control;
  players want simplicity.
- **No adjacent enemy.** A Tithe move that ends adjacent to no enemy
  must still be legal (tempo move). Confirmed: no donation, no counter
  decrement.
- **Self-donation.** Can the Tithe donate to a friendly piece? No —
  enemies only. (Otherwise the mechanic becomes a tempo engine.)
- **Promotion gift transfer.** Definitive answer: gifts transfer to the
  promoted piece. This is the compositional intent.
- **King donation.** Can a Tithe donate to the enemy king? Yes —
  unusual but legal. The enemy king gains a "long" king-move in the
  gifted direction (functionally a small queen-move). This enables
  exotic selfmates where Black's king is gifted directions and forced
  into ranges only it covers.
