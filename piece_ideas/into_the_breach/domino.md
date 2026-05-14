# Domino

> [ENEMY] Stationary until something steps in front of it. Then it
> tips into that square next turn, becoming a new Domino facing the
> direction it fell. Cascade puzzles.

## Inspiration

Domino chains. Sokoban-style triggered movers. The "wake on contact"
enemies in Hoplite. Every puzzle game with "do not nudge the
sleeping thing." Domino is the **conditional cascade enemy** — it
sits idle, but once disturbed, the chain reaction is mechanical and
predictable.

## Mechanic

A Domino carries two pieces of telegraph state:

- `dir ∈ {N, E, S, W}` — the direction it will fall when triggered.
- `state ∈ {Standing, Falling}` — current phase.

### Trigger condition

At the end of the player's turn (before enemy resolution phase), for
each Domino in `Standing` state:

- Check the square directly in front of it (the `dir`-adjacent
  square). If that square is now occupied by *any piece* (player,
  enemy, neutral) and was empty at the start of the player's turn,
  the Domino transitions to `Falling`.

The trigger fires once per turn at most. A Domino that was already
`Falling` doesn't re-trigger.

### Resolution

On the enemy resolution phase, for each `Falling` Domino:

1. The square the Domino currently occupies becomes empty.
2. A new Domino spawns on the `dir`-adjacent square (its target):
   - If the target is unwalkable or off-board, the Domino is
     destroyed (it falls off the edge or into a wall and shatters).
   - If the target has a piece, the piece is captured (removed) and
     the Domino spawns on top of it.
   - If the target has another standing Domino, that Domino is
     **chain-triggered**: it transitions to `Falling` for the *next*
     enemy resolution phase. The current Domino still spawns on the
     target square (replacing the chain-triggered one). Wait — they
     can't both occupy. Re-spec: when a Falling Domino lands on a
     standing Domino, the existing Domino is destroyed (cascaded
     "absorbed") and the falling one takes its place, *with `dir`
     preserved from the falling Domino*. Cascades are achieved by
     chain placement of Standing Dominoes in a row.
3. The new Domino inherits the same `dir`. It starts in `Standing`
   state — if its new front square is empty at the start of next
   turn, it stays standing. Otherwise it triggers again.

Result: the Domino "moves" by one square per turn after the trigger,
preserving its facing, until something stops it (wall or empty
trigger-front).

## Telegraph rendering

The piece sprite shows a tall rectangular tile with a small arrow
indicating `dir`. When `state == Standing`, the tile is upright. When
`state == Falling`, the tile is rendered mid-tip — drawn rotated 45°
toward `dir`, with a translucent ghost on the target square showing
where it lands.

A subtle dashed line projects one square forward (the trigger zone).
When something enters that zone during the player's turn, the line
"lights up" — signaling that the Domino will fall next turn.

This live feedback is essential: it teaches the player that
adjacency-into-the-front is the trigger, while adjacency-into-the-side
is safe.

## Why it's interesting

Dominoes turn the board into a **chain machine**. A line of three
Dominoes facing east acts as a runway: poke the first one and a
projectile travels three squares east, capturing anything in its
path. The player can build their own chain reactions.

The trigger condition is what makes it puzzle-grade: not every
movement triggers — only stepping *in front*. A piece moving past a
Domino's side is safe. A piece moving *behind* is safe. Only the
front square wakes it.

Combinations:

- **Marcher into Domino.** A predictable Marcher path that passes
  through a Domino's front triggers the chain. Player plans the
  Marcher's arrival to coincide with a useful Domino direction.
- **Domino chain into Clock.** Three east-facing Dominoes in a row
  ending at a Clock: poke the first, all three fall, the third lands
  on the Clock and destroys it (the chain absorbs the Clock — or
  detonates it, designer choice).
- **Domino into Conduit.** A Falling Domino landing on a Conduit:
  does the Domino teleport out the other Conduit? Spec'd as: yes,
  Domino movement is "piece movement" not "telegraphed effect," so
  it doesn't route through the Conduit. (Conduit only routes
  telegraphed beams/effects.)

## Example puzzle

```
6 . . . . . . . .
5 . . . . . . . .
4 . . . . . . . .
3 . . D D D . . .         D = Dominoes, all dir=E, all Standing
2 . . . . . . . .
1 . . . . . . . k         k = player king
  a b c d e f g h
```

Three Dominoes on c3, d3, e3, all facing east. King on h1, must
survive 3 turns. Player has **1 Anchor Flag** charge.

Threat analysis: no enemies are moving on their own. The Dominoes
are inert. Unless something disturbs them, nothing happens.

But: suppose the puzzle adds a [Marcher](marcher.md):

```
6 . . . . . . . .
5 . . . . . . . .
4 . . M . . . . .         M = Marcher, dir=E
3 . . D D D . . .
2 . . . . . . . .
1 . . . . . . . k
  a b c d e f g h
```

Marcher on c4, facing east. Marcher's predictable 2×2 orbit:
c4→d4→d3→c3→c4. **On turn 2, the Marcher walks from d4 to d3.** d3
is the front-adjacent square of c3 (the leftmost Domino, which faces
east — its "front" is d3). Wait, c3's front-adjacent E is d3. Yes.
The Marcher entering d3 triggers c3 Domino.

Trigger fires at end of turn 2 player phase: c3 Domino becomes
`Falling`. Enemy resolution: c3 Domino tips east, lands on d3
(which has the Marcher). Marcher captured. New Domino on d3 facing E.

Turn 3 player phase: Marcher is dead; no triggers happen. The new
Domino on d3 has e3 as its front. e3 has another standing Domino,
*not empty*, so the trigger condition ("was empty, now occupied")
doesn't fire (e3 wasn't empty to start). Chain doesn't propagate.

Player did nothing. King still on h1. **Puzzle solved by reading
the Marcher's path.**

A harder version:

```
6 . . . . . . . .
5 . . . . . . . .
4 . . M . . . . .         Marcher dir=E
3 . . D D D k . .         king on f3 — IN DANGER if chain propagates
2 . . . . . . . .
1 . . . . . . . .
  a b c d e f g h
```

Same setup but king on f3. If the chain propagates fully, the third
Domino tips east and lands on f3, killing the king. Does it
propagate?

Turn 2 enemy: c3 Domino falls onto d3 (capturing Marcher). New
Domino on d3 facing E. Its front is e3 — occupied by another
standing Domino. **Trigger condition needs "was empty at turn start,
now occupied."** At start of turn 2, e3 had a Domino. Still has.
No trigger.

But wait — when the new Domino *spawns* on d3, e3 is its front.
Same logic applies. The spawn doesn't re-evaluate triggers (those
fire at end of *player* turn). So at end of turn 3 player phase,
e3's status is checked relative to start of turn 3. e3 had Domino at
start of turn 3, has Domino at end. No state change. No trigger.

**Chain doesn't propagate.** King on f3 is safe — protected by the
fact that the middle and right Dominoes' front-squares were already
occupied at the moment of evaluation.

But: what if a player piece moves *out* of e3 during turn 3? If e3
had a player piece at start, and the player moved it elsewhere, then
e3 is now empty — and d3's Domino's front is empty — and *next* turn
nothing triggers. The chain stays dormant.

The full puzzle teaches: **adjacency alone isn't enough — the trigger
is a state-change.**

A nastier version makes the Marcher's path *miss* the Domino. Then
the player must engineer a trigger themselves (e.g. move their own
piece into the front square) to redirect the chain *as offense*.

## Where it shines

- Chain-reaction puzzles. The player constructs a Rube Goldberg from
  existing telegraphs and a single trigger.
- "Use the enemy against itself" — Marchers walking into Domino
  fronts, Latcher-yanks pulling a piece into a Domino trigger zone.
- Domino-vs-Domino: a Falling Domino destroys the standing one it
  lands on, which prevents accidental infinite chains.

## Where it's awkward

- The trigger condition ("was empty at turn start, now occupied") is
  bookkeeping-heavy. Engine must snapshot board state at the start
  of each player turn for comparison. Slight perf cost; probably
  worth it.
- A Domino with an unwalkable front (Block, edge) is permanently
  inert — fine, but visually confusing if the player doesn't read
  the terrain.
- Dominoes that face into Conduits: spec'd as "no routing," but
  players will *want* it to route. Maybe a flag.
- "Falling" rendering may look like real movement; player might
  expect chess-piece-style capture rules (e.g. that they can capture
  the Falling Domino mid-fall). Spec: no — the Falling state is
  internal, the Domino is not capturable while resolving.

## Engine dependencies

- `Color::Neutral`.
- Signal payload for `dir` + `state`.
- Telegraph resolution phase.
- Board snapshot at player-turn-start for trigger comparison (new).
- Piece-move primitive (for the tip-into-target step).

## New features required

- **`Domino` piece kind.** `Piece::Domino { dir, falling: bool }`.
- **Turn-start board snapshot.** Cache the board's piece map at the
  start of each player turn so triggers can compare. Cheap if just
  an array of `Option<PieceKind>` per square.
- **Trigger evaluation step.** Runs at end of player turn, before
  enemy resolution. Iterates Dominoes, compares front-square to
  snapshot, transitions to Falling if appropriate.
- **Falling resolution step.** Part of enemy resolution. Tips each
  Falling Domino, handles capture / wall-edge / chain absorption.

## FEN encoding

```
(P=DOMINO,C=NEUTRAL,D=E)              # standing, facing east
(P=DOMINO,C=NEUTRAL,D=E,S=F)          # falling, facing east (mid-resolution save)
```

`D=` for direction (same vocabulary as Marcher/Siege). `S=` for
state, with `S=S` (standing) as default and `S=F` (falling) for the
rare mid-phase save. Round-trips cleanly.

## Open questions

- **What pushes/pulls trigger?** A push effect that moves an
  external piece into the Domino's front: triggers? Spec says yes —
  any state change of the front square from empty-at-turn-start to
  occupied-at-turn-end fires the trigger, regardless of how the
  piece arrived.
- **The Domino itself moving onto another Domino's front.** A
  Falling Domino lands on square X. X is the front-adjacent of
  another Domino Y. Does Y trigger? Probably yes — but it's a
  one-turn delay (Y triggers at end of *next* player turn, not
  immediately). Confirm in spec.
- **Diagonal Dominoes.** A `dir=NE` Domino is geometrically clean
  but tips into a diagonal square — slightly weird visually.
  Probably leave it 4-directional.
- **Rotational Dominoes.** Should Dominoes spin like Marchers after
  tipping (e.g. land facing 90° rotated)? Probably no — preserves
  facing so chains are predictable.
- **Trigger-by-projection.** Should a [Siege Engine](siege_engine.md)
  beam crossing the Domino's front square trigger the Domino, even
  though the beam doesn't leave a piece behind? Tempting but
  probably no — keep triggers as physical-occupancy events only.
