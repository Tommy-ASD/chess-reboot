# The Clock

> [ENEMY] Visible countdown 3→2→1→boom. Each turn the number ticks
> down. At zero, detonates in a 3×3. Armored against capture —
> but pushing works. Reposition the bomb.

## Inspiration

Bomberman fuses. Into the Breach's Time Pods. Hoplite's lava-tile
countdowns. Any puzzle where the threat is a number ticking against
the player's verbs. The Clock is the **deadline enemy** — it doesn't
chase, it doesn't aim, it just *runs out*.

## Mechanic

A Clock carries one piece of telegraph state: `countdown ∈ {3, 2, 1, 0}`.

On the enemy resolution phase:

1. **Tick.** `countdown -= 1`.
2. **Check detonation.** If `countdown == 0`:
   - The Clock's square and all 8 neighbors form a 3×3 zone.
   - Every walkable square in the zone has its piece removed (the
     Clock itself is destroyed in the blast).
   - Unwalkable squares in the zone are unaffected; pieces "behind"
     a Block don't survive *because the blast is line-of-sight-free*,
     but tile-occupants like Block stay (Block is terrain, not a piece).
   - The square becomes Standard terrain (any conditions on the
     squares — Frozen, Brainrot — are stripped, since the explosion
     burns them off).

The Clock does **not move** on its own. It is stationary.

The Clock is **armored**: it cannot be captured by ordinary movement
(no piece can land on its square). It blocks slider paths (treat
walkability like Block while a Clock is on the square, except the
piece is a real piece, not terrain).

Push-to-move: an external push effect (from [Shover](shover.md),
from another piece's push action) **can** translate the Clock one
square in the push direction, *if the destination is walkable and
empty*. Push does not affect the countdown.

The countdown can only be reset by pushing the Clock onto a tile
with special terrain (e.g. a Frozen square pauses the tick — TBD if
this interaction is desired).

## Telegraph rendering

The piece sprite shows a large numeral matching `countdown`. At
`3` it's calm (green). At `2` it's warning (yellow). At `1` it's
critical (red, pulsing). The 3×3 detonation zone is shown as a
faint red outline around the Clock at all times — so the player
*sees* the explosion radius the entire fuse.

A countdown of `0` only exists for the one frame between tick and
detonation resolution. The sprite never renders `0`.

## Why it's interesting

The Clock is **the immovable deadline**, but the player has agency
over *where* it deadlines. You can't kill it — you can only push it.
The puzzle becomes "where on the board is the explosion useful?"
Sometimes the answer is "into a cluster of other enemies." Sometimes
it's "into an unreachable corner." Sometimes it's "into a square
where the player needs the tile cleared."

The 3-turn fuse is generous on purpose. The player has time to
plan three push-sequences before the bomb goes off. A 1-turn Clock
would be a different piece entirely (less puzzle, more reflex).

Crucially, the Clock is **predictable on a fixed schedule** — there's
no "did the bomb tick this turn?" ambiguity. Every Clock on the
board ticks every enemy phase. Multiple Clocks on different starting
countdowns create stagger patterns.

## Example puzzle

```
6 . . . . . . . .
5 . . . k . . . .         k = player king
4 . . . . . . . .
3 . . . . . . . .
2 . . . . . . . .
1 . . . . 1 . . .         '1' = Clock, countdown=1 (detonates this enemy phase!)
  a b c d e f g h
```

The Clock on e1 has countdown=1. It detonates this enemy phase
unless pushed. Its 3×3 zone is d1, e1, f1, d2, e2, f2 (plus
hypothetical rank 0 / file off-board — clipped). King on d5 is safe
from the blast. **Player can do nothing.**

But suppose:

```
6 . . . . . . . .
5 . . . k . . . .
4 . . . . . . . .
3 . . . . 1 . . .         Clock on e3, countdown=1
2 . . . . . . . .
1 . . . . . . . .
  a b c d e f g h
```

Now the blast zone is d2, e2, f2, d3, e3, f3, d4, e4, f4. The king
on d5 is *one square outside* the blast — still safe. **Player can
still do nothing.** Reading the rendered red outline confirms d5 is
outside the danger square.

The puzzle gets interesting when:

```
6 . . . . . . . .
5 . . . . k . . .         king on e5
4 . . . . . . . .
3 . . . . 1 . . .         Clock countdown=1
2 . . . . . . . .
1 . . . . . . . .
  a b c d e f g h
```

King on e5 is **inside** the blast zone (e4 corner — wait, the 3×3
centered on e3 is d2-f2 / d3-f3 / d4-f4. e5 is two ranks above e3.
Outside.) Let me redraw with the king in danger:

```
6 . . . . . . . .
5 . . . . . . . .
4 . . . . k . . .         king on e4 — INSIDE 3x3 blast zone
3 . . . . 1 . . .         Clock countdown=1
2 . . . . . . . .
1 . . . . . . . .
  a b c d e f g h
```

King on e4 dies this turn unless saved. Player has **1 Shover**
charge. Shover knight-leaps and pushes adjacent. Goal: survive.

**Option A: push the Clock.** Shover lands somewhere knight-adjacent
to e3 such that the push direction is *away from the king*. Knight
squares around e3: c2, c4, d1, d5, f1, f5, g2, g4. Of these, the
ones adjacent (king-wise) to e3 are c4, d5, f5, g4 (no — knight
squares aren't all king-adjacent to e3). Let me think: from c4,
shover lands; the piece pushed is the one adjacent to c4 in line
*from c4*. The push rule (see Shover doc) is: the piece adjacent to
the Shover's landing gets pushed directly away. So Shover landing
adjacent to the Clock means the Clock is the pushed piece, and it's
pushed in the direction from Shover-landing to Clock-square,
extended one square.

To push e3 south: Shover lands on e4. But e4 has the king on it.
Bad.

Push e3 east: Shover lands on d3. d3 is empty. From d3, the adjacent
square in line is e3 (the Clock). Pushed direction: east. Clock
moves e3 → f3. New 3×3 zone: e2-g2 / e3-g3 / e4-g4. King on e4 is
*still inside*. Doesn't help.

Push e3 west: Shover lands on f3. From f3, adjacent in line is e3.
Pushed direction: west. Clock moves e3 → d3. New 3×3 zone: c2-e2 /
c3-e3 / c4-e4. King on e4 is *still inside*. Doesn't help.

Push e3 north: Shover lands on e2. Push direction: north. Clock
moves e3 → e4. But e4 has the king on it — the push destination is
occupied. **Push fails.** Clock detonates on e3. King dies.

Push e3 south: Shover lands on e4. e4 has the king. **Bad
landing**.

Knight-leaps to e2 from where? Shover lands on e2: from any knight
square (c1, c3, d4, f4, g1, g3). Need a Shover piece pre-positioned
to make that leap legal. Assume yes — the Shover is on, say, d4.

Wait — Shover landing on e4 from a knight square doesn't push the
king, it pushes the *piece adjacent to e4*. The king sits on e4
itself, so the king is the Shover's landing target — landing on a
piece's square is forbidden (Shover can't land on occupied).

The only way out: **move the king first.** King e4 → d5. King now
outside the blast zone. Shover charge unspent for next puzzle's
problem. Clock detonates harmlessly on e3, killing nothing in its
blast (Squares d2, e2, f2, d3, e3, f3, d4, e4, f4 are all empty).

**The puzzle's lesson:** sometimes the bomb is positioned such that
no push helps — and the correct move is to dodge. Reading the red
outline tells you that dodge is the only verb.

## Where it shines

- Stagger puzzles: two Clocks at countdowns 2 and 3 require
  sequenced positioning over two turns.
- Cluster-clearing: push a Clock into a group of [Marcher](marcher.md)s
  to delete them all in one blast.
- Pair with [Shover](shover.md): the canonical "push the bomb"
  toolkit.
- Pair with [Latcher](latcher.md): the Latcher pulls the king toward
  the Clock unless you reposition first.

## Where it's awkward

- A Clock that arrives at countdown=0 in a corner with no pieces
  nearby is a wasted threat. Position design matters.
- The "armored" rule conflicts with the engine's existing capture
  semantics. A standard piece tries to capture and finds it can't —
  the move generator has to know not to generate that as a legal
  capture. New movement-stack predicate needed.
- Push interaction with other Clocks: pushing Clock A into the
  square adjacent to Clock B means B's blast zone now includes A
  (and vice versa). Cascade explosions? Spec says: each Clock
  detonates independently on its own countdown, so a chain doesn't
  auto-propagate.
- What if a Clock is pushed onto a Conduit? See [Conduit](conduit.md)
  — the Clock continues ticking; the Conduit's routing doesn't
  affect it (Clocks aren't beams).

## Engine dependencies

- `Color::Neutral`.
- Signal payload for `countdown`.
- Telegraph resolution phase.
- Push primitive (shared with Shover).
- 3×3 area-of-effect primitive (new).
- Capture-immune piece flag (new).

## New features required

- **`Clock` piece kind.** `Piece::Clock { countdown: u8 }`.
- **Capture immunity.** A piece-level flag `armored: bool` that
  rejects capture moves at generation time. Plan 10's movement stack
  modifier band absorbs this cleanly.
- **3×3 detonation primitive.** A function that, given a center
  square, iterates the 9-square neighborhood (clipped to board
  bounds) and removes pieces. Reusable for any future area-effect
  piece.
- **Push action shared semantics.** A push moves a piece one square
  in a given direction if the destination is walkable+empty;
  otherwise no-op. Spec'd once, called by Shover and any future
  push tool.

## FEN encoding

```
(P=CLOCK,C=NEUTRAL,N=3)
(P=CLOCK,C=NEUTRAL,N=2)
(P=CLOCK,C=NEUTRAL,N=1)
```

`N=` for the countdown number (1, 2, or 3 valid). A missing `N`
defaults to `3` (fresh Clock). `N=0` is technically invalid (the
detonation resolves and the Clock disappears in the same phase) —
parser warns and defaults to 1.

`(P=CLOCK,C=NEUTRAL,N=1)` is the canonical "about to detonate" form.

## Open questions

- **Detonation timing vs other phases.** Does the Clock detonate
  *before* or *after* other enemies act on the same enemy phase?
  Spec says: all Clocks decrement and detonate first, then other
  enemies (Marchers, Siege Engines, Latchers) resolve. Reason: the
  player can predict detonation timing without knowing how other
  enemies might move.
- **Friendly fire.** Does the blast kill other Clocks in its zone?
  Spec says yes (they're pieces). Cascade-explosion combos are
  fun.
- **Push past Block.** A push that would move the Clock into a Block
  square is blocked (push destination is unwalkable). Confirm.
- **Frozen Clock.** If [Frozen](../../plans/09-...) is applied to
  the Clock's square (or to the Clock piece), does the countdown
  pause? Probably yes — Frozen halts all telegraph progression.
  Excellent puzzle interaction.
- **Multiple Clocks in same blast.** If Clock A detonates and Clock
  B is inside the blast, B is destroyed without detonating. Loss of
  threat, but consistent with "blast removes pieces."
