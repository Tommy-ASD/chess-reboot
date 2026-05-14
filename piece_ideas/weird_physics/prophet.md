# Prophet

> A piece that writes a future fact about the board into the
> position, then dies if the future disobeys — and freezes the
> obedient piece in place if it complies. [INFORMATION /
> RETROCAUSAL]

## The law it breaks

Chess positions describe the present. The future is open;
players choose what comes next. The Prophet inverts this. On
its turn, it commits a *future* fact to the FEN: "in N turns,
square Q will contain piece P (with colour C)." This fact
becomes part of the position itself — both players see it,
both players must navigate around it.

When the due turn arrives, the engine checks the prophecy
against reality. If it holds — i.e., square Q contains the
predicted piece — the prophecy *succeeds*: the predicted piece
is frozen in place for one turn. If it fails — Q is empty,
contains the wrong piece, or has the right piece in the wrong
colour — the Prophet dies.

The break is retrocausal in effect: a *future* state constrains
*present* play. Both players plan around a fixed point in
spacetime that neither chose.

## Mechanic

State per Prophet instance, stored in FEN:

- `prophecy: Option<Prophecy>` — single outstanding prediction.
- A `Prophecy` is:
  - `due_turn: u32` — the ply on which the prediction is
    checked.
  - `square: Square` — the predicted location.
  - `piece_id: PieceId` — the predicted piece type.
  - `colour: Color` — the predicted colour.

Movement primitive: ferz (one square diagonally) + wazir (one
square orthogonally) — king-stepper. The Prophet's mobility is
constrained on purpose; its weapon is the prophecy.

Turn flow:

1. **No active prophecy.** On the Prophet's owner-turn, the
   owner *must* either move the Prophet (king-step) *or* write
   a new prophecy:
   - Write: choose `due_turn` (must be `current_turn + 3`,
     fixed delay), `square` (any board square), `piece_id`
     (any non-king piece type), `colour` (W or B). The Prophet
     does not move; the prophecy is stored on the Prophet's
     payload.
2. **Active prophecy.** While a prophecy is outstanding, the
   Prophet *cannot move* and *cannot write a new prophecy*.
   It sits inert on its current square. It can still be
   captured normally.
3. **Due turn arrives.** At the *start* of the side-to-move's
   ply on `due_turn`:
   - **Success:** if `square` is occupied by exactly the
     predicted piece (matching `piece_id` and `colour`), the
     piece is *frozen* — gains a `SquareCondition::Frozen` for
     one ply (the current side-to-move's ply). The Prophet
     survives. The prophecy clears.
   - **Failure:** if `square` is empty, or contains a
     different piece, the Prophet dies — removed from the
     board. The prophecy clears.
4. **Capture by opponent.** If the Prophet is captured before
   its prophecy's due turn, the prophecy dies with it. No
   side-effect.

**Prophet-on-Prophet.** Two Prophets on opposing sides may
write prophecies about the same square. Resolution is by
`due_turn`: whichever resolves first determines whether the
square is what the *second* prophecy predicted. Default: each
prophecy resolves independently against the board at its own
due turn.

**Self-prophecy.** The Prophet can predict any non-king piece
including itself. A White Prophet predicting "in 3 turns, e4
contains a White Prophet" with the Prophet's current square
being e4: this *succeeds* if the Prophet hasn't moved (which
it can't, while a prophecy is active), so this is a free
guarantee — but the Prophet sits still doing nothing. The
self-prophecy is a no-op cost.

**King prediction.** Prophecies cannot target the king (`PieceId`
restricted to non-king). Predicting the king's location would
collapse into checkmate-detection logic. Forbidden.

**Frozen interaction.** `SquareCondition::Frozen` already
exists in the engine. The Prophet reuses it. A frozen piece
cannot move; if its colour is the side-to-move on the prophecy's
due turn, that piece loses its tempo. (Frozen on opponent's
turn is just one turn of "this piece is unavailable.")

## Why it's interesting

The chess novelty: the prophecy is a *self-fulfilling pin*. The
side whose piece is predicted to be on Q has two options:
either *ensure* its piece is there (which means the prophecy
succeeds and that piece is frozen for one turn — possibly
useful, possibly punishing), or *avoid* Q (which means the
prophecy fails and the Prophet dies — a free piece for the
opponent). Both players are forced to play around the
prophecy.

The conceptual elegance: a single FEN tuple encodes a *future
constraint*. The engine reduces "retrocausal" to "deferred
check against a stored predicate." No actual time travel
needed.

## Example scenarios

- **The forced freeze.** White Prophet on b2 writes "in 3
  turns, e4 contains a Black queen." Black's queen is *not*
  currently on e4. Black has two choices:
  1. Move the queen to e4 by turn N+3. The prophecy
     succeeds, the queen is frozen for one turn. Black has
     committed a turn of tempo to defend a piece they didn't
     have a plan for.
  2. Don't move the queen to e4. The prophecy fails, the
     Prophet dies. White loses the Prophet but Black has
     spent the position dodging the predicted square.
- **The self-fulfilling pin.** Black Prophet writes "in 3
  turns, c5 contains a White knight." White's knight on b3
  was *planning* to reach c5 anyway. The prophecy will
  succeed; the knight will be frozen for one turn on c5. The
  prophecy didn't change White's plan but added a one-turn
  pin to it.
- **The deadly miss.** White Prophet writes "in 3 turns, h8
  contains a White rook." White has no rook within range to
  reach h8. White cannot move the Prophet (it has a prophecy
  active). The prophecy fails on turn N+3, the Prophet dies.
  White has spent the Prophet for nothing — a tempo-suicide
  if mis-written.

## Where it shines

- Mid-game tactical positions where prophecy can force the
  opponent's piece into a deadly square.
- Positions where the predicted square is *en route* to
  somewhere the predicted piece wants to go — the prophecy
  doesn't change behaviour, just costs the opponent one
  freeze-turn.
- Variants with multiple Prophets per side: prophecies stack,
  creating a "future fence" of constraints.

## Where it's awkward

- **Three-turn delay.** Players must compute three plies ahead
  for every Prophet. Mental load is high.
- **King-prediction ban.** Awkward to surface but necessary —
  otherwise the Prophet becomes an oracle-checkmate device.
- **Self-prophecy degenerates.** Predicting your own piece on
  its own square is a "wait three turns" no-op. Either ban
  same-side prophecies or accept them as a tempo-burn move.
  Default: accept; it's rarely optimal.
- **Stalemate during prophecy.** A Prophet locked in by an
  active prophecy plus all other pieces of its side immobile =
  stalemate. The Prophet doesn't get to abort the prophecy
  to escape. Resolution: stalemate is stalemate.
- **Multiple prophecies on the same square at the same turn.**
  Two prophets, one White, one Black, both predicting e4 at
  due_turn = 12. Both check independently. If e4 holds both
  predictions (somehow — predictions disagree on piece-id),
  impossible; one or both fail. Resolution: each prophecy is
  evaluated separately; results are independent. Resolved in
  insertion order if any sequencing matters.
- **The Prophet cannot defend itself.** Once a prophecy is
  active, the Prophet is a sitting duck. Capturing the Prophet
  invalidates the prophecy — a soft counter that's strong if
  the Prophet is exposed.

## Engine dependencies

- Per-piece FEN payload (exists).
- `SquareCondition::Frozen` (exists; plan 04 or similar).
- Turn-counter (exists).
- Per-ply hook for "check active prophecies."

## New features required

- **Prophecy payload.** A struct on the Prophet's piece-state
  with the four fields above.
- **Turn-start hook for prophecy resolution.** Before move
  generation on `due_turn`, walk all Prophets, check their
  prophecies, apply Frozen or remove the Prophet. Engine
  already has signal/condition-tick hooks; reuse the
  infrastructure.
- **Move-restriction while prophecy active.** Standard pin-
  style filter on the Prophet's move-generator: if a prophecy
  is active, no legal moves except "do nothing this turn" —
  i.e., the Prophet sits, but the side still moves another
  piece. (The Prophet's owner-turn during an active prophecy
  is *not* its own turn; the side moves other pieces.) This
  is subtle and worth careful spec.
- **Prophecy-write move type.** `MoveKind::WriteProphecy {
  square, piece_id, colour, due_turn }` — recorded in the
  move list distinct from a normal move.

## FEN encoding

Prophet piece-id `P`. Payload appends a `PROPH` field when a
prophecy is active:

```
(P=P,PROPH=e4:Q:B:T15)
```

- `PROPH=<square>:<piece-id>:<colour>:T<due_turn>` —
  four-field colon-separated. Absent when no prophecy active.
- `<piece-id>` is the engine's standard piece-id letter (e.g.
  `Q`, `R`, `B`, `N`, `P`, plus fairy IDs).
- `<colour>` is `W` or `B`.
- `T<due_turn>` is the absolute ply number on which the
  prophecy checks.

A Prophet with no active prophecy is just `(P=P)`. After the
prophecy resolves (succeed or fail), the Prophet's payload
reverts to plain.

The FEN serialiser must round-trip a prophecy that crosses a
save/load boundary; the prophecy is purely state, no implicit
history.

## Determinism notes

- The prophecy struct is FEN-visible. No hidden information.
- Resolution is a pure predicate: at `due_turn`, look at
  `square`, compare to `piece_id`/`colour`, return
  freeze/kill.
- Multiple prophecies are resolved in a defined order (e.g.,
  by Prophet's square in reading order) for any cascading
  effects, but they generally don't interact.
- Frozen is itself deterministic.
- No randomness; no choice during resolution.
- The "self-fulfilling pin" emerges from rational play, not
  engine quirk.

## Open questions

- **Due-turn delay.** Fixed at 3 ply, or per-prophecy choice?
  Default fixed at 3 for sanity. Variant could make it owner-
  chosen within [1, 5].
- **Multiple active prophecies per Prophet.** Default: one at
  a time. Allowing a queue makes the Prophet too dominant.
- **Same-side prediction.** Allowed? Default: yes, but rarely
  optimal. Could be banned for variant clarity.
- **Capture during freeze.** A frozen piece can be captured —
  freeze only prevents *that piece's* moves. Default: yes,
  freeze is non-protective.
- **Promotion interaction.** Can a prophecy predict a pawn
  that will promote on `due_turn`? The predicted `piece_id`
  must match the piece *on* the square *after* the
  side-to-move's promotion. Probably allow it: predict pawn,
  pawn promotes to queen on due turn, prophecy fails. Or
  predict queen, pawn promotes to queen, prophecy succeeds.
  Edge case but well-defined.
- **Captured Prophet mid-prophecy.** Prophecy is voided. No
  partial credit. Default.
- **Two Prophets on same side writing conflicting prophecies
  about the same square.** Each resolves independently. If
  contradictory (same square, same due turn, different
  required piece), at least one will fail — that Prophet
  dies. Default accepted.
