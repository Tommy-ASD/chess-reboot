# Dog

> A piece that periodically deposits "slippery" tiles, causing any piece that moves onto them to slide one extra square in its direction of travel.

## Source

From `engine/src/pieces/ideas.txt:21-22`:

> Dog
>     It shits and the shit is slippery

That is the entire entry. The vibe is clear. The mechanic is not.
This doc takes the joke and designs the absolute minimum coherent
mechanic that respects it.

## Inspiration

This is the silliest piece in the file. We will not pretend it's
not. The honesty principle: design the *mechanic* rigorously and
let the *naming* / aesthetic carry the joke.

The mechanical kernel underneath the gag is interesting: a piece
that emits a passive terrain modifier on its movement path. None of
the implemented pieces (Goblin, Skibidi, Bus, Monkey, Locomotive +
Carriage) creates persistent terrain along their movement trail.
Block-square placement (plan 12, Jackhammer) is point terrain;
Skibidi's brainrot radius is a *radiating* effect that moves with
the piece. A trail-of-deposits piece is a new shape.

So: take the joke seriously enough to build a working trail-deposit
piece, and let the *naming* (Dog) be the comedy. The mechanic
itself is reusable — could be reskinned as a Snail (slime trail), an
Oil Tanker, a Skunk, etc., without changing a line of engine code.

## Mechanic

The Dog moves and, on a regular cadence, leaves a **slip tile**
behind it. Pieces that move onto slip tiles slide one extra square
in their direction of travel.

### Base movement

**Move like a king** (one square, any direction). Captures normally.

Justification: a Dog moves slowly. A queen-mover Dog with a slip
trail would carpet the board in two turns. King-mover Dogs require
deliberate routing for the trail to matter.

### Deposit cadence

**Every other Dog move (every 2nd Dog turn) deposits a slip tile on
the square the Dog just *left*.** First Dog move: no deposit. Second
move: deposit on the square vacated by the second move. Third move:
no deposit. Fourth: deposit. Etc.

State: a single boolean (`will_deposit_next: bool`) on the Dog piece.
Toggled each move.

Justification for every-other-turn:

- Every-move deposits saturate the board in 8 moves.
- Every-3rd is awkward to remember (which third?). Every-other is
  trivial: it deposited last move iff it didn't deposit this move.
- The cadence telegraphs threats: opponent can see "Dog is about to
  deposit" and plan around it.

Alternative cadence considered: *deposits whenever it captures*.
Cleaner trigger, but defeats the "trail" aesthetic — captures are
rare. Stick with every-other-turn.

### The slip tile

A new square type in the engine's terrain vocabulary:

```rust
SquareType::Slip { dir: Option<Direction> }
```

A slip tile is **walkable** (pieces can land on it). When a piece
moves onto a slip tile, *if* its current move had a direction (king,
queen, rook, bishop, pawn-push, knight included), it **continues
sliding one more square in the same direction** if that square is
walkable and unoccupied by an enemy piece.

Resolution:

1. Piece arrives at slip tile S via direction D.
2. Compute next square N = S + D.
3. If N is on the board, walkable, and:
   - Empty → piece continues to N.
   - Friendly piece → piece stops at S (can't push allies).
   - Enemy piece → piece *captures* at N (slide-capture).
4. The slip tile remains. Subsequent pieces are also slipped.

Edge cases:

- **Off-board.** Slip would take the piece off the edge. Piece
  stops at the slip tile. No fall damage; chess board is not lava.
- **Slip onto slip.** If N is also a slip tile, **slide one more**.
  Recursive slipping is in keeping with the bit. Cap at one chain
  per move to avoid pathological "8 slip tiles in a row → cross
  the board" scenarios. **Recommend cap: 1 additional slide per
  origin move.** Telegraph-friendly, easy to teach.
- **Knight slip.** A knight's direction *is* its L-shape jump.
  Knights don't have a "continued direction" — recommend knights
  *do not* slip. Slip is a sliding-piece effect. Knights land on
  the slip tile and stop. Documented.
- **Pawn slip.** A pawn pushing forward onto a slip tile continues
  one more square forward (still subject to walkability /
  occupancy). En-passant onto a slip is a real corner case;
  document and treat as: slip applies post-capture, like any other
  arrival.

### Slip-tile direction memory

The above leaves an option open: does the slip tile *remember a
direction*, or does it just continue the arriving piece's direction?

**Recommend: just continue the arriving piece's direction.** No
direction stored on the tile (`dir: None` always). Tile is
direction-agnostic; the slip uses the *piece's* momentum.

Alternative: tile remembers the Dog's heading when it was laid, and
*overrides* the arriving piece's direction with the Dog's. More
chaotic; harder to reason about. Skip.

So actually:

```rust
SquareType::Slip
```

No payload. (`dir` field above was for the alternative; under the
recommendation it's removed.)

### Slip tile lifetime

Slip tiles persist *until a piece moves over them*. After being
used (slipped) **once**, the slip tile is consumed back to
`Standard`.

Justification: permanent slip tiles would compound forever (Dog
moves 20 times, board has 10 slip tiles). One-shot tiles are
self-cleaning. They also create a *risk/reward* — do you walk over
a slip tile now to clear it, or avoid it and let it sit as an
opponent's threat?

Alternative: permanent. Skip — board pollution risk.

Alternative: decay after K turns. Adds a timer per tile. Skip —
state bloat for marginal benefit.

## Why it's interesting

Slip tiles are the first *trail-deposit* terrain in the engine.
Existing terrain is either hand-placed (Block, Switch, Gate) or
radiated from a moving source (Skibidi brainrot, train track —
though tracks are pre-placed). A Dog *writes* terrain as it walks.

The slipping mechanic also adds a new movement primitive: forced
over-travel. Every other piece in chess stops exactly where you
move it. A piece that arrives at a slip tile is moved one extra
square *by the board*. This is genuinely new.

## Example scenarios

1. **Knight refusal.** Black Dog has deposited a slip tile on e4.
   White knight wants to land on e4 (good outpost). Knight lands
   on e4, doesn't slip (knights immune). Fine. But white's bishop,
   trying to reach f5 *through* e4 on the diagonal, can't — sliders
   don't *land* on the trail unless they stop there, in which case
   they slip. Position-denial without a wall.
2. **Forced overshoot.** Black queen wants to reach d5 to deliver
   check. d5 is a slip tile. Queen slides to d5, then forced to d6
   (continued direction) — d6 is occupied by a black piece, so the
   queen stops at d5. But if d6 were empty: queen lands on d6
   instead, possibly hanging.
3. **Self-slipping.** White Dog has deposited slip tiles in its own
   wake. White's own rook tries to traverse the row — slips one
   extra square. Friendly fire is real. Routing around your own
   trail is part of playing a Dog deck.

## Where it shines

- **Open positions where direction control matters.** Slip tiles
  ruin straight-line attacks.
- **Variants with multiple slow pieces.** A Dog in a board with
  many king-mover units becomes a regional denial tool.
- **Comedy.** Self-evidently.

## Where it's awkward

- **Cognitive overhead.** Every move now has to check "am I landing
  on or sliding through a slip tile?" Multiplicatively annoying
  with brainrot, frozen, etc.
- **Pawn-structure interaction.** Pawn pushes can be forced past
  promotion rank if a slip is on rank 7 and rank 8 is walkable
  and empty. **Decision point**: does the pawn promote at rank 8
  after slipping there? **Recommend yes** — the pawn legally
  landed on rank 8, whatever the route. But note: this means a
  Dog-aided pawn can promote *one move earlier than expected*.
  Could be feature.
- **It is still a piece called Dog whose mechanic is leaving shit
  on tiles.** This will not test well with chess purists. By design.

## Engine dependencies

- Terrain system (existing) — new `SquareType::Slip` variant.
- Walkability check (existing) — slip tiles are walkable.
- Post-move-resolution hook for "did the destination square trigger
  a follow-up slide?" The plan-10 movement-stack framing fits this:
  slip is a *modifier* that fires after the base move.
- Square-mutation hook (Dog deposits slip tile on departure square).

## New features required

- `SquareType::Slip` variant. Walkable, no payload.
- Per-piece field for the Dog: `will_deposit_next: bool`.
- Movement-pipeline modifier: "after a directional move lands on
  Slip, attempt one more step in the same direction."
- Square-deposit hook: when a Dog leaves a square and `will_deposit_next`
  is true, replace the vacated square's type with `Slip` (unless
  the square is already a non-Standard special type — recommend:
  Dog cannot deposit on non-Standard tiles).
- Tile consumption: after a piece slips, the slip tile reverts to
  `Standard`.

## FEN encoding

Two pieces of state need to round-trip:

1. **The Dog itself.** Symbol `D` (white) / `d` (black). Payload:
   `(NEXT=1)` if the next move will deposit (the toggle is currently
   "armed"), omitted otherwise. Default omitted = false.

   Example: `D(NEXT=1)` on e3 — next Dog move deposits.

2. **The slip tile.** Square type `(T=SLIP)`. No further payload —
   tile is direction-agnostic and one-shot, no stored state.

Examples:

```
... 4D(NEXT=1)3 ...
... (T=SLIP) ...
```

Round-trip:

- Encoder: emit `(NEXT=1)` on the Dog iff its flag is set; emit
  `(T=SLIP)` for slip squares.
- Decoder: parse `NEXT` as bool (default 0); parse `T=SLIP` as the
  new variant; default to `Standard` if unknown tag.

## Resolving the source's open questions

The source says one thing: *"It shits and the shit is slippery."*
There are no explicit open questions, only implicit ones. Answered:

- **What does it move like?** King.
- **When does it deposit?** Every other Dog move.
- **Where does the deposit go?** The square the Dog just *left*.
- **What does the slippery tile do?** Forces one extra square of
  travel in the arrival direction.
- **Do slips persist?** One use, then revert to Standard.
- **Are knights affected?** No — slip is a sliding-piece effect.
- **Can slips chain?** One additional slide per origin move
  (cap recursive slipping at 1).
- **Does friendly fire apply?** Yes. Your own pieces slip on your
  own Dog's trail.

The mechanic survives without changing on any single answer being
flipped — these are all dials. The recommended defaults are
"telegraph-friendly, easy-to-teach, doesn't pollute the board."

## Open questions (new)

- **Reskin candidates.** Snail (slime trail), Skunk (stink trail),
  Oil Tanker (oil), Dog (the original). Same mechanic, different
  art. The implemented variant should pick one — recommend Dog,
  honouring the source.
- **Can the Dog deposit on the home square?** Probably yes — the
  Dog moved off, the square is now empty Standard, deposit
  proceeds. Worth confirming.
- **Multi-Dog interactions.** Two Dogs both depositing build a
  trail-of-trails. No special rules needed; trails are independent
  Slip tiles.
- **Slip tile placed onto a Block square?** Block isn't a deposit
  target (Dog can't depart from a Block — Block can't hold a piece
  in the first place). Recommended guard: deposits only convert
  `Standard` → `Slip`; any other source type, deposit fizzles.
- **Aesthetics.** The original entry strongly implies a particular
  visual treatment. Engine doesn't care; frontend will decide if it
  wants to lean in or render the slip tiles as harmless pawprints.
