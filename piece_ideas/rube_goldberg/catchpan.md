# Catch-Pan

> The first thing in stays. The rest pile up. Tips and dumps once the load is right.

## Inspiration

The kitchen scale on a string in a Rube Goldberg cartoon — empty
until the right thing falls into it, frozen until the right weight
arrives, then it pivots and dumps. Sorting and timing puzzles need a
piece that says "no, not yet, I'm waiting for the heavy one."

## Mechanic

A Catch-Pan is a tile (not a piece — closer to a `SquareType`) with
three fields:

- `held: Option<PieceData>` — the first piece to enter during the
  current cascade, or `None`.
- `weight: u8` — the current accumulated weight in the pan,
  including the held piece.
- `threshold: u8` — FEN-fixed, value 1, 2, or 3. The pan tips when
  `weight >= threshold`.
- `tip_dir: Dir` — FEN-fixed, the direction the held piece is ejected
  on tip.

Each piece type carries an integer "weight." For v1, simple rule: all
pieces weigh 1 except `Locomotive` (weighs 2). Customizable later.

- **Trigger.** A piece enters the Catch-Pan's tile during a cascade
  (by slide, by roll, by signal-driven motion — any cascade-step
  movement that ends on this tile).
- **Effect.**
  - If `held` is `None`: the entering piece is placed in `held`,
    `weight += piece.weight`. The entering piece is REMOVED from the
    cascade — it no longer occupies the tile in the visible sense
    (graphically it sits in the pan).
  - If `held` is `Some`: the entering piece is also added to the pan
    — `weight += piece.weight`. The new piece is also removed from
    play, stacked atop the held piece. Only `held` is named (the
    first arrival); the others are anonymous ballast.
  - After the weight update: if `weight >= threshold`, the pan tips.
    The held piece (and only the held piece) is ejected one tile in
    `tip_dir`. The ballast pieces are discarded (captured). The
    held-slot returns to `None`, weight returns to 0, threshold
    unchanged.
- **Refusal.** A "refuse all subsequent entries" rule was in the
  original spec — that's redundant given the held-slot stacking
  rule. Re-spec: the pan accepts entries until it tips. Then it's
  empty again. Re-fillable across cascades.

## Cascade behavior

Resolution priority: Catch-Pan tip-check runs at the END of each
cascade-step, after all motion has resolved. This ensures pieces
arriving in the same step are weighed together, not sequentially.

Per-cascade firing: a single Catch-Pan may tip at most once per
cascade. Once it tips and ejects, its slot is empty for the
remainder of the cascade — any piece arriving later that step or in
a subsequent step starts a NEW held-piece round but cannot trigger a
second tip in the same cascade. (Enforced by a
`tipped_this_cascade: bool` resolver-scratch flag.)

Signal consumption: none. Catch-Pan does not interact with `SignalId`
at all. It is a pure mechanical piece. (Future extension: pan emits
a `SignalId` when it tips. Out of scope v1.)

## Why it's interesting

It's the only piece in this folder that is order-sensitive within a
cascade-step. If two pieces arrive at the pan in the same step, the
designer chooses which becomes "held" via the resolver's tie-break
rule (north-then-west priority). This makes the geometry of arrival
matter for the puzzle's semantics, not just for whether things
capture.

It also enables a class of "sort by speed" puzzles: pieces that
arrive earlier (because their cascade chains are shorter) become
held; pieces that arrive later become ballast. The held piece
survives (and is re-ejected); the ballast is consumed.

## Example chain

Setup: three Dominoes converging on a Catch-Pan from north, west,
and south. The Catch-Pan has `threshold=2, tip_dir=E`. The northern
chain is 1 Domino long; the western chain is 3 Dominoes long; the
southern chain is 2 Dominoes long. All three are triggered
simultaneously by a single PressurePlate firing a signal wired to
all three chain-heads.

```
. . . . . .
. . . v . .       <- Domino facing south (chain head, north arm)
. . . . . .
. . . . . .
> > > [P]   .     <- western chain: 3 Dominoes facing east → [Catch-Pan] → ejects east
. . . . . .
. . . ^ . .       <- southern chain head facing north (1 of 2)
. . . ^ . .       <- southern chain 2nd Domino
```

Cascade:

- **Step 0.** Plate fires. All three chain heads receive their
  signal.
- **Step 1.** Three Dominoes slide:
  - North chain Domino slides south. Now adjacent to pan's north
    edge.
  - Western chain head-Domino slides east, propagates to next Domino
    in chain.
  - Southern chain head-Domino slides north, propagates to next.

  (Geometry note: the figures above are approximate; pretend the
  distances work out as stated below.)

- **Step 2.** The northern Domino, having slid south, is one square
  north of the pan. The pan is east of the western chain's leading
  edge. The southern chain is one Domino north of its start.
  - The northern Domino, this step, slides further south — into the
    pan. ARRIVAL #1.
  - The western chain's second Domino slides east.
  - The southern chain's second Domino slides north.

  End of step 2: the pan holds the northern Domino. `held = NorthDomino,
  weight = 1, threshold = 2`. Not yet tipped.

- **Step 3.** Western chain's third (last) Domino slides east into
  the pan. ARRIVAL #2. Southern chain's second Domino slides north,
  now adjacent to pan's south edge.

  Pan: `held` still NorthDomino (first in stays), weight now 2.
  Threshold reached. Pan tips. NorthDomino is ejected one tile east.
  Western Domino is discarded (ballast). Pan is empty.

- **Step 4.** Southern chain Domino slides north into the now-empty
  pan. ARRIVAL #3. Pan is empty so this Domino becomes `held =
  SouthDomino, weight = 1`. Threshold not reached. Pan stays.

  No further motion queued.

- **Cascade end.** Final state: NorthDomino at the pan's east neighbor,
  pan holding SouthDomino (weight=1, not tipped, will not tip again
  this cascade), WestDomino captured.

The puzzle: the player wanted the NorthDomino delivered to the east
square. The western chain's purpose was solely to provide ballast to
tip the pan. The southern chain was a red herring — it arrived too
late to do anything but fill the pan with a leftover.

## Where it shines

- Sorting-by-arrival puzzles. Three pieces all want to use the pan's
  ejection; only the first one in survives, only if the second-arrival
  adds the right amount of weight.
- "Inject a specific piece eastward" — the pan is the only piece in
  this folder that can re-direct a piece into a tile the cascade didn't
  originally reach.
- Multi-stage cascades where the pan's tip is itself the trigger for
  the next stage (the ejected piece lands on another Domino or
  PressurePlate).

## Where it's awkward

- The "ballast is anonymous" rule discards captured Dominoes etc.
  silently. Players will be confused when a Domino vanishes into a
  pan and "doesn't come out." UI must show stacked weight.
- The tie-break rule (north-then-west priority for simultaneous
  arrivals) is invisible from the FEN. Two puzzles that differ only
  by their tile-priority order can have different outcomes. Document
  prominently or render explicit "arrival order" indicators.
- Threshold of 1 is degenerate: the first piece in tips immediately.
  Valid but unsatisfying — it's just a "shove east" tile. Keep for
  completeness.

## Engine dependencies

- Piece weights — extend `PieceType` with `fn weight(&self) -> u8`,
  default 1.
- Cascade resolver — end-of-step hook for pan-tip evaluation.
- Capture pipeline — ballast pieces are captured (go to the capture
  pool, count toward material balance if the variant tracks it).

## New features required

- `SquareType::CatchPan { held, threshold, tip_dir, tipped_this_cascade }`.
  The `held` payload needs to serialize a full piece (color + type +
  any payload state). FEN already handles parenthesized payloads —
  extend to nested form `CATCHPAN(thresh=2,tip=E,held=DOMINO(facing=N))`.
- End-of-cascade-step hook in the resolver. Both this piece and
  Cogwright use it; share the hook infrastructure.

## FEN encoding

```
CATCHPAN(thresh=1,tip=E)                                   # empty pan
CATCHPAN(thresh=2,tip=N,held=DOMINO(facing=W))             # holding a Domino
CATCHPAN(thresh=3,tip=S,held=PAWN-white)                   # holding a white pawn
```

`held` is optional; if absent, the pan is empty. `weight` is
recomputed from `held` at load time (one held piece = its weight,
since ballast is captured immediately so no mid-cascade saves are
possible — Catch-Pan state only saves AT REST, never mid-cascade).

## Open questions

- Does the pan accept a Marble (Color::Neutral)? Yes — Marbles are
  pieces, they have weight (1), they enter tiles. A Marble entering
  a pan is held; if the pan tips, the Marble is ejected and resumes
  rolling. (This is a great puzzle hook — pan as a "Marble redirector.")
- What weight should the Goblin, Skibidi, Bus, Monkey have? Spec for
  v1: all weigh 1 except Locomotive (2). Catch-Pans with threshold=3
  require three normal pieces or a Locomotive + a normal piece.
  Bus weighing 2 might also make sense. Leave for tuning.
- Multiple pans on the board — independent. Each has its own held
  slot and tip flag. A piece can enter a second pan after being
  ejected from the first.
