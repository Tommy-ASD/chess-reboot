# Paradox

> A piece whose move resolves two ply before it is declared — the
> board commits to an effect now and waits for the cause to catch up.
> [TIME / CAUSALITY]

## The law it breaks

Normal chess obeys strict temporal ordering: a piece moves *because*
its owner declares the move on its owner's turn. Cause precedes
effect. The Paradox inverts this within a bounded two-ply window.
On turn N, the piece's *destination* is locked into the board state.
On turn N+2, the owner finally declares which legal move
*produced* that destination. The move retroactively becomes turn N's
play, and turns N+1 (opponent) plus N+1.5 (the Paradox's "wait
state") are evaluated against a board on which the move has already
landed.

The break is not "the piece sees the future." It is "the piece
*occupies* the future." The square is contested for two ply before
anyone can name the contestant.

## Mechanic

State per Paradox instance, stored in FEN payload:

- `pending: Option<Square>` — the destination square locked on turn
  N, empty otherwise.
- `lock_turn: u32` — the ply number on which the lock was placed.
- `home: Square` — the piece's apparent home, where it visually
  rests while the lock is outstanding.

Movement primitive (king + knight composite — flexible enough that
the *declaration* phase has real choices to make):

- Slides one square in any direction, or jumps as a knight.

Turn flow:

1. **Lock phase (turn N).** Owner picks any square the Paradox
   could legally reach by *some* move from `home`. That square
   becomes `pending`. The board records: "on turn N, the Paradox
   committed to landing here." The piece visually remains on `home`
   but `pending` is now a contested square — opposing pieces
   cannot enter it without being captured the moment the Paradox
   resolves. Engine treats `pending` as occupied-by-Paradox for
   threat-generation purposes.
2. **Wait phase (turn N+1, opponent's move).** Opponent plays
   normally. They may move into or capture the *visual* Paradox on
   `home`. If they do, see *Wait-phase capture* below.
3. **Resolution phase (turn N+2, owner's move).** Owner names the
   specific move that produced `pending`. The named move must have
   been legal *on turn N's board* AND must terminate on `pending`.
   The Paradox vanishes from `home`, materialises on `pending`,
   and `pending` clears. Any opposing piece that walked onto
   `pending` during the wait phase is captured. The named move's
   effects (e.g., captures along the path, if a slider variant is
   added) apply retroactively — captured pieces are removed from
   the position with their post-N+1 state, not their turn-N state.

**Wait-phase capture.** If the opponent captures the Paradox on its
`home` square during turn N+1, the lock dissolves. `pending` clears,
no retroactive move occurs, the Paradox is dead as normal.

**Failed resolution.** If on turn N+2 no legal turn-N move would
produce `pending` (e.g., the route was blocked by a wait-phase
move), the lock is *forfeit*: the Paradox is removed from play and
`pending` clears. The forfeit counts as a normal move for the turn
clock. This is the price of poor lock selection.

## Why it's interesting

The chess novelty: every Paradox lock is a fait accompli. The
opponent knows *where* the piece will be but not *what arrived* —
they cannot block the destination, only contest the staging area.
It punishes both players: the Paradox owner cannot react to the
opponent's reply, and the opponent cannot trade against an
incoming piece they cannot identify.

The conceptual elegance: the engine encodes "an event whose cause
is undetermined" as a finite struct — a one-square reservation
plus a deadline. The future is concrete; the past is the variable.

## Example scenarios

- **The sealed mate.** White Paradox on e4 locks `pending = h7`
  on turn 12. Black king on h8. Black must spend turns 12 and 13
  moving the king off the threatened square — but no Black piece
  can prevent the Paradox from materialising on h7 on turn 14. If
  the king reaches g7 by then, the Paradox arrives onto a
  defended square; if not, it's mate. The lock is a two-turn
  countdown clock.
- **The bluff lock.** White Paradox locks `pending = c5`, a
  square no current legal route reaches. Black sees the
  reservation, treats it seriously, and wastes a tempo. On turn
  N+2, White has no legal turn-N move producing c5 — Paradox
  forfeits. White spent the Paradox to buy two tempi.
- **The contested staging area.** Black Paradox on d4 locks
  `pending = f6`. White's bishop captures the Paradox on d4
  during turn N+1. Lock dissolves; the Paradox is dead with no
  retroactive arrival. Capturing the home is the canonical
  counter.

## Where it shines

- Endgame and king-hunt positions where two-ply lookahead by the
  defender is already at the edge of human capacity.
- Variants with restricted move budgets: the Paradox's
  "declare-late" property means it isn't blocked by the
  opponent's intervening tempo.
- Asymmetric scenarios where one side gets a single Paradox and
  must use the lock as a clock.

## Where it's awkward

- **Slider variants are nasty.** If the Paradox is a slider, the
  lock has to describe the *path*, not just the endpoint, to
  resolve captures along the ray. Composite king + knight is the
  simplest mover that avoids this.
- **Double Paradoxes.** Two Paradoxes locked simultaneously can
  interact: Paradox A's `pending` square is on Paradox B's
  resolution path. The natural rule — resolutions happen in
  declaration order — works but produces opaque interactions for
  players. Recommend a one-Paradox-per-side cap in the canonical
  variant.
- **The forfeit move.** A forfeited lock occupies a turn. New
  players will misread this as a free pass. Needs prominent
  surfacing in the editor and the move list.
- **Visual ambiguity.** `home` and `pending` both display the
  Paradox glyph during the wait phase — the home with a "ghost"
  outline, the pending with a "incoming" marker. The frontend
  has to render two affiliated tiles for one piece.

## Engine dependencies

- Per-piece FEN payload (already exists).
- Threat-generation reading `pending` as Paradox-occupied for
  capture purposes during the wait phase.
- Turn-counter access (already exists).

## New features required

- **Lock state on piece payload.** Three fields: `pending`,
  `lock_turn`, retained alongside the regular position.
- **Two-phase move legality.** Turn-N declaration generates legal
  destinations from `home` and stores one; turn-N+2 declaration
  generates legal moves on the *historical* turn-N board and
  filters to those terminating on `pending`. Engine needs to
  recover the turn-N board, or — simpler — re-derive legality
  from the live board if no intervening moves have altered the
  relevant squares, and forfeit otherwise.
- **Captured-during-wait handling.** When an opposing piece moves
  onto `pending` during the wait phase, it's marked
  "captured-on-resolution" but remains on the board until turn
  N+2. The move-list / display has to communicate this.
- **Forfeit move type.** A new `MoveKind::Forfeit` for the
  no-legal-turn-N-move case, so the move list doesn't try to
  replay a phantom move.

## FEN encoding

Paradox piece-id `X`. Payload appends `pending` and `lock_turn`
when a lock is active:

```
(P=X,PEND=h7,LT=12)
```

- `PEND` — algebraic destination square. Absent when no lock.
- `LT`  — turn number at which the lock was set. Absent when no
  lock. Used to compute the resolution deadline `LT + 2`.

No new keys are needed for the *home* square — the piece-id
position in the rank string already records it.

A position with no active Paradox lock is just `(P=X)`.

## Determinism notes

The break is bounded and finite: a two-ply window with one
reserved square. Everything is observable.

- Both players see `pending` and `lock_turn` in the FEN.
- The resolution-phase move set is derived deterministically
  from the recorded historical board — either by retained
  history (a hash of the turn-N position) or by re-derivation
  from the live board with conservative legality.
- No randomness. No hidden information. The Paradox does not
  "decide" what move it played on turn N — the *player* decides
  on turn N+2, from a fully visible legal-move set.
- Forfeit is deterministic: if the legal-move set on turn N+2
  is empty for `pending`, the lock dies.

The "future occupies the present" framing is metaphor; the
implementation is a reservation with a deadline.

## Open questions

- Should turn-N+2 declaration be required, or may it slip to
  turn N+4, N+6, etc., with each additional wait increasing
  forfeit risk? Default: required at N+2, no extension.
- If the opponent has zero legal moves on turn N+1 (stalemate
  in the wait phase), does the lock resolve on turn N+2 or does
  the stalemate fire first? Recommend: stalemate fires first; a
  pending lock has no resolution because the game has ended.
- Promotion interaction. Can a pawn declare a Paradox-style lock
  on its promotion square? Probably not in the canonical
  variant — Paradox is a distinct piece type.
- Multiple Paradoxes on one side: cap at one, or allow with
  ordered resolution? Default cap: one per side.
