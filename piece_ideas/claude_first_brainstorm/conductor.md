# Conductor

> A King-mover that, instead of moving, can throw any one `Switch` on
> the board from anywhere — solves signal puzzles without parking a
> piece on the trigger.

## Inspiration

Plan 08 establishes the signal substrate: `Switch`, `Junction`,
`Gate`, `PressurePlate`, wired by `SignalId`. Today the only way to
fire a Switch is for a piece to *stand on* it (PressurePlate) or
*step onto* it (Switch trigger-on-entry). That means signal puzzles
become positional logistics — "how do I get a rook onto that
switch without losing it?" — which is fine for some compositions
but limits expressiveness.

The Conductor adds remote actuation. It carries no signal state
itself; it's a *delivery mechanism* for a signal-firing action from
anywhere on the board. The piece exists to make signals usable as
tactical primitives rather than just terrain hazards.

## Mechanic

Movement set: identical to King (one-square in any direction,
captures normally, walkability-filtered).

Special action — **Remote-fire.** Once per turn, instead of moving,
the Conductor names any one `Switch` tile on the board and fires it
as if a piece had just stepped on it. The Conductor itself does not
move and is not on the Switch. Signal propagation (Junctions,
Gates, etc.) runs as normal.

Constraints:
- Target must be a `Switch` tile (not `PressurePlate`,
  `Junction`, or `Gate`). PressurePlates require continuous
  presence and don't fit the "fire once" verb; Junctions and
  Gates are downstream of switches and aren't user-fireable.
- Target need not be visible, adjacent, or reachable. The whole
  point is range.
- The fired Switch behaves identically to a step-fired Switch.
  Same propagation, same one-shot vs. toggle semantics (whatever
  plan 08 specified).

No cooldown, no per-game limit. The Conductor's scarcity comes
from being a piece you can lose.

## Why it's interesting

Three reasons it's mechanically interesting:

1. **Decouples actuation from logistics.** The composer designs a
   signal puzzle for its logical content (gate A opens iff switch B
   fires while plate C is held). The Conductor lets a player solve
   the puzzle by paying *tempo* rather than *positioning*. Two
   independent resources, two independent design dials.

2. **Range without movement.** Most long-range pieces capture at
   range (Bishop, Rook, Queen). The Conductor *acts at any range
   without moving*. The shape of its threat is the whole board,
   filtered to Switch tiles only. That's a unique threat geometry —
   not slider, not jumper, not radial.

3. **Forces a new move-type slot.** The engine has movement moves,
   capture moves, and (per plan 09) probably train-trigger moves.
   `MoveType::FireSwitchRemote { switch: Coord }` is a fourth slot:
   "act on a coord without moving there or being adjacent." This is
   the same slot the Sniper would use (see [sniper.md](sniper.md)).
   Adding it pays off across multiple pieces.

## Example scenarios

**Gate trick.** White Conductor on a1. Black king on h8 with a Gate
adjacent (g8 = `Gate(id=3)`). The wiring: switch on c5 controls
Gate 3. White Conductor remote-fires c5. Gate 3 opens (or closes,
depending on plan 08 semantics). If "opens," the king now has a
non-walkable square g8 turn into a walkable one (or vice versa) —
the Conductor just rearranged the king's escape topology from
seven ranks away.

**Mid-game commitment.** Black's plan revolves around holding a
PressurePlate with a rook to keep a Gate open. White Conductor
fires a Switch with the same `SignalId` *target list* but inverted
polarity — the Gate slams shut. Black's rook is still on the
plate but the plate is now signal-redundant. Black has effectively
lost a tempo of rook activity.

**Defensive use.** White Locomotive bearing down on black's king
along a track. Black Conductor on the other side of the board
fires a Switch that operates a track Junction in the train's path.
Train derails (per plan 09). The Conductor saved the king without
moving a piece.

## Where it shines

- Variant compositions heavy on plan 08 signal infrastructure.
- Puzzle/composition boards. The Conductor lets composers add
  "solve this with X tempi" constraints that decouple from
  positional puzzles.
- Boards where Switch tiles are *defended* by enemy pieces — the
  Conductor bypasses the defence entirely.

## Where it's awkward

- Boards with zero Switches. The Conductor reduces to a King-mover.
- Boards with one Switch that has no significant downstream effect.
  Same problem.
- Designer must consider Conductor presence when wiring signal
  puzzles; an unintended Conductor can trivialize a composition.
  This is a category of design constraint, not a bug.
- Move-gen output bloat: every Switch on the board produces a
  candidate `FireSwitchRemote` move from this Conductor. With many
  Switches, that's a lot of candidates. Probably fine for
  reasonable puzzle sizes (< 20 switches), but worth measuring.

## Engine dependencies

- `SquareType::Switch { targets: Vec<SignalId> }` and the
  signal-firing routine from plan 08.
- King-movement primitive.
- Move-gen ability to enumerate all `Switch` tiles on the board (a
  filter over `board.squares`).

## New features required

- `MoveType::FireSwitchRemote { switch: Coord }`. Apply-side
  implementation calls the same signal-fire routine that
  step-trigger uses. Undo restores any toggled `Gate.open` /
  `Junction.direction` states and reverts wherever the propagation
  changed.
- Move-gen entry: for each `Switch` tile on the board, emit one
  `FireSwitchRemote` move from the Conductor (plus the standard
  King moves).
- Test: Conductor + Switch + Gate scenario. Fire Switch remotely,
  confirm Gate state changes. Undo, confirm Gate state restored.
- Test: Conductor + multiple Switches. Confirm correct one fires.

## FEN encoding

Piece symbol: `C` (Conductor). Single-letter, likely free. If
`C` is already taken by a future piece, `Cn` works as a fallback.

```
(P=C)            # white Conductor
(P=c)            # black Conductor
```

No piece-level state. The Conductor remembers nothing across
turns. Cooldown-free, count-free — fully reconstructible from
move history.

## Open questions

- **Should the Conductor be able to fire its *own* tile if it's
  standing on a Switch?** The wording "instead of moving" admits
  this, but it's redundant — stepping off and back onto the Switch
  would have done the same. Recommend allow + document as a
  no-positional-cost re-trigger.
- **What about `PressurePlate`?** Currently disallowed. The
  semantic mismatch (plate = continuous presence) makes
  "remote-fire" unclear: does the Conductor briefly land on the
  plate? Recommend: no. If a future plan wants
  `MoveType::FirePlateRemote { plate, duration }` it adds it
  separately.
- **Conductor + Conductor interaction.** Two Conductors on the
  same side: do they both get a remote-fire action per turn? Yes;
  each is its own piece's action. Combined they can fire two
  Switches per turn, which dramatically widens puzzle solvability.
  Note as a balance dial.
- **Pin behaviour.** A pinned Conductor: can it remote-fire? It
  doesn't move, so it can't expose its king by moving. Recommend
  yes, remote-fire-while-pinned is legal. Worth a test.
- **Should remote-fire reveal information?** In hidden-information
  variants, the Conductor's existence implies a remote actuation
  threat to any visible Switch. This is fine; the threat is
  legible from the piece's existence, not from any move-time
  reveal. No fog-of-war problem.
