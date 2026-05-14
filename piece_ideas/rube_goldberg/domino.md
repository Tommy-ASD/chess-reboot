# Domino

> One signal in, one square of motion, one signal out the other side — the elementary cascade unit.

## Inspiration

The toppling domino chain is the platonic Rube Goldberg primitive: a
piece that does almost nothing by itself but composes into arbitrarily
long propagating sequences. In Zachtronics terms it's the bond in Opus
Magnum — the smallest reusable connector that turns isolated reactors
into a pipeline.

## Mechanic

A Domino has a `facing: Dir` and a `fired_this_cascade: bool` flag.

- **Trigger.** Receives any `SignalId` it is wired to (Domino is a
  receiver). Also receives the implicit "adjacent Domino fired" signal
  from the adjacency-propagation rule below.
- **Effect.** If `fired_this_cascade` is false:
  1. Set `fired_this_cascade = true`.
  2. Slide one square in the `facing` direction. Standard slide rules
     apply: blocked by walls, blocked by occupied squares, captures
     enemy pieces if the slide-target holds one.
  3. After the slide resolves, look at the four orthogonal neighbors of
     the Domino's NEW position. Every Domino in those neighbors (that
     has not fired this cascade) receives the implicit "adjacent Domino
     fired" signal and queues for firing next cascade-step.
- **State carried in FEN.** `facing` only. `fired_this_cascade` is
  resolver-scratch and never serialized.

The Domino does NOT mint a new `SignalId`. Adjacency propagation is a
direct piece-to-piece message inside the resolver, not a substrate
signal. This keeps the substrate's emitter/receiver topology unchanged.

## Cascade behavior

Resolution priority: Dominoes fire after Switches/PressurePlates have
emitted in the current cascade-step, and before slow pieces (Tally,
Cogwright) tick.

Per-cascade firing: at most once. The `fired_this_cascade` flag is
cleared by the resolver in its post-cascade reset pass (one of three
universal teardown actions — reset Domino flags, reset Mirror-Coil tick
counts, reset Hour-Petal pending-pluck queue).

Signal consumption: the substrate signal that triggered the first
Domino is consumed normally (one fire per receiver per signal). The
adjacency-propagated signal between Dominoes is internal and does not
re-enter the substrate.

## Why it's interesting

It's the cheapest piece in this folder and the most expressive. A line
of Dominoes is a delay line; a fork of Dominoes is a fan-out; a Domino
that slides onto a Pressure-Plate is a one-shot timer for the next
stage. Because each Domino fires at most once, the puzzle designer can
build long chains without worrying about runaway propagation.

Crucially, the slide moves the Domino's POSITION before it propagates.
So a Domino can "walk away from its neighbor and into a new neighbor"
within one cascade — the cascade graph is built dynamically as it runs.

## Example chain

Setup (`>` = Domino facing east, `^` = Domino facing north, `P` =
PressurePlate wired to `S` = Switch's receiver Domino, `G` = Gate wired
to receive from a Switch tied to the far end):

```
. . . . . . .
. . . . . . G   <- Gate (closed)
. . . . . . .
S > > . > > .   <- chain, with a gap at column 4
. . . . . . .
. . . ^ . . .   <- north-facing Domino branching off
. . . . . . .
P . . . . . .   <- PressurePlate (player will trigger by stepping here)
```

Trigger: player moves a piece onto `P`. PressurePlate fires its
`SignalId`, which is wired to `S`.

- **Step 1.** `S` receives the signal. Slides east. New board:

```
. S > > . > > .   (S moved one square east, was at col 0, now at col 1)
```

  Wait — let me re-letter. The original `S` is the first Domino; call
  the chain `A B C D E F` for columns 0..5, with column 3 empty. `A`
  receives the signal.
- **Step 1.** `A` (col 0, east-facing) slides to col 1. After slide,
  `A` is adjacent to `B` (col 2). Both Dominoes. `B` queues.
- **Step 2.** `B` (col 2) slides to col 3 — that's the gap. After
  slide, `B` is at col 3, adjacent to `C` (col 4). `C` queues. ALSO,
  `B`'s new position at col 3 is adjacent to `^` (the north-facing
  Domino one row south, but the resolver only checks orthogonal
  neighbors of `B`'s row). Actually it IS orthogonal — south is one of
  the four. `^` queues too.
- **Step 3.** Two Dominoes fire simultaneously:
  - `C` (east-facing) slides east to col 5 — collides with `E` (or
    whatever's there). Let's say `E` is at col 5 east-facing. `C` is
    blocked by `E`. The slide fails; `C` stays at col 4 but the
    propagation still runs (adjacency check happens regardless of
    whether the slide succeeded — `C` did fire). `E` queues.
  - `^` (north-facing) slides one square north. Its new position is
    now adjacent to the Gate `G` if the geometry lines up. If a Switch
    is on the tile `^` just vacated, no — but suppose `^`'s new
    position triggers a different PressurePlate. That plate fires its
    own substrate signal, which goes to `G`. `G` opens.
- **Step 4.** `E` slides east to col 6, then to col 7 — wait, slides
  are one square. `E` slides one square east, ends at col 6. Adjacent
  to nothing relevant. Cascade quiesces.

Final state: gate `G` is open, the original chain has shifted east by
one tile each, the north-facing Domino has stepped one square up and
incidentally triggered the gate via the plate it stepped onto.

The puzzle: arrange the chain and the plate so that the player's
single move on `P` opens `G` and also positions `B` to block an enemy
piece's escape square. One input, full cascade, multiple effects.

## Where it shines

- Long-distance delayed triggers. A line of Dominoes is a "delay-N"
  signal carrier; the engine can compute the exact step the signal
  arrives.
- Branching cause. A Domino at a T-intersection of three other
  Dominoes fans the signal to all three neighbors.
- Sub-puzzles where the player must orient a Domino correctly before
  triggering — the facing is the puzzle.

## Where it's awkward

- Tile occupancy edge cases: what if a Domino slides into a square
  that holds a Marble? The Marble's "bumped" logic should fire — but
  the Domino is now in the Marble's old square AND the Marble is now
  rolling away. Both pieces moved within the same step. The resolver
  must order this: Domino-slide resolves first, then Marble-bump
  resolves against the new Domino-position.
- Two Dominoes targeting the same empty square. Resolver picks by tile
  index (north-to-south, west-to-east); the loser fails its slide but
  still propagates.

## Engine dependencies

- Signal substrate (plan 08) — Domino is a `SignalId` receiver.
- Cascade resolver (new) — the propagation engine.
- Slide-move infrastructure (existing) — reuse the same capture and
  blocking semantics that pawn/rook moves use.

## New features required

- Receiver-with-side-effect-that-modifies-position. The substrate
  today assumes receivers mutate `SquareType` state in place. Domino
  needs to move the piece occupying the tile, which is a higher-level
  operation than the substrate currently exposes. Add a
  `SignalEffect::SlidePieceOn { dir }` action.
- The "adjacent Domino fired" implicit signal is a resolver-internal
  message type. Introduce `enum CascadeMessage { SubstrateSignal(SignalId),
  AdjacencyFired { from: Coord } }` for the cascade queue.

## FEN encoding

```
DOMINO(facing=N)
DOMINO(facing=E)
DOMINO(facing=S)
DOMINO(facing=W)
```

Pure positional facing only. No firing-state in FEN — that's transient.

## Open questions

- Should the "adjacent Domino fired" propagation fire in all four
  directions, or only the direction the firing Domino is facing? The
  current spec says all four (so a Domino sliding east can trigger
  the Domino it left behind on the west). This is more expressive but
  needs playtest.
- Two Dominoes facing each other, one fires. Does the slide collide?
  Yes — the firing Domino's slide-target is the other Domino's tile,
  which is occupied. Slide fails. The unmoved Domino is now adjacent
  and queues. Next step the second Domino tries to slide back into
  the now-occupied first-Domino tile. Same thing. Cascade terminates
  with both Dominoes having fired. OK.
- Cross-color Dominoes — do they propagate adjacency across colors?
  Yes for now; the Domino is a mechanical primitive, not a chess
  piece. Possibly add `propagates_to_color: Color` if puzzles want
  color-filtered chains.
