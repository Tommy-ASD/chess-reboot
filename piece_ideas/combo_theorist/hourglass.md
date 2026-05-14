# Hourglass

> A piece that accumulates "sand" each turn it survives — when it dies, the board rewinds that many half-moves and the Hourglass leaves.

## Inspiration

The deckbuilder problem: "I want a panic button that punishes my
opponent for clearing my pieces *too efficiently*." Magic has
sacrifice-payoffs ("when X dies, draw two cards"). Slay the Spire has
ribbons that scale with damage taken. Hourglass is the chess version:
the longer it lives, the more devastating its death.

The fiction: a fragile timekeeper. Each turn it remains alive, more
sand falls. Strike it down at the wrong moment and the timeline
itself unwinds. The player who killed Hourglass might unkill their
own most important capture.

This is also a clean **counter** to neutral trains (Locomotive +
Carriage). Trains have to capture pieces in their path — they
*can't* choose not to take. Drop a fat Hourglass on the rails and
the train commits suicide.

## Mechanic

- **Movement:** One square in any direction (king-like). Cannot
  capture.
- **State:** Carries a `S=N` sand counter. Starts at `S=0` on spawn.
- **Per-turn increment:** At the start of *Hourglass's owner's*
  turn, if Hourglass is on the board, `S` increments by 1.
  Hard cap at `S=5`.
- **Capture trigger:** When Hourglass is captured by any piece (of
  any color, including Neutral trains), the rewind activates:
  1. Read `S=N` at the moment of capture.
  2. Undo the last `N` half-moves on the board, **including the
     capturing move itself** (which counts as half-move #1).
  3. Pieces return to their squares. Captures are reversed (captured
     pieces come back).
  4. Hourglass is **removed from the board** (it does not
     resurrect — its sacrifice is permanent).
  5. The turn order resets to whichever side was to move at the
     rewound state.
- **Determinism:** the engine maintains a move-history stack. Rewind
  is deterministic — pop N half-moves, replay none, just unmark.
  All state (signal states, terrain countdowns, piece flags) is
  restored from the snapshot at half-move (current - N).

## Why it's interesting

**Escalation / panic button.** Hourglass starts at zero. By turn 5
it's a nuke. The opponent must decide every turn: "do I kill the
Hourglass now (and lose those plies) or let it grow?"

Critically, **Hourglass doesn't choose when to detonate.** The
opponent does. This makes it a *forced-response* card. The Hourglass
player wants the opponent to commit; the opponent wants to ignore
Hourglass while it grows.

The rewind is also a hard counter to a specific class of plan:
**uninterruptible kill sequences** like trains. A train cannot
refuse to capture. Drop a Hourglass on the rails and the train's
4-tile chase becomes a 4-ply rewind. Train derailed, opposing
captures undone, possibly the train itself reset.

## Combos and counters

**Hard counter to Locomotive (rewind train kills):** Locomotive
runs on Track and captures anything in its path. Park Hourglass
on the next Track tile in front of the Locomotive. The Locomotive
*must* enter (per Plan 09 train semantics — it can't choose to
stop). Hourglass dies, rewind activates with whatever `S` had
accumulated. If `S=4`, the last 4 half-moves unwind: typically
the Locomotive's previous 2 moves and 2 of the captures it made.
Train's victims come back, train is back on its starting tile.

Best case: Hourglass with `S=5`, sacrificed to a Locomotive that
just captured your queen 5 plies ago. The rewind brings the queen
back, removes the train's last 5 moves, and the train is sitting
miles from the action. This is the wincon Hourglass exists for.

**Combo with Goblin (kidnap insurance):** Goblin kidnaps a piece
and runs home. The opponent's queen captures Goblin mid-trip and
recovers the hostage. With Hourglass parked safely behind the
action at `S=4`: opponent must spend a tempo on Hourglass *or*
let the kidnap complete. If they take Hourglass, the kidnap is
reset along with the queen's capture move. Net: Hourglass is the
*insurance policy* on the Goblin attack.

**Combo with Skibidi (rewind a missed phase):** Skibidi's brainrot
pulses are phase-locked. If you mis-time the pulse and burn a
critical turn, Hourglass with `S=3` rewinds back to before the
mis-timing. The opponent has to *commit* to killing Hourglass
to lock the timeline.

**Combo with Lien (rewind cleans the lien-board):** A board with
4 active liens is fragile — Hourglass rewind might undo the
captures that created them. This is bad for Lien-heavy builds.
Conversely, the *opponent* might bait the Hourglass-rewind to
clear lien terrain off the board. The interaction is subtle:
Hourglass and Lien are tempo-opposites.

**Counter-play to Hourglass (the disciplined opponent):** The
only sane response is **kill Hourglass at `S=0` or `S=1`**, on
the first turn it appears. If the Hourglass player has the
opening initiative and deploys Hourglass turn 2, the opponent
must drop everything and assassinate it turn 2. Letting it grow
past `S=3` is a losing line.

This means Hourglass functions as a **tempo tax**: it forces an
early-game move from the opponent. Even if it dies at `S=1`, it
bought one tempo.

**Counter — Plague Doctor:** Hourglass's increment is its
"ability." Plague Doctor's miasma silences abilities. A
miasma'd Hourglass doesn't gain sand that turn. With sustained
miasma coverage, Hourglass is neutralized while still on the
board. Cleanest hard counter Plague Doctor has.

## Example scenarios

**Scenario 1: Train derail.**
Turn 1-5: black Locomotive cruises along rank 4 east, capturing
two white pieces. Meanwhile black Hourglass on f7 reached `S=5`
(it's been alive 5 turns).
Turn 6: White plays Hxf7 with a knight... wait, white wants the
rewind. White instead lets the Locomotive's path drift toward
Hourglass.
Turn 7: Locomotive enters f7 (only legal next tile), captures
Hourglass, rewind `S=5` activates. Last 5 half-moves undone —
Locomotive's last 2 moves and 2 white pieces it captured come
back. White is now up 2 material *and* the train is reset to
rank 4 east end.

(Note: this is black's Hourglass dying to black's own train.
The rewind doesn't care about color — sacrificing Hourglass is
always the owner's choice, and trains are Color::Neutral, so
this only works if black *positioned* Hourglass to die to its
own train deliberately. That's the point.)

**Scenario 2: Goblin insurance.**
White Goblin kidnaps black bishop on d5 at half-move 12. Returns
toward d1. White Hourglass on h1, `S=3`. Half-move 18: black
queen captures white Goblin on d3, recovering bishop. Half-move
19: black knight captures white Hourglass on h1 (just to clear
threats). Rewind `S=3` undoes half-moves 17, 18, 19 — including
the queen capture. Goblin and bishop both alive, kidnap progresses.

**Scenario 3: Skibidi reset.**
Black Skibidi mis-times a pulse on turn 10, wasting it on an
empty square. Black Hourglass on a7, `S=4`. Turn 11: white
captures Hourglass with a rook. Rewind `S=4` undoes turns 9 and
10 — including Skibidi's mis-pulse. Skibidi can now re-pulse next
turn correctly. White spent a rook tempo to *reset black's mistake*,
which is... probably what white wanted to avoid. The lesson: white
should have ignored Hourglass.

## Where it shines

- Train-heavy variants. Hourglass is one of the few clean
  counters to Locomotive's "I must capture" mandate.
- Builds with high-investment slow plans (Goblin kidnap chains,
  Tithe Collector ramps, Skibidi phase setups). Hourglass insures
  the plan against single-turn disruptions.
- Late-game material parity. A 5-sand Hourglass in the endgame
  is functionally a "rewind to a winning state" button.
- Variants with deep move-history (no draws by repetition). The
  rewind needs the half-move stack to be deep enough to matter.

## Where it's awkward

- Move-1 deployment. Hourglass at `S=0` is just a weak king-step
  piece. The early game is its dead zone.
- Compositions where the opponent has many cheap pieces. A pawn
  taking Hourglass at `S=1` is a 1-tempo punishment, almost a
  gift to the opponent.
- Symmetric rewind. The rewind affects *both* sides — if Hourglass
  triggers and you just made a brilliant move, you lose that too.
  Players must plan around their own past plies, not just the
  opponent's.
- Plague Doctor metas. A silenced Hourglass never grows.

## Engine dependencies

- **Move history stack** with deep enough retention for `S=5`
  rewind (so, ≥5 half-moves, but in practice the engine should
  already retain full game history for FEN replay/PGN export).
- Per-piece state snapshot capability — at each half-move, the
  engine must be able to restore the full board state from N
  plies ago. This is heavier than the engine currently supports.
- Per-piece FEN payload for `S=N` counter on Hourglass.

## New features required

- **Half-move snapshot stack.** A bounded ring buffer of the last
  K board states (K=5 minimum, K=10 for safety). Each entry is
  a complete deserializable board state. Rewind pops the top N
  and replaces the live state with the (N+1)th from the top.
- **Rewind trigger hook** in make_move: on Hourglass capture,
  invoke `rewind(S)` *before* applying the capture move's other
  effects (in particular, the captured Hourglass doesn't come
  back — it's the exception to the rewind).
- **Hourglass exception in rewind:** the Hourglass that died is
  excluded from the rewind's piece-restoration. Everything else
  rewinds normally; Hourglass stays gone.
- **Sand increment hook** at start of owner's turn.

## FEN encoding

Hourglass piece:

```
(P=H,S=3)
```

Where `S=3` is current sand count. Defaults to `S=0` on spawn,
capped at `S=5`.

The rewind itself doesn't need FEN encoding — it's an event, not
a state. But the **snapshot stack** must serialize if the engine
saves mid-game. Recommend: snapshot stack is part of the engine
session, not the FEN. FEN saves the current board only; reloading
loses rewind capability but the game continues from the snapshot.

(Alternative: encode the last 5 board states as `(H1=...,H2=...,...)`
in a meta-FEN section. Bulky. Not recommended for v1.)

## Open questions

- **Snapshot scope.** Does the rewind include signal states,
  terrain countdowns (Frozen / Brainrot / Lien)? Recommend yes —
  full board rewind, no exceptions other than Hourglass itself.
  Otherwise the rewind is inconsistent and creates paradoxes
  (a Frozen piece could be un-Frozen by rewind but still bound
  by stale counters).
- **Multiple Hourglasses.** Two Hourglasses on the board, both
  with sand. If one dies, does the other survive the rewind?
  Per the "Hourglass exception" rule, yes — the *dying* Hourglass
  is excluded from rewind. The other Hourglass is rewound back
  to its state N plies ago (which may have been alive with
  different sand). This is consistent but unusual.
- **Stacking rewinds.** If a rewind brings back a *different*
  dead Hourglass with sand on it, can that one then be
  re-killed and re-rewound? Yes — each death is independent.
  Hourglass chains are a real strategy.
- **Maximum sand cap.** `S=5` is the design knob. `S=3` is
  weaker, `S=7` is devastating. Five plies feels like the
  sweet spot — long enough to matter, short enough that the
  opponent can answer the threat.
- **Interaction with Tithe Collector promotion.** Tithe
  Collector promotes to queen on turn 3-or-tithe (see that
  file). If Hourglass rewinds across a Tithe Collector
  promotion, does the queen become a Collector again? Yes —
  rewind restores piece type. This makes Hourglass a
  counter to Tithe Collector's tempo win.
- **Move-history depth.** What if the engine has fewer than `S`
  half-moves in history (early game, after FEN-load)? Rewind
  what's available. `S=5` on turn 2 rewinds 2 plies, not 5.
  Hourglass still dies. Better than crashing.
