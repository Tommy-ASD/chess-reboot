# Marble

> The neutral billiard ball. Bumped once, rolls until it hits something, captures that something.

## Inspiration

The marble track in The Incredible Machine, the steel ball in Mouse
Trap, the ramp segment in Opus Magnum. A Marble is the canonical
"motion-only" piece: it stores no signal state, has no allegiance, and
exists purely to be set in motion by another piece.

## Mechanic

A Marble is a `Color::Neutral` piece with two pieces of state:

- `facing: Dir` — the direction it will roll when bumped.
- `rolling: bool` — whether it is currently in motion.

In FEN at rest: `MARBLE(facing=E)`. While rolling it isn't placed on
the board (it's transient inside the cascade resolver), so the
`rolling=true` state never serializes.

- **Trigger.** Any piece moves into a tile orthogonally adjacent to
  the Marble in such a way that the moving piece's motion vector
  points AT the Marble. (e.g., a Rook moving east, ending on the
  Marble's western neighbor, bumps the Marble's facing east.)
  Equivalently: a piece moves into the Marble's tile from the side
  opposite the Marble's `facing`. The Marble's `facing` is its
  ejection direction; the bumping piece is required to come from
  behind.
- **Effect.** The Marble begins rolling. In subsequent cascade-steps,
  it travels one tile per step in `facing` until:
  - it hits a wall — stops in the last legal tile.
  - it hits an occupied tile holding a piece — captures that piece
    (replaces it on the tile), then stops.
  - it hits a `Track` terrain tile — stops on the Track tile (the
    Track absorbs it; future plan: Tracks redirect Marbles by their
    direction-pair).

State carried in FEN: `facing` only.

## Cascade behavior

Resolution priority: Marbles roll AFTER all Dominoes have slid in the
current cascade-step. A Domino sliding into a Marble's "behind" tile
will bump it, and the Marble's first roll-step is the next
cascade-step, not the same one.

Per-cascade firing: a Marble that is `rolling` keeps rolling each
cascade-step until it stops. Termination is guaranteed by the finite
board — a Marble travels at most `max(W, H)` tiles, then is forced to
stop by a wall.

Re-bumping a rolling Marble: not allowed. Once `rolling=true`, only
the resolver moves the Marble; player and other piece motion cannot
re-aim it within the same cascade. A Marble that has stopped is
bumpable again from a future move.

## Why it's interesting

The Marble is the only piece in this folder that is genuinely
Color::Neutral and captures pieces of both colors equally. This makes
it the "physics substrate" — a Marble sitting on the board is a
pre-staged threat that the puzzle designer has armed, and the player's
job is to bump exactly the right Marble in exactly the right
direction.

It's also a soft cascade-terminator. A Marble rolling into an enemy
King ends the puzzle. A Marble rolling into a Domino captures the
Domino, ending one branch of the chain. The Marble is the cascade's
"executioner" — most cascades end with a Marble hitting something.

## Example chain

Setup (`o` = Marble facing east, `k` = enemy king, `P` = Pressure
Plate, `R` = friendly Rook):

```
. . . . . . . .
. . . . . . . .
. . . . . . . k   <- enemy king
. . . . . . . .
. R . . . o . .   <- Rook on col 1, Marble on col 5 facing east
. . . . . . . .
. . . . . . . .
. . . . . . . .
```

Trigger: player plays `R c5` — Rook slides east from col 1 to col 4,
ending one square west of the Marble. The Rook's motion vector is
east, ending adjacent to Marble's western side — the bump is valid.

- **Cascade-step 1.** Rook's move resolves. Marble's `rolling` is set
  to true. Marble does not move this step; it's queued to roll.
- **Step 2.** Marble rolls east to col 6. No obstacle. Continues.
- **Step 3.** Marble rolls east to col 7. Tile is occupied by the
  enemy king. Marble captures the king. Marble stops.

Result: in a single player move, the cascade resolved a check-and-mate
without the Rook touching the king. The Rook is the trigger; the
Marble is the agent.

A harder variant: put a Domino at col 6 facing south, wire it to a
Switch that the player threw last turn. The Marble's path is now:

- **Step 2.** Marble at col 5 attempts to roll east to col 6 — but col
  6 holds a Domino (which has not fired this cascade and is at rest).
  The Marble captures the Domino. Marble stops at col 6.
- King survives.

The puzzle: the player must choose whether to throw the Switch on the
previous turn (clearing the Domino out of col 6 via its own slide) or
this turn (keeping the Rook free to make the bump-move).

## Where it shines

- Long-range capture by indirection — the player attacks a square
  they can't reach directly.
- Puzzles where multiple Marbles are pre-staged and the player must
  pick which one to fire.
- Combination with Track tiles (existing terrain) — a Marble that
  rolls onto a Track is stopped, but future Track-redirect rules
  could turn the Marble track into a routing problem.

## Where it's awkward

- "Bump from behind" rule is geometrically subtle. A Knight that
  jumps to a Marble's western neighbor doesn't have an east-pointing
  motion vector — does it count? Spec: no, Knight motion is not
  bumping; only sliding pieces and one-tile-step pieces (Pawn, King)
  whose final-step direction matches the Marble's facing-inverse
  count.
- Marbles in flight aren't on the board. If a player tries to read
  board state mid-cascade (debugger), the Marble is "in the
  resolver's hand." Need to render it specially in UI, or just hide
  it until the cascade settles.

## Engine dependencies

- `Color::Neutral` (existing) — the Marble's allegiance.
- Terrain (`Track`, existing in plan 09) — Marble interacts with
  Tracks at rest as an absorber.
- Cascade resolver (new) — Marble is the canonical "multi-step
  motion" example for the resolver design.

## New features required

- Direction-of-arrival tracking for moves. The current move type
  records `from` and `to` but not the unit vector. Sliders' vectors
  are derivable from `from`/`to`; one-tile movers' vectors equal
  `to - from`; Knight is None. Add `motion_vector(&GameMove) -> Option<Dir>`.
- In-flight piece state for the resolver. The cascade resolver needs
  a `Vec<RollingMarble { coord, facing }>` scratch buffer that lives
  for the duration of the cascade and drains as Marbles stop.

## FEN encoding

```
MARBLE(facing=N)
MARBLE(facing=E)
MARBLE(facing=S)
MARBLE(facing=W)
```

Always `Color::Neutral` — the FEN side-letter is unused for this
piece. Decision: encode as a lowercase tag like other Neutral pieces.

## Open questions

- Can a Marble bump another Marble? The first Marble rolls into the
  second's tile. If the second Marble's facing is the same as the
  first's direction of travel, the second is "bumped from behind" by
  the rolling first. Spec: yes, but the first Marble stops AT the
  second's old tile (capture-on-contact) and the second begins
  rolling next step. This creates Marble-trains.
- A Marble that captures a Marble — the first captures and stops;
  the second is gone. Loses the train property but matches the
  general "capture on contact" rule. Choose: cohabitation (train) is
  the cleaner rule, capture is the consistent rule. Probably
  cohabitation for Marble-on-Marble specifically.
- A Marble rolling onto a PressurePlate — does the plate fire? The
  plate's `fires_for: PressureTrigger` accepts `AnyPiece`. A Marble
  is a piece. So yes. Now the resolver has a Marble that just
  triggered a new substrate signal mid-cascade — that's the whole
  point. This is one of the most powerful interaction points.
