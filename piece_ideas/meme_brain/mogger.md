# Mogger

> A passive piece that freezes every weaker enemy in its
> king-radius — its footprint scales with its own material value.

## Inspiration

"Mogging" is internet slang for visibly outclassing someone by
appearance, status, or vibes. To "mog" is to make another person
look bad just by standing next to them. The piece literalizes
this: any enemy weaker than it physically cannot act while in its
presence.

Strip the paint: the mechanic is **passive inhibition with value-
gated targeting.** Square conditions like Frozen already exist;
what's new here is a piece-driven, value-comparative,
auto-refreshing aura. The Mogger doesn't *do* anything per turn
— it sits there and projects a threat that scales with material
asymmetry. This creates dynamic territory control that responds
to capture exchanges: trade a queen for a knight near the Mogger,
and suddenly your remaining pieces are inhibited where they
weren't before.

The most interesting wrinkle is that the Mogger's *own* material
value is a moving target — promote-to-queen-near-Mogger flips
which side gets mogged. The piece punishes mismatched force
concentration.

## Mechanic

### Base form

The Mogger has standard king movement: 1 square in any of 8
directions, captures included. Material value: somewhere around
3 (knight/bishop tier) — the value matters because everything
weaker is in mog range. See open questions on exact value.

### The mog aura

At the **end of every turn**, after all other end-of-turn effects
resolve:

1. For every Mogger on the board, scan its 8 king-radius
   adjacent squares.
2. For each adjacent square containing an **enemy piece**:
3. Compare the enemy's material value to the Mogger's material
   value. If **enemy_value < mogger_value (strictly less)**,
   tag the enemy's square with a fresh `Mogged` condition.
4. The `Mogged` condition is functionally identical to `Frozen`:
   the tagged piece cannot move on its controller's next turn.
5. Conditions decay automatically — `Mogged` lasts exactly 1
   turn (the enemy's next turn). After that turn, the
   condition is removed. The Mogger re-applies it at end of
   that turn if the enemy is still adjacent.

### Stackability and overlap

- Two Moggers each tagging the same enemy: the enemy is still
  just `Mogged` for one turn. Conditions are sets, not stacks.
- An enemy fleeing Mogger A into the radius of Mogger B: at end
  of turn, Mogger B re-evaluates and tags the enemy if value-
  eligible. So fleeing one Mogger to be tagged by another is a
  valid outcome.
- An enemy equal in value to the Mogger: **not mogged**. Strict
  less-than only. A queen next to a queen-Mogger is fine.
- A `Mogged` piece can still be captured normally. Conditions
  don't grant protection.

### Self-targeting

A Mogger does not mog its own controller's pieces (friendly
pieces are exempt). Two opposing Moggers next to each other —
both project mog, neither mogs the other if they're equal value,
both mog the other if value-asymmetric. (E.g., a Knight-class
Mogger and a Pawn-class Mogger adjacent: the Knight-Mogger mogs
the Pawn-Mogger.)

### What "material value" means here

For the comparison to work, the engine must have an authoritative
material value for every piece type. Standard pieces have
canonical values (pawn 1, knight 3, bishop 3, rook 5, queen 9,
king somewhere very large or excluded). Custom pieces need
explicit values:

- Skibidi: 4 (rough)
- Goblin: 4
- Bus: 5 (rook-equivalent)
- Monkey: 3
- Locomotive / Carriage: neutral pieces, value 0 — definitely
  weaker than anything (always mogged? yes; but neutrals
  aren't enemies — see below)
- Mewing base: 1 (pawn-tier); Mewing locked: 5
- Costco Guy: 2
- NPC: 2
- Italian Brainrot: 4
- Gooner: 4
- Sigma: 5 + G (variable — see open questions)

Values are stored in a table consulted at runtime. The Mogger
mechanic doesn't *define* values; it just *queries* them.

### Neutral pieces

A `Color::Neutral` piece (like a Locomotive) is not an enemy of
either side. The Mogger does not mog neutrals. Neutrals are
exempt from the value comparison.

### Mogged + Frozen interactions

A piece that is both `Frozen` and `Mogged` is doubly stuck for
its next turn. The conditions don't compound — they're
parallel inhibitors. Both expire after one turn.

### Capture interactions

When a Mogger is captured, all `Mogged` conditions it placed
remain in effect for their full duration (one turn). The
condition is on the square, not the Mogger. New conditions
won't be applied without the Mogger present, but existing ones
finish their cycle.

## Why it's interesting

1. **Pure positional pressure.** The Mogger doesn't move on its
   own, doesn't have aggressive forks, doesn't capture
   unusually. Its value is entirely in its *presence*. Few
   chess pieces are pure positional pressure plays.
2. **Material asymmetry sensitivity.** The mog radius's
   *effectiveness* depends on what the opponent has nearby. If
   they have only queens and rooks, the Mogger does nothing.
   If they have pawns and knights, the Mogger locks them down.
   This makes the Mogger's value board-dependent in a way most
   pieces aren't.
3. **Promotion dynamics.** Promoting a pawn to queen near a
   Mogger flips the asymmetry — the new queen is no longer
   mogged, but a queen-mogger adjacent might suddenly start
   mogging pieces that were previously equal-value. Promotion
   has new strategic weight.
4. **Trade-based liberation.** Trading off your high-value
   pieces near the Mogger leaves you with weaker remaining
   pieces — *more* mogged. Trading is now a positional
   risk-factor, not a clean exchange.

## Example scenarios

1. **Pawn lockdown.** White Mogger on d4 (value 3). Black pawns
   on c5, d5, e5. End of turn: all three black pawns get
   `Mogged`. Black's next turn — pawns can't move. Black plays
   a different piece. End of black's turn: white Mogger
   re-evaluates, re-mogs the pawns (still adjacent, still
   value < 3). The pawns are perma-frozen until something
   changes.
2. **Trade liberation that backfires.** Black knight (value 3)
   trades with a white piece. Black is now down a knight. A
   new piece (black queen, value 9) shifts to where the knight
   was — adjacent to white's Mogger. Queen value 9 > Mogger
   value 3: not mogged. Trade benefited black despite material
   loss (positional liberation).
3. **Promotion flip.** Black pawn on the 7th, adjacent to white
   Mogger (value 3). Pawn mogged, can't move. Black plays a
   different piece — pushes a different pawn. Eventually,
   black gets the pawn out of the Mogger's radius and
   promotes to queen. The new black queen (value 9) returns
   to the mog radius for tactical reasons. The queen does not
   mog the Mogger (queen is not a Mogger), but the queen is
   safe from being mogged.
4. **Mogger vs Mogger.** White Mogger on d4, black Mogger on
   d5. Both value 3. Neither mogs the other (equal value).
   Both project to their own 8-neighborhoods. Their auras
   overlap on c4, c5, d3, d4 (wait — d4 has white Mogger,
   d5 has black Mogger). On c4 (which has neither Mogger), if
   a black piece sits there, white Mogger projects to it
   (value-eligible). Cleanly handled.

## Where it shines

- **Pawn-rich variants** — pawns are mass-mogged.
- **Mixed-value armies** — sharp positional pressure on the
  weaker pieces while ignoring the stronger ones.
- **Endgame positions with king + minor pieces** — the king
  (typically excluded from value comparison or rated infinity)
  is never mogged. Pawns and any remaining minors are
  vulnerable.

## Where it's awkward

- **Queen-heavy positions** — the Mogger does nothing useful.
- **Material-value calibration sensitivity** — the entire
  piece relies on canonical material values being defined and
  agreed upon for every piece type. Any new custom piece needs
  a value assignment.
- **Adjacent enemy spam** — opponent can repeatedly probe the
  radius with low-value pieces to lock the area. The Mogger's
  controller wants more high-value enemies near it (none get
  mogged), but the opponent strategically avoids placing
  high-value pieces there. Self-defeating in a sense.
- **King value.** If king value is "infinity," kings are never
  mogged. If kings have a finite value, low-value Moggers
  could theoretically mog the king. Probably exclude kings
  explicitly from the value comparison. See open questions.

## Engine dependencies

- **Square conditions** with auto-decay timing. `Frozen` exists;
  `Mogged` is a near-clone.
- **End-of-turn hook** to scan and apply conditions. Already
  present for Skibidi's aura.
- **Material value table** — engine-wide. Probably already
  exists for AI evaluation purposes; expose it as
  `material_value(piece_type) -> u32`.
- **Per-condition expiration tracking** — `Mogged` lasts 1
  turn. The conditions list already supports timed expiry for
  some conditions (presumed).

## New features required

- **`Mogged` square condition.** Same dispatch as `Frozen`,
  applied per-square. Plan stub: extend the conditions enum
  and the move-legality filter.
- **Authoritative material-value table.** Single source of
  truth. Plan stub: define a `material_value(piece_type) ->
  u32` function on a central piece-info module.
- **King value handling.** Either exclude kings from the
  comparison or rate them at `u32::MAX`. Either works.
- **Mogger aura refresh hook.** End-of-turn, scan all Moggers,
  evaluate neighbors, apply conditions. The Skibidi precedent
  applies.

## FEN encoding

Symbol: `MG` for white Mogger, `mg` for black.

No payload — the piece is stateless. All effects are derived
from board state.

```
(P=MG)
(P=mg)
```

`Mogged` conditions on squares follow the existing condition
list syntax — `(C=MOGGED)` or similar, alongside `FROZEN` and
`BRAINROT`. The condition's expiry timer is engine-internal.

## Open questions

- **Exact material value of the Mogger.** Pick a value. 3
  (knight/bishop) mogs only pawns. 5 (rook) mogs minor
  pieces too. 9 (queen) mogs everything except other queens
  and kings. The choice radically affects the piece's power.
  Recommend 3 for a "weak but oppressive" feel; 5 for a more
  dominant version.
- **Sigma's variable value `5 + G`.** Sigmas grow their value
  during the game (via the grindset counter, in a sense — or
  via increased range, which doesn't change material value
  per se). Should the Mogger see a high-G Sigma as
  high-value? If `material_value()` is type-only, no.
  Probably correct.
- **King excluded from comparison entirely?** Yes,
  recommend. Mogging the king would be tactically disruptive
  in unpredictable ways.
- **Should the Mogger immune to being mogged?** A weaker
  enemy Mogger could in principle mog a Mogger if the values
  align (no — value comparison is strict less than, so equal
  values don't mog). Different-value enemy Moggers: the
  weaker one is mogged. Probably fine.
- **Goblin kidnap of a Mogger.** Mog conditions on adjacent
  squares finish their 1-turn duration after the Mogger is
  gone. Fine.
- **Frozen Mogger.** Cannot move. Still projects aura.
  Confirmed by spec — aura is passive, doesn't require the
  Mogger to act.
- **Brainrot'd Mogger.** Brainrot is a separate condition.
  Aura still projects (it's not a move-driven effect).
- **Captured-piece-revival via Goblin** brings the piece back
  to its home square. If a captured piece returns home into a
  Mogger's radius, end of that turn the Mogger may freshly mog
  it. Standard cascade.
