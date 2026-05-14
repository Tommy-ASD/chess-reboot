# Engineer

> A King-mover that, instead of moving, paints a `Track` tile on any
> adjacent empty square — builds rails for friendly trains, or carves
> kill-corridors enemies must walk through.

## Inspiration

Plan 09 lands the train substrate: `SquareType::Track`, neutral
`Locomotive` and `Carriage` pieces, deterministic train propagation
along track directionality. Track tiles today are placed at board
setup (FEN-authored) and never change. The Engineer is the
in-game agent that *grows* the rail network during play.

This pairs naturally with the Architect (walls) as a terrain-maker
piece. Architects close space; Engineers open it. Variant
compositions that ship both can mid-game terraform the board.

## Mechanic

Movement set: identical to King.

Special action — **Track-lay.** Once per turn, instead of moving,
the Engineer designates one empty adjacent square and converts
that square's `SquareType` to `Track { direction: D }`. The
direction `D` is part of the move — the Engineer's player chooses,
which means a single Engineer move emits *multiple* `PaintSquare`
options per neighbour (one per direction).

Constraints:
- Target square must be empty of pieces.
- Current `SquareType` must be `Standard`. No painting over walls,
  switches, gates, or existing tracks.
- Direction `D ∈ {N, E, S, W, NE, NW, SE, SW}` (matches plan 09's
  `TrackDir`).
- Move-gen emits one option per legal (neighbour, direction) pair.
  Roughly 8 neighbours × 8 directions = 64 candidate paint moves
  per Engineer turn before filtering — manageable.

Once painted, the new Track tile behaves exactly like a
FEN-authored Track: trains on it propagate as plan 09 prescribes.

## Why it's interesting

The Engineer turns the train system from a static feature into a
dynamic one. Without an Engineer, a Locomotive's reach is fixed by
the FEN-author's terrain choice. With an Engineer, the *player*
decides where the train goes — at the rate of one rail-segment per
Engineer-turn.

Mechanically novel because it bundles "movement" and "construction"
into the same piece-as-actor abstraction. A Bus carries; a Goblin
kidnaps; an Engineer *builds*. The chess novelty is that the
Engineer extends the *threat geometry of another piece* (the
neutral train) at one-tempo cost. You don't gain a new attacker —
you give your existing attacker a new arm.

There's also a denial use case: paint a single Track tile
adjacent to an enemy piece, in a direction that aims into a
friendly Locomotive's path. The next time the Engineer's side
gets a train turn, the train pushes onto that fresh track and
runs over the enemy. The Engineer is doing the *aiming*; the
train is doing the killing.

## Example scenarios

**Train extension.** White has Locomotive on c1 facing N, with
Track tiles c1–c4 already painted by FEN. Black queen on h6. White
Engineer on d4, white turn. Engineer paints c5 = `Track(N)`. Next
train phase, Locomotive walks c1→c5. Two more Engineer turns extend
to c7. Engineer turns 4–6 extend laterally toward h6. Six tempi
later the train is on h6 and the queen is gone (or has had to flee
six squares, conceding the position).

**Kill-corridor.** Mid-board chase: black king fleeing from c5
toward h8. White Engineer on d6 paints e6 = `Track(E)`, f6 =
`Track(E)`, g6 = `Track(E)` across three turns. White already has
a Carriage on a6 (via a Bus drop or earlier Locomotive run). The
fresh track lets the Locomotive on b6 push the Carriage onto the
king's escape rank. The Engineer never threatens the king
directly; it lays the rails the train rides on.

**Defensive rail.** Black's Locomotive aimed at white's king is
two squares from connection. White Engineer paints a *junction or
sideways track* on the connecting tile. Plan 09's train rules
divert the train; if it derails into a non-walkable square, it
stops. Defensive track-laying is sometimes the cleanest answer
to a hostile train.

## Where it shines

- Compositions with `Locomotive`. Without trains, the Engineer is
  near-useless. With trains, it's the player's hands on the wheel.
- Open boards with sparse FEN-authored track. The Engineer's value
  scales with how much board is *paintable*.
- Long games. Track-laying compounds; a board fully rail-veined by
  move 60 plays nothing like a board at move 1.

## Where it's awkward

- Short tactical positions. Track-laying is a multi-turn investment
  that doesn't pay off if the game ends in 15 moves.
- Variants without `Locomotive` or `Carriage`. The Engineer becomes
  a flavorless King-mover.
- Track direction choice is a 1-of-8 decision per paint, which
  bloats move-gen output. A casual player may find the option
  count overwhelming.
- Architect/Engineer compositions can produce dead-position
  fortresses if both sides over-build.

## Engine dependencies

- `SquareType::Track { direction: TrackDir }` from plan 09.
- The train propagation system from plan 09.
- `PaintSquare` move type (shared with Architect — see
  [architect.md](architect.md)).
- King-movement primitive.

## New features required

- `MoveType::PaintSquare { coord, new_type }` (shared with
  Architect). The `new_type` payload is the full `SquareType`
  including the `TrackDir` selection, so the same move type covers
  both wall-paint (`Block`) and rail-paint (`Track { direction }`).
- Move-gen entry that emits one paint candidate per (neighbour ×
  direction) pair, plus King moves. Filter out blocked / occupied
  / non-`Standard` neighbours.
- Test: round-trip rail-paint through make/undo. Confirm
  `TrackDir` is restored on undo.
- Test: rail-paint followed by train phase. Confirm a freshly
  painted track is walked by a train on the *next* train phase
  (not the same turn).

## FEN encoding

Piece symbol: `E` (Engineer). Single-letter slot likely free; if
contested with an existing piece, fall back to `En`.

```
(P=E)            # white Engineer
(P=e)            # black Engineer
```

No piece-level state. The painted-track FEN is just whatever plan
09 already specifies for `(T=TRACK,D=...)` — no Engineer-specific
syntax needed.

## Open questions

- **Direction-choice UX.** The frontend needs a way to express
  "paint track here, direction = SW" in a single user action. Two
  natural UIs: (a) click neighbour, then a directional popup; (b)
  click-and-drag from the Engineer onto the neighbour, with the
  drag vector picking the direction. Implementation choice, not
  engine.
- **Painting a Track in a direction that immediately points at a
  non-walkable square.** Legal? Useless but legal — the painted
  Track exists; if a train ever reaches it, it stalls. Recommend
  allow + warn in the UI, mirroring plan 09's tolerance of dead-end
  rails.
- **Should Engineers be able to repaint an existing Track to a new
  direction?** Tempting, but the "Standard-only" constraint avoids
  fights with mid-train moves. If a future plan wants
  re-orientation it adds a separate move type (`MoveType::RotateTrack`).
- **Can the Engineer paint *under* a friendly train?** No — the
  Track square is occupied. The Engineer would have to wait until
  the train moves off. This is the right answer mechanically but
  occasionally frustrating. Document, don't fix.
- **Interaction with `Switch`/`Junction` tiles.** Currently
  disallowed by the "Standard-only" rule. If users want
  Engineer-builds-junction, a future plan extends the allowed
  `new_type` set. Out of scope for v1.
