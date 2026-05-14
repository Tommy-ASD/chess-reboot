# Magnet

> A stationary piece that, at the start of each opponent turn, pulls
> every adjacent enemy one square toward it.

## Inspiration

Plan 07's `apply_environment_reactions` (or whatever the engine
calls its between-turns hook for terrain effects — Frozen
condition decrement, train propagation, etc.) is the seam this
piece slots into. The Magnet's only mechanic *is* an environment
reaction; it does nothing on its own turn that any other piece
couldn't do.

The design problem: pieces in chess are agents. Their threat lives
in the moves they could make. The Magnet inverts this — its
threat is a *passive field* around itself. The enemy's positional
agency is what triggers the magnet's effect, but the effect runs
*on the magnet's terms*. This is the same shape as
`SquareCondition::Frozen`: a passive board property that consumes
opponent agency.

## Mechanic

Movement set: **none.** The Magnet cannot move and cannot capture.
It can be captured by enemies that step onto it.

Special property — **Pull field.** At the *start* of the opponent's
turn (before they choose a move), iterate every same-color
Magnet on the board in a deterministic order (top-to-bottom,
left-to-right by coordinate). For each Magnet, iterate its 8
neighbours in a fixed order (N, NE, E, SE, S, SW, W, NW). For
each neighbour that contains an enemy piece:

1. Compute the "toward-magnet" square — the unique square between
   the enemy and the magnet, on the King-step that reduces the
   distance. For an enemy diagonally adjacent: toward-magnet is
   the magnet itself; pulling collapses into capture (open
   question below).
2. If toward-magnet is empty *and* walkable, move the enemy
   there.
3. If toward-magnet is occupied or non-walkable, the pull fails
   silently for that enemy.

After all Magnets resolve, the opponent's turn proceeds normally.

Critical clarifications:
- Pulls do *not* count as a move for that piece (no
  side-effects like en-passant clearing, no `has_moved` flag
  set).
- Pulls *do* trigger square-condition entry effects (a piece
  pulled onto a `Switch` fires the Switch — consistent with
  step-onto semantics).
- Pulls trigger train-track propagation? Likely no — pulls are
  not the train's "step." Worth deciding.
- If an enemy is adjacent to *two* magnets (same color or
  different colors), the deterministic order resolves which pull
  applies first. Pulls are serialized, not simultaneous.

## Why it's interesting

Three reasons:

1. **Threat without movement.** The Magnet is the chess equivalent
   of a tractor beam — its threat doesn't propagate via piece
   moves but via the *opponent's* attempt to play. Enemies near a
   Magnet are losing one move's worth of positional freedom every
   turn.

2. **Deterministic ordering is load-bearing.** Two adjacent
   enemies, one magnet: which gets pulled first? The choice
   matters because the first pull may block the second. The
   coordinate-order rule keeps this engine-deterministic and
   FEN-replayable.

3. **Reuses the environment-reaction tick.** No new turn-phase, no
   new game-loop entry point. Plan 07 already runs a
   between-turns reaction pass; Magnet adds one more reaction.

## Example scenarios

**Pawn vacuum.** Black has Pawns on d6, e6, f6. White Magnet on
e5. Start of black's turn: pulls iterate. Enemy on d6 (NE of
magnet): toward-magnet is e5 — but the magnet is on e5, so the
pull *would* be a capture. Skip (or alternatively: capture-pull,
see open questions). Enemy on e6 (N of magnet): toward-magnet is
e5 = magnet. Same situation. Enemy on f6 (NW of magnet):
toward-magnet is e5 = magnet. Same. Net: no pulls applied (if
no-capture rule). Magnet is being besieged but not destroyed.

**Edge pull.** White Magnet on a1. Black Knight on b2. Start of
black's turn: toward-magnet is a1 = magnet itself. Skip.
Re-position Magnet to a2 in mind (FEN edit): black Knight on b2.
Toward-magnet is a2 = magnet. Skip. The Magnet is hard to use
on enemies that are *immediately* adjacent. Now black Bishop on
c3: not adjacent to a Magnet at all. No pull. The Magnet only
acts on the 8-neighbourhood.

**Mid-board attrition.** White Magnet on e5. Black Queen on g5
(not adjacent, two squares east). Black Knight on d6 (NW
adjacent). Start of black's turn: Knight on d6, toward-magnet is
e5 = magnet itself. Skip. Black plays Q moves; black turn ends.
White turn. Black's next turn: Knight still on d6. Same skip.
The pull only activates when the enemy *enters* the 8-neighborhood
and is not immediately adjacent. Re-spec: change rule from "8
neighbours" to "8 neighbours, two-square pull" — see open
questions.

## Where it shines

- Defensive variants where slowing the opponent is enough.
- Compositions where Magnets are paired with `Block` walls — the
  Magnet pulls toward the wall, trapping enemies against it.
- Train compositions — Magnets near tracks force enemies into the
  train's path.
- Asymmetric variants where one side gets a static-piece
  advantage.

## Where it's awkward

- The "adjacent enemy → toward-magnet is the magnet itself" edge
  case is a recurring source of confusion. Either the pull
  becomes a free capture (powerful) or the pull no-ops (weak).
  Neither is obviously right.
- Pulls don't compose well with multiple magnets pulling in
  opposite directions. The first-applied pull wins; the second
  is wasted. Hard to evaluate intuitively.
- Frontend visualization: a pull is not a player-initiated move,
  but it's a piece movement that needs animating. UI work.
- Stationary pieces have no agency in their own turn — players
  may feel a Magnet "doesn't do anything" on white turns. Mostly
  a perception problem; mechanically fine.

## Engine dependencies

- The environment-reaction tick (plan 07's between-turns hook).
- The deterministic-iteration pattern (already used for Skibidi
  resolution and train propagation).
- `is_walkable()` predicate for pull target validation.
- Color::Neutral handling — Magnets are color-aligned, so this
  is normal piece coloring, not Neutral.

## New features required

- A new piece type `Magnet` with no movement primitive (movement
  generation returns empty; capture-on-step-onto is normal).
- A new entry in the environment-reaction tick: enumerate magnets
  of the *current-turn-opponent's* color (i.e., the side that's
  about to move suffers pulls). Apply pulls in coordinate order,
  neighbour order.
- Pull-target validation: empty + walkable.
- Optional: a pull-event log so the frontend can animate the
  pulls. Not strictly necessary for engine correctness.
- Tests: single pull happy path; pull onto Switch fires signal;
  pull blocked by friendly; multiple magnets ordering;
  immediately-adjacent enemy edge case; FEN round trip.

## FEN encoding

Piece tag: `MAG` (multi-character, since `M` is ambiguous — Monkey
already uses `M`).

```
(P=MAG)              # white Magnet
(P=mag)              # black Magnet
```

No state. The pull effect is fully determined by adjacency + board
state at opponent-turn-start, so no per-Magnet state needs
serializing.

## Open questions

- **Capture-pull?** When an enemy is adjacent and toward-magnet is
  the magnet itself: (a) no-op skip (recommended default — weak
  but unambiguous), (b) enemy is captured by the Magnet (powerful,
  surprising), (c) the Magnet is captured by the enemy
  (anti-magnet, weird). Recommend (a) for v1.
- **Two-square pull?** Currently the Magnet only acts on
  8-neighbours. A two-square pull (radius-2 with intermediate
  squares pulled to radius-1) is more visible mechanically but
  much more complex to specify. Recommend: stick with
  8-neighbour for v1.
- **Diagonal pulls.** An enemy NE-adjacent to a Magnet, with the
  magnet itself blocking the pull target: skip. But if the enemy
  is two squares NE (not adjacent) and we adopt two-square pull,
  the toward-magnet square is the NE-1 square — this is the
  diagonal pull. Worth a separate rule. v1: ignore.
- **Friendly piece adjacent to Magnet.** No pull (only enemies
  pulled). Friendly pieces can stand around a Magnet without
  effect.
- **Magnet adjacent to Magnet (same side).** No interaction; each
  pulls its own enemies. Friendly Magnets don't interfere.
- **Magnet adjacent to enemy Magnet.** Each pulls at start of the
  *other's* turn. Net effect: oscillation if pulls succeed both
  ways. Mostly fine; rare in practice.
- **What if the pull would put the puller's king in check?** A
  pull is not a player move, so it's not subject to the
  no-self-check rule. But: the engine should evaluate
  check-detection *after* all pulls resolve, so the player-side
  whose turn is about to start sees the post-pull board. A
  Magnet that pulls an enemy Rook into a discovered check on
  *its own* king is fine — the side-to-move handles it.
- **Pulls during signal propagation.** If a pull moves a piece
  off a `PressurePlate`, does the signal release? Yes — signal
  semantics don't care how the piece left.
