# Antiphon

> Two paired pieces that each may only move to the 180° board-rotation
> of its twin's current square; symmetry becomes the engine of
> cooperation.

## Inspiration

The musical *antiphon* — call and response, two voices that mirror
across silence. In chess composition, **mirror symmetry** is a
classical compositional theme: a position where Black's geometry
mirrors White's, and the solution exploits the broken or restored
symmetry of the mating geometry. The Antiphon makes the symmetry
*literal* — two pieces whose legal moves are reflections of each
other's current position.

## Mechanic

Antiphons exist in **pairs**. A pair consists of two pieces (any
colours — same colour or opposite, see Open questions) that are
designated as each other's twin. The pair-link is FEN-tracked.

On its turn, each Antiphon may only move to **one specific square**:
the 180°-rotation around the board's geometric centre of where its
twin currently sits.

For an MxN board with file-indices 1..M and rank-indices 1..N:
- The rotation of square (file=f, rank=r) is (M+1-f, N+1-r).

So on an 8x8 board, the rotation of e4 is d5 (file e=5 → 9-5=4=d;
rank 4 → 9-4=5).

An Antiphon's *legal move set* contains at most one square — the
rotation of its twin. The move is legal iff:
1. The target square is empty or contains an enemy piece (normal
   capture rule).
2. The target square is on the board.
3. The path is not relevant — Antiphons *jump* like a knight (they
   teleport directly).

If the rotation-square is occupied by a friendly piece, or off-board,
the Antiphon has no legal move that turn.

When the Antiphon moves, its twin's *next* move's target updates
accordingly. The pair must coordinate.

If one Antiphon of a pair is captured, the surviving Antiphon is
**immobile** for the rest of the game (no twin → no rotation-target).
The surviving piece is still a board obstruction; can be captured;
can give check via threat-set? See Open questions.

State in FEN: pair-link via shared pair-ID. Each Antiphon carries
`PR=<n>` where `<n>` is a small integer identifying the pair. Two
Antiphons with the same `PR` value are paired.

## Why it's interesting (compositionally)

The Antiphon's mechanic makes **mirror-symmetric problems** the natural
class. Specifically:

- **Helpmates** where Black's cooperation is making its own pieces
  symmetric to White's. The Black Antiphon's move *is the move that
  enables White's mating Antiphon move* — they call and respond.
- **Symmetric-key problems** where the solving move sets up a
  rotational geometry, and the obvious symmetric defence fails
  because the *Antiphon's* rotation doesn't match.
- **Forced-pair selfmates** where White's only legal Antiphon move
  is to the rotation of Black's Antiphon, and Black's only response
  delivers mate. The composer engineers the pair-positions so that
  no other move-orderings exist.

The mechanic also enables **rotational zugzwang** — Black to move,
every Black move shifts its Antiphon's position (or the threat against
it), which by rotation shifts White's Antiphon's legal target.
Black exhausts the rotation-space.

## A worked problem

**Helpmate in 2.** Pair-colour convention: both Antiphons on same
pair-ID, one White, one Black. Both kings, one Antiphon each side, no
other pieces.

```
8 . . . . . . . k
7 . . . . . . . .
6 . . . . . . . .
5 . . . . . . . .
4 . . . . . . . .
3 . . . a . . . .
2 . . . . . . . .
1 K A . . . . . .
  a b c d e f g h
```

White: King a1, Antiphon b1 (pair PR=1).
Black: King h8, Antiphon d3 (pair PR=1).

The pair links the white Antiphon b1 and the black Antiphon d3.

White Antiphon b1's legal target = rotation of d3 = (9-4, 9-3) =
(5, 6) = e6. Black Antiphon d3's legal target = rotation of b1 =
(9-2, 9-1) = (7, 8) = g8.

White Antiphon can move b1→e6 (jump, square e6 is empty).
Black Antiphon can move d3→g8 (jump, square g8 is empty — but it's
adjacent to the black king on h8; does the black king's adjacency
matter? Only if g8 puts the king in check from the white side. The
white pieces are King a1 and Antiphon (will be at e6 if it moves
first). Antiphon-at-e6's threats: see below.).

Stipulation: Helpmate in 2. Black moves first; both sides cooperate;
Black is mated on White's 2nd move.

What does an Antiphon *threaten*? An Antiphon attacks the square it
*could move to*, which is the rotation of its twin's current square.
The Antiphon's attack set is its move set (at most one square).

So if the white Antiphon is on b1 and the black Antiphon is on d3, the
white Antiphon attacks e6 (rotation of d3). If a black piece sits on
e6, the white Antiphon attacks it.

Solution attempt:

- **1...Antiphon d3–g8** (Black's first move; jumps to g8 = rotation
  of b1). Black Antiphon now on g8. Black king on h8 is now adjacent
  to its own Antiphon on g8 — no self-check problem.
- **1.Antiphon b1–???.** White Antiphon's legal target = rotation of
  g8 = (9-7, 9-8) = (2, 1) = b1. **White Antiphon's only legal move
  is to stay where it is — but staying isn't a move.** White has no
  legal Antiphon move.

White's other pieces: only the king a1. King moves: a2, b2. Choose
a2.

- **1.Ka1–a2.** King now on a2.

- **2...Antiphon g8–???.** Black Antiphon's legal target = rotation
  of b1 = g8 (same). Antiphon already there. No legal move.
  Black's other piece: king h8. King moves: g7, h7, g8 (occupied by
  own Antiphon). Choose Kh8–h7. But this is Black's *2nd* move; in
  helpmate-in-2, Black makes 2 moves and White makes 2 moves
  alternately, with White's 2nd move delivering mate. So:

  Move sequence: 1...B 1.W 2...B 2.W#

- **2...Kh8–h7** (Black's second move).

- **2.???**. White must mate on move 2. White's pieces: King a2,
  Antiphon b1.

  White Antiphon b1's legal target = rotation of g8 = b1. No move.
  White King a2 moves: a1, a3, b1 (occupied by Antiphon), b2, b3.
  No mating king-move (Black king on h7 is far away).

This sketch does not produce mate in 2. The position is too open.

**Revised: place the black king tighter, give the Antiphon a tactical
role.** Composing helpmates with Antiphons requires the rotation-
targets to land *exactly* on the squares needed for the mating
geometry. The composer chooses the Black Antiphon's starting position
so that its rotation = a key square attacking the Black king's
position after Black's cooperation.

**Kernel.** Position the Black Antiphon at square Q such that Q's
rotation = the unique square attacking the Black king for mate. The
Black king is forced (by helpmate cooperation) to a square where the
White Antiphon (on Q's rotation) gives mate.

For example: place the Black king at b8. The White Antiphon must end
on a square attacking b8. The single square attacking b8 (other than
b-file/8th-rank adjacents which are not Antiphon-style) is — well, the
Antiphon attacks only one square (the rotation of its twin). So the
White Antiphon ends at b8's *neighbour* (no — Antiphons attack the
rotation, not a king-neighbour). Wait — re-read the mechanic. The
Antiphon attacks the square it can move to (= rotation of its twin).
That single square is what's attacked.

So for the White Antiphon to threaten the Black king, the **rotation
of the Black Antiphon must equal the Black king's square**. That means
the Black Antiphon must sit at the rotation of the Black king. If the
Black king is on b8 (file 2, rank 8), its rotation is g1 (file 9-2=7,
rank 9-8=1). The Black Antiphon must be on g1 for the White Antiphon
to attack b8.

But the *White Antiphon* must also be able to move to b8. Its
move-target = rotation of Black Antiphon's position. If Black Antiphon
is on g1, the White Antiphon's target = rotation of g1 = b8. So the
White Antiphon jumps to b8 — landing *on* the Black king and capturing
it. But capturing the king isn't "mate" in chess; check + no-escape is.

The Antiphon's attack on the Black king's square is *check*. The Black
king must escape, block, or capture the attacker. Captures are normal
(king can capture an Antiphon if it's adjacent and unprotected).

Refined: The White Antiphon delivers check by jumping to a square *adjacent
to* the Black king, where the rotation-target lands the Antiphon next to
(not on) the king. But Antiphons attack only their move-target — they
don't have adjacency attacks. So the Antiphon must jump *onto* the king's
attack-from-square… which is a king-adjacent square… but the Antiphon's
attack reach is only that one rotation-target. So the Antiphon must jump
to an adjacent-to-king square *and* be unable-to-be-captured for it to be
mate.

**A specific construction.** Place the Black king on h8. The Black
Antiphon must sit at the rotation of a square adjacent to h8 that the
Black king can't escape to or capture from. Adjacent squares to h8:
g7, g8, h7. The Antiphon's check-square must be one of these. Say g7.
Rotation of g7 = b2. So the Black Antiphon at b2 means the White
Antiphon's target = b2's rotation = g7.

So: Black Antiphon on b2, Black king on h8. White Antiphon needs to be
on a square from which it could move to g7 *as its move* — but the
Antiphon's move-target is always rotation-of-twin, which is g7
regardless of where the White Antiphon currently sits. So *any* current
position of the White Antiphon allows it to jump to g7. Good — only
the *target* matters, not the *origin*.

Place White Antiphon on a1 (out of the way). White king on, say, h6
(adjacent to g7? h6 is adjacent to g7 ✓ — covers the king's escape
squares from h8).

Position:

```
8 . . . . . . . k
7 . . . . . . . .
6 . . . . . . . K
5 . . . . . . . .
4 . . . . . . . .
3 . . . . . . . .
2 . a . . . . . .
1 A . . . . . . .
  a b c d e f g h
```

White: King h6, Antiphon a1 (PR=1).
Black: King h8, Antiphon b2 (PR=1).

Helpmate in 2. Solution:

Black plays a "free" move; White plays Antiphon to g7 (check); Black
makes another forced move; White's 2nd move mates.

But wait — the White Antiphon's move-target depends on the Black
Antiphon's *current* position. If Black moves its Antiphon, the
target changes.

- **1...Antiphon b2–???.** Black Antiphon's target = rotation of a1
  = h8. But h8 is the Black king's square — a Black Antiphon cannot
  move to its own king's square. So Black Antiphon **has no legal move**.

Black is in a kind of mini-zugzwang. Other Black pieces: king h8.
Black king moves: g7 (no attacker on g7 currently — White Antiphon
is at a1, not yet at g7; White king on h6 attacks g7, h7, h5, g6, g5
— so g7 IS attacked by White king). Kh8–g8 (h6 doesn't attack g8;
attacked by anything else? Not yet). Kh8–h7 (attacked by White king).
So Black king's only escape from h8 is g8.

- **1...Kh8–g8.**

White's first move:
- **1.Antiphon a1–???.** Target = rotation of b2 = g7. White Antiphon
  jumps a1→g7. Is g7 empty? Yes. Move legal. White Antiphon now on g7.
  The White Antiphon on g7 attacks the rotation of the Black Antiphon
  (still on b2) = g7. So the White Antiphon on g7 attacks itself? It
  attacks the square *it can move to*, which is g7 — its own square.
  Antiphons don't self-attack (a piece doesn't attack its own square).
  Hmm — but the Antiphon's "threat" is the square it could move to;
  it cannot move to its own square (no-op). So the Antiphon at g7 has
  no current move-target *if* the rotation-target equals its own
  square. Edge case — let's say this means the Antiphon has **no
  threat** in this configuration.

So White Antiphon on g7 threatens nothing. The Black king on g8 is
not attacked by the Antiphon. Is it attacked by the White king on h6?
h6 attacks g7, h7, h5, g6, g5 — not g8. Not check.

This sketch also fails. The problem is that the Antiphon's threat
depends on its twin's position, and any move by either Antiphon
shifts both threats. Composers must engineer the **paired final
configuration** — what both Antiphons look like at the moment of mate.

**The kernel motif (committed):**

For an Antiphon-mate to work, the final configuration must satisfy:

- Black Antiphon at square Q_B.
- White Antiphon at square Q_W.
- White Antiphon attacks rotation(Q_B) = the Black king's square.
- White Antiphon attacks itself? No — it attacks rotation(Q_B).
- For mate, rotation(Q_B) = Black-king-square AND the king cannot
  escape AND the Antiphon cannot be captured.

The White Antiphon's *position* Q_W is irrelevant to its threat
(threats depend only on Q_B). The White Antiphon must be on a square
*reachable* from its previous position by a legal Antiphon move.

This is the elegant constraint: the *threat geometry* depends only on
the *twin's* position, while the Antiphon's own position is constrained
by the rotation-history of moves.

A composer building an Antiphon helpmate plans:
1. Final Q_B, Q_W giving mate.
2. Walks backward: each move's Q_? was determined by the rotation of
   the other's *previous* position.

This produces a tightly-determined puzzle with one solution-chain.

## Compositional notes

- **The threat-source asymmetry.** An Antiphon's threat is a function
  of its *twin's* position, not its own. Compose around the twin's
  trajectory.
- **Stalemate by rotation.** If an Antiphon's rotation-target is
  occupied by a friendly piece, it has no legal move. Use this to
  create rotation-stalemates that force the side to play with non-
  Antiphon pieces.
- **Same-pair colour pairs.** Pairs can be same-colour or
  opposite-colour. Same-colour pairs make both pieces the same
  side's, useful for puzzles where one side has two coordinated
  Antiphons.
- **Self-capture impossible.** An Antiphon cannot jump onto its own
  twin (would require the twin's square = rotation of twin's square,
  which is only true for the centre — and 8x8 has no centre square).
- **Centre-symmetric special case.** On an even-dimension board there
  is no centre square. On an odd-dimension board (e.g. 7x7) the
  centre is a single square; if a twin sits on the centre, its
  rotation is itself, and the Antiphon's only legal move is to its
  twin's square (which is the centre, occupied by the twin) — so no
  legal move.

## Where it shines

- Helpmates in 2-4 moves with one Antiphon-pair across colours.
- Selfmates with mirror-symmetric mating geometries.
- Series-movers where the Antiphon-pair's trajectories trace
  symmetric paths.
- "Tries" exploiting rotation-target arithmetic — a candidate move
  that looks correct but the rotation-target lands the wrong piece on
  the wrong square.

## Where it's awkward

- The mechanic is hard to teach. New solvers must visualise 180°
  board rotation for every Antiphon move.
- One Antiphon-pair is the natural unit. Two pairs (PR=1 and PR=2)
  on one board quadruples the state to track.
- Lone surviving Antiphon (after twin is captured) is immobile dead
  weight. Composers can use this for endgame studies.
- Boards of unusual size (the engine's variable-board feature) re-
  parameterise the rotation. Confirm composer aware of board
  dimensions when setting up.

## Engine dependencies

- Pair-link state: each Antiphon carries a pair-ID.
- Movement function reading the twin's position via the pair-ID
  table; computes rotation; returns a single-element move set (or
  empty).
- Board-dimension awareness (rotation depends on board size).
- Threat function = movement function (Antiphons attack only their
  move-target).

## New features required

- New piece type "Antiphon" with state `pair_id: u8`.
- Per-game pair-table (or just iterate pieces to find the pair).
- Movement function consulting the pair-table and board dimensions
  to compute rotation.
- FEN payload `PR=<id>`.
- Initial setup: pairs declared in FEN by matching pair-IDs.

## FEN encoding

Antiphon piece payload:

```
(P=A,C=W,PR=1)        # white Antiphon, pair 1
(P=A,C=B,PR=1)        # black Antiphon, pair 1 (matches above)
(P=A,C=W,PR=2)        # second pair, white side
(P=A,C=W,PR=2)        # second pair, white side (same-colour pair)
```

Two Antiphons share a `PR=` value iff they are paired. The FEN
validator should enforce that each `PR=` value is used exactly twice.

## Open questions

- **Same-side pairs.** Are pairs allowed to be the same colour? Yes
  by default; the mechanic doesn't restrict.
- **More than one pair per side.** Allowed; each pair tracked
  separately by ID.
- **Three-Antiphon scenarios (broken pair).** Disallowed by FEN
  validation. Each PR-ID must appear exactly twice.
- **Capture removes the twin's mobility.** When one Antiphon of a
  pair is captured, the other has no legal move (no twin to rotate
  to). The surviving Antiphon can be captured by any normal means;
  it cannot move.
- **Pair re-establishment.** Can a captured Antiphon be replaced?
  Not by promotion (pawn-promotion to Antiphon would create a third
  Antiphon with the dead PR-ID — disallowed). Pair death is permanent.
- **Board centre.** On odd-dimension boards, the centre is a single
  square whose rotation is itself. An Antiphon-twin on the centre
  immobilises its partner (rotation target = centre = twin's
  square = occupied).
- **Castling, promotion, en-passant.** None apply to Antiphons.
- **Letter choice.** `A` for Antiphon (unused; available). Confirm.
- **Threat through walls.** Antiphons jump; walls don't block them.
  Confirm with engine.
