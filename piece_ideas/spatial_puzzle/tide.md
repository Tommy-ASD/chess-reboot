# Tide

> Every fourth full move, the board breathes. Dark-square pieces shift toward
> the nearest Tide; next cycle, light-square pieces shift away.

## Inspiration

The geometric primitive is **periodic global rearrangement**. Into the
Breach's Vek emergence cycle; the Bullet Hell genre's wave patterns;
celestial mechanics — tides, eclipses, predictable cosmic events that
all players must plan around. The Tide is the longest-period rule in
this set: it acts every fourth move, but the consequence is total.

It is also the only piece in this set with **inter-turn state** —
specifically a turn-counter that's part of board state, not a piece
property. (Or it can be encoded as a property on each Tide; see [FEN
encoding](#fen-encoding).)

## Mechanic

The board has a **tide counter** `T`, an integer modulo 4, that
increments by 1 after every full move (i.e., after black moves). The
counter starts at 0 on the initial position.

**Pulse events:**

- When `T` transitions to 0 (after a full move that made it equal 0,
  i.e. the cycle just completed): **light-square pulse**.
- When `T` transitions to 2: **dark-square pulse**.

(Phases 1 and 3 are quiet — players move freely.)

**On a dark-square pulse:**

For every piece currently on a **dark square** (excluding Tides
themselves):

1. Find the **nearest Tide** by Chebyshev distance. If multiple Tides
   are tied, pick the **lowest-index** Tide (sorted by `(rank, file)`
   ascending) — deterministic.
2. Compute the **one-step direction** from the piece toward the chosen
   Tide. The direction is the unit-vector of `(Tide.rank - piece.rank,
   Tide.file - piece.file)` rounded to one of the 8 compass directions.
3. **Move the piece one square in that direction**, simultaneously with
   all other pulsed pieces.

**On a light-square pulse:**

Identical, but every piece on a **light square** moves one square
**away** from the nearest Tide. Direction is negated.

**Pulse collisions:**

Pulses move many pieces simultaneously. Collisions:

- Two pieces resolving to the same destination square: the **lower-
  indexed piece** (by `(rank, file)` at start of pulse) arrives; the
  other is **forfeited**, i.e., stays in place.
- Piece resolving to a square that **starts the pulse occupied** by a
  piece not also pulsing in the same direction: forfeited.
- Piece resolving to an **unwalkable square** (Block, Turret, closed
  Gate): forfeited.
- Piece resolving **off the board**: forfeited.

"Forfeited" means: the piece does not move in this pulse. Stays put.

**Captures via pulse:** None. The Tide does not directly capture
anything. (Pieces forfeited because they were blocked stay put, no
capture.)

**Promotion via pulse:** A pawn pushed onto its back rank by a pulse
promotes. (Reuses the existing promotion trigger.)

**The Tide itself.** The Tide piece:

- Is **immobile**.
- Is **capturable** by any piece attacking its square.
- Is **the centre of attraction** for its colour-phase pulse.
- Sits on either a light or dark square; pulses ignore the Tide's own
  square colour. (The Tide attracts both colour phases, just on
  different cycles.)

**Multiple Tides.** Each Tide attracts the **nearest** piece on its
pulsed colour. If Tide A is closer to piece X than Tide B, X moves
toward A. Multiple Tides partition the board into Voronoi-like cells
of attraction.

If a piece is exactly equidistant from two Tides, the lower-indexed
Tide wins (see step 1).

## Why it's interesting

It introduces **long-period predictability**. Every fourth move, the
board rearranges *en masse*. Players must plan multi-move sequences
that anticipate the pulse — sometimes a planned attack is foiled
because the pulse displaces the attacker. Equally, a player can set
up a pulse-driven discovery: place a Tide such that the next pulse
shifts the opposing king onto an attacked square.

The colour-phase alternation is **antisymmetric**: dark squares
contract toward the Tide, light squares expand away. Over a full 8-move
cycle, pieces oscillate. Static pieces on the boundary may not move at
all.

It's the only piece in this set that creates **timing-based puzzles**:
"mate in 3 if you wait for the pulse, mate in 5 if you don't."

## Example scenarios

**Pulse moves king into check:**

```
. . . . . . . .
. . . . k . . .       k = black king on e7 (dark square: e7 is dark)
. . . . . . . .
. . . . . . . .
. . . T . . . .       T = Tide on d4 (dark)
. . . . . . . .
. . . . . . . .
. . . . . . . .
```

Tide counter at 1 (next pulse is on dark squares — i.e. at counter 2).
After white moves and black moves, counter becomes 2. **Dark-square
pulse.** Black king at e7 is on a dark square. Nearest Tide is the only
Tide, at d4. Direction: from e7 toward d4 = `(-1, -1)` (SW). King moves
to d6.

If white had a queen attacking d6, the king pulse pushes into check.
Black loses on the spot. This is the **king-displaced-into-mate**
tactic.

**Voronoi partition by two Tides:**

```
. T . . . . . T       T = Tides on b8 and h8
. . . . . . . .
. . . . . . . .
. . p . . . . p       Black pawns on c5 (closer to b8) and h5 (closer to h8)
. . . . . . . .
. . . . . . . .
. . . . . . . .
. . . . . . . .
```

Pawn at c5 attracted to b8 (Chebyshev distance 3 vs. 5 to h8). Pawn at
h5 attracted to h8 (distance 3 vs. 6 to b8). Dark-square pulse moves
c5→b6 (toward b8) and h5→g6 (wait — h5 toward h8 is north, so h5→h6).
Actually h5 is a light square (h+5 = even? File h is 8, rank 5, sum
13, odd — light). So h5 doesn't pulse this cycle. Only c5 (file 3,
rank 5, sum 8, even = dark) pulses. c5→b6 (toward b8: SW step is
wrong, b8 is NW of c5, so `(+3, -1)` rounds to N or NW). Let's say
NW. c5→b6 actually moves N-W one square = b6. Yes.

**Forfeit example:**

Tide at e5. Two black pawns at d6 (dark) and e6 (light). Dark pulse:
d6 attracted toward e5. Direction `(+1, +1)` rounds to NE? No, e5 is
SE of d6 — direction `(-1, +1)` (SE). d6→e5? But e5 is the Tide's
square. Forfeit (Tide blocks). d6 stays.

(Tide attracts toward itself but is itself a blocker.)

## Where it shines

- **Long-term planning.** The pulse cycle is publicly known; both
  players track it. Plans that span multiple pulses are strategically
  rich.
- **King safety.** Tides shift pieces — including kings — in
  predictable directions. Composers can craft "the king is forced
  into check by the pulse" sequences.
- **Pawn coordination.** Pawns on dark squares contract toward Tides;
  pawns on light squares expand. Over 8 moves, pawn structures
  reshape without any player input.

## Where it's awkward

- **Determinism of simultaneous moves.** Two pieces want the same
  destination — needs a stable tiebreak. Using piece-coordinate order
  is fine. Document it.
- **Promotions during pulse.** A pawn pushed onto rank 8 by a pulse
  promotes. The pulse's direction was determined by the Tide; the
  promotion is incidental. Player picks promotion piece? In a
  *non-interactive* pulse, default to queen, or use a per-pawn
  "preferred promotion" stored in FEN.
- **Tide attractors of own pieces.** The Tide attracts its colour's
  pieces too — they shift toward it. This is symmetric but might
  feel weird if a player loses material to their own Tide's pulse.
  Variant: Tide only attracts enemy pieces. Future option.
- **Counter persistence across reset.** The counter is global board
  state. FEN must include it.

## Engine dependencies

- **End-of-turn hook** — fires after black's move. Increments
  counter, dispatches pulses.
- **Variable boards** — Voronoi partitions depend on board geometry,
  which is already configurable.
- **Square colour function** — `is_dark_square((r, f)) = (r + f) %
  2 == 0` (assuming a1 is dark). Already exists in any chess
  engine.

## New features required

- **Tide counter.** New global state field on `Board`: `tide_counter:
  u8` (values 0..=3). FEN-serialized.
- **Tide piece.** Has no special payload. Pulses respond to its
  presence at pulse time.
- **Pulse resolver.** A new turn-phase function:
  1. Increment counter; mod 4.
  2. If new value is 0: light-square pulse. If 2: dark pulse. Else
     skip.
  3. Build list of all pulsed pieces (colour-phase filter).
  4. For each, compute target (nearest Tide, direction toward/away).
  5. Sort pulsed pieces by `(rank, file)` for deterministic conflict
     resolution.
  6. Apply moves in order; pieces whose destination is now blocked
     forfeit.
- **Voronoi nearest-tide.** Iterate all Tides, compute Chebyshev
  distance, pick min (tiebreak: lowest-index Tide).
- **Pulse-promotion trigger.** Reuse normal promotion logic at
  pulse-apply time.

## FEN encoding

The Tide piece itself:

```
(P=TD,C=W)              White Tide
(P=TD,C=N)              Neutral Tide (works for both sides equally)
```

Probably neutral is the right default — Tides are environmental.

**Board-level state**: tide counter is encoded as a new top-level FEN
field, after the existing castling/en-passant/halfmove fields:

```
... TC=2 ...
```

`TC=` field, value 0..=3. Default 0 if omitted.

## Open questions

- **8-direction rounding.** The "direction from piece to nearest
  Tide" rounds to one of 8 compass directions. Rounding algorithm:
  for delta `(dr, df)`, normalize each component to `-1, 0, +1` by
  `sign(dr)`, `sign(df)`. Zero deltas mean "no motion in this axis."
  A piece directly N of a Tide has `dr < 0, df = 0` → moves S
  (component `(-sign(dr), 0)` = `(+1, 0)`)... wait that's away. Let
  me redo: from piece to Tide = `(Tide.r - piece.r, Tide.f -
  piece.f)`. Sign of each = direction toward Tide. Move = `(sign(dr),
  sign(df))`. Good.
- **Direction toward Tide for *expanding* (light) pulse.** Negate the
  direction. Move = `(-sign(dr), -sign(df))`. If the piece is on the
  Tide's own square (dr=df=0), light pulse can't push it anywhere —
  pick a default. Suggest: no movement (forfeit). Tides on a light
  square pulse no piece (other than the Tide itself, which doesn't
  move).
- **A piece exactly on a Tide.** Can't happen — Tide occupies the
  square. Skip.
- **Pulse with no Tides.** If all Tides are captured, no pulse fires
  even at counter transitions. Counter still increments. Pulses
  resume if a new Tide is placed (variant FEN inject) or never
  (normal play).
- **Pulse-and-Anchor interaction.** A pulse-moved piece — is that a
  "move" that triggers an Anchor's mirror? **No** — pulse moves are
  environmental, not player moves. Document explicitly.
- **Pulse-and-Echo interaction.** Does a pulse update the Echo's
  recorded delta? Each pulse moves many pieces; pick one? Suggest:
  pulses don't update the Echo. Only player moves do. Document.
- **Counter and clock pollution.** Adding a TC field to FEN is mildly
  invasive. Could attach the counter to one of the Tides as a
  payload (`P=TD,C=N,CT=2`), but if there are multiple Tides they'd
  all need to agree. Cleaner as a board-global field.
