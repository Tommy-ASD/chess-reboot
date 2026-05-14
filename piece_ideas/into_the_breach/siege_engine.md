# Siege Engine

> [ENEMY] Spends a turn loading, then fires a line beam down its
> facing that obliterates everything in the row. While loaded, the
> fuse is a visible bar — and the engine is immobile.

## Inspiration

The Vek artillery in Into the Breach. The fuse-bomb in Hoplite. Every
puzzle game's "this thing will kill you in one turn unless you do
something specific" piece. The Siege Engine is the **wide threat** —
it doesn't capture one square, it razes a line.

## Mechanic

A Siege Engine carries two pieces of telegraph state:

- `dir ∈ {N, E, S, W}` — facing.
- `state ∈ {Idle, Loaded}` — loading phase.

On the enemy resolution phase:

- **If `state == Idle`:** the engine moves one square in `dir` if
  legal (walkable + empty or capturable). Then transitions to `Loaded`.
- **If `state == Loaded`:** the engine does not move. It **fires** —
  emits a beam along `dir` from its square. The beam travels until
  it hits the board edge or an unwalkable square. Every walkable
  square in the beam's path is processed: any piece on it (other
  than the Siege Engine itself) is removed. Then `state` transitions
  back to `Idle`.

The beam is one square wide. It does not stop at the first piece —
it passes through and kills everything in line. (Adjust this rule
in a variant if a "pierces 1" Siege Engine is desired.)

The engine itself is a normal-armor neutral piece — capturable by
any standard means while idle. While loaded, it's still capturable,
but capturing it requires landing on its square, which means walking
into the beam's origin. The beam fires *during the enemy phase*; if
the player captures the engine before the enemy phase, the beam
never fires.

The classic puzzle move: **rotate it.** The engine can be pushed
(e.g. by [Shover](shover.md)) one square. It can also be redirected
by [Mirror Plate](mirror_plate.md) — the beam reflects, hitting a
different rank/file entirely.

## Telegraph rendering

The piece sprite shows:

1. The cannon barrel rotated to match `dir`.
2. A fuse bar: empty when `Idle`, half-full when `Loaded`. (The
   "half-full" framing is intentional — players read "halfway to
   exploding" as the alarm signal.)
3. A faint dashed line projecting from the muzzle along `dir`, all
   the way to the next unwalkable square or board edge. This is the
   threat zone. It updates live as the player moves pieces during
   their turn — moving a Block into the line cuts the dashes short.

The threat zone is what the player actually reads. The fuse bar is
the "when": idle = "no beam this turn," loaded = "beam fires now,
plan accordingly."

## Why it's interesting

A line attack is fundamentally different from a point attack. The
Marcher threatens one square; the Siege Engine threatens an entire
file. The puzzle structure flips: instead of "where will the enemy
be?", it's "where is the safe zone in this rank?"

The two-turn loading cycle means every Siege Engine is a metronome.
The player gets exactly one full turn to react to a loaded engine
before it fires. That's the entire game design: a forced rhythm.

Combinations:

- Two perpendicular Siege Engines create a kill-cross.
- A Siege Engine pointed at a [Conduit](conduit.md) splits the beam
  out of every other Conduit on the board — chain firing.
- A Mirror Plate in the beam path turns it into a bouncing puzzle.

## Example puzzle

```
8 . . . . . . . .
7 . . . . . . . .
6 . k . . . . . .         k = player king
5 . . . . . . . .
4 . . . . . . . .
3 . . . . . . . .
2 . . . . S . . .         S = Siege Engine, dir=W, state=Loaded
1 . . . . . . . .
  a b c d e f g h
```

The Siege Engine on e2 is loaded, facing W. Its threat zone is
**rank 2, files d→a**. That doesn't currently hit the king on b6 —
but the engine will rotate (no, it won't; Siege Engines don't rotate
on their own; rotation needs a tool). Wait — re-read the spec.
Confirmed: Loaded engines fire on the upcoming enemy phase. The
beam goes left across rank 2. King on b6 is *safe this turn*.

But after firing, `state` returns to `Idle`. Next enemy phase the
engine moves one square `dir=W` to d2 (still facing W). The turn
after that, it loads. The turn after that, it fires d2's rank.

The puzzle expands:

- Player has **1 Anchor Flag** and **1 Mirror Plate**.
- Goal: survive 4 turns with the king on b6.

Turn 1 (player): Place Mirror Plate on b2 (the column where the
king sits). This sets up the beam reflection later.

Turn 1 (enemy): Engine fires. Beam goes e2→d2→c2→b2. At b2, the
Mirror Plate reflects 90°. The beam now goes b2→b3→b4→b5→b6 — and
kills the king. Mirror Plate was *the wrong move*. Try again.

Turn 1 (player, retry): Place Anchor Flag on f2 (adjacent to the
engine). Anchor Flag freezes adjacent enemies for one turn.

Turn 1 (enemy): Engine is anchored. State stays `Loaded`. No beam.
No movement.

Turn 2 (player): Anchor Flag consumed. The engine is still loaded.
Player has used 0 of their Mirror Plate charge. Place Mirror Plate
on b2.

Turn 2 (enemy): Beam fires e2 W. Beam reflects at b2 down to b1
(reflection direction depends on Mirror Plate orientation — see that
piece's doc; assume the player chose the orientation that bounces S).
Beam dies at b1. King safe.

Turn 3 (enemy): Engine moves to d2 (idle), then loads. Wait —
movement and loading are the same phase? Re-read spec.

Re-spec'd: Engine in Idle state **moves** this turn, then becomes
Loaded. It doesn't both move and fire in one phase. So turn 3 enemy:
e2 → d2, becomes Loaded.

Turn 4 (player): King still on b6. No tools left. Player must move
king out of file. King b6 → b7 (or anywhere off rank-2 *and* off
b-file, but actually the engine fires along rank 2, so king at b6
was always safe from the rank — the threat was the reflected beam,
not the original. With Mirror Plate gone, king on b6 is safe). Done.

The full puzzle teaches: **a tool that seems perfect can be a death
trap.** The Mirror Plate alone, played turn 1, kills the king. The
Anchor Flag must come first to buy the right tempo.

## Where it shines

- The cornerstone enemy. Every Into-the-Breach puzzle has one
  Siege-Engine-flavored "this kills a lane next turn" piece.
- Pair with [Shover](shover.md): the player can push a loaded Siege
  Engine 90° off its rank, missing the target. Wastes the engine's
  shot and resets the loading cycle.
- Pair with [Mirror Plate](mirror_plate.md) for redirect puzzles.

## Where it's awkward

- The fire-pierces-everything rule has weird edge cases. A loaded
  engine pointed at a row of friendly pieces (other neutrals) will
  kill them all. Probably fine in puzzle mode; possibly broken in
  free-play.
- "Beam stops at unwalkable" needs a precise definition. Does the
  beam stop *at* the Block (and kill the piece on the Block-side
  square) or *before* it? Spec'd as: beam stops at the unwalkable,
  i.e. it does not affect the unwalkable square itself but does
  affect the last walkable square before it.
- Two perpendicular engines firing the same turn: which fires first?
  Order-of-resolution rule needed.

## Engine dependencies

- `Color::Neutral`.
- Signal payload for `dir` + `state`.
- Telegraph resolution phase.
- A "beam fire" primitive that walks a line and removes pieces.
- Existing terrain walkability for stopping the beam.

## New features required

- **`SiegeEngine` piece kind.** `Piece::SiegeEngine { dir, loaded }`.
- **Beam emit primitive.** A function that, given an origin square
  and a direction, iterates the line, removes pieces, stops at
  unwalkable. Reuse-friendly — Mirror Plate hooks into the same
  primitive.
- **Loading state transition.** Two-phase enemy resolution: phase A
  fires all `Loaded` engines, phase B moves all `Idle` engines and
  transitions them to `Loaded`. Splitting the phases prevents
  ambiguity where a freshly-moved engine fires the same turn.
- **Telegraph rendering data.** The dashed-line threat zone needs a
  predicate query the frontend can call: "what squares does this
  Siege Engine threaten next turn?"

## FEN encoding

Two payload keys, both required:

```
(P=SIEGE,C=NEUTRAL,D=N,L=0)      # idle, facing north
(P=SIEGE,C=NEUTRAL,D=E,L=1)      # loaded, facing east
```

`L=0` is Idle (default), `L=1` is Loaded. `D=` reuses the Track
direction vocabulary.

Round-trip is symmetric. An unrecognized `L` value warns and
defaults to `0`, matching existing lenient-parse policy.

## Open questions

- **Beam piercing.** Default is "pierce everything." A `pierce=1`
  variant where the beam stops at the first piece would create a
  totally different puzzle space. Worth a flag? Probably yes —
  encode as `(P=SIEGE,...,PIERCE=1)` with `INF` as the default.
- **Diagonal Siege Engines.** A NE-facing engine firing a diagonal
  beam is geometrically clean but harder to read. Probably a
  separate piece — call it "Lancer."
- **Friendly fire.** Does the beam kill other Siege Engines in its
  path? Spec says yes (any piece on a walkable square). That gives
  the player a "redirect engine A to kill engine B" puzzle. Keep it.
- **Push-to-rotate.** When [Shover](shover.md) pushes a loaded
  engine, does it rotate the engine to match the push direction, or
  does it preserve `dir` and just translate it? Probably preserve —
  rotation is a *separate* tool (TBD).
