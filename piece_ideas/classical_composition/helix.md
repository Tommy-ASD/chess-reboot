# Helix

> A rook that rotates 90° clockwise on every move; its short-range threat
> alternates between diagonal and orthogonal as it spins.

## Inspiration

The fairy-chess problemist's love of **switchback** — a piece that travels
out and back, returning to its origin square as the final move of a problem.
A bare switchback is mechanical; the Helix raises the bar by demanding
the piece return not just to its *square* but to its *rotational state*.
A 4-square tour back to home is the cheapest switchback the Helix permits,
and the composer who finds one has built a small machine of pure geometry.

This piece is a Loyd-style novelty: outrageously specific mechanic,
designed to make exactly one kind of problem sing.

## Mechanic

The Helix moves like a rook (any number of empty squares along a rank or
file). On every move it makes — capture or non-capture — its rotation
state advances by one (mod 4). It begins at rotation 0.

The rotation does not change *how it moves* — it always moves as a rook.
It changes *which adjacent squares it threatens for the purpose of check
and capture-on-arrival*, in addition to the rook attack:

| Rotation | Extra threat squares                            |
|----------|--------------------------------------------------|
| 0 (even) | Orthogonally adjacent (N, S, E, W) — already covered by rook attack, so functionally no extra threat |
| 1 (odd)  | Diagonally adjacent (NE, NW, SE, SW)             |
| 2 (even) | Orthogonally adjacent (no extra threat)          |
| 3 (odd)  | Diagonally adjacent                              |

So odd-rotation Helices have a "king-like" diagonal aura layered on top of
the rook attack, and even-rotation Helices are bare rooks. To return to a
*useful* identical state, the Helix must move exactly 4 times.

State in FEN: `R=0`, `R=1`, `R=2`, `R=3`.

The Helix does not "rotate" in any visual sense for piece-on-piece geometry
— it does not gain or lose rook-direction. Only the diagonal aura toggles.

## Why it's interesting (compositionally)

The Helix is a **switchback engine**. Any problem requiring the Helix to
end on its starting square *with the same threat profile* costs exactly
four Helix moves. This forces the composer to budget four tempi for the
Helix and find work for the rest of the army around that constraint.

Beyond switchback, the rotation state enables **parity tries** — a candidate
solution that captures the right piece on the right square but ends with
the Helix in the wrong rotation, leaving its king vulnerable to a defence
that exploits a missing diagonal threat. The Helix punishes solvers who
look only at the geometry and not at the rotation counter.

## A worked problem

White to play and mate in 5. White Helix on d4 (rotation 0), white king on
h1, black king on f6, black pawn on e7.

```
8 . . . . . . . .
7 . . . . p . . .
6 . . . . . k . .
5 . . . . . . . .
4 . . . H . . . .
3 . . . . . . . .
2 . . . . . . . .
1 . . . . . . . K
  a b c d e f g h
```

`H` is the Helix at rotation 0. We need to mate the black king on f6 with
the Helix giving check, and (per the composer's stipulation) end with the
Helix back on d4 at rotation 0 — a full switchback.

The forced sequence:

1. **Hd4–d6+** (rotation 0 → 1). Direct rook check. Black must respond.
   The pawn on e7 cannot interpose (e6 is empty but black pawns push, they
   don't sidestep); the king must move. **1...Kxd6** is illegal because
   the Helix at rotation 1 threatens c7, c5, e7, e5 diagonally — but the
   black king moving to d6 puts itself adjacent to the Helix on its own
   square, which is fine geometrically (king captures attacker), except
   that the rook threat from d6 itself does not protect d6. So *can* the
   king take? No — the d6 square is occupied by the Helix only momentarily;
   after **1.Hd6+**, e7 is attacked diagonally, and Kxd6 walks into…
   actually nothing yet. Let us tighten the position.

   Add a white bishop on b4. Now after **1.Hd6+ Kxd6**, the bishop on b4
   covers the king-recapture square. But this is not the intended line.

Let me restate the problem with sharper constraints — composers do this
all the time when the first sketch leaks.

**Revised position.** White Helix on d4 (R=0), white knight on e3,
white king on h1, black king on f6, black pawn on g7, black rook on a8.

```
8 r . . . . . . .
7 . . . . . . p .
6 . . . . . k . .
5 . . . . . . . .
4 . . . H . . . .
3 . . . . N . . .
2 . . . . . . . .
1 . . . . . . . K
  a b c d e f g h
```

Stipulation: mate in 4, Helix must end on d4 at R=0 (declared switchback).

Solution:

1. **Hd4–f4+** (R: 0→1). Rook check along the 4th rank to f4; the Helix
   now sits one square below the king on f6. At R=1, the Helix threatens
   e5, g5, e3, g3 diagonally — none of which are the king. But the rook
   threat from f4 attacks f5, f6 (the king), f7. The king must move.
   - **1...Ke6** Black plays into a knight fork prep. The knight on e3
     covers d5 and f5; the king goes to e6.
   - **1...Ke5** illegal: e5 is threatened diagonally by the Helix at R=1.
   - **1...Kg5** illegal: g5 is threatened diagonally by the Helix at R=1.
   - **1...Kg6, Kf5** etc.: f5 illegal (rook threat), so the king has
     only e6 and the diagonal escapes are cut.

2. **Hf4–f5+** (R: 1→2). The Helix slides to f5 — back into rook-attack
   range of the king. R=2 has no diagonal aura. The king is checked
   along the f-file? No — the king is on e6. Let us re-examine: Helix on
   f5 threatens e5, f6, f4, g5 (rook attack adjacent) and the entire
   f-file and 5th rank. The king on e6 is attacked along the 5th rank? No,
   e6 is on the 6th rank. The king is attacked from f5 because f5 and e6
   are diagonally adjacent and the Helix at R=2 has *no diagonal aura*.
   So this is not check.

The problem is harder than the sketch admits. Let me commit to honesty
and present the *kernel* of what the Helix enables, rather than a full
mate-in-N that I cannot verify by hand.

**The kernel.** With the Helix on a central square at R=0, four consecutive
Helix moves return it to R=0. If the composer arranges the position so
that:

- The 1st and 3rd Helix moves give rook-checks the king must dodge
  diagonally;
- The 2nd and 4th Helix moves require the diagonal aura (R=1, R=3) to
  cover key squares the king would otherwise escape to;

then no shorter Helix tour solves the problem, and no alternative
white piece can substitute. The Helix's mechanic is *load-bearing*: a
rook at f4 cannot cover e5 + g5 the way an R=1 Helix does. A bishop at
f4 cannot give the f-file rook check.

A composer crafting this puzzle will iterate the position 20–50 times
until exactly one 4-Helix-move sequence mates. That is the joy.

## Compositional notes

- **Budget four tempi.** A switchback Helix problem is at minimum a mate
  in 4 (or longer with non-Helix moves interleaved). Mate in 2 with a
  Helix switchback is impossible.
- **Use the R=1/R=3 aura sparingly.** The diagonal threat is invisible
  to solvers who only see "rook." Tries that look like immediate mates
  often fail because the Helix is at R=2 and lacks the diagonal cover.
- **Even-rotation = boring Helix.** At R=0 and R=2 the Helix is a plain
  rook. Position the start state so that the *first* move enters R=1 and
  the aura becomes load-bearing immediately.
- **Avoid two Helices.** Two Helices on one board multiply state-space
  4×4 = 16, and the solver loses track. One Helix per problem.

## Where it shines

- Mate-in-4 and mate-in-5 problems with a declared switchback stipulation.
- Tries that capture the right piece but end the Helix in the wrong
  rotation, leaving a defensive resource the solution-rotation prevents.
- Endgame studies where White must make four Helix moves to corner the
  king, but no five-move tour also works.

## Where it's awkward

- The mechanic is fiddly to teach. A solver who has never met a Helix
  will spend the first minute working out the rotation table. This is
  fine for problem journals, less fine for casual play.
- The diagonal aura is *not* a rook attack — it does not extend along
  diagonals, only adjacent. Composers who forget this place the Helix
  too far from the king and the aura does nothing.
- Two Helices interact poorly. Each rotates independently and the
  combinatorial state explodes. Recommend one-per-side maximum.

## Engine dependencies

- Piece state in FEN (already established by Locomotive carriage tracking).
- Rotation counter as a small per-piece integer (0..=3).
- Threat function that distinguishes "moves" from "attacks" — the rook
  movement is unchanged, but the *attacked-square* set varies with
  rotation. Existing pieces with this split: Skibidi (kind of), Goblin.
- Move-hook to bump rotation on every successful move.

## New features required

- Per-piece rotation counter, FEN-serialised as `R=0..3`.
- Move-completion hook: increment rotation mod 4.
- Threat function override: read rotation, return rook-rays ∪
  (diagonal-adjacents if R odd, else ∅).
- Check-detection consults threat function, not movement function.

## FEN encoding

Standard piece payload with a rotation field:

```
(P=H,C=W,R=0)
(P=H,C=W,R=1)
(P=H,C=W,R=2)
(P=H,C=W,R=3)
```

The `R=` key is reused from any future rotation-state convention;
unconflicted with existing tags. Default value `R=0` if omitted, matching
the standard leniency policy.

A board placing a Helix at d4 rotation 1 emits:

```
... 3(P=H,C=W,R=1)4 ...
```

## Open questions

- Does the Helix's rotation count when it *fails* to move (e.g. is pinned
  and has no legal move)? Recommended: no — only successful moves advance
  the counter, otherwise the switchback budget breaks.
- Does a king capturing the Helix care about the rotation? Recommended:
  no — capture removes the piece, rotation is irrelevant.
- Should castling count as a move for rotation? The Helix doesn't castle
  (it's not a rook in the castling-rights sense), so moot.
- Should there be an initial-rotation tag in the FEN for setup
  flexibility? Yes — `R=` already provides it.
