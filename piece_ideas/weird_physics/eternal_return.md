# Eternal Return

> A piece that cannot be killed in place; capturing it rewinds the
> game's last K turns with the piece restored to its earlier
> position, costing the attacker their accumulated tempo while
> leaving everyone else's losses permanent. [TIME / CAUSALITY]

## The law it breaks

Chess captures are terminal: a captured piece leaves the board.
The Eternal Return refuses this. When captured, it does not
die. Instead, the engine rewinds its *own* trajectory K turns,
restoring its position from K plies ago, and replays the
intervening K plies of both players' moves *with the Eternal
Return back in play* — but with all *other* captures preserved.
The trajectory of the Eternal Return is overwritten by an
alternate one in which the would-be-fatal capture never
happened.

The break is selective. Time rewinds for the Eternal Return
only; the rest of the game-state — pieces lost, signals fired,
pawns promoted, terrain altered — is *not* unwound. The
attacker has lost K turns of tempo; the Eternal Return has lost
K turns of position; everyone else has kept their losses.

## Mechanic

State per Eternal Return instance, stored in FEN:

- `k: u32` — the rewind window for this piece (positions
  remembered).
- `position_history: VecDeque<Square>` — circular buffer of the
  Eternal Return's own positions over the last `k` owner-turns.
  Each entry is the square it occupied at the end of its turn
  `t - i` for `i in 1..=k`.
- `move_history: VecDeque<GameMove>` — the last `k` plies of
  *both* players' moves (i.e., 2k half-turns? or k full turns?
  see Open questions). Stored at the game level, not per piece.
  Used for replay.

Movement primitive (default): knight + king (composite, similar
to Paradox's mover). The piece's rewind ability is its
strength; its mover is intentionally modest.

Turn flow:

1. **Normal turn.** The Eternal Return moves like a normal
   piece. After the move, its current square is pushed onto
   `position_history`; the oldest entry drops out.
2. **Capture event.** When an opposing piece attempts to
   capture the Eternal Return, the capture is *initiated* but
   triggers rewind logic before resolving:
   a. The would-be capturing piece does not move. The capturer
      retains its current square and continues to exist.
   b. The Eternal Return is *re-placed* on the square it
      occupied K turns ago (front of `position_history`).
   c. The last K *plies* of the game are *replayed* against
      the new piece position:
      - Every move in `move_history` is re-applied in order.
      - The Eternal Return is *not* re-moved during replay —
        its position is fixed at the rewound square during the
        replay window, then it returns to normal mobility on
        the next turn forward from "now."
      - Captures of other pieces during the replay are
        re-evaluated: pieces lost in the original timeline
        stay lost, but a captured piece's *capturer* may have
        ended up somewhere else after the replay. The capture
        itself is preserved; the consequences for the rest of
        the timeline may shift.
   d. The would-be-capturer's last K plies are replayed *with*
      the Eternal Return in its rewound spot. The capturer may
      have made different legal moves had the Eternal Return
      been there; the engine does *not* re-decide for them.
      Their original moves are re-applied; if any move becomes
      *illegal* (e.g., destination now occupied by the Eternal
      Return), see *Illegal replay* below.

**Illegal replay.** If a replayed move's destination is now
occupied by the Eternal Return (in its rewound position), the
move is invalidated: the original piece doesn't move, the move
slot is *consumed* (the player loses that tempo), and the next
move proceeds. The piece that would have moved sits where it
was. This is the cost of the rewind.

**Captures during replay.** A replayed move that *would* have
captured a piece that is now somewhere else (because of the
rewind shifting the position) is also invalidated — same
"tempo consumed" rule. Captures of pieces that *are* still
where they were originally captured proceed normally.

**Eternal Return captured *during replay*.** If, during replay,
the Eternal Return is again attacked, *no* further rewind fires
— one rewind per capture event, not recursive. The Eternal
Return is captured normally during replay if a replayed move
delivers the capture. This bounds the rewind cost.

**K value.** Configurable per instance. Default K = 5 (replay
five plies of each player after rewind). Larger K is more
disruptive; K = 1 is "merely undo my last move and don't move
the capturer."

## Why it's interesting

The chess novelty: the Eternal Return is *not invulnerable* —
it can be killed during replay, or by careful coordination — but
*single-piece captures are wasted*. The attacker pays K turns of
tempo with no material reward, and the defender pays K turns of
his own piece's positional progress (it rewinds too). The piece
forces *plans* against it, not opportunistic captures.

The conceptual elegance: a per-piece history buffer plus a
game-wide replay window. The engine already has a move list —
the Eternal Return reuses it as the rewind source. The break is
in the *semantics of capture*, not in the move-generator.

## Example scenarios

- **The wasted bishop.** Black bishop on a8 takes White's
  Eternal Return on h1 on turn 20. Rewind fires: Eternal
  Return reverts to its turn-15 position on e4. The last 10
  plies replay. Black's bishop, mid-rewind, ends up where it
  was on turn 15. White's Eternal Return now sits on e4. Black
  has burned a capture for net zero — and possibly worse if
  the replay invalidated some of Black's intermediate moves.
- **The double-trade trap.** White Eternal Return on f3, K=5.
  Black knight on g5 captures it. Rewind: Eternal Return
  returns to its turn-N-5 position on b1. Replay: a Black pawn
  that was captured by White on turn N-3 was a real capture —
  the pawn stays captured. But the White piece that captured
  it had its move replayed against a different board; that
  move may still be valid, in which case the capture re-fires.
  White's intermediate material gains *are* mostly preserved.
- **The "this is fine" capture.** Black queen, willing to lose
  five turns of tempo, captures White's Eternal Return as a
  king-hunt clearance. The rewind costs Black five turns, but
  the queen *would* be back in play; whether that's worth it
  depends on the King's exposure during the replay. Sometimes
  the rewind is acceptable damage.

## Where it shines

- Mid-game where the Eternal Return acts as an indestructible
  blockader on a key square — opponents must build positional
  pressure rather than capture.
- Variants where tempo loss is severe (timed games, race
  variants, sacrifice scenarios).
- Endgame king-hunts where the Eternal Return is the *quarry*
  and the attacker must coordinate multiple pieces to ensure
  the rewound piece is killed during replay.

## Where it's awkward

- **Replay cost.** The engine re-executes K plies on every
  capture attempt. For K = 5 in a position with hundreds of
  considered captures during search, the replay cost is
  multiplicative. The board's `make_move` must be cheap and
  idempotent.
- **Replay determinism with stochastic-style state.** No
  randomness in this engine, but: if any piece's *legal* move
  set changed during replay (e.g., a Signal fired differently
  because the Eternal Return blocked a propagation), replay
  diverges from history. The engine must replay against the
  *historical* board, not the post-rewind one — except for the
  Eternal Return's position, which is the rewound one. Hybrid
  history is the source of complexity.
- **Captured-during-replay paradoxes.** If the replay captures
  the Eternal Return again (via a different path), it dies
  this time. But during the *replay capture*, no nested
  rewind. The bound-by-one rule resolves this.
- **Game-replay state.** Signals, triggered terrain, train
  movements — all the dynamic effects already in the engine —
  must be re-simulated cleanly. The engine must be
  deterministic and re-runnable from any move-list prefix.
- **K = large.** K = 20 produces wild rewinds that are
  essentially "restart the position with this piece relocated."
  Cap K at some reasonable value (e.g., 10) in the canonical
  variant.

## Engine dependencies

- Move-list / game-history (exists; needed for PGN-style
  export).
- Deterministic `make_move` / `undo_move` (the undo half is
  needed; not sure if the engine has it today — *new
  feature*).
- Per-piece FEN payload.

## New features required

- **Undo stack.** The engine needs to walk the move list
  backwards and restore prior board states. Either a literal
  undo-each-move primitive (cheap snapshot of just the
  changed-squares + signal-state) or a from-scratch replay
  from the start of the rewind window.
- **Hybrid replay.** Re-apply K plies of moves with a single
  piece's position overridden. Move-application code must
  accept a "with this piece pinned at this square" parameter.
- **Per-piece position-history buffer.** Bounded-size deque of
  the last K positions, pushed every owner-turn.
- **Replay invalidation rule.** A replayed move that becomes
  illegal consumes the player's tempo without applying. New
  move outcome: `MoveResult::Invalidated`.
- **One-rewind-per-capture flag.** A boolean on the rewind in
  progress, preventing recursive rewinds during replay.

## FEN encoding

Eternal Return piece-id `R`. Payload tracks `K` and
`HIST` (recent positions):

```
(P=R,K=5,HIST=e4,d3,c2,b1,a2)
```

- `K` — rewind window size in owner-turns.
- `HIST` — comma-separated list of up to K previous squares,
  oldest first. Absent or empty until the piece has actually
  moved.

The move-history needed for replay is *not* part of the
per-piece FEN — it's part of the game's move list, which
already exists in PGN-style form. The FEN snapshot encodes only
the per-piece state.

A game-record-aware loader is required: loading an FEN with
non-empty `HIST` but no move-list context is permitted, but the
Eternal Return's rewind will be no-op until enough moves
accumulate.

## Determinism notes

- The rewind is fully deterministic: K, HIST, move-list, all
  visible.
- Replay re-executes recorded moves through the same engine.
  Same input, same output.
- The one-rewind-per-capture rule prevents infinite recursion.
- No randomness, no hidden state. The cost of capture is
  computable by either player before committing.
- The interaction with signals, terrain, and trains is
  determined by the existing deterministic substrate.
- If two captures of the Eternal Return are attempted on the
  same ply (e.g., via two Apocrypha-style effects), only the
  first triggers a rewind; the second resolves against the
  rewound board.

## Open questions

- **Per-side or per-instance K?** Per-instance — different
  Eternal Returns can have different windows, set at game
  start.
- **Rewind during opponent's turn or owner's turn?** The
  capture-triggering ply is whoever's. Default: rewind fires
  on the capturing player's ply, at the moment the capture
  resolves, before the move completes.
- **What about pieces *spawned* during the rewind window?**
  E.g., a Mitosis halved on turn N-3, captured Eternal Return
  on turn N. Replay must respect spawn order. Default: spawns
  replay with their original timing.
- **Multiple Eternal Returns.** If two of them rewind on the
  same ply (unlikely but possible), do they both rewind
  independently? Default: yes, but only one captures resolves
  at a time, so the rewinds are sequential.
- **King-Eternal-Return.** A king with this property is
  effectively immortal until tempo-starved. Forbidden in
  canonical variant.
- **History pre-game.** What's in `HIST` at game start?
  Default: empty, or filled with the starting square repeated
  K times (meaning rewinds early in the game return to the
  starting square).
