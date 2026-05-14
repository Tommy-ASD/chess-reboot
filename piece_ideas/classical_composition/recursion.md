# Recursion

> A piece that copies any adjacent piece's identity to move as it,
> permanently becoming that type; series-movers chain seven moves into
> a planned metamorphosis.

## Inspiration

The **series-mover** tradition — one side makes N consecutive moves
(the other side does not move) and on the final move delivers
checkmate. Classical series-movers are puzzles of trajectory and
move-economy: each tempo costs something, and the geometry must be
exact.

The Recursion makes the *piece itself* the variable. The composer plans
a chain of identity-copies — pawn becomes knight becomes bishop becomes
rook becomes queen — that the Recursion threads through over seven
consecutive moves. Each link of the chain requires an adjacent piece
of the correct type at the correct moment. The Recursion arrives at
the mate as exactly the piece needed, having been every previous piece
in turn.

## Mechanic

The Recursion is a piece type with a *current identity*. On
construction (board setup or FEN) it has identity `Empty` — meaning it
has no movement of its own and cannot move on its turn.

On any turn the Recursion is to move, the player must:

1. Choose an **adjacent piece** P (within one king-step, regardless of
   colour, friendly or enemy).
2. The Recursion's identity becomes P's piece-type for this move and
   permanently thereafter (FEN-tracked).
3. The Recursion moves as P moves (using its current square and the
   movement rules of P's type), with normal capture/check rules. Note:
   the Recursion does not move *as the same piece object* — it moves
   from its own square, with P's *type's* movement, not P's specific
   piece state. (E.g. if P is a Solstice with Q=0, the Recursion
   inherits the Solstice *type* — and starts with Q=1 from the
   Solstice's "fresh" state? See Open questions.)
4. If no adjacent piece exists, the Recursion *cannot move*. The
   Recursion has no fallback movement.

After the move, the Recursion has the new identity permanently. Its
next turn, it may re-copy from a new adjacent piece (changing identity
again) and move per that new identity.

Identity-copy is **per move**, not per piece. The Recursion can be a
pawn on turn 1, a knight on turn 2 (copying from an adjacent knight at
its new location), a bishop on turn 3, etc. Each identity-change is
permanent until the next change.

Identity is a piece-type tag — `Pawn`, `Knight`, `Bishop`, `Rook`,
`Queen`, `King` (problematic — see Open questions), `Goblin`, `Skibidi`,
etc. Any piece type in the variant.

State in FEN: `I=<type-letter>` where the letter follows the standard
piece-letter convention (`P`, `N`, `B`, `R`, `Q`, custom letters for
fairy pieces). The Empty identity is `I=_` or absent.

## Why it's interesting (compositionally)

The Recursion enables a *trajectory-with-metamorphosis* puzzle class:

**Series-mover in 7.** White makes 7 consecutive moves; Black does not
move; White's 7th move mates. The composer arranges the position so
that:

- The Recursion's mating move on turn 7 must be a queen-move (e.g. to
  cover a specific mating diagonal-and-rank intersection).
- The Recursion reaches the queen-square only via a chain: pawn (turn
  1) → knight (turn 2) → bishop (turn 3) → rook (turn 4) → bishop again
  (turn 5) → knight again (turn 6) → queen (turn 7).
- At each step, an adjacent piece of the required type must be
  available — and **only** that type, because if other types were
  available the solver could shortcut.

The composition's craft is to populate the board with auxiliary pieces
(friendly or enemy — adjacency doesn't care) such that at each
square-and-turn the Recursion arrives at, exactly one
identity-copy target exists.

## A worked problem

**Series-mover in 5.** White Recursion to play 5 consecutive moves;
Black does not move; 5th move mates.

Position:

```
8 . . . . . . . k
7 . . . . . . . .
6 . . . . . . . .
5 . . . . . . . .
4 . p . n . . . .
3 . . . . . . . .
2 . . R . . . . .
1 . X . . . . . K
  a b c d e f g h
```

White: King h1, Rook c2, Recursion b1 (identity `_`, empty).
Black: King h8, Pawn b4, Knight d4 (these are *White's* auxiliary pieces
   — actually let me restate them as White's to be consistent with
   "auxiliary pieces" being friendly. Black has only the king h8.)

Re-stated:
- White: King h1, Rook c2, Recursion b1 (id=_), Pawn b4, Knight d4.
- Black: King h8.

The Recursion's plan: become pawn, then knight, then bishop, then rook,
then queen — wait, that's 5 identities. A series-mover in 5 has the
Recursion making 5 moves; each move is one identity-copy. Let me trace
a plausible chain:

Turn 1: Recursion on b1, adjacent pieces are: c2 (rook, white), and
nothing else immediately adjacent. b1's neighbours: a1, a2, b2, c1, c2.
Rook on c2 is adjacent. **Recursion copies Rook**, becomes a rook,
moves like a rook.

Where? The Recursion needs to traverse toward the mating geometry near
h8. A rook move from b1: b1-b3 (forward 2), b1-b4 (forward 3, but b4
has the white pawn — same colour, blocked), b1-b8 (entire b-file — but
b4 pawn blocks at b4). So along b-file: b2, b3. Along 1st rank: a1, c1
(empty), d1, e1, f1, g1, h1 (king, blocked at g1). So rook destinations:
a1, b2, b3, c1, d1, e1, f1, g1.

Choose b1→b3 (rook move; identity becomes `Rook` permanently if not
changed).

Turn 2: Recursion on b3 with identity `Rook`. b3's neighbours: a2, a3,
a4, b2, b4 (pawn, white), c2 (rook, white), c3, c4. Adjacent pieces:
pawn b4, rook c2.

Choose to copy **Pawn**. Recursion is now identity `Pawn`. As a pawn,
the Recursion moves forward (one square) — but wait, the Recursion is
white and starts at b3 (not the 2nd rank), so it doesn't have a double-
push option. It can move b3→b4 (blocked by friendly pawn) or capture
diagonally (a4, c4 — both empty, so no capture). No legal move.

Hmm — the Recursion-as-Pawn from b3 has no legal move. The composer
needs to plan the adjacency-and-identity chain so the chosen identity
always has a legal move from the current square.

Re-plan. Turn 2: copy **Knight** instead. But there is no knight
adjacent to b3. The knight is on d4, not adjacent to b3 (b3 to d4 is
two file + one rank — knight's distance is not adjacency-by-king).

So at b3, the only adjacent pieces are Pawn b4 and Rook c2. Identity
choices: Pawn (no legal move from b3) or Rook (already the Recursion's
current identity, but copying doesn't change anything — copy is "set
identity", no-op if same).

The Recursion's plan should re-route through a square adjacent to the
knight. Reposition the knight to c3 (so that from b2, the knight is
adjacent).

Revised position:

```
8 . . . . . . . k
7 . . . . . . . .
6 . . . . . . . .
5 . . . . . . . .
4 . . . . . . . .
3 . . N . . . . .
2 . . R . . . . .
1 . X . . . . . K
  a b c d e f g h
```

White: King h1, Rook c2, Knight c3, Recursion b1 (id=_).
Black: King h8.

Recursion's plan: copy Rook, move to b2 (or stay around there);
re-copy Knight from c3-adjacency; move as Knight to some forward
square; repeat.

Turn 1: Recursion b1 adjacent to Rook c2. Copy Rook; move b1→b2.
Recursion now on b2, identity Rook.

Turn 2: Recursion b2 adjacent to: Rook c2, Knight c3, and various
empty squares. Copy Knight; move as Knight from b2. Knight moves
from b2: a4, c4, d1, d3. (8 knight-squares from b2 minus off-board:
a4, c4, d1, d3.) Choose b2→d3.

Turn 3: Recursion d3, identity Knight. d3 neighbours: c2 (rook), c3
(knight), c4, d2, d4, e2, e3, e4. Adjacent pieces: rook c2, knight c3.
Copy Rook; move as Rook from d3. Rook destinations along 3rd rank:
a3, b3, c3 (knight, blocked), e3, f3, g3, h3. Along d-file: d1, d2,
d4, d5, d6, d7, d8.

Choose d3→d8: but that requires the d-file to be empty. It is (no
pieces on d4..d8 in this position). d3→d8.

Recursion on d8, identity Rook. d8 attacks the entire 8th rank and the
d-file. Black king on h8: along the 8th rank from d8, the rook attacks
e8, f8, g8, h8. **Check.**

But this is only the 3rd move of a series-mover in 5 — the king is
checked too early. In a series-mover, the side moving does **not** put
the opposing king in check until the final move (intermediate checks
are illegal in standard series-mover conventions). So d3→d8 is illegal
on move 3.

Re-plan. On move 3, move the Rook-Recursion somewhere non-checking but
along the route. d3→d7 (also along the d-file). d7 attacks d-file and
7th rank. Does d7 attack h8? d7 along 7th rank: e7, f7, g7, h7 — and
the king is on h8, not h7. No check from d7. Good.

Turn 4: Recursion d7, identity Rook. d7's neighbours: c6, c7, c8, d6,
d8, e6, e7, e8. No pieces adjacent (the board is empty around d7).
**Recursion cannot copy any identity. No legal move.**

The Recursion has stranded itself. The composer needs to pre-place an
auxiliary piece adjacent to d7 (or wherever the chain pauses) to allow
the next copy.

Add a white pawn on c7 (just for the demonstration). Then on turn 4,
adjacent to d7 we have pawn c7. Copy Pawn; move as Pawn from d7.
A white pawn on d7 can push to d8 (promotion!) or capture diagonally
on c8 or e8 (both empty, no capture). Push d7→d8=Q. But that's a
promotion; the identity-change rules don't cover promotion (composer
decision needed). Setting that aside: assume Pawn-id Recursion can
push to d8 and promotes to a Queen by normal rules.

Recursion-as-Queen on d8 attacks the entire 8th rank, the d-file, and
both diagonals from d8. The black king on h8 is attacked along the 8th
rank. Is it mate? Black king escape squares: g7, g8, h7. g8 is on the
8th rank (attacked by queen). g7 is on the d8-g7 diagonal? d8 to g7
is +3 file, -1 rank — not a queen ray. h7 is on the d8-h7 diagonal?
d8 to h7 is +4 file, -1 rank — not a diagonal (a queen ray needs
equal file and rank differences). So h7 is safe… unless something else
attacks h7.

We need another piece to cover h7. Add a white pawn on g6 — attacks
f7 and h7. Now the king on h8 with queen on d8: escapes g8 (attacked by
queen along the 8th rank), g7 (not attacked by queen — d8 to g7 is
not a queen ray; but g6 pawn attacks h7 and f7, not g7), h7 (attacked
by g6 pawn).

g7 is unattacked. Black escapes Kh8→Kg7. Not mate.

This is the *honest* shape of series-mover composition: tight constraints,
multiple defensive escapes, the composer iterating positions until
exactly one piece-chain mates. The Recursion makes this iteration
*about identity*, not just trajectory — every position-adjustment shifts
which identity-copies are available.

**Conclusion of worked problem.** A clean series-mover-in-5 with a
Recursion requires careful auxiliary-piece placement so that:
1. At each turn, exactly one identity-copy target is adjacent.
2. That identity has exactly one move toward the mating goal.
3. No intermediate check is given.
4. The final move achieves mate.

The Recursion's mechanic guarantees that *only one* identity-chain
solves the problem — alternative chains either have no adjacent target
or lead to a non-mating geometry.

## Compositional notes

- **Plan the identity chain backward from mate.** Start with what
  piece-type delivers mate; work backwards to determine the
  Recursion's identity at each step; place auxiliary pieces to
  provide the necessary adjacency.
- **Adjacency is king-distance.** Eight neighbour-squares at most.
  Auxiliary pieces must be placed precisely.
- **Auxiliary pieces are tempo-free.** In a series-mover only the
  Recursion (or the moving side) takes turns; auxiliary pieces sit
  silent. This is the Recursion's playground.
- **The intermediate-check ban.** In standard series-movers, Black is
  not in check between move 1 and move N-1. The composer must route
  the Recursion through *non-checking* squares — adding constraint to
  the identity chain.
- **Enemy-adjacency.** Copying from an *enemy* adjacent piece is legal.
  This allows the Recursion to use enemy material for its
  metamorphosis — wonderful for problems where Black's lone king is
  the only adjacent piece, forcing the Recursion to copy King and move
  as a king for one turn.
- **No-fallback rule.** A Recursion with no adjacent piece cannot
  move. Series-mover problems where the Recursion is stranded for a
  turn lose; the composer must guarantee adjacency at every turn.

## Where it shines

- Series-movers in 5-12 moves with identity-chain trajectories.
- Helpmates where Black's cooperation includes positioning its pieces
  to be adjacent to the Recursion at key turns.
- Endgame studies where the Recursion's path through identities is
  the puzzle's whole content.

## Where it's awkward

- The Recursion needs constant adjacency, which constrains its
  trajectory severely. Position the auxiliary pieces with care.
- Identity-copy from an Empty Recursion (its initial state) is fine,
  but identity-copy from another Recursion is undefined — see Open
  questions.
- Promotion of a Recursion-as-Pawn is messy (does the promoted piece
  inherit "Recursion-ness"?). See Open questions.
- The piece's mechanic is so powerful that play-balance is bad — the
  Recursion can become a Queen anywhere it can get adjacent to one,
  so it's near-omnipotent. Strict composition piece.

## Engine dependencies

- Per-piece identity tag (current piece-type).
- Movement function dispatch: read identity; call the appropriate
  movement function.
- Adjacency lookup (already common — king moves).
- FEN encoder/decoder for identity tag.

## New features required

- New piece type "Recursion" with per-piece state `identity:
  PieceTypeOpt` (None = Empty, Some(T) for any type T).
- Movement function: if identity is Some(T), enumerate as the
  *type T*'s movement; if None, no moves.
- Pre-move identity-copy step: at move-construction, player chooses
  an adjacent piece P; the engine asserts P is adjacent; sets the
  Recursion's identity to P's type for the duration of the move and
  permanently afterward.
- FEN payload `I=<type-letter>` (Pawn=P, Knight=N, Bishop=B, Rook=R,
  Queen=Q, King=K?, fairy letters as defined).
- Default `I=_` (Empty) if omitted.

## FEN encoding

Recursion piece payload:

```
(P=X,C=W,I=_)        # white Recursion, no identity yet
(P=X,C=W,I=N)        # white Recursion currently a Knight
(P=X,C=W,I=Q)        # white Recursion currently a Queen
```

Letter choice: `X` for Recursion (memorable; X for unknown). Confirm.

## Open questions

- **Copying a Recursion.** If two Recursions are adjacent and one
  copies the other, what does it copy? The other's *current
  identity*? Recommended: yes — Recursion-on-Recursion copy yields
  the target's current identity. This makes meta-chains possible.
- **Copying the King.** Can a Recursion become a King? The King is a
  unique royal piece; a copy might create a second king (illegal in
  standard chess). Recommended: copy the King's *movement* (one-square
  any direction) but the Recursion is not royal — capturing it does
  not end the game.
- **Pawn promotion of Recursion-as-Pawn.** When a Recursion-as-Pawn
  reaches the promotion rank, what happens? Two options: (a) it
  promotes by normal rules to a queen/rook/etc., becoming that piece
  type permanently (no longer a Recursion); (b) it promotes by
  changing its `identity` to the chosen promotion piece type but
  remains a Recursion (can re-copy on future turns). Recommended: (b),
  consistent with the "always a Recursion underneath" model.
- **Recursion as auxiliary piece.** Can a Recursion adjacent to another
  Recursion be the source of its identity-copy? Yes; see above.
- **Identity persistence vs reset on capture.** When the Recursion
  captures a piece, the captured piece is removed (normal rules).
  The Recursion's identity is unaffected — captures don't reset it.
- **Initial-game Recursion identity.** Set up by FEN. A puzzle
  starting with `I=_` requires the Recursion's first move to copy from
  an adjacent piece. A puzzle starting with `I=R` lets the Recursion
  move as a Rook immediately on turn 1.
- **Adjacency includes diagonals.** Eight neighbours, king-distance.
  Confirmed.
- **Adjacency through walls / non-walkable squares.** The Recursion's
  adjacency for copy purposes is the eight king-neighbours regardless
  of walkability between them. Confirm with engine behaviour.
