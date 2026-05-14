# Echo

> Records the most recent move on the board. On its turn, instead of moving
> itself, replays that delta onto any friendly piece.

## Inspiration

The geometric primitive is **move as a transferable resource**.
Baba Is You's "X IS MOVE" rules; Patrick's Parabox's recordings; the
Hourglass mechanic in many puzzle games where the last action becomes
ammunition for the next. The Echo turns each turn's delta vector into
ammunition the Echo's owner can spend.

It also has a strong tactical-symmetry flavour: every move the
opponent makes is potentially weaponized against them.

## Mechanic

The Echo records a **delta vector** `(dr, df)`. The vector is
initialized to `(0, 0)` (no recorded move).

After **any** move on the board (any colour, any piece), the Echo's
recorded delta updates to that move's `to - from` vector:

```
Echo.delta := move.to - move.from
```

If multiple Echoes exist, they all update simultaneously to the same
delta. (Each Echo carries its own recorded delta in FEN, but in
practice they're always equal *unless* one of them was placed
mid-game with a stale value, which is allowed.)

**The Echo's own turn.** On its controller's turn, the Echo player may:

1. **Do nothing.** Pass the Echo (no effect; some other piece moves).
2. **Discharge the Echo.** Pick any friendly piece `P` on the board.
   Compute the destination square `P.square + Echo.delta`. If that
   destination is on the board and walkable:
   - `P` moves to that destination.
   - The destination is treated as a normal move endpoint: capture if
     occupied by enemy; illegal if occupied by friend or by impassable
     terrain.
   - **`P` does not need to satisfy its normal move geometry.** A pawn
     can move backwards 3 squares if the Echo's delta is `(-3, 0)`.
     A knight can move in a straight line. A bishop can move
     orthogonally. The Echo bestows whatever vector it has.
3. After discharge, the Echo's recorded delta is **consumed** —
   reset to `(0, 0)`. Echo is empty until the next move on the board
   refills it.

The Echo itself **cannot move**. Capturing it removes its recorded
delta from play.

**Restrictions.**

- The friendly piece chosen for discharge cannot be **the Echo
  itself** (the Echo is stationary).
- Cannot discharge an Echo whose delta is `(0, 0)` (no move to replay).
- Discharge counts as the controller's full turn — they don't also
  move another piece.
- Piece geometry rules **not enforced** on the discharged move, but
  king-in-check rules **are** enforced (you can't echo into self-check).

## Why it's interesting

Every move is now a **risk to the moving side**. If you push your pawn
two squares forward (delta `(+2, 0)`), the opponent's next turn might
be: "discharge the Echo on your queen, moving it `(+2, 0)` — straight
into your defended squares." Counter-tempo built into the game.

It also breaks normal-geometry constraints in a controlled way. A
bishop can suddenly move like a rook, but only by the exact delta of
the last move. This produces **emergent puzzles** where the solver
must find a "key delta" that enables a final tactic.

## Example scenarios

**Steal a king move:**

```
. . . . k . . .       Black king on e8
. . . . . . . .
. . . . . . . .
. . . . . . . .
. . . . . . . .
. . . . . . . .
. . . . . . . .
. . . . K . . E       E = white Echo on h1. Last move: black k e7→e8 (delta (+1,0))
```

White discharges the Echo on the white king (on e1). Apply delta
`(+1, 0)`: white king moves to e2. Not a normal king move? Actually
e1→e2 is a legal king move — but the Echo lets white move the king
**without** spending the king's normal move geometry constraint.
This example doesn't show much.

**Better example: bishop moves orthogonally:**

Same setup but white has a bishop on c1 and the last move was black
rook a8→a4, delta `(-4, 0)`. White discharges the Echo on the white
bishop. Bishop applies delta `(-4, 0)` from c1: lands on c-3 — off the
board. Illegal. Try discharge on white knight on b1: delta `(-4, 0)`
from b1 → b-3, off-board. Illegal.

The Echo gives "any delta to any piece" but the delta must be a legal
landing for the chosen piece. The strategy is **choose your piece such
that the delta + piece's square = useful square**.

**Long-range pawn cascade:**

Black queen plays Q-d8 → Q-d1, delta `(-7, 0)`. Now the Echo holds a
huge southbound vector. White discharges on white pawn at b7 (a pawn
deep in enemy territory): delta `(-7, 0)` from b7 → b0, off-board.
Illegal. Discharge on white rook at h8: h8 + `(-7, 0)` = h1. Rook
teleports across the board. Captures whatever's on h1 (probably
nothing if h1 was empty). **The Echo transformed a queen move into a
rook teleport.**

## Where it shines

- **Counter-tempo defence.** Echoes weaponize opponent moves. Long
  attacking moves are now also long defensive moves for the other side.
- **Composition.** Two Echoes — one for each side — make every move
  reciprocal. The board becomes a delta-mirror.
- **Promotion magic.** A pawn on the 7th rank with a delta of `(+1, 0)`
  available promotes. Easy. But a pawn on the 6th rank with a delta of
  `(+2, 0)` available — that pawn lands on its promotion square in one
  Echo discharge, not two pushes. Big tempo gain.

## Where it's awkward

- **What counts as a "move"?** Castling has two pieces moving — which
  delta gets recorded? Suggest: the king's delta. (Or: castling
  doesn't update the Echo at all. Discuss.)
- **Captures and Echo.** A capture move has a `to - from` delta;
  Echo records it. The captured piece is gone — its position doesn't
  matter to the delta. Fine.
- **Promotion via Echo.** A pawn lands on its back rank via Echo
  discharge. Does it promote? Yes; same rule as any pawn reaching
  back rank. Player picks promotion type.
- **Discharge to the Echo's own square.** A friendly piece's
  `square + delta` equals the Echo's square? Illegal — the Echo is
  there.
- **King discharge.** Player can move the king via Echo. This bypasses
  the "king moves one square" rule — but check rules still apply, so
  the king cannot land on attacked squares. Use this to escape mate?
  Yes, if the delta allows it. Very strong if the delta is a long
  vector.

## Engine dependencies

- **Move-history primitive.** Engine must already track the most
  recent move (for en passant). The Echo just reads it.
- **`is_walkable()` and capture infrastructure** — standard.

## New features required

- **Echo piece.** Has a `delta: (i8, i8)` field. Updated on every
  `make_move` via a post-move hook.
- **Discharge action.** New move type: `DischargeEcho { echo_id,
  target_piece_id }`. Move generator emits one such move per
  friendly piece whose `square + Echo.delta` is a legal landing.
- **Move-history hook.** Existing engine probably has this; just need
  to subscribe to it from the Echo update logic.
- **Geometry-free move primitive.** The discharge move bypasses
  per-piece geometry. The engine's `make_move` already handles
  arbitrary `(from, to)` pairs — confirm.

## FEN encoding

```
(P=EC,C=W,D=+2-1)       White Echo, recorded delta (+2, -1)
(P=EC,C=W,D=+0+0)       White Echo, empty (no recorded move)
(P=EC,C=W)              Defaults to empty if D omitted
```

`D=` payload is a signed `(dr, df)` pair. Notation uses explicit signs:
`D=+2-1` is `(+2, -1)`. `D=+0+0` is the no-move state.

Multiple Echoes can have different recorded deltas if they were placed
mid-game; engine updates all to the same value after each move, so in
play they converge. For hand-crafted positions, allowing different
deltas means the composer can pre-load each Echo with a specific
delta — useful for puzzles.

## Open questions

- **Castling, en passant, promotion delta.** What's recorded for
  multi-piece moves? Recommend: the moving "primary" piece's delta.
  For castling that's the king. For promotion that's the pawn's
  delta (which is `(+1, 0)` or `(+1, ±1)`).
- **Echo discharge as a "move" itself.** When the Echo is discharged,
  the discharged piece's delta is the Echo's recorded delta. Does
  this new move *also* update the Echo for the next turn? Yes — it
  re-records its own delta, which is the same delta. Self-reinforcing
  for one cycle.
- **Multiple Echoes, same player, different deltas.** Player can
  choose which to discharge. Strategic depth.
- **Echo on opponent's piece.** Spec says friendly only. Variant:
  Echo can also force-move an enemy piece by the delta. Much
  stronger — makes Echo more like a remote control. Future variant.
- **Empty Echo on placement.** If a board is set up with no prior
  move, the Echo starts at `D=+0+0`. Cannot discharge. Until the
  first real move plays, the Echo is dead weight.
- **Two Echoes one of each colour.** Each updates after every move.
  Both can be discharged on their respective turns. Symmetric. Fine.
