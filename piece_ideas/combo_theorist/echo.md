# Echo

> A piece whose only job is to be captured — the capturing piece is then locked into replaying Echo's incoming move-vector on its very next turn.

## Inspiration

The deckbuilder problem: "I have a high-tempo board control piece
(Skibidi, Bus) and I need to predict where the enemy will be next
turn." Most fairy pieces solve this by gaining ranged threats. Echo
solves it the inverse way — it **forces the opponent to telegraph**.

The fiction: Echo carries a resonance. The piece that strikes it
becomes possessed by its motion, repeating the last vector before
breaking free. The capturing player still chooses *which* piece does
the capture, but once committed, the *next* move is dictated.

This is a sacrifice piece in the Magic sense: a 1-cost, 1/1 with a
death trigger. The body is irrelevant. The trigger is the whole point.

## Mechanic

- **Movement:** Pawn-like, one square per turn in any of the 8
  directions. Cannot capture (this is critical — Echo never initiates
  combat, only receives it).
- **Capture:** Captured normally. On capture, the **attacking piece**
  receives a `compulsion` flag with two stored values:
  - `dir`: the direction vector of the capturing move (file/rank
    delta).
  - `dist`: the distance traveled to reach Echo.
- **Compulsion resolution:** On the captor's next turn, its legal
  move set is restricted to **exactly one move**: continue in `dir`
  for `dist` squares. If that destination is unreachable (off-board,
  blocked by piece of either color, non-walkable terrain), the
  captor's compulsion is consumed without moving — the piece passes
  the turn. The compulsion flag clears either way.
- **No compulsion stacking.** If the captor is compelled and then
  captures another Echo on its forced move, the new compulsion
  *replaces* the old one's data (which was already consumed by the
  forced move itself). This keeps the rule deterministic.
- **Color::Neutral interaction.** Echo can be played by either color
  but not by Neutral. Neutral trains capturing Echo are also
  compelled — yes, this can wreck your own train. That's intentional.

## Why it's interesting

**Tempo + sacrifice trigger.** Echo's value is entirely in the
trigger. The piece itself sits at 0 raw threat. But the trigger
converts an enemy capture into a predictable enemy move — and
predictability against a high-power piece is gold.

The compulsion is **direction + distance**, not destination. This
matters because the captor might pass through your prepared
brainrot square en route, or end on a square you've baited.

## Combos and counters

**Combo with Skibidi (brainrot trap):** Skibidi parked on e4. Echo
parked on h7. Opponent's queen captures Echo via Qxh7 (a long
diagonal capture, dir=NE, dist=large). The queen is now compelled
to step NE one square at a time — no wait, dist matters: the queen
captured at *some* distance, and must replay *that* distance. Say
Qd3-h7 (NE, dist=4). Next turn the queen *must* play h7-? in the NE
direction for distance 4, which is off-board. The queen passes.
Free tempo, queen wasted two moves to nuke Echo and stand still.

Better: bait the queen onto a *finite* compelled square. Echo on f5,
queen on b1. Qxf5 is `dir=NE, dist=4`. Next turn the queen *must*
play f5 → j9, also off-board. Hmm — long-distance captures often
result in off-board compulsion, which makes them tempo wins for
the Echo player. Short captures are the dangerous ones.

The real Skibidi combo: Echo on c6, Skibidi on f6. A black knight
captures Echo via Nbxc6 (`dir=NW`, `dist=1` — a knight's L-move
encodes as a single non-axial vector). Wait — knights are tricky.
Knight moves are 2:1 or 1:2 offsets. The `dir` vector for a knight
capture is the literal (df,dr) tuple, e.g., (-1, +2). The compelled
move next turn is "from current square, offset by (-1, +2) again."
If that lands the knight on f8 (a square Skibidi can brainrot from
f6 via one of its 4 phases), Skibidi pulses and the knight is
silenced precisely where it was forced.

The key Skibidi-Echo combo is therefore: **place Echo such that the
attacker's compelled landing is inside Skibidi's pulse cone.**

**Combo with Bus (forced detour):** A Bus with passengers is
threatened by an enemy rook on the same file. Park Echo one square
between Bus and rook. Rook *must* capture (or lose Bus next turn).
Rxe4 captures Echo, compulsion = (N, dist=2). Rook's next move is
forced to e6 — past the Bus, not into it. The Bus survives a turn
because the rook telegraphed its own retreat. Net: Echo bought one
critical turn for the Bus to unload.

**Counter to Goblin (kidnap derailment):** Goblin kidnaps and runs
home. If a Goblin captures Echo while carrying a hostage, the
Goblin is compelled to continue in the kidnap direction — meaning
it cannot pivot to its home square. Either the Goblin gets stuck
mid-board with the hostage, or the kidnap timer runs out. Hard
counter to the Goblin-rush opening.

**Counter-play to Echo:** Don't capture it. Echo can't capture, so
it's a sitting duck that *only matters if you take the bait*. The
disciplined response is to develop around it. Unfortunately, Echo
is cheap to deploy in critical squares — by leaving it alive, you
let it sit on a square you wanted. The opponent must decide: cede
the square or telegraph the next move?

The other answer: capture Echo with a piece you don't care about,
or with a piece whose compelled move happens to land somewhere
fine. A pawn capture is often a clean way to disarm Echo — pawn
diagonal capture gives `dir=NW or NE, dist=1`, and the pawn's
compelled move is one square in that direction. Pawns can usually
afford to just advance.

## Example scenarios

**Scenario 1: The off-board sacrifice.**
White Echo on g7, black queen on c3. Black plays Qxg7 — but Qc3-g7
is `dir=NE, dist=4`, and the next-turn forced move puts the queen
at k11. Off-board. The queen's turn is wasted entirely. White
plays freely while black's queen recovers next turn.

**Scenario 2: Skibidi pulse trap.**
Black Echo on c4, white Skibidi on e3 (phase aligned to capture e4
square). White bishop on a2 plays Bxc4 — `dir=NE, dist=2`. Bishop
is compelled to play c4 → e6 next turn. But on the intermediate
turn (Skibidi's), Skibidi pulses and the e6 square is brainrotted.
Bishop lands on e6 and immediately loses its bishop-ness for one
turn. Trade: black gave up Echo + e3-square access, got a
silenced bishop on the wrong rank.

**Scenario 3: Goblin trap.**
White Goblin captures black pawn on e5, becomes a kidnapper
heading back to white's first rank. Goblin's natural path: e5-d4
or e5-e4 backward. Black drops Echo on d4. Goblin captures Echo
via Gxd4 (`dir=SW, dist=1`). Next turn Goblin is forced to d4-c3.
This is **not** the way back to white's home rank — it's a sideways
detour. Goblin spends an extra turn correcting, hostage timer ticks
down, the kidnap fails.

## Where it shines

- Slow positional play where each piece's next move can be planned
  around.
- Builds heavy on Skibidi, brainrot terrain, or any "this square is
  dangerous next turn" mechanic.
- Anti-Goblin metas. Echo is one of the cleanest single-piece
  answers to a kidnap rush.
- Variants with strong centralizing pieces (queens, amazons) — the
  more powerful the captor, the more it hurts to telegraph it.

## Where it's awkward

- Open positions with many pawns. Pawns capture Echo cheaply and
  their compelled move is usually fine.
- When the opponent has multiple attackers. Capture-with-rook,
  compelled-move-shoves-rook-forward isn't a real punishment if the
  rook *wanted* to advance anyway.
- Echo is a tempo investment that pays off later. If you're losing
  on material, Echo is a luxury you can't afford.
- Multi-piece chain captures (Monkey jump-chain): does Monkey get
  compelled on the *last* leg of its chain? See Open Questions —
  this is the messiest rule edge.

## Engine dependencies

- Move-vector capture metadata. The engine already records the
  source and destination of every move; the (dir, dist) decomposition
  is mechanical from that.
- Per-piece state payload (FEN-serializable). Echo doesn't need
  state, but the *captor* of an Echo gains a transient flag — the
  flag rides on the captor's piece payload.
- Move-generation filter that, when the captor is the side to move,
  restricts move set to the single compelled move.

## New features required

- **Compulsion flag** on piece state: `C=(df,dr,n)` where `df,dr`
  is the direction vector and `n` is the distance. Cleared
  immediately after the compelled move resolves (or pass).
- **Movegen short-circuit:** if a piece has a compulsion flag,
  its only legal move is the encoded one. All other moves filtered.
- **Capture-trigger hook** for Echo specifically: when Echo is
  captured, write the compulsion onto the attacker.
- **Monkey chain handling:** decide whether Echo capture mid-chain
  ends the chain immediately (clean) or compels the *next* chain
  start (messy). Recommend clean — Echo capture in a chain ends
  the chain and writes compulsion for next turn.

## FEN encoding

Echo piece (no state of its own):

```
(P=E)
```

Compulsion flag on a piece that captured Echo:

```
(P=Q,C=(1,2,3))
```

Read as: queen, compelled to move (df=1, dr=2, dist=3) on its
next move. (df=1 is "+1 file" i.e. one column right; dr=2 is "+2
ranks" i.e. two rows up.) Cleared after that move resolves.

For knight-like vectors (non-axial), df and dr both nonzero with
dist=1 reproduces the L-shape. A compelled knight encoding
(df=-1, dr=2, dist=1) means "from current square, go to (file-1,
rank+2)."

## Open questions

- **Compulsion + check.** If the captor's forced move would leave
  the captor's own king in check (illegal under standard rules),
  does the compulsion override or yield? Two camps:
  - **Override (chaos):** the king is just exposed. Echo is a
    devastating sacrifice.
  - **Yield (sanity):** the compulsion is consumed without moving,
    the king stays safe. Echo is a tempo win but not a checkmate
    machine.
  Recommend yield — keeps Echo strong but not broken.
- **Multi-leg captures (Monkey).** Strongest open question.
  Proposed: Echo capture in a Monkey chain ends the chain
  immediately, and the Monkey is compelled with the vector of the
  last leg only. Alternative: the compulsion uses the *cumulative*
  net vector of the chain — harder to reason about, but feels right
  for the "Echo absorbs your motion" fiction.
- **En-passant-like captures.** If an Echo is captured by a Bus
  trampling it as part of a multi-square move, what's the dir/dist?
  The Bus's full move, presumably — meaning the Bus is compelled to
  *repeat its full slide* next turn. Probably devastating to the
  Bus. Probably also fair.
- **Echo capturing Echo.** Echo can't capture (rule). But what if a
  variant lets it? The compulsion would write on... itself, then
  resolve immediately. Probably fine. The rule that Echo can't
  capture is the cleaner answer.
- **Two Echoes captured in one turn.** Can't happen normally —
  one move, one capture. Monkey chain edge case: Monkey jumps over
  two Echoes in one chain. Treat as: each captured Echo *would*
  write a compulsion, but only the last one survives (overwrite).
  This is the simplest rule and matches the chain semantics.
