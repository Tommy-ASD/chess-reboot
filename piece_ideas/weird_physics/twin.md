# Twin

> One piece occupying two squares — not a copy, not a link, the
> *same* piece in two locations, mirrored across the board's
> centre. [IDENTITY / NON-LOCALITY]

## The law it breaks

Chess assumes piece identity is local: one entity, one square,
period. A duplicate is a *different* piece. The Twin denies this.
Both squares of a Twin are the Twin — the piece's identity is
distributed across two coordinates simultaneously. Moving "one
half" moves the whole; capturing "one half" kills the whole;
both halves vote on threats; neither half has independent
existence.

This is not the king-and-rook castling co-operative move. Castling
is two pieces acting jointly. The Twin is one piece with two
extents.

## Mechanic

State per Twin instance, stored in FEN:

- `pos_a: Square` — first occupied square.
- `pos_b: Square` — second occupied square, always equal to
  `mirror(pos_a)` across the board centre.

`mirror((f, r)) = (FILES - 1 - f, RANKS - 1 - r)`. For an 8x8
board, `mirror(b3) = g6`. On odd boards the centre cell mirrors
to itself — handle this by forbidding the Twin from occupying
the centre cell pair (the mirror-self position collapses both
halves into one square, which violates the two-extent invariant).

Movement primitive: king (1 square, any direction). The Twin's
short range is the balancing constraint — a long-range Twin
covers too much.

Turn flow:

1. **Move declaration.** Owner picks *one* half (a or b) and a
   legal king-step destination `dest` for it. Legality is checked
   for that half independently — the other half does not need a
   legal step.
2. **Application.** The moved half slides to `dest`. The
   *unmoved* half teleports to `mirror(dest)`. Both halves
   complete the move on the same ply, no extra tempo for the
   second half.
3. **Capture detection.** If `dest` was occupied by an opposing
   piece, that piece is captured normally. If `mirror(dest)` was
   occupied by an opposing piece, *that* piece is also captured —
   one ply, up to two captures.
4. **Self-collision.** If `dest == mirror(dest)` (centre cell on
   odd boards), the move is illegal. If `mirror(dest)` is
   occupied by a friendly piece, the move is illegal. If
   `mirror(dest)` is occupied by another Twin's half (own or
   opposing), the move is illegal. The mirror always lands on an
   empty square or an opposing non-Twin.
5. **Capture by opponent.** Either half can be targeted normally.
   Capturing either half removes the Twin from the board
   entirely; the surviving half *also* vanishes on the same ply
   (no replacement, no remnant). One capture kills both extents.

## Why it's interesting

The chess novelty: the Twin makes the player think in
*reflections*. Every move is two moves. Every threat is two
threats. Every fork is doubled — but every fork against the Twin
only needs to land one tine. The piece is simultaneously
unusually strong (controls two areas) and unusually fragile (one
hit, total death).

The conceptual elegance: identity is encoded as a multiset of
positions with a hard invariant (`pos_b = mirror(pos_a)`). The
break is a one-line constraint maintained across every legal
move. No special-casing, no hidden state.

## Example scenarios

- **The unstoppable mirror-fork.** White Twin on c4/f5 (mirrored
  across centre). Moves c4 -> d5; the mirror auto-moves f5 -> e4.
  Both d5 and e4 simultaneously attack Black's king on e5 and
  Black's queen on d4. One ply, two threats, both pre-positioned
  by the auto-mirror.
- **The single-hit kill.** Black knight on h6 captures the Twin's
  b3 half. The g6 half — which Black was not threatening — also
  dies. The Twin is gone in one capture from across the board.
- **The blocked mirror.** White Twin on b2/g7 wants to step to
  c3. The mirror lands on f6, but f6 is occupied by White's own
  bishop. Move illegal. The Twin's mobility is constrained by
  *both* half-environments simultaneously.

## Where it shines

- Open positions with empty mid-board: both halves have room to
  mirror without collision.
- Counter-attacking play: the Twin's auto-mirror moves create
  threats on the far side that the opponent cannot anticipate
  from the local context.
- Variants on non-square boards (e.g., 6x10): the asymmetric
  mirror produces more interesting geometry than 8x8.

## Where it's awkward

- **Centre cell on odd boards.** The mirror-self collapses both
  halves into one square. Resolution: forbid the centre cell
  entirely for Twins on boards with odd dimensions. Documented
  but ugly.
- **Castling rights.** Twins don't castle, but if a Twin is the
  king of a variant, the castling logic explodes (two halves
  moving along two paths simultaneously). Recommend: Twins
  cannot be promoted to king roles.
- **Pinning.** A pin against the Twin's a-half is also a pin
  against the b-half — but moving the b-half along the *b*
  half's pin ray relieves *neither* pin. Legality is the
  intersection, which is unintuitive.
- **Promotion / pawn-stuff.** A Twin pawn — if such a thing exists
  — is too strange. Restrict to Twins-as-distinct-pieces, not
  Twin-modifier on existing pieces.
- **Two Twins on one side.** Their halves can collide. The
  collision check is N^2 per move. Cap at one Twin per side, or
  accept the cost.

## Engine dependencies

- Per-piece FEN payload (exists).
- `Board::for_each_piece` enumeration (exists).
- Move-generation hook that allows a single piece-id to produce
  moves with two simultaneous effects.

## New features required

- **Multi-square piece representation.** Currently every piece
  is rooted at exactly one `Square`. The Twin needs the
  board-state model to admit a piece with two positions. Two
  options:
  1. Store both halves as separate piece entries with a shared
     `twin_id` cross-reference; move-application code looks up
     the partner and moves it.
  2. Promote `Piece::position` to `Vec<Square>` or a
     `Position::One | Position::Twin` enum. Wider blast radius
     but cleaner invariant.
  Recommend option 1: less type-system churn, the cross-reference
  is the same single u32 used for signal IDs.
- **Mirror function.** `Board::mirror_square(sq) -> Square`,
  parameterised on board dimensions. Currently the engine has
  `Square::flip_rank` for pawn-direction purposes; the centre
  mirror is a new function.
- **Simultaneous-move application.** `make_move` needs to apply
  two relocations and two captures in one move-record. Suggests
  extending `GameMove` with an optional `mirror_effect` field.
- **Twin-aware threat generation.** Threats on `pos_a` and
  `pos_b` both threaten the Twin. The existing threat machinery
  already handles per-square threats; the only new logic is
  "Twin dies on either square's capture."

## FEN encoding

Twin piece-id `Y`. The Twin appears on *both* halves' squares in
the rank string, with a shared `TWIN` payload tagging them as
the same piece:

```
... (P=Y,TWIN=1) ... (P=Y,TWIN=1) ...
```

`TWIN` is a small integer (1, 2, ...) scoped per side; pieces
with the same colour and the same `TWIN` value are halves of the
same Twin. Two `(P=Y,TWIN=1)` entries on the same side declare
one Twin. Two `(P=Y,TWIN=2)` entries declare a *second* Twin.

The mirror invariant is *checked* on parse, not declared:
parser computes `mirror(pos_a)` and confirms it equals `pos_b`,
warning + dropping a half if not. This keeps FEN tolerant of
hand-edits.

Example: White Twin on c4 and f5 (mirrored on 8x8):

```
... (P=Y,COL=W,TWIN=1) on c4
... (P=Y,COL=W,TWIN=1) on f5
```

## Determinism notes

- The mirror function is pure — same square, same dimensions,
  same output.
- Both halves move on one ply, in a defined order (a-half first,
  b-half resolves automatically by mirror computation). No
  "which half moves first" ambiguity.
- Capture is fully deterministic: the predicate "is the captured
  square part of a Twin?" gives a yes/no; if yes, both halves
  are removed in one effect block.
- No hidden information: the Twin's identity link is in FEN.
  Both players see which squares are Twin halves.
- The two-captures-per-move case is bounded (always exactly 0,
  1, or 2 captures from a Twin move; never more).

## Open questions

- **En-passant interaction.** A Twin doesn't en-passant, but if
  a Twin half lands on an en-passant capture square, does the
  pawn die? Default: yes — the Twin's normal capture rules
  apply on the moved half, and en-passant is a normal capture
  for this purpose. Mirror half doesn't trigger en-passant.
- **Discovered check via mirror.** Moving the a-half can clear
  a pin on the b-half's ray, exposing the b-half side to check.
  Handle as normal check resolution on the post-move board.
- **Promotion of one half.** A Twin pawn (if allowed in some
  variant) reaching the back rank: does it become a regular
  piece, splitting the Twin? Default: Twin is not a pawn type;
  no promotion.
- **Captures-per-Twin tally.** When the Twin captures two pieces
  in one move (one per half), does this count as one move with
  two captures or two moves? Default: one move, two captures.
