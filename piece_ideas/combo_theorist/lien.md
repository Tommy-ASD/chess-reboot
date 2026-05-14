# Lien

> Every square it captures becomes legally untouchable to the opponent for 3 turns — no piece may move *to* that square, period.

## Inspiration

The deckbuilder problem: "I need to deny **specific squares**, not
just block paths." Walls (Trellis vines, Block tiles) deny squares
permanently and symmetrically. Lien denies squares **asymmetrically
and on a timer**. Only the opponent is locked out. Only for 3 turns.
Only on squares where a capture happened.

This is the legal-fiction version of denial — the square isn't
physically obstructed, it's "owed." Pieces can fly past it, glide
through it as a slider, but they can't *stop* on it. The capturing
side reads the lien; the opposing side reads a wall.

Pure denial card. Lien doesn't out-trade — it makes trades sticky.

## Mechanic

- **Movement:** Like a rook (orthogonal slider), 3-square max range.
  Captures normally.
- **Capture trigger:** On any successful capture by Lien, the
  captured square becomes **liened**:
  - It is marked with `T=LIEN` terrain (or equivalent terrain
    overlay on the existing square type) with a 3-turn countdown.
  - For the next 3 of the opponent's full turns, no piece of the
    opposing color may move ONTO that square.
  - Pieces of the lien-owner's color may move onto it freely.
  - Sliders of either color may *pass through* it (it doesn't
    block paths — only landings).
- **Lien expiry:** Each opponent's turn decrements the countdown. At
  0, the lien is cleared and the square returns to whatever it was.
  (If the underlying square was Standard, it becomes Standard
  again.)
- **Cap:** Maximum 4 active liens per Lien-piece (per color, if
  multiple Liens — see Open Questions). The 5th capture either
  fails to lien, or expires the oldest. Recommend "expires oldest"
  for play feel.
- **Lien on a non-Standard square:** if Lien captures on a Track
  tile, signal square, or other special terrain, the lien overlays
  *on top of* the underlying type. Track is still walkable as Track
  to the owner, but the opponent's train cannot enter. Conflict
  resolution is by AND: a square must satisfy both layers.

## Why it's interesting

**Denial / countdown.** Lien punishes captures *after the fact* — it
turns each kill into a 3-turn no-fly zone. The opponent has to
re-plan around squares that were fine yesterday.

The asymmetry matters: only the opponent is locked out. Lien's owner
can re-occupy the square instantly (and often will, to reinforce).
This makes Lien a positional dagger — the captures don't just remove
material, they reshape the geography.

The 3-turn timer is short enough that the opponent can wait it out,
long enough that "I'll just wait" costs the game.

## Combos and counters

**Combo with Locomotive (rail denial):** Locomotive runs on Track
tiles. The Track has a fixed path; Engineers can rebuild track
mid-game (per plan stub). Lien captures on the rail-adjacent
squares — *not* the track itself, but the buffer squares the
opponent would use to repair, derail, or interact with the train.
For 3 turns, the opponent cannot send a piece to the rail to
disrupt the train. Locomotive cruises uncontested.

Concrete: White Lien captures on e3, f3, g3 across three turns.
Black's track-disrupting bishop has nowhere to park on the
rank-3 buffer. White's Locomotive crosses rank-4 unmolested.

**Combo with Goblin (post-kidnap buffer):** Goblin kidnaps a piece
on, say, e6, and starts the return trip. The square e6 was just
vacated by the kidnap victim — a Lien capture on e6 (by a
following piece, not the Goblin itself) would lien the square,
denying the opponent any chance to re-occupy and intercept the
Goblin from there. Lien is the cleanup crew behind a Goblin raid.

**Combo with Skibidi (denial overlap):** Skibidi's brainrot terrain
silences abilities on the squares it covers. Lien denies the same
squares to opposing pieces. Stacked: a square that's both
brainrotted and liened means the opponent can't even *send a
piece* to clear the brainrot. The brainrot tile sits unanswered
for 3 turns minimum, possibly the rest of the game if Lien
re-captures nearby.

**Counter to Bus (passenger deployment):** Bus drops passengers on
adjacent empty squares as part of its move. A Lien on those
adjacent squares means Bus cannot deploy there — passengers stay
trapped in the Bus. A Lien-walled corridor forces Bus to detour or
keep its passengers stuck.

**Counter-play to Lien (waiting game):** The lien expires in 3
turns. Disciplined opponents will simply route around the lien zone
for one round and reclaim it on turn 4. This makes Lien an
*opportunistic* card — devastating in critical moments, useless in
slow positions where the opponent has alternatives.

The other counter: capture the Lien piece itself. Liens persist
after the Lien dies (already-active liens still count down), but
no new liens are generated. Variant designs might rule "liens
clear on Lien's death" for an even sharper counter. Recommend
liens persist — the death is its own win, you don't get to undo
the captures.

**Hard counter — Echo:** If Lien captures Echo, Lien is compelled
to move in Echo's vector next turn. The compulsion might force
Lien away from its lien-management range. Echo + a fast attacker
behind it: Echo absorbs the Lien's first capture, Lien moves off,
attacker scoops the remaining liens.

## Example scenarios

**Scenario 1: Rail denial.**
White Locomotive on a4, track running east. Black Bishop on c1
wants to attack the train at d4. White Lien on c3 captures the
black knight on d3 (Lxd3). Square d3 is now liened for 3 turns.
Black bishop's plan was Bc1-d2-d3-d4; the d3 step is illegal.
Bishop detours via Bc1-e3 — but Lien now plays Lxe3 (the bishop
was there, capture), creating a *second* lien. Now d3 and e3 are
both locked. Bishop is locked out of rank 3 entirely for 3 turns.

**Scenario 2: Goblin escort.**
Black Goblin captures on f5 and starts kidnap-return. White rook
wants to chase via Rf1-f5 — but black Lien plays Lxf3 capturing
white pawn on f3 the same turn. f3 is liened. White rook can't go
through f3 (well, it can pass through, but can't *stop*; doesn't
matter since rook had nowhere else to go that turn). Goblin
completes escape.

**Scenario 3: Lien overload.**
White Lien on rank 1 has captured 4 black pieces, building 4
active liens on c5, d5, e5, f5. A wall of denial across the
middle. Black plays a 5th piece into striking range; white plays
Lxg5 — but the cap is 4. Cleanest rule: the *oldest* lien (c5)
expires immediately, g5 becomes the new lien. Pattern slides
east as white pushes the line.

## Where it shines

- Trains + tracks: rails are linear and easy to lien-wall.
- Mid-board positional play where 3 turns is enough to win an
  exchange.
- Heavy-capture endgames — every kill becomes a buffer turn.
- Anti-Bus, anti-deploy compositions. Lien snipes the deployment
  squares.

## Where it's awkward

- Fast games. Three turns is too long when each turn is a sprint.
  Lien needs to be the long-game piece.
- When the opponent has many alternative move options. If every
  liened square has 3 equivalent neighbors, the denial is paper.
- The 4-active cap. Lien with no captures recently is a 3-square
  rook — underwhelming on its own.
- Compositions without slow pieces. Liening a square the opponent
  never wanted to enter anyway is wasted output.

## Engine dependencies

- Terrain layer with per-square countdown (the Frozen/Brainrot
  precedent is the model — per-square turn-based decay).
- Move-generation filter: "moving piece's color != lien's owner"
  rejects this destination.
- Per-square FEN payload supporting `LIEN=N,O=W` (owner + countdown).
- Hook in make_move: on Lien's capture, write the lien onto the
  destination square.

## New features required

- **Lien terrain overlay:** `T=LIEN` (or as an additive overlay if
  the engine supports stacked terrain). Carries `N=3` countdown
  and `O=W` owner.
- **Walkability predicate update:** a liened square is non-walkable
  for the opposing color (lands only — slider paths still allowed
  to pass through).
- **Per-piece cap tracking:** Lien's piece payload tracks the IDs
  or squares of its active liens, so the 4-cap is enforceable
  and the oldest-expiry rule is computable.
- **End-of-turn decay:** at the start of each player's turn, all
  liens belonging to the OTHER player decrement by 1 (the
  player-they-bind has one fewer turn locked out).

## FEN encoding

Lien piece:

```
(P=L,A=[c5;d5;e5;f5])
```

Where `A=[...]` lists the squares this Lien has active liens on
(needed to enforce the 4-cap and oldest-expiry rule).

Liened square:

```
(T=LIEN,N=3,O=W)
```

`N` decrements per opposing-turn, `O` is the owner color.

If the underlying square needed to retain its original type
(Track, Switch), the lien is encoded as an additive overlay on the
piece-state side: `(T=TRACK,DIR=E,LIEN=3,LO=W)`. Specifics depend
on the engine's chosen "stacked terrain" representation. Either
form is FEN-clean.

## Open questions

- **Lien on a piece-occupied square.** When Lien captures, the
  captured square becomes empty *and* liened. But what if the
  square already had a lien from another Lien-piece? Refresh the
  countdown to 3? Stack two liens (one per Lien)? Recommend
  refresh — keep it simple.
- **Multi-color liens.** If both players have Lien pieces, liens
  belong to colors. A square could be liened by both — meaning
  neither side can occupy it. Functionally a temporary Block. Rare
  edge but worth defining: yes, dual liens stack as a temporary
  no-go.
- **Sliders passing through.** Recommend liens block landings, not
  paths. This is the cleaner rule and the more flavorful one ("you
  owe this square; you can't stop here"). Alternative: liens block
  slider passage too, which makes them a temporary Block tile.
  Latter is too strong.
- **Capture by other pieces of liened squares.** A friendly piece
  can stop on its own lien. But can the *opponent* capture a
  piece sitting on a liened square (the lien-owner's piece)?
  Recommend yes: the lien prevents the opponent from *moving onto*
  the square, but capture *removes* the obstruction first, then
  lands. Otherwise lien-pieces are unkillable once seated, broken.
- **Train interaction.** A Locomotive belongs to Color::Neutral.
  Can Neutral move onto a liened square? Cleanest: yes — liens
  bind the opposing player's pieces, not Neutral. Trains
  unaffected. This preserves the locomotive's neutrality.
- **Lien cap = 4 vs. unbounded.** 4 is the design knob. Unbounded
  is too strong (Lien with 6 captures locks half the board for 3
  turns each). 4 is the round-number cap. 3 might be tighter.
  Playtest.
