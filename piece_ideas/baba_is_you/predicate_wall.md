# WALL (predicate)

> "...IS WALL." The bound SUBJECT becomes impassable terrain.

## Role in the grammar

WALL is a **PREDICATE**. It fills the slot to the right of an IS in a
clause. The semantic effect: every piece matching the SUBJECT-payload
becomes treated by the engine as a wall — it cannot move, and its square
blocks gliders' paths.

The parser identifies WALL by `token_kind == Predicate, payload ==
WallProperty`. The predicate enum has multiple variants; WALL is the first
and most foundational.

## Inspiration

In Baba Is You, the property tile WALL turns its bound nouns into solid
terrain. Pushable rocks become wall-rocks: pushing fails, stepping into
their square fails. The chess version is identical in effect — a pawn
that is `IS WALL` cannot move, and a queen sliding along the rank treats
the walled-pawn's square as a stopping blocker.

## Mechanic

### Effect on bound pieces

When the parser registers a `SUBJECT(X) IS WALL` rule, every piece of type
X on the board is flagged "walled" for the duration of the next
legal-move generation. Walled pieces:

- **Cannot move.** Their `legal_moves` returns the empty set.
- **Block gliders.** Rook/bishop/queen/etc. paths terminate on a walled
  piece's square (the glider cannot land on or pass through it, even to
  capture). The walled piece is *not* capturable by glider line attacks —
  the glider can't reach its square. *Leapers* (knight, etc.) may still
  capture it, because they don't trace a path.
- **Still cause game-end if YOU.** If the same SUBJECT also has `IS YOU`
  applied, the walled-and-royal piece can be captured by leapers but
  cannot move out of trouble. The classic Baba immovable-king setup.
- **Are still owned by their original side.** WALL doesn't change color
  or alignment. A walled white pawn is still a white pawn that white must
  manage.

### Effect persists clause-by-clause

When the clause dissolves (IS captured, SUBJECT moved, etc.), the WALL
flag is removed from those pieces at the next parser pass. They become
normal pieces again, potentially mid-game.

This means **a walled piece in the middle of an exchange can become
unwalled** by the opponent breaking the clause, suddenly able to move into
the threat. The puzzle solver must account for this: leaving a piece
walled-and-attacked is only safe so long as the clause holds.

### Toggle and stacking

If two clauses both flag the same SUBJECT as WALL (e.g. two identical
clauses on different rows), the effect is idempotent — the piece is just
walled. No double-walling.

If `KNIGHT IS WALL` and `KNIGHT IS NOT WALL` are both valid clauses, NOT
wins (see modifier_not.md). The pieces are not walled.

If `KNIGHT IS WALL` and `KNIGHT IS YOU` are both active, both apply.
Knights are walled AND royal. Lose-by-leap-capture is the win condition.

## Composition rules

- **WALL ∩ YOU**: piece is both impassable and royal. Compatible.
  Mechanically: leapers can capture (game-end), gliders cannot reach.
- **WALL ∩ NOT WALL**: NOT wins. Piece is not walled.
- **WALL on multiple SUBJECTs via AND**: all listed types are walled.
  `KNIGHT AND BISHOP IS WALL` walls both.
- **WALL applied via category SUBJECT**: a category-SUBJECT like ROYAL
  walls the current YOU pieces. If YOU is the king (default), `ROYAL IS
  WALL` walls the king. Combined with `KING IS NOT YOU`, the king is no
  longer YOU and so no longer in the ROYAL category, so the WALL no
  longer applies. Order matters: see "Resolution order" below.
- **Resolution order across clauses.** Each parser pass:
  1. Collect all clauses, atomize to (subject, predicate) pairs.
  2. Resolve category-SUBJECTs against the *previous* pass's category
     memberships (avoids paradox). v1 freezes category-membership at the
     start of each pass.
  3. Apply NOT overrides.
  4. Register effects.
  This is a one-pass resolver. No fixpoint iteration; chains of "X IS Y,
  Y IS Z" do not propagate to "X IS Z" within a single pass. (See open
  questions.)

## Why it's interesting

WALL is the predicate that turns the position itself into the puzzle.
The same chess board with the same pieces but with a WALL-clause
restricting one side's mobility is a different problem. Solving requires
either:

- Breaking the clause (capture IS, slide SUBJECT off, slide WALL off).
- Working with the walled pieces (use unwalled pieces, or capture
  walled pieces via leapers).
- Re-binding the clause to target the opponent's pieces instead.

The third option is the most Baba: rearranging tokens to point at the
opposite color.

## Example sentences

```
[SUBJECT PAWN] [IS] [WALL]
  → no pawn (of either color) can move; pawns block glider lines.

[SUBJECT QUEEN] [IS] [WALL]
  → the queens are stone cover. Useless to their owners but devastating
    as defensive terrain for whoever's behind them.

[SUBJECT BLACK_KING] [IS] [WALL] [AND] [YOU]
  → the black king is immobile and royal. White wins by getting a
    leaper-capture onto the king (knight check that becomes
    knight-take-king under Baba's relaxed king-safety).
```

## Example puzzle

```
 8/8/8/8/4k3/4(R=SUBJECT:QUEEN)(R=IS)(R=WALL)/4P3/3QK3
```

White queen on d1, white king on e1, white pawn on e2 (blocking advance),
SUBJECT(QUEEN)-IS-WALL clause on rank 3, black king on e4.

The queen is walled, so it cannot move. The pawn cannot advance because
e3 is occupied by SUBJECT. White must break the clause to use the queen
for mate.

Solution: capture IS with — well, the queen can't move. The pawn can
capture diagonally onto SUBJECT or WALL (d3 or f3 don't hit them — IS is
on e3? let's adjust the puzzle so this works). Move the king or use the
pawn to push SUBJECT off the line. Once the clause breaks, queen reaches
e3 or e4, mate.

The instructive moment: a walled queen is *worse than useless* — it
blocks its own king's escape. Walls have negative tactical value for
their owner.

## Where it shines

- Restriction puzzles. The clue is the WALL clause; the solution is the
  unlock.
- Defensive setups where the player wants their own piece walled
  intentionally (e.g. a walled pawn becomes an unkillable blocker on a
  rank — except by leapers).
- King-fortress positions where `KING IS WALL` creates an immovable
  monarch the player must defend or relocate via grammar change.

## Where it's awkward

- **Walled piece in a fork.** A piece can be walled-and-attacked-by-a-
  leaper, and there's nothing the owner can do directly — they have to
  break the clause from elsewhere. Feels good in puzzles, can feel
  arbitrary in open play.
- **Walled king.** If the king is the only piece the side controls and
  it's walled, the side is essentially mated unless they can break the
  clause. v1 says this is legal: the side just loses if they can't
  unwall. Document that "walled king" doesn't trigger stalemate — the
  side still has rule-piece moves available (if they own any) and the
  game continues until a real loss condition fires.
- **Walls and en-passant.** A walled pawn cannot be captured en-passant
  (it didn't move). A pawn that was walled, then unwalled, then made a
  two-square jump can be en-passant'd. Standard rules apply, just
  observe that the wall-window matters.

## Engine dependencies

- `VariantId::Baba`.
- Glider path-tracing in `engine/src/pieces/standard/{rook,bishop,queen}.rs`
  must consult the walled-piece set when stepping along a ray.
- Move-gen for the walled piece-type must short-circuit when that type is
  walled.

## New features required

- **`PredicateKind::Wall` enum variant.**
- **Walled-piece set** in the rule-effect registry. Cheaply queryable by
  move-gen and threat-resolution code.
- **Glider-blocker modifier** in the movement stack (plan 10, 100–199
  band) reading the walled set.

## FEN encoding

```
(R=WALL)
```

No payload. WALL is a single property; future predicates (STOP, MOVE,
PUSH) get their own labels.

## Open questions

1. **Walled rule-pieces.** Can a rule-piece be walled? `SUBJECT(SUBJECT)
   IS WALL` is forbidden (no meta-SUBJECTs), but `(category that includes
   rule-tokens) IS WALL` — does that work? v1: no, rule-tokens are not
   in any chess-piece category. Document as: WALL applies only to chess
   pieces.
2. **Multi-hop predicate chains.** If `KNIGHT IS WALL` and `WALL IS YOU`,
   are knights YOU? The resolution-order rule (one pass, no fixpoint)
   says no — but Baba Is You's grammar does support chains. v1 declines;
   future enhancement could add a fixpoint resolver, capped at depth.
3. **Wall pieces during animation/threat-resolution.** When a piece moves
   onto a glider's path and a walled piece is the next square, the
   glider was already blocked. Make sure the move-gen and threat-
   resolution code consult the walled set at the right moment.
4. **Wall pawns and promotion.** A walled pawn that gets unwalled mid-game
   and reaches the back rank promotes normally. If the promotion target
   type is also `IS WALL`, the new piece is born walled. Consistent;
   documented.
