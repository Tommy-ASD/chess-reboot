# Antipode

> A piece that mirrors every move of its opposite-colored twin one
> tempo later. When the twin is captured, the Antipode freezes
> permanently and a `frozen-at-ply-N` marker is etched into its FEN
> payload as evidence of when the twin died.

## Inspiration

Smullyan's puzzles frequently turn on the question "when was this
piece captured?" — a question with no answer from the position alone
unless some surviving piece bears witness. The Antipode is a
dedicated witness: it does nothing of its own volition, only echoes
its twin, and the *moment its echo stops* is recorded indelibly. The
date of death is on the board.

## Mechanic

An Antipode is paired with a twin of the opposite color. Both pieces
share a base move profile (the simplest choice is "moves as a
knight," though any profile works; the canonical pair is white
Antipode + black twin or vice versa).

On the twin's turn, the twin moves normally (subject to the
constraints of its own piece type). On the Antipode's owner's next
turn, the Antipode must move in the same *displacement vector* as
the twin's most recent move (i.e., `(dx, dy)` matches). If the
displacement is illegal for the Antipode at its current square
(blocked, off-board, friendly fire), the Antipode forfeits its turn
— the player must play some other move that turn, and the
displacement-debt is dropped.

The Antipode does not have free moves. Its only legal motion is the
echo of its twin's last move. If the twin has not moved yet (start
of game) or has died, the Antipode cannot move at all.

When the twin is captured, two things happen:

1. The Antipode is *frozen* — it can never move again.
2. The Antipode's FEN payload acquires `F=N`, where N is the ply
   number at which the twin was captured. This marker is permanent.

The frozen Antipode is a normal target — it can be captured. But it
cannot move, period. Other pieces may interact with it (it blocks
slider paths, can be attacked, etc.). It just sits.

## The deduction it enables

A position is presented: white Antipode on c4 with payload
`F=14`. The puzzle is to determine the move number on which the
Antipode's black twin was captured.

The marker `F=14` directly states: the twin was captured on ply 14.
Ply 14 of a standard game is *White's 7th move* (plies 1-2 are
moves 1, 3-4 are moves 2, ..., 13-14 are move 7). So the twin was
captured by a White piece on White's 7th move.

This pins:

1. The capture was made by a White piece. (The twin is Black; only
   White can have captured it.)
2. The capture occurred precisely on the seventh White move. Not
   earlier, not later.
3. Up through ply 13 (Black's 7th move), the twin was still alive
   — therefore the Antipode was still echoing. The Antipode's
   moves on White's 1st through 6th moves were echoes of the twin's
   plies 1 (twin's move 1) through 6 (twin's move 6). The Antipode
   has executed exactly six echo-moves.
4. The Antipode currently sits on c4. Its starting square is known
   (the puzzle's initial setup). The displacement from start-square
   to c4 is the *sum* of the six echo-vectors. Each echo-vector is
   the twin's corresponding move. So the *path the twin walked
   through plies 1-6* is uniquely determined by the c4 position
   plus the Antipode's starting square plus a small combinatorial
   check on the order of the six echo-vectors.
5. From the twin's reconstructed path, the puzzle composer can
   work backward to constrain other facts: the squares the twin
   visited, the captures *it* may have made, etc. The Antipode's
   current location *encodes the trajectory of a piece that no
   longer exists.*

This is the central trick. The Antipode is a memorial. Its position
in space records where its dead twin walked. The `F=14` marker
times the death. Together they reconstruct a piece's entire
biography.

## Why it's interesting

The Antipode does not act on its own. It is a pure mirror — and the
moment the mirror cracks, the crack records the *time* of the
breakage. This dual role (mirror in life, witness in death) is what
makes it forensic: it offers two distinct kinds of evidence
depending on whether its twin is alive or dead.

A live Antipode pair is an *ongoing* historical record — the
Antipode's current square is the running sum of all the twin's
displacements. A dead twin's Antipode is a *frozen snapshot* — the
biography is sealed, and the timestamp is etched.

## Example puzzle setup

```
. . . . . . . k
. . . . . . . .
. . . . . . . .
. . . . . . . .
. . A . . . . .
. . . . . . . .
. . . . . . . .
K . . . . . . .
```

White Antipode `A` on c4 with `F=14`. White king on a1, Black king
on h8, no other pieces.

The puzzle: the Antipode's starting square was c1 and it is a
"knight-mover" Antipode. How many of the twin's knight moves are
consistent with the Antipode arriving at c4 after exactly six
echoes? Reconstruct the twin's path.

Displacement from c1 to c4 is `(0, +3)`. Six knight-moves summing
to `(0, +3)` is a constrained enumeration. There are exactly a
handful of solutions; the puzzle's other constraints (pieces that
were captured along the way, squares the twin must have visited)
prune to one.

## Where it shines

- **Time-of-death puzzles.** "When did Black's bishop die?" The
  `F=N` marker is the literal answer.
- **Path-reconstruction puzzles.** Given the Antipode's final
  square and starting square, the twin's full trajectory is
  encoded.
- **Tempo-budget puzzles.** Because the Antipode forfeits its turn
  when echoing is illegal, the Antipode's history *also* records
  forfeitures — a careful composer can use the count of free
  Antipode moves (vs total ply count of its side) as a budget
  constraint.

## Where it's awkward

The "echo or forfeit" rule entangles the Antipode's owner's
choices. The owner may be forced into a turn where they have to
move *something else* because the Antipode can't echo. This is a
real gameplay restriction with retrograde flavor — but it makes
the Antipode an awkward fit for any normal game; it's a puzzle
piece.

A live Antipode does not yet have its `F` marker, so its position-
plus-starting-square is the *only* evidence of the twin's path.
Compositions hinging on a live Antipode must also pin the starting
square in the puzzle preamble; otherwise the evidence is
underdetermined.

## Engine dependencies

- Pair linkage (same as Tether — an Antipode is tied to a specific
  twin).
- Per-piece payload system.
- Access to the most recent move's displacement vector, queryable
  by the Antipode's owner at move-generation time.
- Engine knowledge of the global ply counter.

## New features required

- **Echo-move generator.** A piece-trait method that, given the
  twin's last move, returns the single legal echo destination (or
  the empty set if blocked).
- **Capture-of-twin hook.** When a piece is captured, check if it
  is a twin in an Antipode pair. If so, freeze the Antipode and
  stamp `F=<current_ply>` on it.
- **Frozen-piece state.** A frozen Antipode is a new
  immutable-piece state; engine must respect it (move generator
  returns empty for the Antipode forever).
- **Twin-died-but-no-Antipode case.** If the Antipode itself is
  captured before its twin, the twin continues to move normally
  (no echo needed) and the Antipode is just gone. No special case.

## FEN encoding

Piece payload keys: `T` (pair designator: pair-id integer matching
the twin) and optionally `F` (frozen-at-ply integer; present only
if the twin is dead and the Antipode is frozen).

Examples:

```
(P=AN,T=1)              — live Antipode, paired with twin id 1
(P=AN,T=1,F=14)         — frozen Antipode, twin died on ply 14
(P=AN,T=2,F=27)         — frozen Antipode, second pair, twin died on ply 27
```

The pair-id is needed because the engine may support multiple
Antipode pairs (one per side, or even multiples). Each pair has a
unique id assigned at game start or at editor placement.

The `F` flag's absence is itself evidence: a live Antipode has no
`F`; a dead-twin Antipode always has `F`. Round-trip rule: emit `F`
only when frozen; never emit `F=0` as a placeholder.

## Open questions

- **What if the twin's last move was a special move (castle, en
  passant)?** Echo of castling is ill-defined for a non-king
  Antipode. Decision: special moves do not generate echo-debt; the
  Antipode forfeits that turn.
- **What if the Antipode is forced to move into a check on its
  echo?** The echo is illegal (king-safety rule overrides), and the
  Antipode forfeits the turn. Echo-debt is dropped, not deferred.
- **Multiple pairs on the same side.** Allowed. Each has its own
  pair-id and `F` flag.
- **Can the engine echo a *capture* by the twin?** Yes — the
  Antipode's echo lands on the corresponding square; if that square
  is occupied by an enemy piece, the Antipode captures it. If
  occupied by a friendly piece, the echo is illegal and the
  Antipode forfeits.
- **Antipode promotion.** Possible if the echo is a pawn-promotion
  move and the Antipode is pawn-like. Out of scope for the
  canonical knight-mover Antipode.
