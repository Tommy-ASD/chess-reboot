# Tithe Collector

> Every 3 turns it demands a sacrifice — the opponent removes a pawn-equivalent piece, or Tithe Collector promotes to a queen on the spot.

## Inspiration

The deckbuilder problem: "I want an **alternate win condition**.
Checkmate is one path; this piece is the other." Magic has alternate-
win cards (Approach of the Second Sun, Helix Pinnacle). Slay the
Spire has ribbons that scale into wincons. Inscryption has the
Skull. Tithe Collector is the chess version: it doesn't directly
threaten the king, but it pressures the opponent's *piece economy*
on a fixed schedule.

The fiction: an unsubtle tax collector. Every third turn, it
presents its claim. Pay the tithe or it ascends to power. The
opponent cannot ignore it — there's no "wait out" strategy. The
pressure is **unrelenting and asymmetric.**

This is one of the only pieces in this set that offers a path to
victory **without checkmating**. If you can build a board where
Tithe Collector survives 3+ promotions, you have multiple queens
without ever attacking the king.

## Mechanic

- **Movement:** Bishop-like (diagonal slider). 3-square max range.
  Captures normally.
- **Tithe state:** Carries a `T=N` tithe counter, where N counts
  turns since spawn (or since last tithe). At the start of its
  owner's turn:
  - `T` increments by 1 (so after spawn, T=1, T=2, T=3...).
  - When `T` reaches 3, the **tithe trigger** fires.
- **Tithe trigger:**
  1. The opponent is presented a choice: surrender one pawn-equivalent
     piece, OR allow Tithe Collector to promote.
  2. "Pawn-equivalent" means: any piece with point-value ≤ 1 in the
     standard valuation (pawns; in variants with cheap fairy pieces,
     designer's call — recommend pawns + neutral-spawned vines/sand
     don't count, only owned material).
  3. **If opponent has no pawn-equivalents:** they cannot pay. Tithe
     Collector promotes automatically.
  4. The choice is **opponent-driven**: opponent picks WHICH
     pawn-equivalent dies (if multiple), if any.
  5. After resolution, `T` resets to 0.
- **Promotion:**
  - If the opponent does not pay (refuses, or has nothing to pay
    with), Tithe Collector becomes a queen on its current square.
  - The Collector itself is replaced by a queen of its color.
  - The new queen is a normal queen — no Tithe state, no special
    ability. It can be captured, traded, etc.
- **Limit:** No cap on the number of Collectors a player can
  field. Multiple Collectors trigger their tithes independently
  (each on its own 3-turn cadence from its own spawn).

## Why it's interesting

**Escalation / alternate wincon.** Tithe Collector creates a
**forced-economy** pressure. The opponent must shed material to
prevent promotion. Each tithe paid weakens their position. Each
tithe refused strengthens yours. Both are bad for them.

The opponent's choice — which pawn to lose — is genuine and
agonizing. They cannot pay with a queen or rook; they have to lose
the pawns. Eventually, when pawns run out, every tithe becomes a
free queen.

The wincon is **don't checkmate, just keep promoting**. Multi-queen
boards with Tithe Collector are achievable by turn 18-20 against
unprepared opponents. The opponent loses on material long before
any king attack.

The piece also creates **rhythmic pressure**: every 3 turns the
opponent is on the clock. The synergies cluster around those
trigger turns.

## Combos and counters

**Combo with Skibidi (defender silence on trigger turn):** Tithe
Collector's tithe is paid by a chosen pawn — usually a defended
one. On the trigger turn, the defender often matters. Skibidi
pulses on the defender's square, silencing it for one turn. The
opponent's options narrow: their best paying option is now too
exposed. They either lose the *good* pawn or take the promotion.

Specific: white Tithe Collector ticks to T=3 on turn 12. Black has
two pawns left: b7 (defended by knight on a5) and h7 (undefended).
Black wants to pay h7, lose the marginal one. White's Skibidi
pulses on a5 — knight silenced. Now h7 is fine but b7 is also
"undefended" for the trigger turn. Black still pays h7. ... Hmm,
this combo is weaker than I thought; let me re-examine.

Better combo: Skibidi silences pieces *that would attack the
Tithe Collector during T=3*. Tithe Collector is a soft target;
during the trigger turn the opponent often dumps a piece to kill
it. Skibidi silences the attacker, Collector survives, queen
arrives next turn.

**Combo with Trellis (defender wall):** Tithe Collector needs to
survive 3-turn cycles. Walling off the Collector with vines
(Trellis) makes it nearly unkillable. Wall + 3-turn cycles = 1
queen every 3 turns. By turn 15: 4 queens.

**Combo with Quartermaster (mid-cycle attack):** Tithe Collector
is a 3-square diagonal slider. With Quartermaster's +1 range, it's
a 4-square diagonal — meaning the Collector can reach further
squares to capture during the turns between tithes. Tithes are
the wincon; captures are the secondary value. Quartermaster makes
the Collector a real piece in between.

**Combo with Goblin (kidnap the only paying pawn):** Tithe demands
a pawn. If the opponent has only one pawn left and that pawn is
kidnapped by your Goblin earlier, the opponent literally cannot
pay. Automatic promotion. The Goblin-Tithe Collector pair
manufactures forced-promotion scenarios.

**Counter to Tithe Collector (cheap pawn meta):** The simplest
answer is **have a lot of cheap pawns to spend**. A starting
position with 12 pawns instead of 8 makes the Tithe tax sustainable
for longer. Variants with pawn-spawning pieces (or a chess960-like
"extra pawn row") fully neutralize Collector.

In standard 8-pawn games, Collector burns through 8 pawns in 24
turns. If the game lasts 30+ turns, Collector wins by promotion.
If the game ends in 15 turns by checkmate, Collector contributes
1-2 queens at best.

**Counter — kill it:** Tithe Collector is a 3-square diagonal,
fragile. Knights, queens, Skibidis, and Monkeys can all reach it.
The economic optimal play is often **lose 3-4 pawns to a tithe
chain, then commit a piece-for-Collector trade in turn 12-14**.
This is the dance.

**Hard counter — Plague Doctor:** Collector's promote-or-collect
trigger is its "ability." If Collector is on miasma (Plague Doctor)
when T reaches 3, the trigger is silenced — the tithe doesn't
fire. T... does it still reset? Recommend: T does NOT reset (the
trigger never happened), but doesn't increment further while
silenced. Next turn after miasma clears, T=3 still, trigger fires
then. So Plague Doctor delays but doesn't permanently prevent.
This is the cleanest counter rule.

**Hard counter — Echo:** If Tithe Collector captures Echo, Echo's
compulsion fires. Collector is forced into a specific direction
next turn — meaning it cannot stay on its current square and
might walk into danger. Echo doesn't directly stop the tithe but
disrupts Collector's positioning around it.

**Hard counter — Hourglass:** Hourglass rewinds plies. A 5-sand
Hourglass capture rewinds through a tithe trigger turn — undoing
the promotion (if it happened) and reverting Collector. This is a
brutal counter: the opponent eats a tithe, then sacrifices
Hourglass to undo it. Collector player must defend the Hourglass
from being killed.

## Example scenarios

**Scenario 1: The pawn drain.**
White Tithe Collector spawned turn 1. Turn 4: T=3, white declares
tithe. Black has 8 pawns. Black gives up h7 (their worst pawn).
T resets. Turn 7: T=3, black pays g7. Turn 10: T=3, black pays
f7. Turn 13: T=3, black pays e7 (now sacrificing useful pawns).
Turn 16: T=3, black has 4 pawns. Pays a7. Turn 19: T=3, black
has 3 pawns. Pays b7. Turn 22: T=3, only c7 left. Pays c7.
Turn 25: T=3, NO PAWNS. Collector promotes to queen. White has
14 pieces (no pawns lost), black has 8 (4 pawns + 4 other) +
nothing pawn-equiv left. The queen ends the game.

**Scenario 2: Stop the Collector early.**
White Tithe Collector turn 1, black knight on g8. Turn 3: black
Ng8-f6 attacking the Collector area. Turn 4: T=3, tithe fires
— black pays a pawn. Turn 5: black Nf6 closes in. Turn 6:
black Nxc3 — Collector dead. Black lost 1 pawn, gained 1 piece
exchange. Net: black is up. The Collector earned 1 tithe but
that's it.

**Scenario 3: Trellis wall + Collector.**
White Tithe Collector on b1, white Trellis on a2 growing vines
covering all approaches to b1. Black cannot reach Collector by
turn 8. T=3, T=6 trigger twice → 2 tithes paid. T=9 trigger → 3
tithes. By turn 12, Collector promotes (no pawns left), queen
on b1. Trellis vines persist around the new queen. Black has
been bled white.

## Where it shines

- Long games. The longer it lasts, the more tithes fire.
- Variants where pawn count is fixed or low.
- Compositions with Trellis (wall protection), Quartermaster
  (range buff), Goblin (deny pawns proactively).
- Anti-rush metas. Tithe Collector punishes opponents who try
  to play slow defensively.

## Where it's awkward

- Fast checkmate games. Tithe Collector contributes nothing in
  10-turn games — it just dies.
- Compositions with strong leapers (Monkey, knight) that can
  reach the Collector before T=3.
- Plague Doctor-rich metas — silencing the tithe over and over.
- When the opponent has many cheap pawns. Eight pawns is 24 turns
  of tithes — survivable for many games.

## Engine dependencies

- Per-piece FEN payload `T=N` counter.
- Promotion mechanic (already exists for pawns; generalize to
  any piece type → any other piece type).
- "Piece point-value" registry — the engine must know which pieces
  are "pawn-equivalent." Per-piece designer-defined value.
- Opponent-choice mechanic: when the tithe trigger fires, the
  opponent must respond before the next move. This is a
  blocking choice — the engine pauses, the opponent picks, then
  the game continues. (Or, in async play, the choice is queued
  and resolved at the start of the opponent's next turn — see
  Open Questions.)

## New features required

- **`T` counter on Tithe Collector piece state.** Increments at
  start-of-owner's-turn. Resets to 0 after tithe resolves.
- **Tithe trigger event:** when T reaches 3, pause, present
  choice to opponent.
- **Choice resolution:** opponent selects pawn-equivalent to
  remove. If none available, auto-promote. The choice is
  presented in UI; engine maintains a "pending tithe" state if
  the choice can't be resolved immediately.
- **Promotion mechanic generalized:** Tithe Collector replaces
  itself with a queen on its current square. Existing pawn
  promotion logic generalizes; the piece-type table just needs
  to know Collector's promotion target.
- **Pawn-equivalent registry:** a method `is_pawn_equivalent()` on
  each piece type. Returns true for standard pawns. False for
  everything else by default (custom pieces opt-in or out).

## FEN encoding

Tithe Collector piece:

```
(P=T,T=2)
```

T=2 means "this is the 2nd turn since spawn / last tithe."
Increments to 3 at start of next turn, triggers tithe.

Pending tithe choice (in async play):

```
(P=T,T=3,PENDING=1)
```

`PENDING=1` indicates the trigger fired but resolution is
pending (waiting for opponent's choice). Block normal move
generation until resolved.

After promotion, the FEN just shows a queen at that square:

```
(P=Q)
```

The Collector is gone. The tithe history isn't preserved.

## Open questions

- **Sync vs async choice resolution.** In sync play (live game),
  the opponent makes the choice immediately at trigger. In async
  play (correspondence), the trigger fires, pauses the game until
  the opponent's turn, and the opponent must resolve the choice
  before making their own move. The `PENDING=1` flag handles
  async. Sync just blocks the game flow.
- **Pawn-equivalent definition.** Currently: standard pawns only.
  Should custom cheap pieces (e.g., a hypothetical "Footsoldier"
  fairy piece) count? Recommend designer-defined per piece — each
  piece declares `is_pawn_equivalent()`. Default false.
- **Multiple simultaneous tithes.** Two Tithe Collectors trigger
  on the same turn. Opponent gets two choices. Cleanest: resolve
  in spawn-order (oldest Collector first). Opponent can pay one
  pawn for the first, and either pay a second pawn for the
  second or take a second promotion. No interaction with each
  other.
- **Tithe of a Goblin-kidnapped pawn.** The pawn isn't on the
  board — it's in Goblin's possession. Does it still count as
  pawn-equivalent material the opponent can pay? Recommend no —
  only board-active pawns count. If all your pawns are kidnapped,
  you can't pay. (This is the Goblin-Collector combo.)
- **Tithe Collector capturing its own pay target.** If Collector
  is poised to capture a pawn, and that capture triggers the
  tithe (because T was already 3 from the previous turn — no,
  T increments at start of turn, not on capture), then the
  captured pawn is the natural payment. But it's already
  captured — moot. The interaction doesn't arise.
- **Skibidi pulse on Collector during T=3.** Skibidi silences the
  Collector's tithe trigger (per Plague Doctor mechanic, if
  Brainrot terrain functions similarly). Recommend: Brainrot
  silences abilities the same way miasma does. Collector on
  brainrot misses tithe. T stays at 3, fires next clean turn.
- **Tithe Collector promotion target.** Queen is the default.
  Variant designers might allow Collector to promote into other
  pieces (rook + bishop, amazon, hyper-queen). For v1, hard-code
  queen.
- **Promotion overruns king-mate.** If the opponent has only their
  king and no pawns left and the Collector promotes... the
  promoted queen is right there. If the queen checks, fine —
  normal mate flow. If not, the game continues. Collector wins
  by attrition rather than direct check. This is the alt-wincon
  path: the game ends when one player runs out of meaningful
  material to lose.
