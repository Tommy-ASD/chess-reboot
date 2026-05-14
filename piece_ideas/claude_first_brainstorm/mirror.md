# Mirror

> On its turn, can either move like a King — or replay the *shape*
> of the opponent's last move from the Mirror's own square.

## Inspiration

A piece whose moveset is determined by what just happened. The
engine already has `MoveType` variants enumerable across the
whole move history (the move log). Extracting "shape of last move"
out of a `MoveType` is a small projection — direction + distance
+ capture-flag for sliders, exact knight-jump offset for jumpers,
etc.

The chess problem it answers: most fairy pieces are *static* —
the same threat geometry every turn. The Mirror's threat
geometry *changes per turn* based on opponent play, forcing the
opponent to consider how their *own* move shape will weaponize
the Mirror against them.

State lives in `BoardFlags` (the engine already has variant
flags, halfmove clock, etc. in this struct) rather than on the
piece, because "last opponent move" is a board-global property,
not Mirror-specific. Multiple Mirrors share the same
`last_seen_move`.

## Mechanic

Movement set on the Mirror's turn — chooses one of:

**A. King-move.** Standard one-square any direction with capture.

**B. Mirror-move.** Replays the *shape* of `BoardFlags.last_seen_move`
from the Mirror's own square. "Shape" means:

- For a slider move (Bishop/Rook/Queen/Bus): the direction
  vector and the distance. So if the opponent's Queen moved
  `(+3, +3)`, the Mirror moves `(+3, +3)` from its own square.
  Capture-by-mirror is allowed if the target square has an
  enemy.
- For a Knight-like jump: the exact offset (e.g., `(+1, +2)`).
- For a King-step: just a one-square direction. (Equivalent to
  Mode A; the Mirror could've King-moved anyway.)
- For a Pawn push: a single forward step *in the Mirror's color's
  direction* (so a white Mirror mirroring black's pawn push moves
  forward = up the board for white).
- For a Pawn capture: a forward-diagonal step in the Mirror's
  color's direction, target must be enemy.
- For a special move type (Skibidi rotate, Goblin kidnap, Bus
  passenger swap, train trigger, paint-square, fire-switch-remote,
  reanimate, vampire absorb, etc.): **does not mirror.** The
  Mirror falls back to King-only when the previous move was a
  special.

Constraints:
- The mirrored shape's destination must be on the board, walkable,
  and not blocked by friendlies along the path (for sliders).
- The mirrored shape's destination can have an enemy (capture) or
  be empty (move).
- If the mirrored shape is invalid from the Mirror's square
  (off-board, blocked, would self-check), Mode B is unavailable
  this turn and the Mirror is limited to Mode A.

## Why it's interesting

Three layers:

1. **Threat geometry is opponent-dependent.** The opponent can
   make the Mirror weak (move a Pawn one square) or strong (move
   a Queen seven squares). Their choice indirectly chooses the
   Mirror's threat. This couples the two sides' move spaces in
   a way nothing else does.

2. **Information-theoretic interesting.** A skilled opponent
   tracks both their *own* move's value and the *Mirror's*
   downstream value. Trading short-tempo gain for Mirror-threat
   reduction is a new decision-theoretic axis.

3. **Stresses `BoardFlags.last_seen_move`.** No current piece
   needs this. Adding it lays groundwork for other history-aware
   pieces (a Defender that re-positions to the previous capture
   square? An Echo that copies your *own* last move?). The
   one-piece justification is thin; the substrate justification
   is rich.

## Example scenarios

**Queen mirror.** Black's last move: Queen `Qd1-d7` (slide
`+0, +6`). White Mirror on a1, white turn. Mode B candidate:
slide `+0, +6` from a1 = a7. If a7 is empty + walkable + path
clear, white Mirror jumps a1 → a7 in one move. A 6-square slide
on a King-like piece.

**Knight mirror.** Black's last move: Knight `Nf6-e4` (jump `-1,
-2`). White Mirror on h1. Mode B candidate: jump `-1, -2` from
h1 = g(-1) — off-board. Mode B unavailable. White Mirror
restricted to King-only this turn.

**Mirror chess.** Both sides have Mirrors. White moves Mirror as
King (Mode A, since black hasn't moved yet, or
`last_seen_move = None`). Black mirrors white's last move (one
square step). The game becomes mostly King-shaped until someone
makes a "richer" move that gives the opponent's Mirror a real
threat — and then the dance escalates.

## Where it shines

- Positions where the opponent has to make committal slider
  moves. The Mirror punishes long Queen slides.
- Variants with diverse piece sets — more move shapes = more
  Mirror options.
- Mid-game tactical play where each side is making strong
  moves; each strong move opens a strong Mirror reply.

## Where it's awkward

- Opening play. `last_seen_move = None` means Mirror is
  King-only until the first move. Probably fine — the opening
  move is by definition not a Mirror-side piece (or, if both
  sides have Mirrors, the first move is itself a Mirror with
  no prior move = King-only).
- Special-move-heavy variants. If the opponent moves a Goblin
  (special), the Mirror falls back to King. This "downgrade
  by special-move" is a real strategic tool for the opponent.
- The single-state `last_seen_move` slot is *board-global*. In
  a future free-for-all variant with three+ players, this
  doesn't generalize cleanly. Defer.
- Visualization: explaining to the player *why* their Mirror
  can or can't slide six squares this turn requires showing
  the opponent's previous move shape. UI design problem.

## Engine dependencies

- The move log (already maintained for undo and FEN).
- Per-piece-type "shape" extractor — a function `MoveType ->
  Option<MoveShape>` where `MoveShape` is something like
  `enum MoveShape { Slide { dx, dy, dist }, Jump { dx, dy }, Pawn { capture }, None }`.
- King-movement primitive.

## New features required

- `BoardFlags.last_seen_move: Option<MoveType>` — or a derived
  `Option<MoveShape>` to avoid recomputing on every Mirror's
  move-gen. The full `MoveType` is more general and FEN-able;
  recompute the shape lazily.
- `MoveShape` enum and the projection function.
- Per-side `last_seen_move` if multi-player variants ever need
  it. v1: single board-global slot, set to the previously-moved
  piece's move.
- Move-gen entry for Mirror: emit King moves + (if Mode B
  available) the shape-replayed move from the Mirror's square.
  Filter for legality.
- Apply-side: when any move is made, update
  `BoardFlags.last_seen_move = Some(that_move)`. Mirror's own
  Mode-B move updates it to the *Mirror's* move, which is
  what the next opponent's Mirror would copy. Recursive
  Mirror-of-Mirror games are mechanically defined.
- FEN encoding of `last_seen_move` — needs a serialized form.
  Probably the same encoding the move log uses already.

## FEN encoding

Piece tag: `MIR` (multi-character; `M` ambiguous).

```
(P=MIR)              # white Mirror
(P=mir)              # black Mirror
```

Piece-level state: none. The Mirror's "memory" lives on
`BoardFlags`, not on the piece.

`BoardFlags.last_seen_move` FEN representation — a new field in
the variant-payload section of the FEN. Suggested:

```
... lsm=Qd1d7 ...    # last seen move as algebraic-style string
... lsm=- ...        # no last seen move (game start)
... lsm=special ...  # last move was a non-mirrorable special
```

Exact encoding aligns with whatever format the move log uses.
Open question on what's preferred.

## Open questions

- **Mirror-on-Mirror.** If both sides have Mirrors and white
  mirrors black's Queen slide, then black mirrors white's
  Mirror-Queen-slide. Black's Mirror has same shape available as
  white's, which means black's Mirror also makes a long slide.
  Recursive escalation is fine and game-theoretically interesting.
- **Mirror copying its own side's previous move.** When white's
  Mirror's turn comes, the previous move was *black's*, not
  white's. Good. But two consecutive white-side moves never
  happen (no double-move in standard turn structure), so this
  edge case doesn't arise.
- **Castling.** Castling is a special move; Mirror falls back to
  King. Probably right — copying castling from another square
  is nonsensical.
- **Promotion.** A Pawn-push that promotes is a special. Mirror
  falls back. Right call.
- **Pin / check legality.** Mirror's Mode-B move can pin/be-pinned
  same as any move. Standard legality.
- **Skibidi/Brainrot adjacency.** A Mirror that's Brainrot-stunned
  can't move at all, regardless of Mode A vs Mode B. Per
  existing Skibidi semantics.
- **`MoveShape::None` from special moves.** Document: a Mirror
  facing a special opponent move is King-only. This is a
  designed downgrade, not a bug.
- **What about Mode-B sliding into a Switch/PressurePlate?**
  Trigger as normal (consistent with stepping onto). Mirror moves
  trigger square-conditions same as any other move.
- **`last_seen_move` mutation by environment reactions.** When a
  Magnet pulls an enemy, does that count as a "move" for
  `last_seen_move`? Recommend no — pulls aren't player moves.
  `last_seen_move` only updates on player-issued
  `MoveType`s. Worth a test.
