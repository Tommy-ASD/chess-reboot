# Gooner

> A target-locked piece that shuffles relentlessly toward the nearest
> enemy, shoving everything in its path — and can only ever capture
> the thing it's locked onto.

## Inspiration

The "gooner" meme is internet slang for someone in a state of
single-minded obsessive focus on one thing, unable to deviate. The
chess version takes that to its logical extreme: the piece sees an
enemy, picks one, and walks toward it forever.

Strip the paint: the mechanic is **autonomous targeting with
forced collateral.** Most chess pieces are infinitely flexible
each turn — the controller picks any legal move. This piece picks
its own moves under a fixed deterministic rule, and the controller's
only choice is whether to *use* it at all this turn or wait. The
piece behaves more like a missile than a soldier. The friendly-
shove mechanic creates the chess-design problem: the controller
must arrange the board so the Gooner doesn't trash their own
position.

## Mechanic

### State

The Gooner carries a per-piece state field `T` — a target square,
encoded as algebraic notation (e.g. `T=e4`) or `T=NONE` if no
target. Default `T=NONE`.

### Turn 1: target acquisition

At the start of the controller's turn, if `T = NONE` *or* the
target square does not contain an enemy piece (target was
captured or moved), re-acquire:

1. Scan all 8 king-ray directions and 8 knight directions from
   the Gooner's square, in a fixed order (N, NE, E, SE, S, SW, W,
   NW, then knight moves in fixed order). Cast a ray from the
   Gooner's square along each direction.
2. The first enemy piece reached on any ray, *unobstructed by
   friendlies on the ray*, is a candidate. Friendlies on the
   ray block visibility — line-of-sight is required.
3. Among all candidates, pick the **closest** by Chebyshev
   distance. Tiebreak by the fixed direction order above.
4. If no enemy is visible: `T = NONE`. The Gooner does nothing
   this turn.
5. Otherwise: `T = <square of chosen enemy>`. Continue to phase 2.

Target re-acquisition only happens when the existing target is
gone. If the original target is still on its original square and
still in line-of-sight, the Gooner stays locked on the same
piece. **It does not switch targets opportunistically.**

If the target moves to a *different* square but is still the
same piece, the Gooner stays locked — but the `T` field updates
to the new square. This requires per-target tracking; an
alternative is to store the target piece's ID rather than its
square. See open questions.

### Turn 2: shuffle toward target

If `T != NONE`, the Gooner moves one square along the shortest
path toward `T`:

1. Compute `dx = sign(T.x - G.x)`, `dy = sign(T.y - G.y)` where
   `G` is the Gooner's square. Each is in `{-1, 0, 1}`.
2. The target step is `(G.x + dx, G.y + dy)`.
3. If the target step is the same square as `T` (the Gooner is
   king-adjacent to the target), the Gooner **captures** the
   target. Standard capture — enemy piece removed.
4. Otherwise, evaluate the target step:
   - **Empty walkable square:** Gooner moves there.
   - **Enemy piece (not the target):** The Gooner cannot capture
     anything except `T`. The square is blocked; see "blocked
     step" below.
   - **Friendly piece:** **Shove.** The friendly piece moves one
     square in the **opposite** direction (away from the
     Gooner, i.e. `(F.x + dx, F.y + dy)` where `F` is the
     friendly's square — note this is the same `(dx, dy)` as
     the Gooner's step direction). Then the Gooner moves into
     the vacated square. See "shove resolution" below.
   - **Non-walkable square (Block, closed Gate, etc.):**
     Blocked. See "blocked step" below.

### Shove resolution

The friendly piece's destination must be **walkable and empty**.
If the destination is:

- Empty walkable: friendly moves there. Gooner advances. Done.
- Off the board edge: the friendly piece is **captured by the
  edge** — removed from play. The Gooner advances. (The meme
  demands this. The strategic implication is large.)
- Non-walkable square (Block, etc.): the shove fails. The Gooner
  is blocked — see below.
- Another friendly piece: **chain shove** — recursively try to
  shove that piece in the same direction. If the chain
  terminates (last piece can move into an empty square or off
  the edge), the entire chain moves. If any link is blocked by
  a non-walkable square, the whole chain fails.
- An enemy piece (not the target): the friendly cannot capture
  the enemy via a shove (shoves are not captures). Shove fails.
  Gooner is blocked.

### Blocked step

If the Gooner cannot complete its step (because the path is
blocked by a non-walkable square, an enemy that isn't the
target, or a failed shove chain), the Gooner **does not move
this turn.** The controller still spent the turn-action on the
Gooner — it's not a free pass. (Variant: blocked Gooner does
not consume the controller's turn. See open questions.)

### Diagonal-vs-orthogonal step selection

When both `dx` and `dy` are nonzero, the Gooner prefers the
diagonal step `(dx, dy)`. If that step is blocked but `(dx, 0)`
or `(0, dy)` is available, the Gooner tries those *in
deterministic order*: first `(dx, 0)`, then `(0, dy)`. This
ensures the Gooner is fully deterministic.

If `dx` or `dy` is zero, only the single nonzero-component step
is tried.

### Capture rule

The Gooner can only ever capture its locked target. If it
arrives king-adjacent to its target with the target still on the
target square, the next step captures. Any other enemy piece in
the Gooner's path is treated as an obstacle.

## Why it's interesting

1. **Player as architect, not pilot.** The Gooner runs itself.
   The controller's job is to position the board so the Gooner
   does damage on its way to its target — not to micromanage
   each move.
2. **Asymmetric threat profile.** The Gooner threatens *one*
   piece, but its passage through the board can shove friendly
   pieces off the edge. The controller has to defend against
   their own piece. This is a kind of friendly-fire risk new to
   chess.
3. **Predictable terror.** Both players know exactly where the
   Gooner is going. The piece is a slow-motion siege engine —
   the opponent can plan around it, the controller has limited
   ability to redirect it. Mind games happen at the level of
   "can I get my Gooner unstuck before my opponent's queen
   trades into it."
4. **The meme is the mechanic.** A piece that compulsively
   shuffles toward one enemy, ignoring everything else, is what
   a "gooner" would do. The mechanic and the joke are unified.

## Example scenarios

1. **Standard advance.** Gooner on a1, white. Black knight on
   d4. Gooner acquires target: `T=d4`. White's next turn: Gooner
   steps to b2. Black moves the knight to e6. White's turn:
   target is still the knight (same piece, new square `T=e6`),
   Gooner steps to c3. And so on.
2. **Friendly catastrophe.** White Gooner on c3, target is on
   h8. White king is on d4. Gooner's step direction is `(1, 1)`
   — onto the king. The king is *shoved* to e5. Castling
   rights are lost (the king moved); the Gooner advances to
   d4. The opponent will probably destroy this.
3. **Edge capture by shove.** White Gooner on f6, target on h8.
   White pawn on g7. Gooner steps northeast: pawn at g7 is
   shoved to h8 — wait, h8 is the target square (black piece).
   The shove destination is occupied by an enemy. Shove fails.
   Gooner blocked.
4. **Successful edge capture by shove.** White Gooner on f6,
   target on h6. White pawn on g6. Gooner steps east: pawn at
   g6 is shoved to h6 — but h6 has the target. Shove fails.
   Try `(1, 0)` only path. Same result. Gooner stuck.
   *Alternative:* White pawn on a7, Gooner on a6, target on h8.
   Gooner steps `(1, 1)` to b7 — empty. Pawn at a7 is not in
   the way. Pawn is untouched. Gooner advances.
5. **Capture-and-redirect.** White Gooner on d4, target black
   queen on d6 (`T=d6`). Gooner steps to d5. Black moves queen
   to f6 — target still alive, same piece, `T` updates to f6.
   Gooner now steps `(1, 1)` from d5 to e6. Black's queen
   takes the Gooner — done.

## Where it shines

- **Open-file killshots** — a Gooner with a clear runway and a
  high-value enemy at the far end is a guaranteed trade.
- **Variants with edge-capture exploitation** — getting your
  Gooner near the edge with a sacrifice-able friendly
  alongside lets you punt minor pieces off the board for
  position.
- **Goblin pairings** — a Goblin can kidnap the Gooner's target,
  forcing re-acquisition. Combo possibilities.

## Where it's awkward

- **Closed positions** — the Gooner gets stuck. A blocked
  Gooner is dead weight.
- **No king adjacency restriction** — a Gooner that locks onto
  the enemy king and shuffles toward it is brutally effective.
  May need a rule: Gooner cannot target the king. Variant
  question.
- **Self-trampling** — a player with a developed back rank
  cannot deploy a Gooner without shoving their own pieces.
  The piece is most useful from a starting position on the
  rim.
- **Determinism feels like loss of agency** — players who like
  pure tactical control will dislike this piece. That's fine;
  it's a variant piece.

## Engine dependencies

- **Per-piece state** for `T`. Square-coordinate serialization
  in FEN (new — existing payloads are integers and short
  strings).
- **Line-of-sight check** — standard ray-cast, present in
  every slider piece.
- **Turn-start hook** for target acquisition.
- **Move-generation override** — Gooner doesn't expose a normal
  "legal moves" list to the controller; it has *exactly one*
  legal move per turn (the auto-step). UI may want to expose
  that single move as the only option, or expose a "skip
  Gooner turn" option.
- **Shove primitive** — moves a piece against its will. New
  surface. Reusable for Costco Guy's transport, perhaps.
- **Edge-capture handler** — pieces shoved off the board are
  captured. Plan stub: integrate with the capture pipeline so
  shoved pieces follow the same "captured piece" semantics
  (Goblin pickup, etc.).

## New features required

- **Target-square FEN payload.** Encode squares as
  letter-number strings (e.g. `e4`). On variable board sizes,
  use the existing coordinate format. Plan stub: extend the
  per-piece payload type system to allow square refs.
- **Shove move type.** A new `GameMove` variant for shoved
  movement — like a capture, but the captive moves rather than
  being removed. Apply via the existing `relocate_pieces`
  pipeline.
- **Chain-shove resolution.** Walk the chain forward, validate
  end-of-chain, then commit the entire shift atomically.
- **Deterministic targeting order constant.** Document the
  direction-priority array as part of the engine spec.

## FEN encoding

Symbol: `GO` for white Gooner, `go` for black.

Payload: `T=<square>` or `T=NONE`.

```
(P=GO,T=NONE)
(P=GO,T=e4)
(P=go,T=h1)
```

Default `T=NONE` if omitted.

For variable board sizes, the square format follows whatever the
engine's `Square` algebraic notation uses (probably file letters
extended past `h` for wider boards, plus rank digits).

## Open questions

- **Target by piece ID vs target by square.** Storing by square
  is simpler but fragile (if the target moves, target tracking
  requires inferring "same piece, new square"). Storing by
  piece ID is more robust but requires piece-ID infrastructure.
  Recommendation: square-based, with re-acquisition on miss.
- **Can the Gooner target the king?** Suggests yes for
  symmetry, but Gooner-vs-king ends the game on capture. May
  want a variant rule.
- **What if `T` exists but the piece on `T` is no longer the
  original target** (e.g., a different enemy moved into `T`'s
  square after the original was captured)? Treat as re-acquire.
- **Blocked Gooner consumes a turn or not?** The chess-design
  question. Recommend: yes, consumes the turn (otherwise the
  controller can spam Gooner-turn-then-real-turn). Variant
  rule could flip this.
- **Two Gooners locking onto each other.** Both target the
  other. Step direction is `(±1, ±1)` toward the other. They
  walk toward each other, meeting in the middle, one captures
  the other. First-to-move wins. Deterministic and fine.
- **Gooner with no enemies in line-of-sight at start of game.**
  Probably never happens in standard setup (the opening
  position has enemies visible), but in a sparse puzzle, the
  Gooner sits inert.
- **Frozen Gooner.** Cannot move. Target stays locked but no
  step. Brainrot Gooner — probably still moves (Brainrot
  doesn't prevent the action, it scrambles it; semantics need
  defining).
