# Italian Brainrot

> A 3-legged shark-shaped zigzag mover that pulses every Switch on
> the board the moment it's captured. Also known as Tralalero.

## Inspiration

The "Italian brainrot" meme cluster — Tralalero Tralala (the
shark in Nikes), Bombardiro Crocodilo, Tung Tung Tung Sahur, etc.
— is a category of AI-generated absurdist character art with
nonsense names. The Tralalero shark's "tralala tralala" cadence
became the genre's signature audio. This piece weaponizes that
cadence: when the shark dies, it emits its sound and the whole
signal substrate twitches.

Strip the paint: two interesting mechanics layered.
1. **Forced zigzag.** A piece that *cannot* move in a straight
   line — every step must change direction. This creates
   genuinely novel reachability patterns. The set of squares
   reachable in 3 steps via forced direction-change is its own
   geometry, distinct from king, knight, bishop, rook.
2. **Death-rattle signal pulse.** A capture-triggered global
   toggle of every Switch on the board. The signal substrate
   (plan 08) already supports per-Switch toggles; what's new is
   a piece-driven global broadcast. Turns Italian Brainrot into
   a panic button — you can deliberately sacrifice it to flip
   the signal state.

## Mechanic

### Movement: 3-step zigzag

Each turn, the Italian Brainrot moves **exactly 3 squares total**
in a single turn — three sequential single-square steps in 8
king-style directions. Each successive step must be in a
**different direction** from the immediately previous step. No
straight lines of two consecutive steps.

Concretely:

1. Step 1: choose any of 8 king directions, move 1 square.
2. Step 2: choose any of 8 king directions *except the one used
   in step 1*. Move 1 square.
3. Step 3: choose any of 8 king directions *except the one used
   in step 2*. Move 1 square.

Step 3 may reuse step 1's direction (the constraint is "different
from immediately previous," not "all three different").

Each step must land on a walkable square. If a step lands on:

- **Empty walkable:** continue.
- **Friendly piece:** the step is illegal. Try a different
  direction. If no legal step 2 exists, the path is
  abandoned.
- **Enemy piece:** the step is a **capture**. The captured
  piece is removed. The Italian Brainrot continues moving.
  *Multiple captures in one turn are possible* — each capture
  during the 3-step path is a separate capture event.

The piece **must** complete all 3 steps if any legal 3-step path
exists. If no legal 3-step path exists (the piece is boxed in
such that no legal direction-change sequence reaches 3 steps),
the piece does not move this turn. Partial paths (1-step or
2-step moves) are **not** legal.

### The "tralala" pulse

When the Italian Brainrot is **captured** (not when it captures
others — when it itself is captured by an enemy):

1. The capture resolves normally — Italian Brainrot is removed
   from the board.
2. Immediately, *all* Switches on the board are **toggled**.
3. Signal propagation from those toggles resolves immediately
   via the standard signal substrate. Gates open/close, plates
   reconfigure, etc.

The pulse is **simultaneous** for all Switches. Order of
resolution follows the existing signal-substrate ordering rules
(plan 08). The pulse is unconditional — it fires regardless of
which player captured, which piece captured, or which Switches
were already in which state. Every Switch flips its boolean.

A Goblin kidnap **is not a capture** for this purpose — the
Goblin pickup doesn't fire the pulse (the piece returns home
later). Only true removal-via-capture fires it.

If an Italian Brainrot is captured indirectly (e.g., shoved off
the board edge by a Gooner), this also counts as a capture and
fires the pulse.

### En passant analog

The Italian Brainrot's mid-step captures behave like a multi-
capture sequence. There is no en-passant analog — captures
happen at each step landing.

## Why it's interesting

1. **Forced direction change.** This single constraint produces
   surprisingly intricate reachability. The set of squares 3
   steps away is the *target lattice*: not concentric rings,
   but a non-trivial shape that excludes some adjacent squares
   (you can't reach a square diagonally adjacent to the start
   via 3 zigzag steps to all-different-directions — wait, you
   can if step 1 = N, step 2 = E, step 3 = S, ending diagonally
   NE-ish... actually let's prove it empirically per square,
   not in this doc). Players will need a board-overlay UI hint.
2. **Multi-capture in one turn.** Three-step path means up to
   three captures per turn. This is rare in chess. Properly
   defended pieces are safe; loose pieces in zigzag range are
   vulnerable.
3. **Sacrifice as signal weapon.** When Switches drive Gates and
   plates drive Switches, deliberately throwing your Italian
   Brainrot into a capture is a way to *reset the signal
   substrate.* This is huge in signal-heavy variants — the
   piece is a chaos button.
4. **Predator/prey duality.** The piece is dangerous (multi-
   capture) but its death has the largest non-piece consequence
   in the game. Both players have incentive to keep it alive
   in some board states and incentive to kill it in others.

## Example scenarios

1. **Triple-capture rampage.** Italian Brainrot on d4. Enemy
   pawns on e5, e6, and d7. Path: d4 → e5 (capture, dir NE) →
   e6 (cannot — same direction N). Try: d4 → e5 (capture, NE)
   → d6 (NW, valid) → e7 (NE, different from NW, valid).
   Captures one piece, ends on e7. Other paths exist.
2. **Sacrifice for signal flip.** Italian Brainrot is on a
   square attacked by enemy queen. Switches on the board
   currently control gates that block white's attack lanes.
   White doesn't move the Italian Brainrot away — enemy queen
   captures it next turn. Pulse fires, gates flip open, white's
   attack is now possible.
3. **Direction-change deadlock.** Italian Brainrot on a1 with
   friendly pieces on b1 and a2. Step 1 options: NE (b2),
   N (a2 - friendly, blocked), E (b1 - friendly, blocked).
   Only NE works. Step 2 from b2: any direction except NE.
   Pick N to b3. Step 3 from b3: any direction except N. Many
   options. Path: a1 → b2 → b3 → c4 (say).
4. **Switch chaos.** Board has 8 Switches in various states.
   Italian Brainrot captured. All 8 flip. The combinatorial
   consequence is large — Gates open/close, plates reconfigure.
   Worth a tactical study.

## Where it shines

- **Signal-heavy variants** — the pulse is most disruptive
  when Switches actually control important things.
- **Crowded middlegames** — multi-capture potential is real.
- **Defensive parity puzzles** — designing a board where the
  signal flip *helps* or *hurts* you predictably.

## Where it's awkward

- **Sparse signal boards** — if there are no Switches, the
  pulse fires harmlessly. Most of the piece's flavor is wasted.
- **Path enumeration** — the engine has to enumerate 3-step
  zigzag paths. 8 × 7 × 7 = 392 maximum step combinations per
  turn before walkability filtering. Tractable but
  non-trivial.
- **Tracking which directions are "previous"** — adds a
  per-step state to move generation. Slight code complexity.
- **Mandatory full path** — if no 3-step path exists, the
  piece is stuck. Frustrating UX. Could be relaxed to "as
  many steps as possible" but determinism of the rule
  matters.

## Engine dependencies

- **Signal substrate** (plan 08) — Switch toggle as an action
  exists. New: global broadcast trigger.
- **Multi-step move primitives** — Monkey already does jump
  chains; the Italian Brainrot's path is similar in structure
  but with direction constraints instead of jump targets.
- **Capture pipeline with broadcast-on-removal hook** — new
  surface. Plan stub: piece-types can register a "on captured"
  callback that runs after the piece is removed but before
  control returns to the next turn.
- **Multi-capture in one turn** — already supported by Monkey's
  jump chains.

## New features required

- **Direction-change-constrained path generator.** Plan stub:
  add a `zigzag_path(start, length, direction_constraint)`
  primitive that enumerates valid n-step paths. Useful for
  future fairy pieces.
- **On-captured callback.** Plan stub: piece types can declare
  an effect that fires after their removal. The capture pipeline
  invokes it post-removal. Plan-10's modifier stack is the
  natural home.
- **Global Switch toggle action.** Plan stub: a new signal
  action type "broadcast toggle" applies to every Switch on
  the board atomically.

## FEN encoding

Symbol: `IB` for Italian Brainrot (white), `ib` (black). Or `TR`
for Tralalero if the name "Italian Brainrot" is too long. Pick
one.

No per-piece state — the piece is stateless. (The direction
constraint is per-move, not persistent.)

```
(P=IB)      # white Italian Brainrot, no payload
(P=ib)      # black
```

The "tralala" pulse is a runtime event, not encoded in FEN.

## Open questions

- **Is the multi-capture intentional?** It's powerful. Probably
  yes — the meme demands the shark eats things in a frenzy.
  But balance-wise, it might need a "captures at most one per
  turn" cap.
- **Pulse-on-capture vs pulse-on-move.** Currently: pulse on
  capture-of-Italian-Brainrot. Alternative: pulse on every
  move of Italian Brainrot. Probably weaker; the death-rattle
  framing is sharper.
- **What about Switch chains?** If Switch A controls Gate B
  which when open lets Track tile C be reached by Train D
  which trips Plate E which fires Switch F — does the
  cascading propagation happen synchronously after the pulse?
  Plan 08 already specifies signal-substrate timing; reuse it.
- **Does the Italian Brainrot pulse fire when it dies to its
  own owner's pieces** (e.g., Costco Guy carrying it gets
  captured)? Costco Guy capture removes the carried IB →
  that's a capture → pulse fires. Yes.
- **Frozen IB** cannot move. Captures normally if attacked.
  Pulse on capture still fires.
- **Brainrot'd IB** — the irony is delicious. Brainrot already
  has defined mechanics; the IB obeys them. The pulse on
  capture is independent.
- **Path through Ohio.** A 3-step zigzag may end on Ohio.
  Rotation applies. The IB's own moveset is direction-
  constrained, not direction-specific, so the rotation has no
  meaningful effect (the constraint is "different from
  previous," symmetric under rotation). Worth a confirming
  test.
