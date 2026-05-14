# Anchor Flag

> [TOOL] Player places it on a square. Any enemy adjacent to the
> Flag at the start of its turn skips its action. One charge, lasts
> one turn, then consumed. The stun bubble buys exactly one tempo.

## Inspiration

The Pylon tile in Into the Breach. The freeze grenade in countless
roguelikes. The "buy one turn" verb — every puzzle needs a way to
pause threats temporarily. Anchor Flag is the **tempo tool**: it
doesn't kill, it doesn't push, it just *delays*.

## Mechanic

The Anchor Flag is a player tool with 1 charge per puzzle (default).
Each use places one Flag on a chosen empty square. The Flag is a
**neutral piece** (not a square condition — it occupies the square).

### Placement

The action `UseTool { tool: AnchorFlag, params: { square } }`:

- **`square`** — any empty walkable square on the board. The Flag
  occupies the square as a piece.

The Flag has a one-turn lifetime tracked by a `turns_remaining`
counter, initialized to `1`.

### Stun effect

At the start of the enemy resolution phase, before any enemy
acts:

1. For each Flag on the board, compute its 8-neighbor adjacency set
   (king-adjacent: N, NE, E, SE, S, SW, W, NW).
2. Any enemy (Neutral-colored piece) on a flagged-adjacent square is
   marked `stunned` for this resolution phase.
3. During resolution, `stunned` enemies **skip their action
   entirely**:
   - Marchers: don't step, don't rotate.
   - Siege Engines: don't load, don't fire. (A Loaded engine adjacent
     to a Flag at the start of the phase does *not* fire this turn —
     critical interaction.)
   - Latchers: don't yank. The leash still renders for the next
     turn's threat preview.
   - Clocks: don't tick. The countdown freezes at its current value.
   - Dominoes: don't fall, even if triggered. The trigger is lost
     (re-evaluates next turn).

The stun is **only** active for the one phase. After resolution, the
Flag's `turns_remaining` decrements to 0. The Flag is removed at the
end of the enemy resolution phase.

### Lifetime

A Flag exists for exactly one enemy resolution phase: placed during
the player's turn, stuns enemies during the immediately-following
enemy phase, then removes itself. The player can't extend it.

A Flag is **capturable** by enemies during their resolution — but
only by enemies *not* in its adjacency zone. A Marcher one square
outside the flag's 8-neighborhood that steps *into* the flag's
square captures it. This destroys the flag without expending its
effect; if it captures *during* resolution (and the marcher's
adjacent flag stunned other enemies), the captured flag still
counted for the start-of-phase calculation. So: enemies inside the
zone stayed stunned; enemies outside the zone could capture the flag
en route to other actions.

## Telegraph rendering

The Flag is a small banner-pole sprite with a faint glowing aura
covering its 8-neighbor zone. The aura color (e.g. soft yellow)
matches the stun-effect color used elsewhere.

When the player begins placement, all empty walkable squares glow as
placement candidates. Hovering a candidate previews the 8-neighbor
aura *and* highlights which enemies in the aura would be stunned by
this placement. That preview is essential — the player must see
which threats get neutralized before committing.

A small `1` icon on the Flag indicates `turns_remaining`. It's
always `1` in default (since the Flag lives one turn), but a future
multi-turn variant could display longer counts.

## Why it's interesting

Anchor Flag is the **"buy one turn" verb.** It's the cheapest puzzle
tool conceptually — just freeze stuff — but it interacts deeply with
threat *timing*.

The key insight: many puzzles are unsolvable in turn 1 but trivial
in turn 2. The Flag converts "I need one more move" into a placed
piece. Examples:

- [Siege Engine](siege_engine.md) loaded, threatening your king
  next turn: Flag adjacent to the engine. Engine skips its phase.
  Next turn you have time to move the king *or* push the engine.
- [The Clock](the_clock.md) at countdown=1: Flag adjacent. Clock
  doesn't tick. Countdown stays at 1. Next turn it's still 1 — the
  player has *one more turn* to push the clock to safety.
- [Latcher](latcher.md) about to yank your king into a beam: Flag
  adjacent to the Latcher. Latcher skips. Yank doesn't happen.
- [Marcher](marcher.md) about to step on your king's square: Flag
  adjacent to the Marcher. Marcher skips its step+rotate. Position
  preserved.

The Flag is **conservative** — it doesn't reposition or eliminate,
just delays. That's its design strength: it composes cleanly with
*all* other tools. A puzzle solved by "Flag turn 1, push turn 2,
shove turn 3" has a clean rhythm.

The single-charge limit means the player chooses one threat per
puzzle to delay. Most puzzles have multiple threats; the Flag picks
the one *worst* threat to defang, and the other tools handle the
rest.

## Example puzzle

```
6 . . . . . . . .
5 . . . . . . . .
4 . . . k . . . .         k = player king
3 . . . . . . . .
2 . . . . . . . .
1 S . . . . . . .         S = Siege Engine loaded, dir=E
  a b c d e f g h
```

Engine on a1 loaded, fires across rank 1. King on d4 — rank 4,
outside the beam. **Safe; no action.** Anchor Flag unused.

Now make the king vulnerable:

```
6 . . . . . . . .
5 . . . . . . . .
4 . . . . . . . .
3 . . . . . . . .
2 . . . . . . . .
1 S . . k . . . .         king on d1, INSIDE the beam path
  a b c d e f g h
```

Engine on a1 fires E this turn. King on d1 dies (beam passes
through d1 heading E). Player has **1 Anchor Flag**.

Place Flag on b1 (or any of a1's 8 neighbors: a2, b1, b2). Engine
on a1 is in the Flag's adjacency. Engine is stunned this enemy
phase. **Beam doesn't fire.**

Engine remains Loaded. Next enemy phase (turn 2): Flag is gone
(consumed after one phase). Engine fires E this turn. King still on
d1 dies *unless the player moved them*.

Turn 2 player phase: king moves d1 → d2 (or anywhere off rank 1).
Engine fires turn 2 enemy phase. Beam passes rank 1 harmlessly.
King survives.

**Lesson: Flag buys one turn of breathing room. The player must use
the bought turn for actual repositioning.**

A combined puzzle:

```
6 . . . . . . . .
5 . . . . . . . .
4 . . . . . . . .
3 . . . . . . . .
2 . . . 1 . . . .         '1' = Clock countdown=1
1 S . . k . . . .         engine + king
  a b c d e f g h
```

Two threats: engine fires rank 1 (kills king at d1), and Clock at
d2 detonates next phase (3×3 zone c1-e1, c2-e2, c3-e3 — includes
king at d1).

Player tools: **1 Anchor Flag, 1 Shover.**

Engine threat kills king on rank 1. Clock blast also includes d1
(c1-e1 row in the blast). Two ways to die simultaneously.

Option A: Flag adjacent to both. The Flag covers an 8-square aura.
If placed on c1, aura is {b1, c1, d1, b2, c2, d2, b0, c0, d0 (off
board)}. The engine on a1 is NOT in this aura. The Clock on d2 IS.

Engine fires, kills king. Bad.

Place Flag on b1: aura = {a1, b1, c1, a2, b2, c2, a0, b0, c0}.
Engine on a1 IS in aura. Clock on d2 is NOT.

Engine stunned. Clock detonates. King in blast zone. Dies.

No single Flag placement covers both. **Need a second tool.**

Plan: Flag the engine (b1), Shover the clock (push d2 north to d3,
e.g.).

Shover landing for pushing Clock on d2: must be knight-adjacent
from king, and adjacent to d2. Knight squares from d1: b2, c3, e3,
f2. Knight squares from king d1: b2 (adj to a2, b1, c2, b3 — not
d2), c3 (adj to b2, b3, b4, c2, c4, d2, d3, d4 — c3 adj to d2: yes
wait c3 and d2 are diagonal, that's adjacent? c3 to d2 is one file
right, one rank down — yes king-adjacent). So Shover lands on c3.

From c3, push direction toward d2 is SE. `L+D = d2` (Clock).
`L+2D = e1`. e1 empty. **Push succeeds.** Clock moves d2 → e1.

But wait — e1 is rank 1, in the engine's beam path. The Clock at e1
gets killed by the beam... except the engine is *Flagged*, stunned,
doesn't fire. So Clock lives at e1.

New 3×3 blast zone of Clock (still countdown=1, since being moved
doesn't reset the timer): d0-f0 (off-board), d1-f1, d2-f2. King on
d1 IS in the zone. **Detonates this enemy phase. King dies.**

Oh no. Push made it worse.

Try push direction NW from c3: `L+D = c3+NW = b4`. b4 is empty.
No piece to push at b4. Shover leap happens, no push effect. Waste.

Try landing the Shover from a different knight square. Knight squares
from king d1: b2, c3, e3, f2. Of these, knight-adjacent to d2:
b2 (b2 adj to d2? b2 to d2 is 2 files, 0 ranks — not adjacent). 
c3 (adj to d2: yes). e3 (e3 adj to d2: yes). f2 (f2 adj to d2:
two files away — not adjacent).

Shover landings adjacent to d2 from king's knight moves: c3 and e3.

Shover on e3, push toward d2 (SW direction): `L+D = d2`, `L+2D =
c1`. c1 is empty (assuming). **Push succeeds.** Clock moves d2 →
c1. New blast zone: b0-d0 (off-board), b1-d1, b2-d2. King on d1 IS
in the zone. Same problem.

Push direction E from e3: `L+D = f3`. Empty. No push.

Push from c3, direction E: `L+D = d3`. Empty. No push.

**No Shover push gets the Clock out of king-adjacency.** The blast
zone is 3×3 — any single-square push leaves the Clock adjacent to
its old position, which is adjacent to the king.

Solution: don't push the clock; **move the king instead.**

King d1 → d2 — but d2 has the Clock. King d1 → c2 (legal, empty).
Now king on c2.

But: place Flag on b1 first (to stun engine), Shover unused, king
moves c2.

Turn 1 enemy phase: Engine stunned (Flagged). Clock at d2 detonates
(countdown=1, no flag adjacent). 3×3 blast: c1-e1, c2-e2, c3-e3.
King on c2 IS in the blast. Dies.

Hmm. Move king elsewhere.

King d1 → c3: empty? Yes. Is c3 in the Clock blast? Clock on d2,
zone c1-e3 (3×3 around d2). c3 is in the zone (c3 = file c, rank 3,
which is one of c2/d2/e2's row... wait, 3×3 around d2 is files c-e,
ranks 1-3). c3 IS in. Dies.

King d1 → e2: e2 in blast (yes). Dies.
King d1 → e1: in blast (no — actually yes, e1 is files e, rank 1,
in the c1-e1 row of the zone). Dies.
King d1 → c1: in blast. Dies.

Every king-adjacent square is in the blast.

What if king moves further away? King is on d1; king's moves are
1-square. Can only reach adjacent squares. Adjacent to d1: c1, c2,
d2, e2, e1. All in or adjacent to the blast. Only e1, c1, d2 are
in the blast itself. c2 and e2 are too, since the 3×3 of d2 includes
ranks 1-3 and files c-e. ALL king-adjacent squares are in the blast.

**Without Flag-on-Clock, king dies.** Reassign Flag.

Place Flag on d1 (king's square)? No — Flag needs an *empty* square.

Place Flag on c2 (adjacent to clock at d2): aura includes d2. Clock
stunned. Countdown stays at 1.

Now engine fires (not flagged anymore). Beam rank 1: a1-h1. King on
d1 dies.

So: Flag stuns one threat, leaving the other to kill. **No
single-tool solution exists for this puzzle.**

What if the player also has 1 [Mirror Plate](mirror_plate.md)?

Flag on c2 (stuns Clock). Plate on c1 with `Backslash` (reflects E
beam → S, off-board). Engine fires rank 1 a1→b1→c1. At c1 reflects
S, off-board. Beam dies. King safe.

Turn 2: Flag gone. Clock still at countdown=1 (frozen by Flag). Push
Clock with Shover... but we don't have Shover in this puzzle. Stuck.

The fully-equipped puzzle:

Tools: **1 Flag, 1 Plate, 1 Shover.**

Turn 1 player:
- Flag on c2 (stun Clock).
- Plate on c1 `\` (redirect engine beam).
- Shover: need to set up turn 2 success. King move? No, Shover is a
  tool, not a king move. King can move *separately* from tool use
  (each turn the player gets a king move + one tool use, assuming
  variant rules).

Let's say turn-rule: one king move plus one tool use per turn. King
d1 → d2? d2 has Clock; can't move there (Clock is armored,
unpassable). King → c2? c2 has Flag (after placement). c2 occupied.
King → e2? Empty, in blast — but Clock is stunned, no detonation
this turn. e2 safe this phase.

Turn 1 actions:
- King move: d1 → e2.
- Tool 1: Flag on c2.
- Tool 2 (if rules allow): Plate on c1 `\`.

(Two tool uses per turn? That depends on variant rules. Some
variants: one tool per turn. Then we need 3 turns of king-move +
single-tool-use.)

Assume one tool per turn:

Turn 1: King d1 → e2 (out of rank 1, into blast zone but Clock will
be flagged so no detonation). Tool: Flag on c2 (stun Clock).
Engine fires rank 1: a1-h1, no king. Clock stunned, no detonation.

Turn 2: King moves. e2 → e3 (out of blast zone). Tool: Plate on c1
`\` (in case Clock isn't stunned this turn, redirect engine when it
reloads).

But: engine fired turn 1, returned to Idle, will move turn 2 enemy
phase to b1 (D=E means it moves east one). Then loads. Won't fire
again until turn 3.

Clock turn 2: Flag expired. Clock ticks to 0 → detonates. 3×3 around
d2: zone c1-e3. King on e3 IS at the corner. e3 is in the zone. Dies.

Alternative turn 2: King e2 → f3 (out of zone entirely). Tool:
Shover (push the engine off rank 2 or push the clock).

Actually the Clock is going to detonate turn 2 regardless. Need to
prevent detonation. Push Clock with Shover — but as we computed,
every push leaves it adjacent to its old position, and the zone
shifts but always includes the king's escape path. Unless the king
*also* moves far enough...

This puzzle is too constrained for the tools given. The point isn't
to solve it cleanly — it's to demonstrate that **Flag is one half of
a multi-tool answer**. A puzzle designed around Flag specifically
would give the player one threat where Flag-once + king-move solves
it.

## Where it shines

- The "I just need one more turn" tool. Composes with every other.
- Pair with [Shover](shover.md): Flag stuns turn 1, Shover repositions
  turn 2.
- Pair with [Mirror Plate](mirror_plate.md): Flag the engine, Plate
  the rerouted beam.
- Stun-stack: a Flag adjacent to multiple enemies stuns all of them.
  The 8-neighbor zone is wide enough that 2-3 enemies can be defanged
  in one placement.

## Where it's awkward

- One-turn lifetime is brutal — players who place it wrong waste a
  charge. A 2-turn variant exists but trivializes some puzzles.
- "Adjacent to flag" rule means enemies at distance 2+ are
  unaffected. A long-range Siege Engine 3 squares from the king
  can't be flagged unless the player places the flag near the
  engine itself. Spec'd: yes, place the flag near the engine, not
  near the king.
- A Flag's adjacency aura overlaps enemy positions confusingly.
  Frontend rendering must distinguish "this enemy is in the aura
  but won't be stunned (e.g. not telegraphed)" from "this enemy is
  stunned."
- Can a Flag be placed adjacent to another Flag? Probably yes (no
  rule against it). Two Flags adjacent each other: their stun
  zones overlap, just doubles the aura coverage. Harmless.

## Engine dependencies

- Player tool inventory (shared with Shover/Plate).
- Neutral-colored piece slot for the Flag piece itself.
- Stun flag on enemies (per-phase, not persistent).
- Telegraph resolution phase with a "stun check" prelude step.

## New features required

- **`AnchorFlag` piece kind.** `Piece::AnchorFlag { turns_remaining: u8 }`.
  Neutral color. No movement.
- **Stun marker.** Per-piece transient flag set during stun check,
  cleared after resolution. Doesn't persist in FEN (since it's
  recomputed every phase).
- **Stun check phase.** Runs at the start of enemy resolution.
  Iterates flags, marks adjacent enemies stunned. Each enemy's
  resolution code checks the stun flag before acting.
- **Flag decrement and removal.** After enemy resolution,
  `turns_remaining -= 1`; if zero, remove flag from board.

## FEN encoding

The Flag is a piece, encoded standardly:

```
(P=FLAG,C=NEUTRAL,T=1)
```

`T=` for `turns_remaining`. Default 1. A `T=2` flag (variant) lasts
two phases.

Charges in tool inventory: `T:FLAG=1`.

## Open questions

- **Stun timing.** Does the Flag stun enemies *placed* turn N during
  enemy phase N (same turn placement)? Spec: yes — Flag placed
  during player turn N is active for enemy phase N. The whole point
  is buying *the immediate* turn.
- **Stun and Clock countdown.** Spec says: Clock under a Flag's
  aura doesn't tick. This is the cleanest "freeze" interaction.
  Alternative: Clock ticks but doesn't detonate (so countdown=0
  occurs but the blast is skipped). That's more nuanced but creates
  weird state. Stick with "doesn't tick."
- **Stun and Domino trigger.** A Domino in the Flag's aura: does
  it still *check* for triggers? Spec: the trigger check happens at
  end of player turn (before stun check). So the trigger fires
  (Domino transitions to Falling), but during enemy phase the
  Falling resolution is skipped. The Domino stays Falling — next
  turn it falls. The Flag delays the fall by one turn.
- **Flag-on-Flag.** Stacking? Spec: not allowed (one Flag per square,
  since they're pieces).
- **Flag capturable.** Yes, by an enemy adjacent to the flag's
  square but *not* in the flag's aura — i.e. an enemy stepping
  *into* the flag's square. Such an enemy is not in the aura at
  start-of-phase, doesn't get stunned, executes its move, captures
  flag, ends turn. The captured flag does NOT consume the stun
  effect on other enemies — they were already marked stunned.
- **Tool inventory shared with player king?** Or per-side? In a
  two-player chess game with Anchor Flags, both sides have their
  own inventory. In single-player puzzle, the inventory belongs to
  the puzzler.
