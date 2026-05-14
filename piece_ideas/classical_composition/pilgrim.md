# Pilgrim

> A knight that must visit squares in alphabetical file order; the only
> legal mate is the move whose file is the currently-required letter.

## Inspiration

Sam Loyd's love of the **unique-key puzzle** — a problem where the
solving move is the *only* legal first move, and every other candidate
fails to a single defence. The Pilgrim takes "unique key" to an
absurdist extreme: the piece *can move* only to one file per turn, so
the unique-key constraint is mechanical rather than tactical.

The pilgrimage motif: a knight on a tour, bound to follow the
alphabet, walking from a-file to h-file and starting over. Any solver
who forgets the rule will propose obvious-looking knight checks that
land on the wrong file and are *illegal* — not merely losing, illegal.

## Mechanic

The Pilgrim moves like a knight (8 L-shaped jumps from its current
square, standard chess knight movement). It is constrained by a
**file counter**: the next move must land on a square whose file equals
the current `next_file` value.

The counter starts at `a` (file 1). After each Pilgrim move, the counter
advances to the next letter:

```
a → b → c → d → e → f → g → h → a → b → ...
```

So:
- Pilgrim's first move must land on the a-file.
- Pilgrim's second move must land on the b-file.
- ...
- Pilgrim's eighth move must land on the h-file.
- Pilgrim's ninth move resets to the a-file.

If no knight-jump from the Pilgrim's current square lands on the
required file, the Pilgrim has no legal move that turn. It is
immobile until other pieces' moves either reposition the Pilgrim
indirectly (impossible without specific mechanics) or the game
proceeds with the Pilgrim stranded.

The Pilgrim does not advance the counter when it has no legal move;
the counter advances only on successful moves.

State in FEN: `F=<letter>` where `<letter>` is the next required file
(`a`, `b`, ..., `h`).

Captures, checks, and check-detection follow normal knight rules — the
Pilgrim attacks all eight L-squares from its current position, but
only those on the `F` file are *legal moves*. Whether attacks on
non-`F` files count as **check** is the key design question:

**Compositional decision:** the Pilgrim's *attack set* equals its
*move set*. The Pilgrim only attacks squares on its required file. A
black king sitting on a knight-L away from the Pilgrim is **not in
check** unless the king's square is on the file matching the Pilgrim's
counter.

This makes the file-counter the spine of the entire problem: tries
that put a king under "knight-fork" from the Pilgrim on the wrong file
do nothing.

## Why it's interesting (compositionally)

The Pilgrim enables a tight class of **unique-key** and **try**
problems:

- **Unique-key motif.** From any given Pilgrim position, the legal
  moves number 0, 1, or 2 (knight-jumps that happen to land on the
  required file). The composer engineers positions where the single
  legal move *is* the solution, and the *appearance* of other knight-
  options misleads the solver.
- **File-counter zugzwang.** The composer can force the Pilgrim into
  a sequence where, on a critical turn, the required file is one that
  has no knight-jump available — the Pilgrim is "out of phase" and
  cannot move. The mating geometry collapses because the Pilgrim
  cannot deliver the expected check.
- **Try defences that change the file-counter.** A Black defensive
  move can sometimes capture a friendly piece on the Pilgrim's
  required file, removing the target without changing the counter —
  the Pilgrim still must land on that file, but now lands on the
  capture-square doing nothing useful.
- **Mate by file-alignment.** The Pilgrim mates only when its
  required-file landing square gives check. The composer arranges the
  game so that the file-counter reaches the *correct* letter exactly
  on the move that delivers mate.

## A worked problem

**Mate in 3.** White Pilgrim with F=`d`, plus a white king and a black
king with limited mobility.

```
8 . . . . . . . .
7 . . k . . . . .
6 . . . . . . . .
5 . . . . . . . .
4 . . . . . . . .
3 . . . . . . . .
2 . . . . . . . .
1 . . . K . P . .
  a b c d e f g h
```

White: King d1, Pilgrim f1 (F=`d`).
Black: King c7.

The Pilgrim on f1 with F=`d`: knight-jumps from f1 are d2, e3, g3, h2.
Of these, only **d2** is on the d-file. So the Pilgrim has exactly
**one legal move**: f1→d2.

- **1.Pilgrim f1–d2.** F advances to `e`.

After 1.Pf1–d2, the Pilgrim is on d2 with F=`e`. Knight-jumps from d2:
b1, b3, c4, e4, f1, f3. Of these on the e-file: **e4**. One legal move.

- **1...??? Black's turn.** Black king on c7 moves: b6, b7, b8, c6,
  c8, d6, d7, d8. Choose a "neutral" move; in a mate-in-3 stipulation
  Black plays to delay mate. Suppose Black plays Kc7-c6.

- **2.Pilgrim d2–e4.** F advances to `f`. Pilgrim on e4 attacks
  (knight-L from e4): c3, c5, d2, d6, f2, f6, g3, g5. Of these on the
  f-file: f2, f6. Two legal moves. Choose the one toward the mating
  geometry: e4→f6+? Does the Pilgrim on f6 give check? The Black king
  is on c6. f6 to c6 is a knight-jump? File diff 3, rank diff 0 —
  not knight. So no, f6 doesn't attack c6. e4→f2 also doesn't attack
  c6.

This sketch is too open — the Pilgrim's geometry isn't lining up with
the Black king. The composer would constrain Black's king position
much more tightly.

**Tighter position.** Black king on a4 (forced to corner-zone by
auxiliary white pieces).

```
8 . . . . . . . .
7 . . . . . . . .
6 . . . . . . . .
5 . . . . . . . .
4 k . . . . . . .
3 . . . . . . . .
2 . . . . . . . .
1 . . . K . P . .
  a b c d e f g h
```

White: King d1, Pilgrim f1 (F=`d`). Black: King a4.

Plan: Pilgrim's F-counter advances through d, e, f, ... until it
reaches a value where a knight-jump from the current Pilgrim square
gives check to the Black king on a4.

Knight-squares around a4: c3, c5, b6 (no, b6 to a4 is file 1, rank 2 —
knight? +1 +2 yes), b2 (yes, +1 -2). So knight-attackers of a4 sit
on b2, b6, c3, c5. Their files: b, b, c, c.

For the Pilgrim to give check from one of these squares, F must equal
that file's letter (b or c) at the moment of arrival.

F starts at `d`. After move 1: `e`. After move 2: `f`. After move 3:
`g`. We're heading away from `b` and `c`. The counter wraps after `h`
back to `a`, so move 5: `h`, move 6: `a`, move 7: `b`. So the Pilgrim
can give check from b2 or b6 *on move 7* at earliest, after 6 moves
of setup. That's not mate-in-3.

The composer adjusts the starting F counter. Set F=`a` initially —
then move 2 lands on `b`. So if the Pilgrim's first move lands on the
a-file (move 1, F=`a`), then the second move lands on the b-file
(F=`b`), and that second move can be to b2 or b6.

Starting position revised:

```
8 . . . . . . . .
7 . . . . . . . .
6 . . . . . . . .
5 . . . . . . . .
4 k . . . . . . .
3 . . . . . . . .
2 . . . . . . . .
1 . . . K . P . .
  a b c d e f g h
```

Pilgrim f1, F=`a`.

- **1.Pilgrim f1–???.** Knight-jumps from f1: d2, e3, g3, h2. None on
  the a-file. **The Pilgrim has no legal move.**

The Pilgrim cannot start its tour from f1 with F=`a` because no
a-file square is a knight-jump away. The composer must place the
Pilgrim on a square with knight-access to the a-file.

Pilgrim on b3 has knight-jumps: a1, a5, c1, c5, d2, d4. Two on the
a-file (a1, a5). Place the Pilgrim there.

```
8 . . . . . . . .
7 . . . . . . . .
6 . . . . . . . .
5 . . . . . . . .
4 k . . . . . . .
3 . P . . . . . .
2 . . . . . . . .
1 . . . K . . . .
  a b c d e f g h
```

Pilgrim b3 (F=`a`), white king d1, black king a4.

- **1.Pilgrim b3–a5+?** Pilgrim on a5 attacks (knight-L): b3, b7, c4,
  c6. The Black king on a4: from a5, the knight-attacker squares of a4
  are b2, b6, c3, c5. a5 is not among them. So a5 does not attack a4.
  Not check. Move is legal (lands on a-file ✓), F advances to `b`.

After 1.Pa5: Pilgrim on a5, F=`b`. Black's turn. Black king on a4
moves: a3, b3 (empty? yes), b4, b5 (attacked by the Pilgrim on a5 —
b5 is one square N of the Pilgrim, not knight-attacked but king-
adjacent — Pilgrim doesn't have king-aura, just knight-jumps). Wait —
is b5 attacked by the Pilgrim? Pilgrim is a knight-mover; b5 to a5 is
one square, not a knight-jump. So no, b5 isn't attacked. Black king
escapes: a3, b3, b4, b5.

Black plays Ka4-a3 (toward the corner — helpful for the composer if
the mating square aligns).

- **2.Pilgrim a5–???.** F=`b`. Knight-jumps from a5: b3, b7, c4, c6.
  On the b-file: b3, b7. Two options. From b3 or b7, does the Pilgrim
  attack a3 (the Black king's current square)?
  - b3 knight-attacks: a1, a5, c1, c5, d2, d4. Not a3.
  - b7 knight-attacks: a5, c5, d6, d8. Not a3.

Neither attacks a3. **No mate on move 2 from this position.** Continue:

  Choose 2.Pa5-b3 (Pilgrim returns to b3, F=`c`).

- **2...** Black plays Ka3-?. Choices: a2, a4 (attacked? a4 is
  knight-attacker of b3? b3 to a4 — file -1, rank +1 — not knight.
  Not attacked. Safe), b2 (attacked? b3 to b2 — one square — not
  knight. Safe), b4 (attacked? b3 to b4 — one square — not knight.
  Safe). Black king has multiple escapes.

This problem doesn't mate-in-3 cleanly. The geometry is *too* open and
the Pilgrim's attacking range is *too* narrow.

**Lessons from the sketch.** The Pilgrim mechanic is genuinely
constraining: at any given moment, the legal Pilgrim moves are usually
0–2, and the attack-pattern shifts to a single file per turn. This
restricts the composer to:

- Tightly-confined Black kings (against an edge or in a corner).
- Pre-planned F-counter sequences where the file-counter hits the
  "right" letter exactly when the mating geometry is ready.
- Multiple white auxiliary pieces (king + minor pieces) to handle the
  king-cornering work while the Pilgrim waits for its turn.

The composer crafts a position where the Pilgrim's *single legal move*
on each turn is also the *only mating move*. Tries that try to "play
the Pilgrim faster" (skip a file) are illegal; tries that "play the
Pilgrim slower" (waste a turn) miss the file-counter window.

## Compositional notes

- **Plan the file-counter sequence backward from mate.** Determine
  the mating square's file; work backward N letters (= N moves before
  mate); set the starting F-counter to mating_file − N (mod 8).
- **Pre-place the Pilgrim with knight-access to the starting file.**
  An immobile Pilgrim from move 1 wastes the puzzle.
- **The Pilgrim's two-option turns are danger zones.** If two
  knight-jumps both land on the required file, the puzzle has two
  candidate solutions on that turn. Engineer the position so that
  only one of the two is the "mating-trajectory" move; the other
  loses to a Black defence.
- **Stuck-Pilgrim positions.** If the F-counter requires a file the
  Pilgrim can't reach, the Pilgrim is stuck. Use this for compositional
  jokes: "and then the Pilgrim cannot move and Black mates."
- **Wraparound from h → a.** A Pilgrim that has made 8 moves resets
  to F=`a`. Long problems (series-movers) can use wraparounds for
  unexpected geometry.

## Where it shines

- Mate-in-3 and mate-in-4 problems with cornered Black kings.
- Series-movers in 8 or 16 (full tours of the file-counter).
- Try-rich problems where multiple knight-jumps *exist* but only one
  is *legal*.
- Studies emphasising the file-counter constraint as the puzzle's
  whole content.

## Where it's awkward

- The attack-set restriction (Pilgrim attacks only its required file)
  is unintuitive. Solvers will assume knight-fork threats that are
  not threats.
- The mechanic produces many positions where the Pilgrim has zero
  legal moves; composers must be careful that Black isn't accidentally
  put into stalemate by Pilgrim-immobility.
- Long-game balance is poor — a Pilgrim with bad file alignment can
  be useless for many turns. Composition piece, not playable.
- Two Pilgrims per side multiplies F-counter bookkeeping. Recommended:
  one Pilgrim per problem.

## Engine dependencies

- Per-piece state (`F` counter).
- Movement function returning knight-jumps filtered by file.
- Attack function = movement function (Pilgrim attacks only what it
  could move to).
- File-counter advance on every successful move.
- FEN encoder/decoder for the counter.

## New features required

- New piece type "Pilgrim" with state `next_file: u8` (1..=8 or
  'a'..'h').
- Movement function: enumerate 8 knight-jumps, filter by file ==
  next_file, filter by normal walkability/capture rules.
- Move-completion hook: advance next_file = (next_file mod 8) + 1.
- FEN payload `F=<letter>`.
- Default F=`a` if omitted.

## FEN encoding

Pilgrim piece payload:

```
(P=I,C=W,F=a)            # white Pilgrim, must move to a-file next
(P=I,C=W,F=d)            # white Pilgrim, must move to d-file next
(P=I,C=W,F=h)            # white Pilgrim, must move to h-file next; wraps to a after
```

Letter `I` for Pilgrim (memorable: "Pilgrim" → unused capital; P, R, etc.
are taken). Confirm.

## Open questions

- **Does the F-counter advance if the Pilgrim is checkmated /
  stalemated on its turn?** No — the counter advances only on a
  *successful* Pilgrim move. If the side-to-move plays a non-Pilgrim
  move, F stays.
- **Pilgrim attack set = move set (decided).** Yes, the Pilgrim only
  attacks the file matching F.
- **Initial file.** Composers set this in the FEN; default is `a`.
- **Multiple Pilgrims per side.** Each has its own F-counter. Allowed.
- **Capturing the Pilgrim.** Standard capture rules. The Pilgrim's
  F-counter dies with it.
- **Promotion to Pilgrim.** Allowed; promoted Pilgrim's F-counter
  starts at `a` (fresh tour). Or composers can specify in FEN.
- **Pilgrim castling.** Pilgrim does not castle.
- **Letter conflict.** `I` is currently unused; confirm.
- **F-counter increment on capture-only or all moves.** All moves
  including non-captures. A Pilgrim move is a move.
- **Pilgrim attacks the king from required file = check; player must
  respond to check.** Confirmed.
- **What if F is the king's file but no knight-jump lands on the
  king?** No check, no attack — the Pilgrim must *reach* the king's
  square (or attack-square) via a knight-jump on the required file.
