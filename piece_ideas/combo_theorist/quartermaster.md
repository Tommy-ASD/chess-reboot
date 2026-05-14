# Quartermaster

> Skips its own turn to lend +1 range to one adjacent friendly piece — an amplifier, not a fighter.

## Inspiration

The deckbuilder problem: "I have a strong attacker (Goblin, Skibidi,
Bus) whose range is one square short of what I need." Most fairy
pieces solve this by being more attackers. Quartermaster solves it by
being a **support card** — the relic that buffs your relics.

In Slay the Spire terms: this is a "+1 strength" power. In Magic: a
mana rock. The piece's individual contribution is unimpressive, but
it makes the rest of the deck function. Without supports, every piece
must self-justify; with supports, the design space opens up to pieces
that almost-work-but-not-quite.

## Mechanic

- **Movement on Quartermaster's own turn:** Quartermaster does **not**
  move under its own power. Its turn is spent issuing a buff.
- **Buff issuance (alternative to a passive turn):** at the start of
  Quartermaster's turn, the player may designate **one orthogonally
  or diagonally adjacent friendly piece**. That piece gains a
  `R=+1` token (extra range) **that the same player consumes on
  their next turn for that piece**.
- **Buff consumption:** when the buffed piece moves on a subsequent
  turn, its range is extended by 1 for that move only. Specifics:
  - For sliders (rook, bishop, queen, Bus, Skibidi-slide), max
    slide distance +1.
  - For leapers (knight, Monkey hop, Goblin leap), the leap vector
    is scaled by +1 in the dominant direction (knight goes from
    1-2 to 1-3 or 2-3, designer's call — recommend 2-3).
  - For step pieces (king, pawn), one extra step in the chosen
    direction.
- **Buff lifetime:** persists until the buffed piece moves *or* until
  Quartermaster moves a buff to another piece *or* until the buffed
  piece is captured. If Quartermaster skips its turn explicitly (no
  buff target), no change.
- **Single active buff per Quartermaster.** Buff tokens stack across
  multiple Quartermasters: two Quartermasters each adjacent to the
  same Bus = `R=+2`.

## Why it's interesting

**Tempo amplifier.** Quartermaster trades its own move for a *better*
move from a piece that matters more. This is asymmetric tempo: you
spend 1 ply to convert your strongest unit's next move into a
super-move.

The piece is also a positional anchor — it must be adjacent to its
target, so the opponent can break the combo by harassing the
Quartermaster itself. This makes it a juicy target despite being
mechanically passive.

## Combos and counters

**Combo with Goblin (deep kidnap):** Goblin's strength is grabbing a
piece and running. The depth of the kidnap is limited by Goblin's
leap range. A Quartermaster shadowing a Goblin extends each leap
by one square — meaning the Goblin can reach kidnap targets one
rank deeper than the opponent expects.

Sequence: Quartermaster on c2, Goblin on b1. Black piece on f6.
Normally Goblin's leap range maxes at 4 squares; b1-f6 is dist=4
file + dist=5 rank = unreachable. Quartermaster buffs Goblin (`R=+1`).
Now Goblin leaps b1 → f5 (dist=4+5 via the diagonal allowance, see
Open Questions on how +1 scales 2D leaps), captures, and starts
kidnap return. Without the buff, that piece was safe.

**Combo with Skibidi (extended brainrot pulse):** Skibidi's pulse
radius depends on its current phase. A buffed Skibidi gets +1
radius for one pulse — wide enough to silence two threats at once.
Quartermaster sits next to Skibidi for the whole game, eating
attacks for it, just to enable the one critical wide-pulse turn.

**Combo with Bus (passenger-extending caravan):** Bus moves like a
rook, carrying passengers. A buffed Bus moves one extra square,
crossing the half-board in a single turn instead of two. The Bus
+ Quartermaster pair is a deployment machine: load passengers,
buff, fire across the board, unload. Repeat next round.

The double-Quartermaster Bus is the wincon. Two Quartermasters
adjacent to a Bus means `R=+2` — Bus crosses an entire 8-rank board
in one move. Drop 5 passengers at the back rank turn 3. This is
the deckbuilder "infinite combo" archetype.

**Counter to Quartermaster (assassination meta):** Quartermaster has
zero defensive capability. It doesn't move, doesn't attack. The
counter is "kill it before the combo fires." A Knight-rush or
Skibidi-pulse aimed at Quartermaster's parking square neutralizes
the build. Players running Quartermaster must invest in defenders
or hide it behind walls (see Trellis synergy).

The other counter is **positioning denial**: keep your high-value
pieces away from any Quartermaster radius. If the Bus never gets
adjacent to its support, the support is dead weight.

**Counter-combo with Plague Doctor:** Plague Doctor's miasma
silences abilities. A Quartermaster on miasma loses its ability to
buff next turn. One miasma tile, two Quartermasters silenced if
they share an adjacent square. A Plague Doctor running through a
buffed Bus build can dismantle it in 3 turns.

## Example scenarios

**Scenario 1: Goblin reaches the queen.**
White Goblin on c1, white Quartermaster on c2, black queen on d6.
Turn 1: Quartermaster buffs Goblin. Turn 3 (after black plays):
Goblin leaps c1 → d6 (dist=1+5, normally too far, with `R=+1`
just reachable). Capture, queen kidnapped, Goblin starts return
trip. Trade: one Quartermaster pinned to c2 forever, one queen
extracted from the game.

**Scenario 2: Cross-board Bus.**
White Bus on a1 with 4 passengers, two Quartermasters on a2 and b1.
Both buff Bus on turn 1. Bus has `R=+2`. Bus moves a1 → a10 in one
move (assuming a wider board variant). All four passengers
deploy on the far rank turn 1. The two Quartermasters are now
stuck on a2 and b1, but the game is effectively over.

**Scenario 3: Skibidi wide pulse.**
Black Skibidi on e5 mid-phase, threatening to pulse on e4 (range
2). Black Quartermaster on f5 buffs Skibidi. Next turn Skibidi
pulses with `R=+1` — range 3 instead of 2. The pulse catches
white's Bus on e2 *and* white's Goblin on f2 in one stroke. Two
key pieces silenced for one tempo.

## Where it shines

- Builds with a single dominant attacker that needs one more move.
- Variants with wide boards (8x10, 10x10) where +1 range is the
  difference between reach and out-of-reach.
- Trellis-walled openings where Quartermaster is safe behind vines.
- Skibidi-centric decks where the wide pulse is the wincon.

## Where it's awkward

- Fast all-out positions where pieces never sit adjacent long enough
  to build buff chains.
- When the opponent has a Knight or Monkey that can leap onto
  Quartermaster's square directly — passive support pieces hate
  leapers.
- Variants with no slide-range-relevant pieces. Quartermaster
  buffing a king's +1 step is rarely the wincon you wanted.
- Buffing the wrong piece. Quartermaster's value is unrealized
  if the chosen target doesn't move next turn (king in check,
  Frozen piece). Buff allocation is a real planning burden.

## Engine dependencies

- Move generation gets a per-piece range-modifier input. Many
  pieces already parameterize range; the +1 token is one more
  modifier.
- Per-piece FEN payload for the `R=+N` token.
- Adjacency check at Quartermaster's turn — already a primitive in
  movegen.
- The buff token's lifetime is tied to "next move of the buffed
  piece" — a standard event hook.

## New features required

- **Range-modifier token** on piece state: `R=+N`. Consumed on the
  next move of the carrying piece. Cleared on capture, on
  Quartermaster moving the token, on Quartermaster's death.
- **Quartermaster movegen:** the legal "moves" at its turn are
  "buff adjacent friendly piece X" (one option per adjacent
  friendly) plus "pass." The buff is the move.
- **Range scaling logic per piece:** how +1 applies to knight L's,
  Monkey chains, Skibidi pulses. Per-piece interpretation,
  centralized in a `apply_range_buff(piece, n)` helper.

## FEN encoding

Quartermaster:

```
(P=M)
```

(No persistent state — its buff lives on the *recipient* piece.)

Buffed piece:

```
(P=R,R=+1)
```

Rook with one accumulated range buff. Stacking is additive:

```
(P=R,R=+2)
```

means two Quartermasters' buffs stacked. Cleared after the rook moves.

## Open questions

- **2D scaling for non-axial leapers.** Knight leap is (1,2). Does
  +1 mean (1,3) and (2,3) become legal, or only the "primary"
  axis +1? Recommend: each non-axial leaper defines its own +1
  shape. For knight: (1,3) and (3,1) and (2,3) and (3,2) all
  become legal. This is generous — alternate is "only the strict
  +1 of the standard pattern." Designer call.
- **Buff on the move turn itself.** Can Quartermaster buff a piece
  that has already moved this turn? Standard interpretation: no,
  buff applies to the *next* friendly turn. But variants with
  Bus-driven multi-piece moves (Bus + passenger deployment) need
  clarity — does the unloaded passenger benefit from a buff stored
  on the Bus? Recommend: no, buffs are piece-specific.
- **Buff overwriting.** If Quartermaster buffs Bus on turn 1, then
  buffs Skibidi on turn 3 before the Bus has moved, does the Bus
  lose its buff? Cleanest rule: yes, single active buff per
  Quartermaster. The Bus is unbuffed at turn 3.
- **Buffing a Goblin holding a hostage.** Goblin's leap is its
  return trip. +1 range definitely helps. The kidnapped piece is
  along for the ride. No new rules needed, but worth flagging that
  Quartermaster makes Goblin kidnap chains significantly stronger.
- **Quartermaster + Locomotive.** Locomotives are neutral trains
  that move along Track. Can Quartermaster buff a neutral train?
  Cleanest answer: no — Quartermaster's buff targets *friendly*
  pieces and Neutral isn't friendly to anyone. Avoids the train
  being weaponized by both sides simultaneously.
