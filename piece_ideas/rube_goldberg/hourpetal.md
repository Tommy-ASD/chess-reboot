# Hour-Petal

> Pluck the petals one by one. The final pluck fires the four-direction burst. Overshoot kills it.

## Inspiration

The flower in he-loves-me-he-loves-me-not: a piece that's about exact
counting, not about what you do with each individual count. Also the
glyph of unmaking in Opus Magnum — finite, consumable, and the
finishing move only happens when the last unit is used.

## Mechanic

An Hour-Petal piece has two pieces of state:

- `petals: u8` — current petal count, decrements monotonically.
- `max_petals: u8` — FEN-fixed. Domain: 1..=6. Used only for display
  and not for mechanics; only `petals` matters at runtime.

- **Trigger.** Receives a `SignalId` (Hour-Petal is a substrate
  receiver, wired by the editor).
- **Effect on receiving a signal:**
  - If `petals > 1`: `petals -= 1`. No external effect. Piece
    remains in place.
  - If `petals == 1`: the FINAL pluck. Emit a four-direction signal
    burst — i.e., walk N, E, S, W from this tile and for each
    direction, find the FIRST `SignalId` receiver in that direction
    along the orthogonal ray (any distance, blocked by walls and
    opaque pieces only). Fire each found receiver's signal. Then
    remove the Hour-Petal from the board (it's spent).
  - If `petals == 0`: this case must not occur (the piece is removed
    when petals would go to zero via the final-pluck path). Defensive:
    if somehow it does, the piece is removed silently. Assert in
    debug.

The four-directional burst is emitted at the cascade-step AFTER the
final pluck, so that receivers wired to the Hour-Petal's substrate
input have already had their effect this step.

## Cascade behavior

Resolution priority: Hour-Petal pluck-handling runs at the same
priority as Mirror-Coil — both consume signals. Specifically, on
receiving a substrate signal, the pluck is applied IMMEDIATELY in
the same step (decrement-only). The four-direction emission, when
triggered, queues for the NEXT cascade-step (so it acts like a
secondary emitter).

Per-cascade firing: Hour-Petal can receive multiple signals per
cascade, plucking once per signal. The final pluck is one-shot
(piece removed). The four-direction burst is also one-shot (because
the piece is gone after).

Termination: each pluck monotonically decrements `petals`. After
at most `max_petals` plucks, the piece is removed. Bounded.

Signal consumption: every signal received is consumed (one fire per
receiver per signal). The four-direction burst is its own emitted
signal, fired as a NEW substrate event.

**Overshoot rule.** A signal received when `petals == 1` causes the
piece to fire its burst and be removed. A signal received AFTER the
piece is gone is a "dangling reference" — the substrate already
tolerates these (plan 08 warns at load time but doesn't crash at
runtime). However, the puzzle-design notion of "overshoot kills it
first" requires the player-facing rule: an extra pluck above the
exact count means the final pluck's burst happens earlier than
intended, potentially missing the target. The piece is gone; the
burst already fired; the puzzle is over.

So "overshoot kills it" reads as: the BURST fires when the count
hits 1, regardless of how many extra signals the player intended to
deliver later. Mistime the count by sending too many signals, and
the burst fires before the rest of your cascade is ready to receive
it.

## Why it's interesting

Counting puzzles are absent from the rest of the substrate. Switches
fire once per throw; PressurePlates fire once per entry; Dominoes
fire once per cascade. There is no piece that is sensitive to "how
many times" something happened. Hour-Petal is the only piece that
remembers a count across the cascade AND across turns. (The pluck
state survives between cascades — Hour-Petal at 3 petals stays at
3 until a signal arrives.)

This also makes Hour-Petal the only natural way to require a
specific number of throws of a Switch over multiple turns. "Pluck
this Hour-Petal exactly four times across four turns to fire its
burst on the fourth turn" is a clock puzzle.

## Example chain

Setup: an Hour-Petal `H` with `max_petals = 3, petals = 3` at the
center of a cross. Four receivers wired to listen for `H`'s burst:
a Gate `G_n` north, a Gate `G_e` east, an Hour-Petal `H2` south
(`max_petals = 1`), and a Mirror-Coil `M` west. All four are
orthogonal-aligned with `H`.

Two Switches `S1, S2` upstream. `S1` is wired only to `H`. `S2` is
wired to `H` and to `H2`.

Turn 1: player throws `S1`. `H` receives a signal. `petals: 3 → 2`.
Cascade ends.

Turn 2: player throws `S1`. `H` plucks again. `petals: 2 → 1`.
Cascade ends.

Turn 3: player throws `S2`.

- **Step 1.** `S2` fires. Signals propagate to both `H` and `H2`.
  - `H` (petals=1): final pluck triggered. Queue the four-direction
    burst for the next cascade-step.
  - `H2` (petals=1): final pluck triggered. Queue ITS four-direction
    burst for the next cascade-step.
- **Step 2.** Both bursts emit.
  - `H`'s burst walks north, east, south, west from H's tile.
    - North: finds `G_n`. Fires it. Gate toggles open.
    - East: finds `G_e`. Fires it. Gate toggles open.
    - South: finds `H2`. But `H2` has just been removed (its final
      pluck also fired this step). Dangling. No effect.
    - West: finds `M`. Fires it. Mirror-Coil receives a signal; it
      will reflect — but the burst is a single emission, not a
      directional one, so Mirror-Coil treats it as a signal arriving
      from the east (since the burst originated to its east). Mirror
      reflects it as a new emission to the north-south axis (see
      `mirror_coil.md` for the rule).
  - `H2`'s burst, similarly, walks four directions from H2's old
    tile. Whatever it finds, it fires.
  - Both Hour-Petals are removed at end of step.
- **Step 3.** Mirror-Coil's reflected signal propagates to whatever
  it's aimed at.
- Cascade ends.

The puzzle: the player had to deliver exactly 3 signals to `H` over
3 turns, and on the 3rd turn also deliver a signal to `H2`. If they
had thrown `S1` four times instead of `S1`-`S1`-`S2`, `H` would
have fired its burst on turn 3 alone (with `H2` still at petals=1
and not yet plucked), and the south-direction find would have hit
`H2` and immediately plucked it. Different outcome.

## Where it shines

- Multi-turn puzzles. The cross-turn state is what distinguishes
  Hour-Petal from the rest of the substrate.
- Burst-router puzzles. The four-direction emission with
  variable-distance reach is the only way to fire four signals from
  a single tile in v1.
- "Mistime and lose" puzzles. The piece is destroyed by overshoot.

## Where it's awkward

- The four-direction burst can hit a Mirror-Coil from a direction
  the player didn't anticipate, leading to surprising reflections.
  Editor UX needs to preview burst paths.
- `max_petals` is decorative — the actual mechanics depend only on
  `petals`. Slight FEN asymmetry; keep `max_petals` for display
  (rendering N petals on the piece sprite) but document that it has
  no runtime effect.
- Cross-turn state means the puzzle isn't reproducible from a single
  FEN unless the FEN captures the current `petals`. It does — but
  this makes "reset the puzzle" require a separate "initial FEN"
  alongside the live state.

## Engine dependencies

- Signal substrate (plan 08) — Hour-Petal is a receiver AND a
  burst-emitter (a new emitter type within the substrate).
- Cascade resolver — needs to schedule the burst for the next step,
  not the same step.

## New features required

- New emitter behavior: directional ray-scan to find first receiver.
  Currently, emitters carry an explicit `Vec<SignalId>` listing
  their targets. Hour-Petal's burst is dynamic — at burst-time it
  walks four rays and fires whatever it finds. This means the
  substrate gains a runtime "find first receiver along a ray"
  helper, used only by burst-emitters.
- `ReceiverKind::Aware` for Mirror-Coil — Mirror-Coil needs to know
  the cardinal direction from which a signal arrived, which is not
  exposed by the current substrate. The burst-emitter has to pass
  direction-of-emission metadata along the signal. Add an optional
  `arrived_from: Option<Dir>` field to the cascade message.

## FEN encoding

```
HOURPETAL(max=3,petals=3)
HOURPETAL(max=6,petals=4)
HOURPETAL(max=1,petals=1)
```

If `petals` is omitted, defaults to `max`. So a fresh Hour-Petal can
be written as just `HOURPETAL(max=3)`.

## Open questions

- Should Hour-Petal be capturable? It's a piece, so by default yes.
  But capturing it during a cascade (e.g., a Marble rolls into it)
  destroys it without firing its burst. The puzzle designer may want
  this protected. Decision: Hour-Petal is capturable; design the
  puzzle to keep it safe. Optional later: `HOURPETAL(max=3,armored=true)`
  for capture-immune variant.
- Direction priority in the burst — N, E, S, W or some other order?
  Use N → E → S → W consistently. Documented in resolver spec; ties
  in tie-break geometry (two receivers found in two directions
  fired simultaneously, but their effects queue per direction).
- What if a burst fires four signals at four Gates and the resolver
  must process them in some order? Each is a separate substrate
  event, processed in the canonical N-E-S-W order. All four resolve
  in the same cascade-step.
