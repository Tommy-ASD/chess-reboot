# Plague Doctor

> Every square it captures on becomes 4-turn miasma terrain; pieces ending their turn on miasma lose their special ability the next turn.

## Inspiration

The deckbuilder problem: "The enemy deck is full of ability-stacked
threats — Skibidi pulses, Bus deployments, Monkey chains, Goblin
kidnaps. I need a **silence card**." This is the hard counter slot in
every deckbuilder. Magic has Pithing Needle. Inscryption has Hodag.
Slay the Spire has Weakened. Plague Doctor is the silence in chess.

The fiction: the doctor leaves diseased ground in their wake. Pieces
that linger on infected squares lose their faculties — kingside
bishop becomes a normal bishop, Skibidi becomes a regular slider,
Bus becomes an over-sized rook. Standard chess piece moves remain,
because the *body* is unaffected. Only the *ability* is silenced.

This is the answer card the combo-heavy meta needs. Without Plague
Doctor, ability-stacked builds dominate. With Plague Doctor, the
opponent has to actually move.

## Mechanic

- **Movement:** Standard king's-move (one square in any direction).
  No special leap or slide. Captures normally.
- **Capture trigger:** On any successful capture by Plague Doctor,
  the captured square becomes **miasma** terrain:
  - Layered with `T=MIASMA` (overlay on existing square type, like
    Lien).
  - `N=4` countdown. Each turn (either side's) decrements it.
  - At `N=0`, miasma clears; the underlying square reverts.
- **Miasma effect on pieces:**
  - When *any* piece (either color, including Neutral) ENDS its
    turn on a miasma square, that piece gains a `Q=1`
    (quieted/silenced) flag at end-of-turn.
  - On the piece's NEXT turn, while `Q=1`, the piece's
    "special ability" is suppressed. The piece can still make all
    its standard chess moves; only the unique ability is muted.
  - `Q` decrements to 0 at the end of the silenced turn.
- **What "special ability" means per piece** (the silence
  manifest — must be documented per piece):
  - **Skibidi:** cannot pulse (brainrot AoE). Can still slide.
  - **Bus:** cannot pick up or drop passengers. Can still rook-slide.
  - **Monkey:** cannot jump-chain. Can still take a single jump or
    standard move.
  - **Goblin:** cannot kidnap (the capture works but the piece
    isn't carried home; or alternatively, can't initiate the
    home-return move). Pick one — recommend: cannot capture-and-
    carry. The capture happens, the kidnap doesn't trigger.
  - **Locomotive / Carriage:** cannot move along Track. Effectively
    paralyzed for the turn. (See Open Questions on whether trains
    should be silenceable.)
  - **Echo:** N/A; Echo's "ability" is its on-capture trigger
    on the attacker. Plague Doctor capturing Echo leaves miasma,
    but the compulsion still fires on Plague Doctor. (Plague
    Doctor doesn't have a "special ability" the compulsion would
    suppress — the miasma is on the *square*, not on Plague
    Doctor.)
  - **Quartermaster:** cannot grant buffs. (Plague Doctor's
    canonical hard counter.)
  - **Beacon:** cannot be swap-targeted next turn. (Beacon-swap is
    initiated by the friendly piece, not Beacon — so technically
    the miasma'd piece is the one silenced. But if Beacon itself
    is miasma'd, swaps targeting it are also forbidden.)
  - **Hourglass:** cannot increment sand that turn.
  - **Tithe Collector:** cannot demand tithe / cannot promote.
  - **Lien:** lien-on-capture trigger suppressed. Captures normally
    but doesn't create the lien.
  - **Plague Doctor:** captures normally but doesn't lay miasma.
    Yes, a Plague Doctor can be silenced by another miasma square.
  - **Trellis:** cannot grow a vine that turn.
- **Standard chess pieces (R/N/B/Q/K/P)** have no "special ability"
  to silence. Miasma is inert on them (they still trigger the
  `Q=1` flag but no behavior changes).

## Why it's interesting

**Denial / silencer.** Plague Doctor is the answer card to the
ability-stacked builds that make Chess 2 distinct. Without it, the
opponent can pile abilities (Skibidi + Bus + Goblin + Monkey) and
overwhelm you with multi-trigger combos. With Plague Doctor, the
miasma trail forces the opponent to dodge — and abandoning a
critical square to escape miasma is its own positional cost.

The piece's mechanic is *passive* in the sense that miasma persists.
You don't have to act every turn; you just lay miasma in the
opponent's traffic lanes and they bleed tempo.

It's also a *meta-aware* design choice. In a tournament with
mostly-vanilla decks, Plague Doctor is mediocre. In a tournament
with ability-stacked builds, Plague Doctor is essential.

## Combos and counters

**Hard counter to Skibidi (silence the pulse):** Skibidi's wincon
is the wide AoE pulse. Plague Doctor captures a piece on a square
Skibidi will need to pass over. Skibidi ends a turn on that
square, `Q=1` next turn, pulse silenced. Skibidi can still slide,
but its identity is gone for one critical turn. Now: chain the
miasma — capture multiple pieces in a row, build a 3-square
miasma corridor, and Skibidi spends 3 turns unable to pulse.

**Hard counter to Bus (silence the carrier):** Bus loaded with 4
passengers approaches the back rank. Plague Doctor captures a
piece on the unload square. Bus arrives, ends turn on miasma,
`Q=1`. Bus cannot deploy passengers next turn. Passengers remain
trapped, the opponent gets a free tempo to set up defenders. If
miasma chains: passengers might never deploy.

**Hard counter to Monkey (chain-breaker):** Monkey's jump-chain is
the wincon. Plague Doctor captures a piece on Monkey's expected
landing pad. Monkey lands on miasma, ends turn, `Q=1`. Next turn
Monkey can take a single jump but cannot chain. Net: Monkey is a
slow knight for one turn. In a Monkey-rush opening, this single
turn is often the game.

**Hard counter to Goblin (kidnap-break):** Plague Doctor lays
miasma on a tile Goblin must traverse during return. Goblin lands
on miasma carrying a hostage, ends turn, `Q=1`. Goblin's kidnap
ability is silenced — recommend interpretation: the hostage is
released onto a nearby empty square, and Goblin can no longer
carry until cleared of miasma. Effective ransom.

**Combo with Lien (denial overlap):** Plague Doctor and Lien both
lay terrain layers from captures. Plague Doctor captures, miasma;
nearby Lien captures, lien on its square. Now the opponent
navigates a board with both temporary walls (lien) and ability
suppression (miasma). The two layers stack mechanically — a
square can be both miasma'd and liened. This is the "answer
card" stack, and arguably the strongest denial archetype.

**Counter to Plague Doctor itself:** The simplest answer is **don't
end your turn on miasma**. Plan moves to land on clean squares.
This works if the opponent has space; on a tight board, it doesn't.

The other counter is **Plague Doctor's own range**. King-step
movement is slow. A long-range attacker can pick off Plague Doctor
from outside its range. Bishops, queens, Skibidis-with-clean-squares
all answer it. The piece is fragile if exposed.

**Counter — Echo:** Plague Doctor captures Echo. Echo's
compulsion fires on Plague Doctor. Plague Doctor is forced to
repeat Echo's vector. Next turn Plague Doctor is locked into a
move that may not capture — meaning no new miasma. Echo is a
one-turn silence on the silencer. Trade favorable for Echo's
side.

**Counter — Hourglass:** Plague Doctor's miasma counts down per
turn. Hourglass rewinds plies. A 4-sand Hourglass capture rewinds
through miasma's countdown — but the miasma itself is restored
(it was on the board 4 plies ago). Actually, if the miasma was
*laid* 2 plies ago, the rewind un-lays it. So Hourglass can clear
miasma by sacrificing itself. Specific edge: this only works if
the Plague Doctor's capture is within the rewind window.

## Example scenarios

**Scenario 1: Skibidi silenced.**
Black Skibidi on e5, planning a phase-3 pulse on white's queen at
e2. White Plague Doctor on f6 captures black pawn on f5 (Pf6xf5).
f5 is miasma'd, `N=4`. Skibidi must cross or stand near f5 to
reach pulse position. Skibidi ends turn on f5 in transit, `Q=1`.
Next turn Skibidi can move but cannot pulse. White's queen lives
another turn, white re-positions, the attack dies.

**Scenario 2: Quartermaster lockout.**
Black has a Quartermaster + Bus combo set up. Quartermaster is on
c2, Bus on b1, buff ready. White Plague Doctor captures the
adjacent black piece on c3 (now miasma on c3). Quartermaster
remains on c2 but is *adjacent* to miasma — not directly on it.
Plague Doctor moves to c2 next turn? No, only captures lay miasma.
White must capture a black piece *on* c2 to silence Quartermaster
directly. The miasma trail is a real positional knife — Plague
Doctor has to land the kill on the right square.

**Scenario 3: Self-silence accident.**
Black Plague Doctor and black Skibidi sharing tight space. Plague
Doctor captures a white pawn on f4. Miasma laid. Black Skibidi
needs to traverse f4 next turn. Skibidi ends on f4, gets `Q=1`,
loses pulse next turn. Friendly fire. Plague Doctor positioning
matters for *both* sides.

## Where it shines

- Ability-heavy enemy decks. Plague Doctor is the answer slot.
- Tight-board positions where miasma trails cover key squares.
- Endgames where the opponent has one big ability piece left —
  Plague Doctor silences it permanently with a chain capture.
- Compositions with Lien for stacked denial.

## Where it's awkward

- Vanilla openings. Against R/N/B/Q/K/P only, miasma is inert.
  Plague Doctor is a weak king-step piece in those games.
- Self-block. A Plague Doctor with allied special-ability pieces
  in tight quarters creates friendly-fire risk.
- Slow tempo. Plague Doctor needs captures to lay miasma. Without
  targets, it's just a king-step.
- Against fast attackers. Plague Doctor's king-step range can't
  catch a Bus or a Skibidi on the run.

## Engine dependencies

- Terrain overlay system (the precedent: Frozen, Brainrot, Lien).
  Miasma is another instance of "per-square countdown terrain
  with an effect."
- Per-piece `Q=N` flag (one turn of silence). Decrements at end
  of the piece's turn.
- Per-piece "ability" registry — the engine must know how to map
  `Q=1` to "this piece's special move is suppressed this turn."
  This is a per-piece-type table in movegen.
- End-of-turn hook: any piece ending its turn on a miasma square
  gains `Q=1` if not already set.

## New features required

- **Miasma terrain overlay:** `T=MIASMA` with `N=4` countdown.
  Same shape as Lien. Per-square FEN payload.
- **Per-piece `Q` flag:** `Q=1` silences for one upcoming turn.
  Cleared at end of the silenced turn.
- **Ability-suppression dispatch:** in movegen, before generating
  moves for a piece, check `Q`; if `Q=1`, skip the ability-move
  branches and emit only standard-chess moves. Per-piece
  implementation — Bus's movegen skips passenger ops, Skibidi's
  movegen skips pulses, etc.
- **Capture-on-miasma trigger:** when a piece's move ends on a
  miasma square (capture or not), set `Q=1` at end-of-turn.

## FEN encoding

Plague Doctor piece:

```
(P=D)
```

(No persistent state on the Doctor itself.)

Miasma square:

```
(T=MIASMA,N=4)
```

Per-piece silence:

```
(P=S,Q=1)
```

A Skibidi silenced for one turn. Cleared after the next move.

Stacked terrain example (lien + miasma on the same square):

```
(T=LIEN,N=2,O=W,M_N=3)
```

Where `M_N=3` is miasma countdown overlaid on a Lien tile.
Specifics depend on the engine's terrain stacking strategy. If the
engine supports multiple `T=` entries, write them as comma-
separated overlays.

## Open questions

- **Does miasma silence on entry or on dwell?** Two camps:
  - **Entry:** stepping onto miasma triggers `Q=1` immediately,
    next turn silenced. Aggressive; punishes a single step.
  - **Dwell:** ending the turn on miasma triggers `Q=1`.
    Step-and-leave is safe. Less aggressive; rewards quick
    traversal.
  Recommend **dwell**. Encourages clever movement (skim across
  miasma) and makes Plague Doctor a positional rather than tactical
  threat. The mechanic above is written for dwell.
- **Stack two Q flags?** A piece dwells on miasma turn 1, silenced
  turn 2. If it dwells on miasma *again* turn 2 (which it can't
  do if it just moved — but a Frozen piece can't move and might
  remain on miasma), does silence extend? Recommend: yes,
  re-trigger refreshes `Q=1`. The flag is "next turn silenced,"
  not a global counter.
- **Standard pieces with no ability.** Should miasma do *anything*
  to a rook or knight? Currently: nothing. Recommend leaving it
  inert — adds no design value to silence "pawn's diagonal
  capture" or "knight's L-shape." Simpler this way.
- **Trains and Neutral color.** Should miasma silence Locomotive
  movement (preventing it from moving along Track that turn)?
  This is a strong counter-to-trains play, similar to Hourglass.
  Recommend yes — trains are silenceable. Track movement IS the
  train's special ability. A miasma'd train sits still that turn.
- **Plague Doctor silencing itself.** A Plague Doctor on miasma
  is silenced — its "lay miasma on capture" trigger is
  suppressed. Cool emergent property. No new rules.
- **Promotion through miasma.** A pawn promoting on a miasma
  square: the promoted piece is born `Q=1`? Probably yes, since
  it ends its turn on miasma. Consistent.
- **Multi-Plague Doctor stacking.** Two Plague Doctors capturing
  in sequence lay miasma on two squares. Easy. No new rules.
