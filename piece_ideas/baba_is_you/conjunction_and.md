# AND (conjunction)

> Joins two SUBJECTs or two PREDICATEs into a single clause. The Baba
> equivalent of set-union.

## Role in the grammar

AND is a **CONJUNCTION**. It occupies the slot between two SUBJECTs (in
the `subj-group`) or between two PREDICATEs (in the `pred-group`). It
cannot cross the verb — `KNIGHT AND IS WALL` is invalid because AND
joins like-kinded tokens, not a SUBJECT to a VERB.

Grammar reminder:

```
subj-group := SUBJECT (AND SUBJECT)*
pred-group := [NOT] PREDICATE (AND [NOT] PREDICATE)*
```

AND can chain arbitrarily: `KNIGHT AND BISHOP AND ROOK IS WALL` is valid
and produces three atomic rules.

The parser identifies AND by `token_kind == Conjunction, payload == And`.

## Inspiration

Baba Is You's AND tile joins nouns and properties identically. "BABA AND
ROCK IS YOU" makes both controllable. The chess version preserves this:
AND is set-union over the joined slot.

## Mechanic

### Placement and movement

AND is a Color::Neutral, movable rule-token. Like SUBJECT, it slides one
square orthogonally per move. Like all rule-tokens, it can be captured.

### Parser interaction

When parsing a token run as a clause:

1. Identify the VERB (IS) position.
2. Tokens to the left of IS form the `subj-group`.
3. Tokens to the right of IS form the `pred-group`.
4. Within each group, the parser walks the tokens and applies the
   grammar rule for that group.

For `subj-group`: the parser expects `SUBJECT (AND SUBJECT)*`. Any
deviation (two adjacent SUBJECTs without AND, an AND at the start, a
trailing AND, an AND between non-SUBJECTs) makes the entire clause
invalid.

For `pred-group`: same pattern but `[NOT] PREDICATE` is the unit being
joined.

A single token (`SUBJECT`) or `[NOT] PREDICATE` is also valid — no AND
needed for a single-element group.

### Effect of AND

When the parser sees `SUBJECT(KNIGHT) AND SUBJECT(BISHOP) IS WALL`, it
atomizes to:

- `KNIGHT IS WALL`
- `BISHOP IS WALL`

Both atoms register. The behavior is identical to writing two separate
clauses (one for knights, one for bishops) — AND is grammatical sugar
that lets a single clause carry multiple bindings.

For multi-AND on both sides: `KNIGHT AND BISHOP IS WALL AND YOU` produces
the Cartesian product:

- `KNIGHT IS WALL`
- `KNIGHT IS YOU`
- `BISHOP IS WALL`
- `BISHOP IS YOU`

Four atomic rules from one clause. AND multiplies meaning.

### Splitting an AND

Moving (or capturing) the AND tile splits the clause. If the parser sees
`SUBJECT(KNIGHT) AND SUBJECT(BISHOP) IS WALL`, then a piece slides the AND
tile out of the line, the parser re-runs and sees:

- A truncated clause: `SUBJECT(KNIGHT)` (now isolated — only valid if
  parsed as part of a different run; in v1 single SUBJECT is inert).
- Or, if `SUBJECT(KNIGHT)` is still followed by `IS WALL` (perhaps the
  AND was the rightmost subj before IS): `SUBJECT(KNIGHT) IS WALL`. A
  valid one-knight clause. The bishop is no longer walled.

This is the "peel a rule off one piece" power. Surgical.

## Composition rules

- **AND atomizes via Cartesian product.** Multi-AND on either side
  multiplies the atom count.
- **AND does not interact with NOT in special ways.** `KNIGHT IS NOT
  WALL AND YOU` atomizes to `KNIGHT IS NOT WALL` and `KNIGHT IS YOU`.
  The NOT applies to the predicate immediately to its right (WALL), not
  to YOU. Use a second NOT (`KNIGHT IS NOT WALL AND NOT YOU`) to negate
  YOU as well.
- **Heterogeneous AND**: `SUBJECT(KNIGHT) AND WALL IS YOU` is invalid —
  AND can't join a SUBJECT and a PREDICATE.
- **AND adjacent to the VERB**: `KNIGHT AND IS WALL` (AND immediately
  before IS) is invalid; the parser expected another SUBJECT after AND.
  Inert.
- **Long chains.** `A AND B AND C AND D IS WALL` produces four atoms.
  No upper limit (other than board-size).

## Why it's interesting

AND is the **multiplier**. It lets a player build a single clause that
hits many piece-types or applies many predicates, with relatively few
tokens. Especially powerful for:

- **Multi-royal traps.** `KNIGHT AND BISHOP AND ROOK IS YOU` makes 12+
  pieces royal. The opponent has 12 win conditions to attack.
- **Compound predicates.** `IS WALL AND ROYAL` (where ROYAL is shorthand
  for YOU) makes pieces both immovable and game-ending — the
  immovable-king trap.
- **Surgical peeling.** Constructing the multi-AND clause is a long
  setup, but splitting one AND with a single move surgically removes
  one piece-type from the trap.

The grammar move "slide AND off the line" is the precise opposite of
"slide AND into the line." Symmetry of construction and destruction.

## Example sentences

```
[SUBJECT KNIGHT] [AND] [SUBJECT BISHOP] [IS] [WALL]
  → both knights and bishops are walls.

[SUBJECT KING] [IS] [WALL] [AND] [YOU]
  → kings are walls AND royal. Immobile, lose-on-capture.

[SUBJECT WHITE_PAWN] [AND] [SUBJECT BLACK_PAWN] [IS] [NOT] [WALL]
  → no pawn (either color) is a wall, regardless of other clauses.
```

## Example puzzle

```
 . . . k . . . .
 . . . . . . . .
 . [SUBJECT KNIGHT] [AND] [SUBJECT BISHOP] [IS] [WALL] .
 . . . . . . . .
 . . . . . . . .
 . . N . . B . .
 . . . . . . . .
 . . . . K . . .
```

White has king e1, knight c3, bishop f3. Black king on d8. Both white
minor pieces are walled by the multi-AND clause.

White's task: deliver mate by mobilizing one of the minor pieces.

Solution: slide the AND tile off the clause line. Pick which (knight
side or bishop side) to free.

Suppose AND is at d5, between SUBJECT(KNIGHT) at c5 and SUBJECT(BISHOP)
at e5. If a piece can capture AND (say a black pawn can't reach, but
imagine setup permits) — or if the player can shove SUBJECT(BISHOP) into
AND's square — the AND-clause splits.

After split: the leftmost run is `SUBJECT(KNIGHT) IS WALL` (still walls
knights). The rightmost run is just `SUBJECT(BISHOP)` isolated (does
nothing). Bishop is now free. Bishop delivers mate.

Or: free the knight instead by splitting from the other side. Two
possible solutions, depending on which piece is better for the mate.

## Where it shines

- Multi-piece binding setups. AND is essentially the only way to express
  "this rule applies to multiple piece types" in one clause.
- Compound predicate setups. `IS WALL AND YOU` is the canonical
  immovable-target form.
- Puzzles where the choice of *which* AND to split is the puzzle.

## Where it's awkward

- **AND tile density.** Multi-AND clauses are long. A
  three-piece-type AND-clause on the SUBJECT side is 7 tokens; with two
  predicates on the PREDICATE side it's 11. Boards get crowded.
- **AND adjacency to the wrong neighbor.** A SUBJECT tile sitting next to
  an AND tile in a *different* clause is ambiguous: is this AND
  continuing the SUBJECT's clause, or starting a new one? v1 resolves
  via the strict "maximal orthogonal run" rule — the AND belongs to the
  longest contiguous run of rule-tokens it sits in. Adjacent clauses
  must be separated by a non-token square.
- **AND in a clause with a single SUBJECT.** `SUBJECT(KNIGHT) AND IS
  WALL` is invalid (AND is the last subj-position but there's no second
  SUBJECT). The AND tile must always have a like-kinded peer on each
  side within the group.

## Engine dependencies

- `VariantId::Baba`.
- The grammar parser must atomize multi-AND clauses correctly. Cartesian-
  product expansion.
- The rule-effect registry receives atoms identically whether they came
  from a multi-AND clause or independent clauses — the AND mechanism is
  parser-side only.

## New features required

- **`Conjunction::And`** in the RulePiece enum.
- **Atomizer** that expands `subj-group × pred-group` to atoms. Already
  needed for `SUBJECT IS WALL AND YOU` (multi-predicate, one subject);
  AND on the subject side just adds another dimension.

## FEN encoding

```
(R=AND)
```

No payload.

## Open questions

1. **AND vs OR.** OR is not in v1 but could be added later. OR-semantics
   in Baba would be "rule applies to this subject *or* that subject" —
   which is exactly AND's current behavior, since the rule is bound by
   the disjunction of types. So OR and AND would mean the same thing for
   SUBJECT-side; for PREDICATE-side OR is meaningful (`IS WALL OR YOU`
   could mean a 50/50 random binding — but Baba is deterministic, so
   not v1). Defer.
2. **Empty AND chains.** `SUBJECT(KNIGHT) AND AND SUBJECT(BISHOP) IS WALL`
   is invalid (two ANDs in a row). Inert.
3. **AND across clauses on perpendicular lines.** If a horizontal clause
   has its AND at position (3,5) and a vertical clause runs through
   (3,5), does the AND participate in both? v1: yes — the parser runs
   each orthogonal direction independently, so the AND can serve double
   duty. Same as how IS can anchor two perpendicular clauses (see
   copula_is.md).
