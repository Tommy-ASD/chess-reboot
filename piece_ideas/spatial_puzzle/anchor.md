# Anchor

> Names one enemy piece at placement. Whenever that piece moves, the Anchor's
> controller also moves it equally far in the opposite direction.

## Inspiration

The geometric primitive is **mirror-coupling of two trajectories**.
Into the Breach's hostage-taking mechanics; the "you move, I move"
dynamics of puzzle games where two characters share momentum; physical
linkages in classical mechanics where a constraint enforces equal-and-
opposite motion between two bodies.

The Anchor is a **leash made of geometry**. The named enemy piece is
not merely watched — its motion is mathematically inverted and applied
back, as if the Anchor's controller is gripping the other end of an
invisible rigid rod.

## Mechanic

An Anchor is placed with one mandatory parameter: the **target
coordinate** of a specific enemy piece at the time of placement. The
coordinate is encoded by absolute (rank, file). The Anchor "names" the
piece currently on that square; the engine tracks the named piece by
identity (piece-ID) across the game — if the target moves to a new
square, the Anchor's target moves with it.

If the target is captured (by anyone, including the Anchor's
controller's own move), the Anchor becomes inert. It remains on the
board, occupying a square, but has no further effect. (Composers may
want this for fixed positions where the Anchor's "tether" was already
spent.)

**The coupling rule.** Whenever the named target piece is moved by its
controller, the engine then immediately gives the Anchor's controller
a **free mirror move** of the target piece:

- Let `(dr, df)` be the delta vector the target just moved (e.g. knight
  from b1 to c3 has delta `(+2, +1)`).
- The Anchor's controller selects the target piece (no other piece) and
  must move it by `(-dr, -df)` — i.e. the inverse vector applied from
  the target's *new* square.

The mirror move:

1. Does **not** need to respect the target piece's normal move
   geometry. A pawn can be moved diagonally if that's the inverse of
   the just-made delta.
2. Must respect **board edges** — if the mirror destination is off the
   board, the mirror move is *forfeited*. (Anchor placement is partly
   strategic for this reason.)
3. Must respect **square walkability** — the mirror destination must be
   a walkable square.
4. **Can capture** anything on the mirror destination, including the
   target's own teammates (suicide is on the table).
5. **Cannot be declined.** The Anchor's controller must make the
   mirror move if it's legal; the move is part of the same logical
   turn as the original.

If the mirror move is illegal (off-board, blocked by terrain,
self-square = target's original square is occupied), the move is
forfeited but the original move stands.

**Multiple Anchors.** Each Anchor names exactly one target. Two
Anchors on the same target — both fire, sequentially, in order of
their placement (FEN order). The mirror moves compose.

## Why it's interesting

It introduces **non-local consequences for routine moves**. The
opponent must reason: "if I move my queen, the enemy gets to drag my
queen backward — is the backward square safe?" Every move costs two
moves.

It also creates **leash puzzles**: position the Anchor such that the
target's natural escape routes all have illegal mirror destinations,
trapping the piece.

## Example scenarios

**Pawn-anchor trap, 5×5 sketch:**

```
. . . . .
. . k . .       Black king on c4
. . . . .
. A . . .       Black Anchor on b2 names white pawn at b1
. P . . .       White pawn on b1, named target
```

White wants to push the pawn b1→b2. Move delta `(+1, 0)`. Mirror
move would push the pawn b2→b1 — but b2 is the Anchor's square,
i.e. occupied. So the mirror move would have to land on b1, which is
the pawn's original square, also empty after the push. Wait —
re-derive. Pawn moves from b1 to b2 — pawn is now at b2 — that's
the Anchor's square. Illegal: collision with Anchor. So the push is
illegal at the get-go.

White pushes the pawn b1→b3 instead (double push). Delta `(+2, 0)`.
Mirror: from b3, apply `(-2, 0)` — destination b1, which is empty.
Black plays the mirror, pawn returns to b1. **Net effect: white
just moved its pawn back to its starting square.**

Pawn b1→a2 is impossible (not a pawn move). Pawn b1→c2 also not a
pawn move. So pawn is geographically locked.

**Bishop-anchor leash:**

```
. . . . . . . .
. . . . . . . .
. . . . . . . .
. . . k . . . .       k = black king on d5
. . . . . . . .
. . . . A . . .       A = black Anchor on e3 names white bishop at f2
. . . . . . . .
. . . . . B . .       B = white bishop on f2
```

White bishop f2→a7. Delta `(+5, -5)`. Mirror from a7: `(-5, +5)` →
f2 again. Pure swap — bishop returns. Net: no movement.

White bishop f2→h4. Delta `(+2, +2)`. Mirror from h4: `(-2, -2)` →
f2 again. Net: no movement.

White bishop f2→g3. Delta `(+1, +1)`. Mirror from g3: f2. Same.

For *any* slide, the bishop returns. The only way to break free:
capture the Anchor (on e3, reachable diagonally from f2? No, f2 and
e3 are knight's-move). The bishop has no diagonal access to e3.
**Bishop is permanently leashed.** This is a one-piece prison.

**Knight-anchor with off-board forfeit:**

White knight on b1, Anchor on a3 names the knight. Knight b1→a3
(captures Anchor) — delta `(+2, -1)`. Mirror: from a3, `(-2, +1)` →
b1, empty, legal. Anchor's controller would move the knight back, but
the Anchor was just captured — it's already inert. **Capturing the
Anchor breaks the leash.** This is the only escape.

Knight b1→c3. Delta `(+2, +1)`. Mirror: from c3, `(-2, -1)` → b1,
empty. Knight returns. Net: no move.

Knight b1→d2. Delta `(+1, +2)`. Mirror: from d2, `(-1, -2)` → b0 —
off the board! Mirror is **forfeited**. Knight stays on d2.

This is the **strategic key**: find a knight move whose mirror falls
off the board.

## Where it shines

- **Forced sequences.** A puzzle composer can build "anchor mazes"
  where the only escape route is the one move whose mirror is forfeit.
- **Material asymmetry.** A single Anchor can effectively neutralize a
  queen — every queen move costs two queen moves.
- **King hunts.** Anchoring the king forces a positional response: any
  king move drags the king back. (Unless king moves over the edge of
  the mirror frame.)

## Where it's awkward

- **Triggers on every move.** Every single move by the named piece
  triggers the Anchor. The engine must check this in `make_move`.
- **Identity tracking.** The named piece must be tracked by stable
  piece-ID, not by coordinate. Engine needs unique piece IDs that
  survive moves but die on capture.
- **Promotion.** If the target is a pawn that promotes, does the
  Anchor still target the promoted piece? Yes — it's the same
  piece-ID, just a different type. (This makes pawn promotion under
  anchor strategically rich: promoted-queen-still-leashed.)
- **Anchor capturing the target via mirror.** If the mirror move's
  destination is the target's original square, and that square is
  empty, fine. If it's occupied by an enemy, mirror move captures.
  If the mirror move's destination is the Anchor's own square? Then
  the target captures the Anchor — and the leash breaks mid-turn.
  Edge case: handle as "mirror move legal, target captures Anchor,
  leash ends."
- **Castling.** If the target piece is a king and the player castles —
  the delta is `(0, ±2)`. Mirror is `(0, ∓2)`. Castling-back is
  almost certainly illegal as a literal move, so mirror is forfeit.

## Engine dependencies

- **Stable piece IDs.** Required across the engine. May not exist yet.
- **Move post-hooks.** A way for a piece's mere presence to inject a
  follow-up move on the other player's behalf. The signal substrate
  may already provide a "post-move" event we can subscribe to.
- **FEN coordinate-pair payloads.** New payload type: an absolute
  coordinate, not a relative offset.

## New features required

- **Piece ID system.** Each piece has a `PieceId(u32)` set at placement.
  Anchor's `target` field is a `PieceId`. ID persists through moves
  and promotions, dies on capture. Engine-wide.
- **Anchor's mirror-move resolver.** A new resolution phase: after a
  normal move is applied, scan all Anchors for matching `target_id`;
  for each, compute the mirror destination, check legality, apply if
  legal.
- **Forced-move primitive.** The mirror move is selected by the
  Anchor's controller — but only as a *piece selection*; the
  destination is determined by the engine. For an Anchor naming a
  single piece, there's no choice. So really the mirror move is
  fully engine-determined — no player input required.
- **FEN coord-pair payload.** Parse `T=(c5)` style absolute coordinate
  in payload, separate from `D=(...)` delta vectors used by Echo.

## FEN encoding

The Anchor itself is a piece. Its payload includes the target piece
ID (or coordinate-at-placement, resolved to ID at parse time):

```
(P=AC,C=B,T=f2)         Black Anchor naming the piece currently at f2
```

`T=` is the target. At parse time, the engine looks up the piece
currently on `f2` and records its `PieceId`. If `f2` is empty, the
Anchor is born inert.

Alternative: store the ID directly: `T=#7` referring to piece ID 7.
More robust but requires the FEN to encode piece IDs everywhere,
which is intrusive. Recommend coord-at-placement for human-edited
FEN; engine-emitted FEN can include a dual `T=f2;TID=#7` for
disambiguation across promotion/capture history.

## Open questions

- **Multiple Anchors on the same target.** Compose sequentially? Or
  do they all simultaneously demand mirror moves, and the controller
  picks one? Sequentially is cleaner and FEN-deterministic.
- **Anchor naming a piece on the Anchor's own team.** Allowed?
  Strategically weird — you'd be giving the opponent a free move.
  Probably forbid at placement.
- **Anchor naming another Anchor.** Self-referential. If the target
  Anchor moves (it can't), it would trigger the naming Anchor.
  Anchors don't move, so this is vacuous but legal. Edge case
  documented.
- **Promotion-during-mirror.** The target's mirror move lands the
  pawn on the back rank. Does it promote? Yes — apply promotion
  rules normally. (The pawn does not choose the promoted piece type
  on the mirror; force-promote to queen, or use the target piece's
  default.)
- **Stalemate via Anchor.** Player to move has only legal moves that
  trigger Anchor mirror moves resulting in being placed in check —
  is that stalemate? Yes; the rule "you cannot move into check" must
  account for the post-Anchor-mirror position, not just the
  pre-mirror one. Significant change to legality check.
