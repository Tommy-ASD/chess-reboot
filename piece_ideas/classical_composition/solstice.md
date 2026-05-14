# Solstice

> A king that may, exactly once per game, move as a queen instead;
> spending the privilege early or late changes the helpmate solution.

## Inspiration

The helpmate composer's tradition of **resource budgeting** — Black
helps White mate in N moves, and the composition's elegance lies in
making each move *uniquely* necessary. The Solstice is a one-shot
queen, and the helpmate composer chooses *which move number* must spend
that shot. Move-order variations all fail except the one the composer
intended.

The piece's name evokes the once-a-year astronomical event: a single
moment of maximum reach.

## Mechanic

The Solstice moves as a king (one square in any of eight directions) on
every move, *except* that exactly once per game it may move as a queen
instead.

State in FEN: one flag per Solstice — `Q=1` (queen-move still
available) or `Q=0` (queen-move already spent).

The Solstice's queen-move is a normal queen move (any number of empty
squares along rank, file, or diagonal). Captures are normal. Check
delivery is normal. The only restriction is the once-per-game budget.

The privilege does not regenerate. There is no Solstice-promotion or
Solstice-multiplication.

A Solstice in check follows normal king-in-check rules (must escape or
block or capture). The queen-move can be used to escape check — and
often must be, when ordinary king moves are insufficient.

## Why it's interesting (compositionally)

The Solstice is built for **helpmates**: problems where Black moves
first (or both sides move in alternation, depending on convention) and
both sides cooperate to checkmate Black's own king in exactly N moves.
Helpmates live or die on solution uniqueness — if any move-order
produces the mate, the problem is "cooked" and discarded.

With a Solstice, the composer guarantees one queen-move exists in the
solution. The puzzle becomes: *which* of the N White moves must be the
queen-move? Spending it early leaves the Solstice as a slow king for
the rest of the problem; spending it late requires the king-only moves
to traverse impossible terrain in the lead-up.

The composer engineers a position where exactly one move-number admits
the queen-shot. Tries that spend the queen-move on move 1, 2, 3, etc.,
all fail except the intended one.

## A worked problem

**Helpmate in 3.** Black moves first; both sides cooperate; Black is
checkmated on White's 3rd move.

```
8 . . . . . . . .
7 . . . . . . . .
6 . . . . . . . .
5 . . k . . . . .
4 . . . . . . . .
3 . . S . . . . .
2 . . . . . . . .
1 . . . . . . . .
  a b c d e f g h
```

`S` = white Solstice on c3 (queen-move available, Q=1). White king on
some far-away square (a1) — not relevant to the mating geometry.
Black king on c5. Black has no other pieces. White has no other pieces
besides the off-board-stage king.

Stipulation: Helpmate in 3. Both sides cooperate so that Black is mated
on White's third move.

Black moves first. We need a sequence:

1...Black1, 1.White1, 2...Black2, 2.White2, 3...Black3, 3.White3#

Solution (one of the canonical helpmate solutions for this kind of
position):

- **1...Kc5–b5** (Black king walks toward the corner).
- **1.Sc3–c4** (Solstice as king, one square forward). Q=1 still.
- **2...Kb5–a5** (Black king to a5, hugging the edge).
- **2.Sc4–b5** (Solstice as king, diagonal step). Q=1 still.
- **3...Ka5–a6** (Black king to a6 — forced cooperation: the only
  square that allows White's mating queen-move).
- **3.Sb5xQUEEN–a6**? — no, the king is on a6. **3.Sb5–b6+**? Let's
  check: Solstice on b6 (as king move from b5) attacks a5, b5, c5, a6,
  c6, a7, b7, c7. Black king on a6 is attacked. Is it mate? Black's
  king escapes: a5, a7 (both attacked), b5 (occupied — actually b5 is
  vacated by Solstice's move), b6 (now Solstice), b7 (attacked).
  Escapes: a5 (attacked), a7 (attacked). Black has no escape. **Mate
  without the queen-move.**

This means the queen-move was never necessary — the problem is *cooked*
because it doesn't require the Solstice's special privilege. The
composer would discard this sketch.

**Revised position.** Place an obstacle so that the king-move sequence
cannot reach the mating square, but a queen-move can.

```
8 . . . . . . . .
7 . . . . . . . .
6 . . k . . . . .
5 . . . . . . . .
4 . . . . . . . .
3 . . . . . . . .
2 . . . . . . . .
1 . . S . . . . .
  a b c d e f g h
```

White Solstice c1 (Q=1). Black king c6. White king far away (h1).
Empty 4th and 5th ranks.

Helpmate in 3 with the constraint that *only the second White move can
be the queen-move.* The composer wants:

- White move 1: king-step (preserves Q=1).
- White move 2: queen-move (consumes Q=1).
- White move 3: king-step that mates.

Sample line:

- 1...Kc6–b6 (Black king sidesteps).
- 1.Sc1–c2 (king-step; Q=1 preserved).
- 2...Kb6–a6 (Black hugs corner).
- 2.Sc2–b7 (queen-move along diagonal c2-b7? c2, b3, a4 — not b7.
  Actually c2-h7 diagonal: c2, d3, e4, f5, g6, h7. b7 is not on that
  diagonal. The c2 diagonals are c2-a4, c2-h7, c2-b1, c2-d1. None
  reach b7).

The geometry needs more care. Reposition: white Solstice on a1.

```
8 . . . . . . . .
7 . . . . . . . .
6 . . k . . . . .
5 . . . . . . . .
4 . . . . . . . .
3 . . . . . . . .
2 . . . . . . . .
1 S . . . . . . .
  a b c d e f g h
```

a1 to b7 is on the a1-h8 diagonal? a1, b2, c3, d4, e5, f6, g7, h8 — no
b7. The Solstice's queen-move from a1 reaches the entire 1st rank, the
entire a-file, and the a1-h8 diagonal.

Useful queen-targets from a1: a7 (reaches via a-file). From a7 the
Solstice (now a king-mover after spending Q) attacks a6, b6, b7, b8,
a8 — and a6 includes the black king on a6 (if the king has moved
there).

Helpmate in 3 sketch:

- 1...Kc6–b6 (Black hugs).
- 1.Sa1–a2 (king-step; Q=1).
- 2...Kb6–a6 (Black hugs further).
- 2.Sa2–a7 (queen-move; Q=0 now). Black king on a6 is now adjacent to
  Solstice on a7.
- 3...Ka6–a5 (Black moves out of adjacency — but to where? a5 is empty;
  legal). Actually wait, in a helpmate Black *cooperates* to be mated.
  Black would not move *away* from mate.

In a helpmate, Black plays the move that *leads to* mate, not the move
that escapes. Black's "cooperation" can include suicidal-looking moves,
constrained by legal-move requirements (Black cannot move into self-mate
on its own move — Black moves are legal chess moves, but Black chooses
the move that helps White's mating plan).

Continuing:

- 3...Ka6–a5 — does this help White? Solstice on a7. White's move 3
  must mate. Solstice on a7 attacks a6, b6, b7, b8, a8. Black king on
  a5: not attacked. Is there a Solstice king-move that mates the
  king on a5? Sa7–a6 attacks a5 (yes) but Black king on a5 escapes to
  b5 (empty, not attacked by Solstice on a6 — a6 attacks a5, a7, b5,
  b6, b7. Yes b5 is attacked! So a5 has no a5→b5 escape) or b4 (a6
  doesn't attack b4) — yes, b4 escape. Not mate.

The position needs another piece to cover b4 and b5. A composition with
just the white Solstice and the two kings rarely produces a clean
helpmate in 3 — the lone king cannot cover enough squares.

**Composed kernel.** A practical helpmate-in-3 with a Solstice requires:
- One white Solstice (Q=1 at start).
- Optionally one other white piece (e.g. a rook in the corner).
- A black king.
- Sometimes a black pawn to constrain Black's "cooperation" to one move.

The composer's *art* is to find the unique starting position where:
- The mating geometry on move 3 demands the Solstice at a specific
  square X.
- X is reachable from the Solstice's starting square in 3 king-moves
  XOR 2-king-moves + 1-queen-move XOR 1-king-move + 1-queen-move +
  1-king-move XOR 1-queen-move + 2-king-moves.
- *Exactly one* of those four orderings is consistent with the black
  cooperation (Black's legal moves at each ply, none of which produce
  premature stalemate or check from a different angle).

The four orderings correspond to spending the queen-move on White's
move 1, 2, 3, or not at all. The cooked-test is whether multiple
orderings work; the composition is sound only if exactly one does.

This is the *kind of problem* helpmate composers love. The Solstice's
mechanic guarantees the move-order question has bite.

## Compositional notes

- **Budget the queen-move.** Declare in the stipulation which move
  number spends it, or leave it implicit and let solvers discover.
  Implicit is more elegant.
- **The four-way move-order branch.** A helpmate in 3 has four
  candidate queen-spend positions (move 1, 2, 3, or never). Compose so
  that exactly one is sound. The other three must fail to specific,
  identifiable defences.
- **Pair with constrained Black.** Helpmates need Black to have few
  legal moves. Pin Black's king against a corner or a pawn; use Black
  pawns one square from promotion to force pawn-promotion-into-mate
  cooperations.
- **Saving the queen-move for check escape.** Selfmates can also use
  the Solstice: White is forced into check, and the only legal escape
  is the queen-move. The composer engineers a position where the
  Solstice's queen-move is the unique forced reply.
- **Two Solstices.** Per side is doable but increases the move-budget
  combinatorics. Two queen-moves available, 8 spend-orderings in a
  3-move helpmate — soluble for a brave composer.

## Where it shines

- Helpmates in 2, 3, 4 with one Solstice per side.
- Studies where the queen-move must be saved for an endgame
  breakthrough.
- Selfmates where the queen-move is the unique forced reply to a
  Black-induced check.

## Where it's awkward

- The Solstice without its queen-move is just a king. After Q=0 the
  piece is functionally weak; long problems where the queen-move is
  spent early can drag.
- The queen-move's *availability* must be tracked in FEN. A position
  cannot be analysed without knowing Q for each Solstice.
- Players in actual games will hoard the queen-move forever; the
  piece's compositional shine doesn't translate to playable balance.

## Engine dependencies

- Per-piece state flag (Q=0 or Q=1).
- Movement function reading the flag: king-moves always available;
  queen-moves available only when Q=1.
- Move-completion hook: if the move was a queen-style move (one that
  could not have been a king-move — i.e. distance > 1), set Q=0.
- FEN encoder/decoder for the flag.

## New features required

- Per-piece boolean flag on Solstice: `queen_available: bool`.
- Movement function returning `king_moves(square) ∪ (queen_moves(square) if queen_available else ∅)`.
- Move-hook: post-move, if `Chebyshev_distance(from, to) > 1`, set
  `queen_available = false`.
- FEN payload extension `Q=0|1`.

## FEN encoding

Solstice piece payload:

```
(P=O,C=W,Q=1)        # white Solstice, queen-move available
(P=O,C=W,Q=0)        # white Solstice, queen-move spent
```

Letter `O` for Solstice (S is Skibidi; O is unused).

Default `Q=1` if omitted (composers may rely on the default for fresh
problem-start positions).

## Open questions

- **Castling.** Does the Solstice castle? It is not the actual king
  (the actual white king is a separate piece). No castling.
- **Queen-move that is also a king-move.** A move from c1 to c2 is
  reachable both as a king-move (one step north) and as a queen-move
  (one step north). Does this consume Q? Recommendation: **no**, the
  consumption rule is "Chebyshev distance > 1" — single-square moves
  are always king-moves and never spend Q.
- **Queen-move that is a one-square diagonal slide.** Same as above:
  one-square diagonal is reachable by king; does not consume Q.
- **Promotion into Solstice.** Can a pawn promote to a Solstice?
  Probably yes; the promoted Solstice starts with Q=1 (a fresh
  queen-move budget). This makes promotion-to-Solstice strategically
  rich (you choose between a full queen now and a king-with-queen-move
  later).
- **Multiple Solstices per side.** Allowed; each tracks Q independently.
- **Stipulation phrasing.** Should the helpmate stipulation declare
  the queen-move must be spent? Or is it implicit (the composer
  ensures it). Either works; implicit is more elegant.
