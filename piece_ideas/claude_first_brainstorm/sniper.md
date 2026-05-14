# Sniper

> Moves like a King but doesn't capture by moving — instead emits a
> dedicated "Snipe" move that captures any enemy on its file, rank,
> or diagonal with a clear line.

## Inspiration

The Conductor introduces `MoveType::FireSwitchRemote` — a move that
acts on a coordinate without moving the piece. The Sniper sits in
the same slot: act-at-range-without-moving, with a different
predicate (target = enemy on line-of-sight rather than target =
Switch tile).

The design problem: capturing-while-moving is the default chess
shape. Capturing-*not*-while-moving exists in some fairy variants
(displacement-vs-rifle captures), but it's not in the engine yet.
The Sniper builds the abstraction.

Two pieces sharing the "act-at-coord" pattern justifies the
generalized `MoveType` slot. Future pieces (a Healer that targets
a friendly square, a Cursebearer that drops a Brainrot condition
on a target square) can reuse the same infrastructure.

## Mechanic

Movement set:
- **King-step.** One square any direction, *non-capturing only*.
  Friendly captures impossible (standard). Enemy captures *not*
  allowed by stepping. A Sniper that's adjacent to an enemy
  cannot capture by moving into it.

Special action — **Snipe.** For each enemy piece on the Sniper's
file, rank, or diagonal with a clear line (no pieces or
non-walkable squares in between), the Sniper can — instead of
moving — capture that enemy in place. The Sniper itself does not
move.

Constraints:
- The line is the standard Queen-slider line: 8 directions
  (rank, file, two diagonals).
- "Clear line" means every square between the Sniper and the
  target is `is_walkable()` *and* empty of pieces. Walls,
  closed Gates, Block tiles, and other pieces all break line.
- One snipe per turn (it's a move). No double-snipe.
- The Sniper *can* be captured by enemy moves stepping onto its
  square (it's not invincible).

## Why it's interesting

Three reasons:

1. **Threat geometry and movement geometry disagree.** The
   Sniper's threat extends along eight rays to any distance, but
   its movement is one-square King. A normal piece's threat and
   movement coincide; the Sniper's don't. The opponent has to
   plan around two different shapes for the same piece.

2. **Reuses the `act-at-coord` MoveType.** Same slot as
   Conductor's remote-fire. The engine pays for the slot once
   and gets two pieces from it.

3. **Forces line-of-sight management as a tactical concern.**
   Interposing a piece (friendly or enemy) blocks a Sniper.
   This makes ordinary pieces relevant in ways they weren't —
   a friendly Pawn that happens to sit on the line from the
   Sniper to an enemy King is now a critical defender.

## Example scenarios

**Long-range pick.** White Sniper on a1. Black Queen on h8. The
diagonal a1–h8 is empty. Snipe: white Sniper captures Queen on
h8 without moving. Black Queen gone, Sniper still on a1.

**Defensive interpose.** White Sniper on e4, black King on e8.
Black plays Pe7. Now the e-file from e4 onwards is blocked by
the Pawn at e7. The Sniper cannot reach the King. Black has
defended by interposition.

**Sniper duel.** Both sides have Sniper. White Sniper on a1,
black Sniper on h8. The a1–h8 diagonal is clear. White to move:
snipe black Sniper. Black Sniper gone. Note: black couldn't have
preemptively sniped the white Sniper on white's previous move
(it wasn't black's turn). Turn order is the resource that
distinguishes the two.

**Sniper + Architect combo.** White Sniper on d4, white Architect
on c5. Black King on h8. Architect paints walls to seal the
king's escape squares. Sniper snipes the king when no friendly
piece interposes. Architect's walls don't block Sniper *lines*
unless the wall is *on* the line — but a wall *adjacent* to the
king sometimes prevents the king from stepping into a Sniper-safe
square, which is what the Architect wants.

## Where it shines

- Open boards.
- Endgames with low piece count (fewer interposing pieces).
- Compositions where line-of-sight is the puzzle.
- Variants with `Block` walls authored as line-segments — gives
  composers a knob for tuning Sniper effectiveness.

## Where it's awkward

- Dense middlegame positions. Lines are perpetually blocked.
- Closed pawn structures negate the Sniper completely.
- Move-gen output cost: enumerate 8 rays, each potentially
  Queen-distance, find first enemy on each. Linear in board
  size. Manageable.
- The "King-step but no-capture" rule is subtle — the Sniper has
  movement *and* a separate capture mode, with the move-capture
  *not* coupled the way every other piece couples them. Players
  routinely will try to step the Sniper onto an enemy. UI must
  reject + explain.

## Engine dependencies

- King-movement primitive.
- The 8-ray slider enumeration (Queen move-gen's internals).
- The `is_walkable()` predicate for line-blocking.
- The same `MoveType` slot Conductor uses (the
  "act-on-coord-without-moving" generalization).

## New features required

- `MoveType::Snipe { target: Coord }` — or, more generally,
  fold this into a shared `MoveType::RemoteAct { target, kind }`
  that also subsumes `FireSwitchRemote`. Up to the engine
  designer.
- Move-gen entry for Sniper: emit King-moves (non-capturing
  only) + for each direction, the first enemy on the ray with
  clear line ⇒ emit Snipe move.
- Apply-side: capture target piece, Sniper unchanged. Standard
  capture pipeline (Vampire absorb, Bomb trigger, Reanimator
  graveyard observation — all apply to the snipe-captured piece
  as if it were a regular capture).
- Tests: snipe along each of 8 directions; line blocked by
  friendly; line blocked by Block; line blocked by closed Gate;
  Sniper cannot capture by stepping onto enemy; FEN round trip.

## FEN encoding

Piece symbol: `Sn` (multi-character; `S` is ambiguous — Skibidi
already uses `S`, and Switch tiles use `S` in some contexts).

```
(P=SN)               # white Sniper
(P=sn)               # black Sniper
```

No state. Snipe-availability is fully recomputable from board
geometry.

## Open questions

- **Does Snipe trigger conditions?** Standard capture pipeline
  involves removing the victim and running on-death hooks
  (Bomb explosion, Reanimator banking). Snipe should run these
  unchanged. Document.
- **What about Brainrot stun?** A Brainrot-stunned Sniper —
  can it snipe? Snipe is a move (consumes the turn), so
  Brainrot rules apply: if the piece can't move this turn, it
  can't snipe either. Worth a test.
- **Snipe through a `Vent`/`Turret`.** Both are non-walkable
  per plan 12's predicate. The line-of-sight rule says
  non-walkable breaks line. So no, Snipe doesn't go through
  Vents or Turrets. Conservative; matches walkability
  semantics elsewhere.
- **Snipe a piece on a Track tile.** Track is walkable. Snipe
  works normally. The sniped piece dies; if it was a Carriage
  attached to a Locomotive, the engine's train-decoupling
  logic handles it.
- **Friendly fire.** A friendly piece on the line breaks line
  (the Sniper sees an opponent only beyond friendlies that
  block the view? No, the rule is *anything* on the line
  blocks). So a friendly Bishop one square in front of the
  Sniper blocks all 8 rays from the Sniper's perspective in
  *that* direction. Recommend: friendly pieces block too. Same
  as enemy. This is the natural reading and the simplest
  predicate.
- **Range cap.** Currently unlimited (Queen-line). Could be
  capped (e.g., 5 squares) for balance. v1: unlimited.
- **What if multiple enemies are on the same line?** Snipe
  the first one (the one with clear path to). The Sniper
  *cannot* snipe a piece behind another piece — line of sight
  is strict. Future "Piercing Sniper" variant could allow.
- **Snipe legality vs. check.** A Sniper in check, with a
  snipe candidate that would capture the checker, is a
  check-escape. Worth a test.
