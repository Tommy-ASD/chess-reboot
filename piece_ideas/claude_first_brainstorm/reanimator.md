# Reanimator

> Banks every friendly piece that dies; on cooldown, drops one back
> onto an adjacent empty square, recolored to its own side.

## Inspiration

The `Bus.passengers` pattern (a piece carrying a list of other
pieces' types) is the template. The Reanimator reuses it but with
a different semantic: instead of *carrying* pieces it has *eaten*
or *received*, it carries pieces it has *witnessed dying*. The
list grows passively (via the engine's existing death-observation
hook) and is consumed actively (via a drop move).

The chess problem it answers: in standard chess, captured pieces
are gone. Crazyhouse / Bughouse fix this by letting players drop
captures back on the board — but only on *empty* squares,
anywhere, with no piece-specific constraint, and only for pieces
*you* captured (Crazyhouse) or were *handed by your partner*
(Bughouse). The Reanimator is a chess-internal version of this
mechanic: a specific piece controls a specific bank, with a
cooldown, with placement adjacent to the Reanimator.

## Mechanic

Movement set: identical to King.

State:
- `graveyard: Vec<PieceType>` — the bank of dead friendlies.
- `cooldown: u8` — turns until next reanimation is available.
  Starts at 0; resets to `N` (initially `N = 3`) after each
  reanimation.

Hooks:
- **On friendly death.** Whenever a piece of the Reanimator's
  color dies (captured, killed by Bomb explosion, run over by a
  train, etc.), append that piece's `PieceType` to *every*
  same-side Reanimator's `graveyard`. (If there are two
  Reanimators, both banks grow.) The Reanimator itself does not
  need to be adjacent; it observes deaths globally.
- **On Reanimator death.** The graveyard is lost. (Open question:
  could a same-side Reanimator inherit? Recommend no — keeps the
  state local.)

Special action — **Reanimate.** If `cooldown == 0` and
`graveyard.len() > 0`, the Reanimator can — instead of moving —
pick one entry from `graveyard`, remove it, and place a fresh
piece of that type, of the Reanimator's color, on any empty
adjacent square. Set `cooldown = N`.

Constraints on the drop square:
- Empty of pieces.
- Walkable (`is_walkable()` true; no painting a Knight onto a
  Block).
- The dropped piece is a fresh instance — empty
  `passengers`/`absorbed`/`graveyard`, default cooldowns. The
  Reanimator does not preserve the original piece's state.

End-of-turn cooldown decrement: `cooldown = cooldown.saturating_sub(1)`
on each Reanimator turn that did *not* reanimate. Or
equivalently, decrement every same-side turn — same shape as the
Goblin's return cooldown if plan 04 ships one.

## Why it's interesting

Three reasons:

1. **Inverts the capture-equals-loss principle.** Sacrificing a
   Knight for tempo is normally a clean material trade. Sacrificing
   a Knight while you have a Reanimator means the Knight is in the
   bank — you'll get it back eventually. The cost-of-loss
   calculation changes per-piece.

2. **Two stateful slots in one piece.** The Reanimator has *both*
   a list (`graveyard`) and a counter (`cooldown`). Most fairy
   pieces have one or the other. Co-existing FEN-serializable
   states stress-test the payload encoding (plan 06).

3. **Global observation, local action.** The graveyard updates
   from anywhere on the board; the drop is constrained to
   adjacency. That asymmetry mirrors a real military medic — sees
   the whole battlefield, can only treat what's close.

## Example scenarios

**Material miser.** White Reanimator on e4. Black captures white
Knight on g5 (move 18). Reanimator's `graveyard = [N]`. Move 21
(after 3 turns of cooldown — assuming `N=3` and cooldown ticked
each Reanimator turn): white Reanimator drops a fresh Knight on
e5. Net material exchange across the sequence: Black lost
nothing, White lost a Knight for three moves. Tempo cost real,
material cost zero.

**Defensive resurrection.** Black Reanimator on h8. White's
attacking pieces have captured black Pawns on f7, g7, h7 (moves
14, 15, 17). Reanimator's `graveyard = [P, P, P]`. Black has been
cycling Reanimator with `cooldown = 3`: move 17 drop on g8 = P;
move 20 drop on g7 = P (after recapture by White); move 23 drop
on h7 = P. Three resurrections rebuild the pawn shield while the
Reanimator pivots around the corner.

**Reanimator captured.** White Reanimator has `graveyard = [Q,
R, R, N, N, B]` after 30 moves of attrition. Black captures it.
Bank lost. Black just neutralized six future drops with one
capture. The Reanimator becomes the priority target.

## Where it shines

- Long games where captures accumulate.
- Defensive players who can afford to invest in piece-protection
  around the Reanimator.
- Variants where pieces die to non-capture sources (Bomb
  explosions, Locomotive runs, Skibidi stuns into board edge).
  The graveyard grows from *all* friendly deaths.
- Compositions where a specific piece is going to die and the
  Reanimator can recover it — turning a tactical sacrifice into
  a material-neutral one.

## Where it's awkward

- Short games. Empty graveyard = wasted King-mover.
- Boards with low capture frequency. Same problem.
- Adjacent drop constraint can be frustrating if the
  Reanimator is cornered — drops have nowhere to land.
- Pieces with state (Bus passengers, Goblin victims, Vampire
  absorbed) are dropped *fresh*. A player who lost a fully-loaded
  Bus gets a stock Bus back. This is the right call (saving
  state would be exploitable and weird) but feels lossy.
- Cooldown counter on top of graveyard is two layers of
  bookkeeping. UI must surface both clearly.

## Engine dependencies

- A global friendly-death hook (already needed for any
  piece-on-death effect — Bomb, plan 04's Goblin-on-death).
- `Vec<PieceType>` payload precedent from Bus.
- King-movement primitive.
- FEN payload extension for multi-field state (graveyard + cooldown).

## New features required

- `Piece::Reanimator { graveyard: Vec<PieceType>, cooldown: u8 }`
  (or as payload on the unified `Piece` struct).
- A `MoveType::Reanimate { piece_type: PieceType, target: Coord }`
  variant. Apply-side: pop `piece_type` from `graveyard`, drop a
  fresh piece on `target`, set `cooldown = N`, this Reanimator
  did not move.
- Death-observation hook: a centralized "on piece death" callback
  that finds all same-side Reanimators and appends to each
  graveyard.
- Cooldown decrement rule: same-side turn-start tick.
- FEN encoder/decoder for `(P=REAN,G=(N,N,Q),CD=2)`.
- Tests: friendly death appends to graveyard; reanimate consumes
  + drops; cooldown ticks; Reanimator death wipes its own bank
  but doesn't wipe a same-side Reanimator's bank; FEN round
  trip.

## FEN encoding

Piece tag: `REAN` (multi-character; `R` is already Rook, `Re` may
be ambiguous). Payload:

```
(P=REAN,G=(N,Q,P,P),CD=2)        # white Reanimator, banked N+Q+2P, cooldown 2
(P=rean,G=(),CD=0)               # black Reanimator, empty bank, ready
(P=REAN)                         # equivalent: empty G, CD=0 (both elidable)
```

Keys:
- `G` = graveyard (list of PieceType tags, same format as Bus's
  `P` list and Vampire's `A` list).
- `CD` = cooldown counter (integer 0..N).

Both elidable at zero / empty.

## Open questions

- **Initial `N` (cooldown).** Default 3. Could be variant-tunable
  (`(P=REAN,CD_MAX=5)`). Recommend hard-coded for v1, with the
  variant system absorbing tuning later.
- **Multiple same-side Reanimators.** Recommended: each tracks
  its own graveyard, both observing deaths. So two Reanimators
  on the same side double the resurrection capacity. Risk of
  abuse — but each Reanimator is itself a vulnerability, so
  tactical pressure scales.
- **Reanimating a Reanimator.** A Reanimator's graveyard
  includes other dead Reanimators. Dropping one drops a *fresh*
  Reanimator (empty graveyard, default cooldown). This is fine
  but slightly anticlimactic — the dead Reanimator's stored
  graveyard is permanently lost. Document.
- **Reanimating a unique piece.** Some piece-types are
  one-per-side (Kings; possibly variant-restricted). Dropping a
  second King is illegal. Recommend: filter `graveyard` against
  variant-specific uniqueness rules at drop-move-gen time. Skip
  King entries; warn on FEN parse if graveyard contains a `K`.
- **Drop creates check.** A drop produces a position with the
  reanimating side in check. Illegal (same rule as any
  self-check move). Already covered by the engine's existing
  legality filter — no special-case needed.
- **Inheritance on Reanimator death.** Currently: graveyard
  lost. Alternative: nearest same-side Reanimator inherits.
  Recommend simple v1, revisit if competitive play wants it.
- **Drop respects square conditions.** Dropping onto a `Frozen`
  square or `Brainrot` square: the dropped piece picks up the
  condition immediately? Or is "drop" considered a special
  appearance that doesn't trigger conditions? Recommend: drop
  triggers conditions (consistency with "step onto"). Worth a
  test.
