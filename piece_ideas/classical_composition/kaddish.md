# Kaddish

> A bishop that may only land on squares where enemy pieces have died;
> the mate is set up by a sacrifice made three moves earlier.

## Inspiration

The composition tradition's love of **deep keys** — a problem where the
solving move is the *first* move and looks irrelevant, but enables a
geometry four moves later. The Kaddish takes this further: the key may
have been played *before the problem's stipulation begins*. A composer
can construct a position where the only solving sequence begins with a
sacrifice that creates a death-square the Kaddish needs three moves
hence.

Hebrew *kaddish* — the prayer for the dead. The piece honours its
victims; it walks only where they have fallen.

## Mechanic

The Kaddish moves like a bishop (any number of empty squares along a
diagonal). It is constrained to land *only* on squares that have, at
some prior point in the game, hosted the death of an enemy piece —
where "death" is any removal of an enemy piece from the board:
capture-by-displacement, capture-en-passant, or any future
piece-removal mechanic.

The death must be of an *enemy* piece relative to the Kaddish's colour.
A white Kaddish lands only on squares where black pieces have died. A
black Kaddish lands only on squares where white pieces have died.

The Kaddish's *path* — the squares it passes over while sliding — is
unconstrained. The path follows normal bishop rules (must be empty,
sliders blocked by intervening pieces). Only the *landing square*
requires death.

The Kaddish itself dying does not turn its own square into a Kaddish
landing site for the opposite-colour Kaddish — death-squares are
recorded once and never re-evaluated.

A square that has hosted multiple deaths is still just one landing
site; the death is a binary property, not a counter.

State in FEN: a global death-square map, encoded as two square-sets
keyed by colour:

```
death_white = { f3, e5 }       # squares where white pieces have died
death_black = { c4 }           # squares where black pieces have died
```

The white Kaddish can land on c4 (black died there); the black Kaddish
on f3 or e5.

## Why it's interesting (compositionally)

The Kaddish enables a problem class I'll call the **delayed-key sacrifice**:

1. The composer's stipulation begins with the Kaddish *unable to move*
   anywhere useful (every empty diagonal landing-square is "barren" — no
   death has occurred there).
2. White's first move sacrifices a piece on a specific square, *not* to
   open a line or attack anything, but solely to mark that square as a
   landing-pad for a future Kaddish move.
3. Three or four moves later, the Kaddish lands on the sacrifice-square,
   and from that square delivers mate via diagonal pin or check.

Tries that look like the solution (an immediate Kaddish move, a different
sacrifice, a capture instead of a giveaway) fail because they do not
create the *exact* death-square the mating geometry requires.

This is **pure puzzle design**. The Kaddish is a clock face, and the
sacrifice three moves earlier is what makes the hour hand land where it
must.

## A worked problem

White to play and mate in 4.

```
8 . . . k . . . .
7 . . . . . . . .
6 . . . . . . . .
5 . . . . . . . .
4 . . . . . . . .
3 . . . . . . . .
2 . . . . . . . .
1 . . . K x . . .
  a b c d e f g h
```

White: King d1, Kaddish e1 (denoted x), Knight on b4. Wait — let me
re-set with care. Pieces:

- White king d1.
- White Kaddish e1 (a bishop-mover with the death constraint).
- White knight b4.
- Black king d8.
- Black pawn d4 (this pawn is the sacrifice target).

```
8 . . . k . . . .
7 . . . . . . . .
6 . . . . . . . .
5 . . . . . . . .
4 . N . p . . . .
3 . . . . . . . .
2 . . . . . . . .
1 . . . K X . . .
  a b c d e f g h
```

`K` is the white king, `k` the black king, `X` the Kaddish, `N` the
knight, `p` the black pawn. Death-squares: empty (no deaths yet).

The Kaddish on e1 can move along the diagonals e1–a5 (toward NW) and
e1–h4 (toward NE). Every diagonal square is empty. None of them have
hosted a death. **Therefore the Kaddish has no legal moves.**

The solution begins:

1. **Nb4–d5? — no, the knight doesn't reach d5 from b4.** Let me
   correct the knight position. **Nb4xd5? — there is no black piece on
   d5.** I want to set up a sacrifice that puts white meat on d4, then
   white captures it back, then the Kaddish lands on d4.

Restart with a tighter position:

```
8 . . . k . . . .
7 . . . . . . . .
6 . . . . . . . .
5 . . . R . . . .
4 . . . p . . . .
3 . . . . . . . .
2 . . . . . . . .
1 . . . K X . . .
  a b c d e f g h
```

Pieces:
- White king d1, white Kaddish e1, white rook d5.
- Black king d8, black pawn d4.

Death-squares: empty.

**1.Rd5xd4.** White rook captures black pawn on d4. The black pawn dies
on d4. `death_black = {d4}`. The white Kaddish on e1 can now potentially
land on d4 if it lies on a Kaddish diagonal — does it? e1 to d4 is not
on the e1's diagonals (e1's diagonals are e1-a5 and e1-h4; d4 is on
the latter? e1, f2, g3, h4 — no. e1, d2, c3, b4, a5 — no. d4 is not
reachable by Kaddish from e1).

Reposition the Kaddish. Put it on c1 instead. From c1, diagonals are
c1-a3 (NW) and c1-h6 (NE: c1, d2, e3, f4, g5, h6). d4 is not on either
diagonal. Hmm.

Put the Kaddish on a1. Diagonal a1-h8 includes d4? a1, b2, c3, d4 — yes!
And the second diagonal a1 has no NW (off-board). So from a1 the Kaddish
reaches d4 via the long diagonal, *if* the path b2, c3 is empty.

Revised position:

```
8 . . . k . . . .
7 . . . . . . . .
6 . . . . . . . .
5 . . . R . . . .
4 . . . p . . . .
3 . . . . . . . .
2 . . . . . . . .
1 X . . K . . . .
  a b c d e f g h
```

White king d1, white Kaddish a1, white rook d5, black king d8, black
pawn d4.

The Kaddish on a1 has bishop access along a1-h8. Empty squares: b2,
c3, d4 (has pawn), e5, f6, g7, h8. Without any deaths, the Kaddish
cannot land. After 1.Rxd4, d4 has a death-square. But d4 is now
occupied by the white rook, not empty — the Kaddish cannot land on an
occupied square either (bishop rules).

The solution needs the death-square *and* an empty square. So after
the rook captures the pawn, something must move off d4 before the
Kaddish lands. Add a tempo move:

**1.Rxd4** (death on d4, rook on d4)
**1...Kd8–c8** (Black tempo — assume forced)
**2.Rd4–d3** (rook vacates the death-square)
**2...Kc8–b8** (Black tempo)
**3.Kaddish a1–d4+** (Kaddish lands on d4, the death-square, now empty)

From d4, the Kaddish (a bishop) attacks d8? No — d4 to d8 is a file, not
a diagonal. d4's diagonals: a1-h8 (a1, b2, c3, d4, e5, f6, g7, h8) and
a7-g1 (a7, b6, c5, d4, e3, f2, g1). The black king on b8 is on neither
diagonal. **Not check.**

The geometry doesn't trivially line up. This is the *honest* state of
problem composition: the Kaddish mechanic is rigorously specified, but
constructing an actual mate-in-4 around it takes hours of position
massage. The point of the worked problem above is to show:

- The Kaddish cannot move at game start (no deaths).
- A specific sacrifice creates a specific death-square.
- A specific Kaddish landing exploits that death-square.
- Wrong sacrifices (captures on other squares) do not enable the same
  landing.

The composer's craft is to engineer the position so that:

1. Only one white piece can create the needed death-square (the rook
   on d5 — no other white piece attacks d4).
2. The Kaddish from a1 attacks the mating square *through* d4 (after
   vacating it).
3. Every alternative white move misses by exactly one tempo or one
   square.

## Compositional notes

- **Death-squares are permanent.** Once recorded, they never expire.
  Long problems (mate in 8+) can accumulate many landing options. Keep
  the death-set sparse by choosing problems where Black has few capturable
  pieces.
- **The path matters.** The Kaddish's *path* over a diagonal is normal
  bishop sliding — empty squares only. The path does not require deaths;
  only the *landing* does.
- **Empty death-squares.** A landing requires the square to be both
  empty (or hold an enemy to capture) AND on the death-set. Captures
  on death-squares are legal (and produce a new death-square for the
  *other* colour's Kaddish).
- **The barren-board start.** Open the problem with no deaths recorded.
  This makes the Kaddish entirely immobile until the composer's plan
  unfolds, and rules out all "obvious" first moves involving the Kaddish.
- **Sacrifice locations.** The sacrifice must be a *capture by Black* of
  a White piece (creates `death_white` square — used by black Kaddish)
  or a *capture by White* of a Black piece (creates `death_black`,
  used by white Kaddish). Composers usually want the latter.

## Where it shines

- Mate-in-3 and mate-in-4 problems with a "what is this sacrifice for?"
  feel.
- Problems where the obvious capture wins material but misses the
  mating geometry; the obscure sacrifice creates the landing pad.
- Two-mover and three-mover composition tourneys with "delayed key"
  themes.

## Where it's awkward

- The death-set must be tracked in the FEN, growing with every capture.
  Long games produce long FENs.
- Solvers who don't know the rule will play "obvious" bishop moves the
  Kaddish cannot make and get frustrated. A clear note in the problem
  diagram is essential.
- The Kaddish is *very* weak in actual play. It is a composition piece
  first; balancing it for the variant system is awkward unless paired
  with other strong fairy pieces.
- Position re-construction from FEN must include the death-set, which is
  not part of standard FEN. Either extend the FEN or require a side-car
  death-set string.

## Engine dependencies

- Global game state: two `Bitboard` death-sets, one per colour.
- Capture hook: any time a piece is removed, mark its square in the
  appropriate death-set.
- Kaddish movement function: standard bishop movement, filtered by
  death-set membership on the landing square.
- FEN extension to encode death-sets.

## New features required

- New game-state fields `death_white: Bitboard`, `death_black: Bitboard`.
- Hook in capture resolution to update the death-sets.
- Kaddish movement function consulting death-sets.
- FEN payload extension: a top-level `D=` or per-square `dW=`/`dB=` flag.
  Recommendation: a single top-level segment in the FEN suffix listing
  death-squares, e.g. `D[W=f3,e5;B=c4]`.

## FEN encoding

Piece payload:

```
(P=K,C=W)              # white Kaddish (use K_ or some non-conflicting letter — see Open questions)
(P=K,C=B)              # black Kaddish
```

Death-set segment (appended to the FEN, comparable to en-passant / castling segments):

```
D[W=f3:e5;B=c4]
```

`W=` lists squares where white pieces died (used by the black Kaddish);
`B=` lists squares where black pieces died (used by the white Kaddish).
Empty if no deaths: `D[]` or omit segment entirely.

Setup FEN (problem start):

```
... w - - 0 1 D[]
```

After 1.Rxd4:

```
... b - - 0 1 D[B=d4]
```

## Open questions

- **Letter conflict.** `K` is taken by the king. Propose **`Z`** for
  Kaddish (mnemonic: from end of alphabet, reserved). Update the
  payload to `(P=Z,C=W)`.
- **Death-set retention across captures of friendly pieces.** The rule
  specifies *enemy* deaths. A white piece capturing another white piece
  (impossible in chess but possible in fairy variants with mind-control,
  treason, etc.) does not produce a death-square for the white Kaddish.
  Recommend: death-squares are stamped by the *victim's* colour. White
  Kaddish lands on `death_black`. Symmetric.
- **Initial-position deaths.** Can the composer specify pre-existing
  death-squares in the setup? Yes — the FEN segment supports it.
- **Promotions and en-passant deaths.** Promotion removes a pawn —
  does that count as a death on the promotion square? Recommend: no,
  the pawn is *transformed*, not killed. En-passant captures count
  normally — the death-square is the captured pawn's square (not the
  capturing pawn's destination).
- **The Kaddish capturing on an opposite-colour Kaddish's death-square.**
  When a white Kaddish captures a black piece on d4 (which was in
  `death_black`), the black piece's square stays in `death_black`. The
  white Kaddish now occupies it. If the white Kaddish is later
  captured, d4 gets added to `death_white` as well — both sets contain
  d4. This is allowed; sets are independent.
