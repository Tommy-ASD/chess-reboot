# Eclipse

> A bishop whose threats are gated by square-colour and distance parity;
> defenders walking to the wrong colour at the wrong distance are
> invisibly checked.

## Inspiration

The fairy-chess **zugzwang** problem: a position where the defender's
every legal move worsens their situation. Classical zugzwang relies on
the defender running out of safe squares. The Eclipse weaponises a
*subtler* exhaustion — the defender exhausts not safe squares but safe
*colour-and-distance combinations*. Every reasonable-looking defensive
move steps into an invisible attack.

The name evokes the moon's shadow falling only at specific points along
its arc — and the Eclipse's threat falling only at specific point along
its rays.

## Mechanic

The Eclipse moves like a bishop (slides any number of empty squares
along a diagonal). Its *attack pattern* is more restrictive than its
*movement pattern*.

The Eclipse occupies either a light or dark square. Its attacks are
gated by **target-square colour** and **distance**:

| Eclipse on  | Attacks occupants of dark squares      | Attacks occupants of light squares     |
|-------------|----------------------------------------|----------------------------------------|
| light       | at **odd** chessboard distances        | not at all                             |
| dark        | not at all                             | at **even** chessboard distances       |

Where:
- *Chessboard distance* = max(|file_diff|, |rank_diff|), i.e. Chebyshev
  distance, counting squares from the Eclipse to the target along its
  diagonal ray.
- A target square has a *colour*. A target *occupant* lives on a target
  square; its square's colour is what matters.
- "Distance 1" means the immediately adjacent diagonal square along
  the ray.

Note that *along a bishop diagonal*, every square is the same colour as
the Eclipse itself, because diagonals never alternate colour. **So the
Eclipse can never attack any piece along its own ray.**

This is intentional and the central twist. The Eclipse moves diagonally
through empty squares, but its *threat* is to the **adjacent
non-diagonal** occupants — specifically, those reachable by the Eclipse
"extending" one square orthogonally off its diagonal ray, evaluated
under the colour/distance rules above.

More precisely: the Eclipse's **threat radius** is, for each square Q
on one of its bishop diagonals (empty path required), the four
orthogonal neighbours of Q. The Eclipse threatens the piece occupying
one of those neighbours, *if and only if* the square Q (the diagonal
point) is at the correct distance and the target neighbour is the
correct colour, per the table above.

Restated cleanly:

- The Eclipse imagines casting bishop-rays through empty squares.
- At each intermediate square Q on its rays, the Eclipse projects its
  threat **one square orthogonally** to Q's four neighbours.
- A neighbour N is *attacked* iff:
  - When Eclipse is on a light square: N is on a *dark* square (true
    by orthogonal adjacency to Q where Q is light) AND the
    Eclipse-to-Q chessboard distance is *odd*.
  - When Eclipse is on a dark square: N is on a *light* square AND the
    Eclipse-to-Q chessboard distance is *even*.

Movement is **not** restricted by this rule — the Eclipse moves like a
plain bishop. Only the *attacks* are colour-parity-gated.

State in FEN: none beyond piece colour and position. The Eclipse is
stateless.

## Why it's interesting (compositionally)

This is **pure parity zugzwang**. The defender's safe and unsafe squares
follow a chessboard pattern, but it is not a pattern the defender can
*see* without computing distances. Squares that look defensively
identical (same orthogonal escape) are radically different under the
Eclipse's threat: the wrong distance class hides an attack.

Composition motifs:
- **The defender's exhaustion of parity classes.** The Eclipse on a
  light square attacks dark squares at odd distance. The defender
  retreats to even-distance dark squares — but then a White tempo move
  shifts the Eclipse one diagonal step (now its distances reparity) and
  the defender is back in attack range.
- **Tries that capture the right piece on the right square but ignore
  parity.** A solver examining the Eclipse's threats by hand will
  miscount distance often.
- **The Eclipse's own diagonal is safe.** Counterintuitive — the
  defender can park a piece directly on the Eclipse's diagonal ray
  (Eclipse's own square colour) and be totally safe. This produces
  positions where the defender's *only* refuge is on the Eclipse's
  ray. The composer can build problems around this.

## A worked problem

White to play and **mate in 2** via zugzwang.

```
8 . . . . . . . .
7 . . . . . . . .
6 . . . . . . . .
5 . . . . . k . .
4 . . . . . . . .
3 . . . . . . . .
2 . . E . . . . .
1 . . . . . . K .
  a b c d e f g h
```

White: King g1, Eclipse c2. Black: King f5.

c2 is a light square (a1 dark → b2 light → c3 dark → c2 light:
let's verify: a1 dark, a2 light, ..., c2: file 'c'=3, rank 2; (file +
rank) = 5, odd → light). Yes, c2 is light.

Eclipse on c2 (light square). Per the table, it attacks occupants of
**dark** squares at **odd** chessboard distance.

The Eclipse's diagonal rays from c2:
- NE: c2, d3, e4, f5, g6, h7
- NW: c2, b3, a4
- SE: c2, d1
- SW: c2, b1

Each ray has empty intermediate squares (the board is mostly empty).
The Eclipse projects orthogonal threats from each intermediate square.

Black king is on f5. Let's check whether f5 is currently attacked.

The Eclipse's threat reaches f5 if some intermediate square Q on its
diagonals has f5 as an orthogonal neighbour, with the right parity:
- Q=e4 (distance 2 from c2 along NE). e4's orthogonal neighbours: e3,
  e5, d4, f4. Not f5.
- Q=f5 itself (distance 3 from c2 along NE). f5's orthogonal neighbours:
  f4, f6, e5, g5. Not f5. The Eclipse cannot attack pieces on its own
  ray (same-colour diagonal).
- Q=g6 (distance 4 from c2 along NE). g6's orthogonal neighbours: g5,
  g7, f6, h6. Not f5.

Wait — let me check Q=f4. f4 is on which ray? c2-NE goes c2, d3, e4,
f5, g6 — f4 is *not* on the NE ray from c2. f4 is on a different
diagonal entirely.

So f5 cannot be reached as an orthogonal neighbour of any square on
c2's diagonals — because all the c2-diagonal squares have colour =
c2's colour = light, and the four orthogonal neighbours of a light
square are all dark. The black king on f5: f5 is (6+5)=11 → odd →
**dark**. So an orthogonal-neighbour-of-light-diagonal square is a dark
square, and f5 is dark.

Which light squares on c2's diagonals are orthogonally adjacent to f5?
- f5's orthogonal neighbours: e5 (light), g5 (light), f4 (light), f6
  (light). All light.
- Of these, which lie on a c2 diagonal? e5: c2-h7 diagonal goes c2, d3,
  e4, f5, ... — e5 is not on c2's NE diagonal. e5 is on b2-h8? b2 is
  light, but Eclipse is on c2, not b2. The Eclipse's diagonals are
  specifically NE/NW/SE/SW from c2. e5 is two squares NE and one
  square N from c2 — not on a diagonal.

So f5 is *not* threatened by the Eclipse from c2. The black king is
safe… for now.

**1.Eclipse c2–d3.** The Eclipse moves to d3. d3 is dark (3+3=6, even
→ dark)? Wait, let's get the colour convention right. Standard: a1 is
dark. (file=1, rank=1, file+rank=2, even → dark). So **even
file+rank = dark, odd = light**. c2: 3+2=5, odd, light ✓. d3: 4+3=7,
odd, light. So d3 is also light. The Eclipse remains on a light
square; the same threat rule applies.

This doesn't change the colour-parity class. Let me try a move to a
dark square: c2 to b1 — b1 is 2+1=3, odd, light. c2 to d1 — 4+1=5,
odd, light. The Eclipse on light stays on light because diagonals
preserve colour.

This is the deep structural fact about the Eclipse: **its
movement-colour never changes**. Born on light, always on light. Born
on dark, always on dark. The colour class is permanent.

So a c2-Eclipse (light) **always** attacks only dark squares at odd
distance — forever. This is a *strong* compositional invariant.

Re-examining: From c2, which dark squares are orthogonally adjacent
to a c2-diagonal at odd distance?

Diagonal squares and their distances from c2:
- d3 (dist 1, odd), e4 (dist 2, even), f5 (dist 3, odd), g6 (dist 4),
  h7 (dist 5)
- b3 (dist 1), a4 (dist 2)
- d1 (dist 1), b1 (dist 1)

Odd-distance diagonal squares: d3, b3, d1, b1, f5, h7.

Each has 4 orthogonal neighbours. The dark ones:
- d3 neighbours: d2 (dark? 4+2=6, even, dark ✓), d4 (4+4=8, dark ✓),
  c3 (3+3=6, dark ✓), e3 (5+3=8, dark ✓). All dark.
- b3 neighbours: b2, b4, a3, c3. All dark.
- d1 neighbours: d2, c1, e1. All dark.
- b1 neighbours: a1, c1, b2. All dark.
- f5 neighbours: f4, f6, e5, g5. f4 (6+4=10, dark ✓), f6 (6+6=12,
  dark ✓), e5 (5+5=10, dark ✓), g5 (7+5=12, dark ✓). All dark.
- h7 neighbours: h6, h8, g7. All dark.

So the Eclipse on c2 threatens: d2, d4, c3, e3, b2, b4, a3, c1, e1, a1,
f4, f6, e5, g5, h6, h8, g7. A wide net of dark squares.

Now suppose the black king is on f4. f4 is dark, distance from c2 is
chess-king distance from c2 to f4 — but we're concerned with whether
f4 is orthogonally adjacent to a c2-diagonal-square at odd Eclipse
distance. f4 is orthogonal neighbour of f5 (which is at c2-NE distance
3, odd ✓). So **the Eclipse on c2 threatens the black king on f4**.

This means I should redesign the position. Let me place the black king
on a square that is *almost* in the threat net but escapes by parity.

**Composition kernel.** A black king on g6: g6 is (7+6)=13, light. The
Eclipse on c2 (light) does *not* attack light squares — g6 is safe.
The king can run to g6 as a refuge. The composer's puzzle: arrange
the white pieces so that the black king must move *off* g6, into the
dark zugzwang net.

A pawn on g7 (white) attacks f6 and h6 (already threatened by Eclipse
— redundant). A pawn on g5 (white) attacks f6 and h6 — irrelevant.
But a white *king* on h6 attacks g5, g6, g7, h5, h7 — including g6.
Black king must vacate g6. Available escapes: f7, g7, h7, f5, f6, g5,
h5. Of these:
- f7: 6+7=13, light. Safe from Eclipse colour-parity? Light, so Eclipse
  on light does not attack light. **Safe.**
- g7: 7+7=14, dark. f7-g7 orthogonal? Yes. g7 is dark. Adjacent to h7
  diagonal-square at c2-distance 5 (odd ✓). Threatened.
- h7: 8+7=15, light. Safe (light).
- f5: 6+5=11, dark. Adjacent to f5 diagonal-square? f5 is itself a
  c2-diagonal-square at distance 3 (odd) — Eclipse cannot attack on its
  own diagonal. f5's orthogonals: f4, f6, e5, g5. f5 itself is on a
  c2-diagonal, but the *attack* projects from f5 to its orthogonal
  neighbours, not back to f5. So is f5 itself attacked? No — only
  *neighbours of* diagonal-squares are attacked; f5 is the diagonal-
  square, not a neighbour. Safe (from Eclipse).
- f6: 6+6=12, dark. Neighbour of f5 (c2-dist 3, odd ✓). Threatened.
- g5: 7+5=12, dark. Neighbour of f5 (odd). Threatened.
- h5: 8+5=13, light. Safe (light).

Black escapes: f7, h7, f5, h5. Four squares.

If we add white pieces to cover f7, h7, f5, h5, the only Black escape
is back into the Eclipse's threat net — **zugzwang**.

Cover f7 with a white bishop on g8 (attacks f7, h7, e6, d5, ...).
Cover h5 with a white knight on g3 (attacks h5, f5, e4, ...) — and the
knight covers f5 too. So one knight on g3 covers f5 and h5; the bishop
on g8 covers f7 and h7.

Position:

```
8 . . . . . . B .
7 . . . . . . . .
6 . . . . . . k K
5 . . . . . . . .
4 . . . . . . . .
3 . . E . . . N .
2 . . . . . . . .
1 . . . . . . . .
  a b c d e f g h
```

Pieces: White Eclipse c3? Wait — I had it on c2. And white King h6,
white Bishop g8, white Knight g3, Black King g6.

But the position has the white king *adjacent to* the black king (h6
and g6 are king-adjacent). This is illegal in chess (kings can't be
adjacent). Need to rework.

Move the white king to h8 instead. h8 attacks g7, g8, h7. But g8 has
the white bishop — attacks blocked by own piece? Kings don't attack
through friendlies in the normal "attacks" sense, but the white king
on h8 *occupies* its own square; the bishop on g8 isn't "blocked" by
the king — they coexist. h8's attacks: g7, g8 (own bishop, no
threat), h7. So h8 covers g7 and h7. Combined with the bishop on g8
covering f7, e6, d5 — but not h7 because h7 is adjacent to g8 (yes,
bishop on g8 attacks h7? bishop moves diagonally, g8 diagonals are
h7 (one step SE) ✓, f7 (one step SW) ✓. So bishop g8 attacks f7 and
h7. White king h8 redundantly covers h7 and g7.

Need to also cover f7 by something other than the bishop, in case
black plays Kg6→f7 and we want it disallowed. The bishop on g8
already attacks f7 directly. Good.

What about f5 and h5? The knight on g3 attacks f5 and h5 — good.

So Black on g6 has no safe escapes:
- f7: bishop attacks.
- g7: king (h8) attacks and Eclipse threatens.
- h7: king attacks and bishop attacks.
- f6: Eclipse threatens.
- g5: Eclipse threatens.
- f5: knight attacks and Eclipse-on-light doesn't attack f5
  (f5 is on Eclipse diagonal, not a neighbour).
- h5: knight attacks.

What about staying put on g6? Black must move (it's Black's turn and the
king is not in check — wait, is the king in check?). g6 is light; the
Eclipse on light doesn't attack light squares; the bishop on g8 attacks
g6 along the file? No, bishops are diagonal-only. g8 diagonals: a2-g8,
h7-g8, etc. — the diagonal from g8 through g6 is not a diagonal (it's a
file). So bishop on g8 does *not* attack g6. The white king on h8: does
it attack g6? h8 to g6 is two squares — not king-adjacent. No.

So the black king on g6 is not in check. Black is *not* in zugzwang in
the classical "must move into worse" sense — Black can just sit on g6.

But this is a **mate-in-2 stipulation**, meaning Black must be forced
to move. Stalemate is also a draw, not a win. The composer needs to
ensure Black has *at least one* legal move (no stalemate) and that
every Black move leads to mate.

Add a black pawn on a2 that has a legal move. Black plays 1...a2-a1=Q,
and White plays 2.??? — but the white plan was zugzwang on the king,
not the pawn. Adding a pawn lets Black escape the zugzwang by moving
the pawn instead.

The classical solution: make the pawn move *also* lose, by a tactical
follow-up. This is exactly the multi-line refinement that makes
problem composition hard.

**Compositional commitment.** The Eclipse mechanic is genuine and
unambiguous; the *worked problem* above is a kernel-sketch that
demonstrates the parity-zugzwang motif. A clean mate-in-2 with the
Eclipse requires several more iterations of position refinement —
typical of problemist work.

## Compositional notes

- **Same-colour invariant.** The Eclipse never changes its square-colour
  (diagonals preserve colour). The colour-distance rule is therefore a
  permanent property of each Eclipse on the board. Two Eclipses can be
  on opposite colours and cover the *complete* parity space.
- **The Eclipse's own diagonal is a refuge.** A defender parked on the
  Eclipse's diagonal is safe (only orthogonal-neighbours-of-diagonals
  are attacked, and the diagonal itself is not).
- **Distance parity flips on every Eclipse move.** Each diagonal step
  changes the distance from Eclipse to every target by 1, flipping
  every target's attacked/safe status. A single Eclipse move is a
  total threat repaint — extremely useful for tempo zugzwang where
  the defender's previously-safe square becomes attacked after the
  Eclipse repositions.
- **Composing distance-exact tries.** A try where the solver moves
  the Eclipse to the "obvious" square but ends up with the wrong
  distance parity — and Black exploits it — is the Eclipse's signature
  failure pattern. The composer can always engineer such tries.

## Where it shines

- Pure-zugzwang mate-in-2 and mate-in-3 problems.
- Mutual zugzwang positions (both sides in zugzwang; only the
  side-to-move loses) where the Eclipse's distance-parity is the
  fulcrum.
- Endgame studies where the Eclipse's threat net rotates as it slides,
  herding the defender along a parity wall.

## Where it's awkward

- The threat function is computationally expensive: every diagonal
  intermediate square's four neighbours, distance-parity check, target
  colour check. Engine implementation must be careful.
- Solvers must visualise odd/even distance from each Eclipse to each
  square. Diagrams should be annotated.
- The Eclipse cannot attack along its own diagonals — counter-
  intuitive for anyone who reads "bishop" and assumes bishop threats.
  Beginner solvers will misplay constantly.

## Engine dependencies

- Square-colour function (already implicit in any chess engine; just
  exposed for the Eclipse's threat function).
- Bishop-ray generation (existing — Skibidi uses diagonals).
- Custom threat function distinct from the movement function.

## New features required

- New piece type with its own movement (bishop) and *attack-set*
  generator that:
  1. Enumerates diagonal rays through empty squares.
  2. For each diagonal-square Q on a ray, enumerates Q's four
     orthogonal neighbours.
  3. Filters by (Eclipse-square colour, Q's distance parity, neighbour
     square colour) per the table.
  4. Returns the set of attacked squares.
- Check-detection consults the attack-set, not the move-set.
- No new FEN state (stateless piece).

## FEN encoding

Standard piece payload:

```
(P=L,C=W)        # white Eclipse (L for eclipse — letter choice; E is engine-taken? confirm)
(P=L,C=B)        # black Eclipse
```

Letter choice TBD — `L` (for "lunar"), `E` if free, or any unused
single letter. The Eclipse has no state.

## Open questions

- **Distance metric.** Chebyshev (max of file-diff and rank-diff) vs
  Manhattan (sum). Recommended: **Chebyshev**, because the Eclipse's
  diagonal steps each increase Chebyshev by 1 (a clean ray-length
  measure), while Manhattan increases by 2 per diagonal step.
- **What about the Eclipse's own square?** Distance 0; not on the
  diagonal (which starts at distance 1); not attacked.
- **Does the Eclipse attack through pieces?** A diagonal blocked by
  any piece truncates the ray — only the diagonal squares *up to and
  including the blocker* are considered for attack projection. A piece
  on a diagonal blocks further projection past it.
- **Does the ray-blocker itself count as a diagonal-square for
  projection?** Yes — the blocker's four orthogonal neighbours are
  evaluated as potential attack targets (subject to colour/parity).
  The blocker itself, being on the Eclipse's diagonal, is *not*
  attacked (same colour, on the ray).
- **Friendly vs enemy blockers.** Both block the ray. Standard chess
  bishop rules.
- **Letter choice for FEN.** Pin down before implementation.
