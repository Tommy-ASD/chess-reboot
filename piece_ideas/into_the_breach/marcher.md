# Marcher

> [ENEMY] A relentless walker that steps one square in its current
> facing, then rotates 90° clockwise. The arrow is painted on the
> piece — its next five turns are public information.

## Inspiration

The "spinning patrol" enemy from Hoplite and roguelike chess
descendants. Also the procession piece from Imbroglio: a unit whose
*future* is the puzzle, not its present. The Marcher is the
calibration enemy — the one new players use to learn that telegraphs
are real and exploitable.

## Mechanic

A Marcher carries one piece of telegraph state: `dir ∈ {N, E, S, W}`,
its current facing.

On the enemy resolution phase, in canonical iteration order:

1. **Step.** Marcher attempts to move one square in `dir`. If the
   destination is walkable and either empty or holds a capturable
   piece, the Marcher moves there (capturing if applicable).
2. **Bump.** If the destination is blocked (unwalkable square, friendly
   neutral piece, off-board), the Marcher does not move. It still
   rotates.
3. **Rotate.** Regardless of whether the step succeeded, `dir` rotates
   90° clockwise: N→E→S→W→N.

Capture-by-Marcher is destructive in the standard chess sense (the
captured piece is removed). Marchers themselves can be captured by any
ordinary piece — they have no armor and no special evasion.

The Marcher is Neutral-colored. It is not aligned with either chess
player; in puzzle mode the player wins by either capturing it or
surviving the Marcher's path.

## Telegraph rendering

The piece sprite is a directional arrow rotated to match `dir`. The
player reads "N-facing Marcher on c4" and can mentally play forward:

```
turn 1: c4 → c5, then rotates to E
turn 2: c5 → d5, then rotates to S
turn 3: d5 → d4, then rotates to W
turn 4: d4 → c4, then rotates to N
turn 5: c4 → c5  (back to start — the cycle is a 2×2 square)
```

The five-turn lookahead is the entire point. A confident player draws
the future path mentally and chooses interventions.

## Why it's interesting

A perfect-information walker turns the empty board into a calendar.
The player isn't asking "what will this piece do?" — they're asking
"where is the cheapest square to interrupt it?" The puzzle is route
planning against a fully-known trajectory.

The 90°-clockwise rule makes every Marcher trace a 2×2 loop in open
space. Add obstacles, and the bump-but-still-rotate rule means the
loop can morph into surprising shapes. Two Marchers near each other
can lock into mutual orbit or annihilate by walking into each other.

## Example puzzle

```
6 . . . . . .
5 . . . k . .       k = player king (must survive 3 turns)
4 . . M . . .       M = Marcher facing E
3 . . . . . .
2 . . . . . .
1 . . . . . .
  a b c d e f
```

Player has one **Shover** charge (knight-leap + push) and three
turns. The king must not be captured.

The Marcher's three-turn future:

- Turn 1 enemy: c4 → d4, rotates to S.
- Turn 2 enemy: d4 → d3, rotates to W.
- Turn 3 enemy: d3 → c3, rotates to N.

The king on d5 is safe from the actual path — but the player loses if
they sit still, because the king starts adjacent and a stray Marcher
post-rotation could re-enter d5 in turn 5. Wait, no: the cycle is
c4-d4-d3-c3, and d5 is never visited. The king survives by doing
nothing.

**The trick:** the puzzle isn't "save the king" — it's "spot the
no-op." Beginners burn their Shover charge unnecessarily. Veterans
read the rotation and confirm the king is outside the orbit.

A harder variant adds a Block on d4:

```
6 . . . . . .
5 . . . k . .
4 . . M B . .       B = Block
3 . . . . . .
```

Now turn 1: Marcher tries c4→d4, bumps (Block), stays on c4, rotates
to S. Turn 2: c4→c3, rotates to W. Turn 3: c3→b3, rotates to N. The
orbit broke; the Marcher is now spiralling. The player must reason
about the bump.

## Where it shines

- Tutorial enemy. Teaches "the telegraph is law."
- Combo with [Mirror Plate](mirror_plate.md): the mirror doesn't
  redirect movement, only effects — so Mirror Plate is *not* the
  answer to a Marcher. Forces the player to use [Anchor Flag](anchor_flag.md)
  or [Shover](shover.md) for actual interruption.
- Combo with [Domino](domino.md): a Marcher walking into a Domino
  triggers a cascade. Predictable Marcher motion makes Domino setups
  solvable.

## Where it's awkward

- A lone Marcher in open space is trivial — just sidestep its 2×2
  orbit. Needs companions or terrain to be a real threat.
- The bump-and-rotate rule is unintuitive on first read. Players
  expect "blocked → don't rotate." Documenting clearly matters.
- Two Marchers can deadlock head-to-head (each tries to step into
  the other's square). The resolution order has to be canonical, or
  symmetry breaks differently each run.

## Engine dependencies

- `Color::Neutral` for the piece.
- Signal payload for `dir` state.
- Enemy resolution phase (new — see below).
- Existing piece-movement primitives.

## New features required

- **Telegraph resolution phase.** A new turn-order step that runs
  after the player's move and before the next player turn. Iterates
  all telegraphed pieces in a canonical board order (e.g. file then
  rank ascending), resolves each one's queued action.
- **Marcher piece kind.** `Piece::Marcher { dir: Direction }`. The
  `Direction` enum already exists for tracks.
- **Resolution order rule.** When two Marchers contest the same
  square, the one earlier in iteration order moves first. The second
  bumps and rotates. Document this.

## FEN encoding

Marcher uses one payload key, `D=` (direction), reusing the
Track-direction vocabulary:

```
(P=MARCHER,C=NEUTRAL,D=N)
(P=MARCHER,C=NEUTRAL,D=E)
(P=MARCHER,C=NEUTRAL,D=S)
(P=MARCHER,C=NEUTRAL,D=W)
```

Default if `D` omitted: `N`. A FEN without `D` parses leniently — the
warn-and-default behaviour stays consistent with existing payload
handling.

## Open questions

- **Capture-on-move.** Does a Marcher walking into an enemy piece
  capture it, or bump and rotate? Current spec says capture. Argument
  for bump: makes the Marcher a pure mover, never a killer — pushes
  the player toward positioning solutions. Argument for capture:
  raises stakes, makes the lookahead matter.
- **Diagonal Marchers.** A `D=NE` variant rotates 45°? Probably a
  separate piece (call it "Drifter") rather than a Marcher variant.
- **Rotation direction.** Always clockwise, or should some Marchers
  rotate counter-clockwise? A `(P=MARCHER,D=N,R=CCW)` payload is
  cheap. Probably worth adding for puzzle variety.
- **Pre-rotation vs post-rotation rendering.** The arrow shows `dir`
  *before* the upcoming step (the next-action telegraph). Confirm
  the frontend draws it that way; otherwise the player misreads by 90°.
