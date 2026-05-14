# Rube Goldberg — cascade-machine pieces

The board is a machine. The player places a piece (or makes one move),
and the rest of the turn is the engine resolving a deterministic cascade
of triggers, slides, ejections, signals, and pressure changes. No
opponent move — just cause and effect that the puzzle designer wired up.

These pieces are the spiritual descendants of plan 08's signal
substrate. Switches, Junctions, Gates, and PressurePlates already
implement one-shot emit-and-respond. The Rube Goldberg pieces are the
mechanical primitives that turn that single-step substrate into a
multi-step cascade: dominoes that re-emit, marbles that travel,
seesaws that re-aim, counters that gate, mirrors that route.

## Design philosophy

- **The piece IS the mechanism.** No piece in this category captures
  by being moved by a player; capture happens because the cascade
  pushed it into something. The player's job is to choose the input;
  the board's job is to compute the output.
- **One input, one resolution.** A cascade starts from a single trigger
  (a player move, a signal pulse, a piece entering a tile) and runs to
  completion before control returns to the turn system.
- **Cause is local; effect propagates.** Every piece in this category
  reacts to immediately adjacent state or to signals from named
  emitters. Nothing reads the whole board.
- **Determinism above all.** Resolution order is fixed. Same input
  produces same output, every time.

## How these extend the signal substrate

Plan 08 introduced four building blocks:

- `Switch` — emitter, fires on player action.
- `Junction` — receiver, cycles its outgoing track direction.
- `Gate` — receiver, toggles open/closed.
- `PressurePlate` — emitter, fires when a piece enters its tile.

That substrate is intentionally one-shot. Plan 08 reads:
> "Receivers cannot themselves emit during the same call. This forbids
>  cascades (and the cycles they enable) in v1."

These pieces are the v2: they re-emit, they move, they reshape the
board. To keep determinism, the engine grows a **cascade resolver**
that runs after every signal-producing event, processes pieces in a
fixed priority order, and terminates when a step produces no further
state change.

## The bounded-propagation invariant

Every piece in this folder satisfies one of three termination rules:

1. **Single-fire per cascade.** The piece keeps an internal
   `fired_this_cascade: bool` flag, set on first activation, cleared
   when the cascade ends. (Domino, Mirror-Coil, Hour-Petal-on-last-pluck.)
2. **Monotone state change.** Each activation strictly decrements a
   counter or consumes a finite resource. (Hour-Petal's petal count,
   Catch-Pan's "held first piece" slot, Tally-Pillar's once-per-cascade
   trigger.)
3. **Spatial bounded motion.** The piece moves a bounded number of
   squares per activation and stops at the first obstacle. (Marble,
   Domino, Fulcrum's ejection.)

These three rules compose: a chain of Dominoes terminates because each
fires at most once; a Marble terminates because the board is finite and
it halts at the first piece it touches.

## The cascade-step concept

A "cascade-step" is one atomic unit of the resolver. Within a step,
every piece that is currently eligible to fire fires simultaneously and
their outputs are queued. Between steps, the queue is drained into the
board state. The cascade ends when a step queues no outputs.

This is the natural extension of plan 08's emit-then-update pattern —
just iterated until quiescent, with the safety rules above to prove
quiescence is reachable.

## Index

| File | Piece | Role in the machine |
| --- | --- | --- |
| `cogwright.md` | Cogwright | Adjacency-counted rotator, fires inert pulse stones |
| `domino.md` | Domino | Signal-to-slide-to-signal propagator |
| `catchpan.md` | Catch-Pan | First-come holder; tips at weight threshold |
| `marble.md` | Marble | Bumped-then-rolling neutral projectile |
| `hourpetal.md` | Hour-Petal | Exact-count consumer; finale signal burst |
| `fulcrum.md` | Fulcrum | 1x3 seesaw, ejects opposite tip on weight |
| `mirror_coil.md` | Mirror-Coil | Single-reflection router; two-in-one short |
| `tally_pillar.md` | Tally-Pillar | File-census trigger; freezes on match |

## Reading order

`domino.md` and `marble.md` first — they define the basic
"trigger-to-motion" primitive everything else assumes. Then `catchpan.md`
and `fulcrum.md` for the weight/timing primitives. Then `mirror_coil.md`,
`hourpetal.md`, and `tally_pillar.md` for signal-routing and
gating. `cogwright.md` last — it composes everything else into
phase-counter puzzles.
