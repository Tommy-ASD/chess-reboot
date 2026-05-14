# Mirror-Coil

> Reflects a signal back along the perpendicular axis. Two signals at once, short-circuits to a four-direction blast.

## Inspiration

The signal-reflector cell in Spacechem (the bonder-on-rails that
flips waldo direction), the periscope/mirror combinator in Opus
Magnum, and every "deflector" in a Goldberg cartoon. A piece that
takes a one-direction input and turns it into a one-direction output
on a perpendicular axis — but with a failure mode (overload) that
produces a richer effect.

## Mechanic

A Mirror-Coil is a tile-type with two pieces of state:

- `axis_pair: AxisPair::NS_EW | AxisPair::EW_NS` — FEN-fixed, but
  symmetric, so collapse to a single boolean or just store the
  "primary axis" (the axis along which the mirror is laid). For v1
  use a flag `oriented_ns: bool`.
- `signals_received_this_tick: u8` — resolver-scratch, cleared each
  cascade-step.

Actually for clarity:

- `signals_received_this_step: SmallVec<[Dir; 4]>` — the directions
  signals arrived from in the current cascade-step. Cleared between
  steps.

- **Trigger.** Receives a substrate signal. Mirror-Coil is a
  receiver. The signal must carry direction-of-arrival metadata
  (`arrived_from: Dir`), which is provided by burst-emitters and by
  Domino-adjacency propagation and by any future emitter that has a
  spatial origin. PressurePlate-fired signals do NOT carry direction
  (they originate from the plate's own coord); Mirror-Coil treats a
  direction-less signal as "arrived from the geometrically nearest
  emitter tile," which is computed by the resolver.
- **Effect.** End-of-cascade-step processing:
  - If `signals_received_this_step.len() == 0`: nothing.
  - If `signals_received_this_step.len() == 1`: REFLECT. The signal
    arrived from direction `D`. Emit a signal along the perpendicular
    axis to `D`. Specifically, walk the perpendicular axis (the axis
    that `D` is not on) in BOTH directions from this tile, find the
    first receiver in each direction, fire it. So a single-input
    Mirror-Coil produces two outputs on the perpendicular axis.

    Example: signal arrived from the north (`arrived_from = N`).
    Perpendicular axis is east-west. Walk east, fire first receiver.
    Walk west, fire first receiver. Two output signals.
  - If `signals_received_this_step.len() >= 2`: SHORT-CIRCUIT. Emit
    a four-direction signal burst (same as Hour-Petal's final pluck
    — walks all four cardinals, fires first receiver in each).

State carried in FEN: `oriented_ns` only (and arguably even that is
unused mechanically, since the perpendicular axis is computed from
the arriving signal's direction). Actually the `oriented_ns` flag
affects WHICH axis the mirror reflects to when the input is
direction-less. Keep it.

## Cascade behavior

Resolution priority: Mirror-Coil resolves at the END of each
cascade-step. All signals arriving in the step are gathered first,
then the reflection (or short) is computed and queued for the NEXT
step's substrate firing.

Per-cascade firing: a Mirror-Coil may reflect many times per cascade
— once per cascade-step in which it receives signals. The
`signals_received_this_step` buffer is cleared between steps.

To prevent perpetual loops: a Mirror-Coil only reflects signals that
were emitted in the CURRENT cascade. If a player throws a switch
each turn, that's a new cascade and a fresh reflection. Within one
cascade, suppose Mirror A reflects to Mirror B, B reflects back to
A, A reflects again — termination relies on the fact that each
reflection consumes the inbound signal (substrate signals fire each
receiver once per emission), and there are only finitely many
emitters in the system. The bounded-propagation invariant for
Mirror-Coil is: the resolver tracks total signals emitted in the
cascade; if it exceeds `4 * num_mirror_coils + initial_signals`, the
cascade is forcibly terminated and a warning logged. (Tunable bound;
proof-of-concept guard.)

Signal consumption: each arriving signal is consumed (normal
substrate behavior). The emitted signals are NEW substrate events.

## Why it's interesting

This is the routing primitive. The substrate already has Junctions
(which cycle their direction on signal); Junctions are stateful
choice points. Mirror-Coil is the stateless directional transformer
— same input direction always produces same output direction. Pair
them: Junctions for "did the player set the right state?" puzzles,
Mirror-Coils for "is the geometry right?" puzzles.

The overload-to-burst rule is the failure-mode twist. A Mirror-Coil
that receives one signal is a quiet, predictable router; a
Mirror-Coil that receives two is a flashbang. The puzzle designer
can exploit this for "make sure exactly one signal arrives, or else"
constraints. It's the inverse of Hour-Petal's "make sure exactly N
signals arrive" rule.

## Example chain

Setup: a Mirror-Coil `M` at center. Four receivers on the four
cardinal rays at varying distances: `Gate G_n` north, `Gate G_e`
east, `Gate G_s` south, `Gate G_w` west. Two Switches `S1` (wired
to `M` only) and `S2` (also wired to `M`).

Case A: Player throws `S1` only.

- **Step 1.** `S1` fires. `M` receives a signal from `S1`'s tile.
  `S1` is, say, west of `M`. Direction-of-arrival: W.
- **Step 1 end-of-step:** `M` has one signal, arrived from W.
  Perpendicular axis is N-S. Walk N: find `G_n`, fire it. Walk S:
  find `G_s`, fire it. Queue both for step 2.
- **Step 2.** `G_n` opens; `G_s` opens. No further effects. Cascade
  ends.

Result: two gates open from one switch throw.

Case B: Player throws `S1`, then on the same turn throws `S2`
through some sequence (e.g., via a Hour-Petal final pluck that
fires both, all from the player's single move).

- **Step 1.** Both switches fire (whatever the upstream cascade does).
- **Step 1 end-of-step:** `M` has two signals arrived from two
  different directions (or even the same direction — count is what
  matters). Short-circuit triggers. Queue a four-direction burst for
  step 2.
- **Step 2.** Walk N/E/S/W. Find each gate, fire each. All four
  gates open.

Result: four gates open. The puzzle is "did you want two gates or
four?" — answer depends on whether you arranged for one signal or
two to converge on `M` in the same cascade-step.

## Where it shines

- Routing puzzles. Use a Mirror-Coil to split a single signal into
  two perpendicular outputs, then route those into other Mirror-Coils.
- Convergence-failure puzzles. The player must AVOID double-input on
  a Mirror-Coil — for example, two Hour-Petals on opposite sides
  both want to fire signals at the same Mirror-Coil, but only the
  early one should be allowed to.
- "Pick your output count": one input gives two outputs (axis), two
  inputs give four outputs (burst). A built-in escalation.

## Where it's awkward

- Direction-of-arrival metadata isn't present on all signal types in
  plan 08. Adding it retroactively means changing the substrate's
  cascade message type — a real refactor of plan 08. Worth it for
  Mirror-Coil; also useful for the cleaner Hour-Petal burst semantics.
- The "perpendicular axis from direction-less signal" rule (using
  `oriented_ns`) is hard to teach. Most signals will be direction-ful
  in practice; the direction-less fallback is a corner case players
  will hit by accident.
- Cycles between two Mirror-Coils are bounded by signal-emission
  count, not by physical geometry. The forced-termination warning
  is correct safety but feels janky. Most puzzles avoid this by
  construction.

## Engine dependencies

- Signal substrate (plan 08) — Mirror-Coil is a receiver and a
  perpendicular-axis-walk emitter (similar to Hour-Petal's burst,
  with the two-vs-four output count rule).
- Cascade resolver — end-of-step gathering of multiple signals per
  Mirror-Coil per step.

## New features required

- Direction-of-arrival on substrate signals. Add `arrived_from:
  Option<Dir>` to the cascade message. Plates emit `None`; Hour-Petal
  bursts emit the burst-direction; Domino adjacency emits the relative
  direction; player-thrown Switches emit `None`.
- Resolver step-buffer for Mirror-Coil: per-Mirror-Coil
  `SmallVec<[Dir; 4]>`, cleared between steps. Lives in the resolver,
  not in the persistent `SquareType` state.
- Cascade emission counter for the bounded-propagation guard.

## FEN encoding

```
MIRRORCOIL(orient=NS)
MIRRORCOIL(orient=EW)
```

`orient` is the fallback axis used when reflecting a direction-less
signal — at NS-orientation, direction-less signals reflect along EW,
and vice versa. (Choice of name: `orient=NS` means "mirror surface
lies along the NS axis," so incident signals reflect to EW.)

## Open questions

- Should "two signals from the same direction in one step" count as
  one or two? Spec: count is `signals_received_this_step.len()`, so
  same direction twice counts as two and short-circuits. This
  matches the intent of "any two signals overload."
- Should the perpendicular-axis reflection fire BOTH directions on
  the axis, or only the OPPOSITE direction? Spec above says both
  directions (giving 2 outputs). Alternative: only the opposite
  direction (1 output). The "two outputs" choice makes Mirror-Coil
  more powerful and the short-circuit-to-four rule a smoother
  escalation. Keep two outputs.
- Should Mirror-Coil interact with non-signal events (e.g., a Marble
  rolling through its tile)? Spec for v1: no. Mirror-Coil is a tile
  type, not a piece; pieces can pass through it freely (it's like
  a Standard tile for movement purposes).
