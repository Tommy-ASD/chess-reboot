# NOT (modifier)

> A negation-token. Inverts the predicate it points at. NOT wins any
> contradiction.

## Role in the grammar

NOT is a **MODIFIER**. It does not stand alone as a clause-component; it
attaches to a PREDICATE in the `pred-group` slot of a clause. Grammar:

```
pred-group := [NOT] PREDICATE (AND [NOT] PREDICATE)*
```

The optional NOT immediately precedes a PREDICATE and negates it. Multiple
NOTs in a row (`NOT NOT WALL`) are not valid in v1 — at most one NOT per
PREDICATE.

The parser identifies NOT by `token_kind == Modifier, payload == Not`.

## Inspiration

Baba Is You's NOT tile is the override mechanism. "BABA IS NOT WIN" turns
off baba's win property. In Baba, NOT-rules typically *defeat* positive
rules of the same form via Baba's specific priority system.

The chess version inherits this: when both `KNIGHT IS WALL` and `KNIGHT
IS NOT WALL` are valid clauses, the NOT wins. The piece is **not** walled.

This is the opposite of "additive composition" — NOT is *subtractive*.
It removes effects rather than adding them.

## Mechanic

### Placement and movement

NOT is a movable rule-token, like SUBJECT. It can be slid one square
orthogonally per turn (by either player — NOT is color-Neutral). Or it
can be captured, removing the token from the board.

NOT is the lightest rule-token to move (no payload), and the most
impactful per-move-spent: a single tile-slide can invert an entire
clause.

### Parser interaction

When parsing the `pred-group` of a clause, the parser checks each token
position. If the position holds a NOT-token, it sets a `negated = true`
flag for the *next* PREDICATE token. The PREDICATE-with-negation produces
a negative atomic rule like `KNIGHT IS NOT WALL`.

NOT appearing in the `subj-group` slot is **not** valid. NOT cannot
negate a SUBJECT. v1 simply discards such clauses as ungrammatical.

NOT appearing without a following PREDICATE is **not** valid (e.g.
`SUBJECT IS NOT` with nothing after the NOT). Clause is inert.

### Contradictory clauses

When the rule-effect registry sees both `KNIGHT IS WALL` (positive) and
`KNIGHT IS NOT WALL` (negative) at the same parser pass:

1. **NOT wins.** The negative rule is registered, the positive is
   discarded.
2. The knights are not walled.

This applies pairwise per (subject, predicate). `KNIGHT IS NOT WALL` does
NOT block `KNIGHT IS YOU` — those are different predicates.

If three clauses produce `KNIGHT IS WALL`, `KNIGHT IS WALL`, `KNIGHT IS
NOT WALL`: NOT still wins. It's not a vote; it's an override.

### Multiple NOT clauses

`KNIGHT IS NOT WALL` and `BISHOP IS NOT WALL` are independent — each
applies to its own SUBJECT. No interaction.

`KNIGHT IS NOT WALL` and `KNIGHT IS NOT YOU` are independent — different
predicates. Both apply.

`KNIGHT IS NOT WALL` and `KNIGHT IS NOT NOT WALL` — the second is
invalid (double-NOT not allowed in v1). Only the first applies.

## Composition rules

- **NOT applies to the predicate immediately to its right.** In a
  multi-AND `pred-group` like `WALL AND NOT YOU`, the NOT applies to
  YOU only. The atomic rules are `WALL` (positive) and `NOT YOU`
  (negative).
- **NOT wins contradictions.** As above.
- **NOT does not chain.** `KNIGHT IS NOT WALL` does not produce
  `KNIGHT IS WALL'`; it produces `KNIGHT IS NOT WALL`, which means "this
  predicate's effect is explicitly off for this subject."
- **NOT during the resolution pass.** The parser collects all clauses,
  atomizes them, then groups by (subject, predicate). Within each group,
  if any atom is negative, the predicate is off for that subject. If all
  atoms are positive, the predicate is on.
- **Order independence.** NOT clauses are not ordered. `KNIGHT IS WALL`
  appearing earlier on the board does not pre-empt `KNIGHT IS NOT WALL`
  appearing later — the final state is governed by the override-wins
  rule, regardless of board layout order.

## Why it's interesting

NOT is the **override / sabotage** mechanism. Critical for puzzles
because it lets the player:

- **Defang a hostile clause** without destroying it. If the opponent has
  set up `WHITE_KNIGHT IS WALL` to paralyze the player's knight, the
  player can answer with a `WHITE_KNIGHT IS NOT WALL` clause elsewhere
  on the board. The original clause stays intact; the NOT-clause defeats
  it. The opponent must then either destroy the player's NOT-clause or
  rearrange around it.
- **Strip default behavior.** `KING IS NOT YOU` removes the king's
  implicit royalty without binding a new YOU somewhere else (see
  predicate_you.md edge cases).
- **Express explicit exceptions.** `PAWN IS WALL` plus `BLACK_PAWN IS NOT
  WALL` walls only white pawns.

The grammar move "construct a NOT clause to neutralize an opposing
clause" is the most direct counter-play in this category.

## Example sentences

```
[SUBJECT KNIGHT] [IS] [NOT] [WALL]
  → knights are explicitly not walls, overriding any other clause that
    says they are.

[SUBJECT KING] [IS] [NOT] [YOU]
  → the king is explicitly not royal. With no other YOU-clause for the
    side, the side has no royal piece (degenerate; see predicate_you.md).

[SUBJECT QUEEN] [IS] [WALL] [AND] [NOT] [YOU]
  → queens are walls, AND queens are not royal (irrelevant if no other
    clause made them royal, but covers the case where the queens
    inherited royalty via category).
```

## Example puzzle

```
 . . . . k . . .
 . [SUBJECT KNIGHT] [IS] [WALL] . . .
 . . . . . . . .
 . . . . . . . .
 . . N . . . . .
 . . . . . . . .
 . . . . . . . .
 . . . . K . . .
```

White has king on e1, knight on c4, the clause "KNIGHT IS WALL" on rank
7. Black king on e8.

The knight on c4 is walled. White cannot move it.

White has, off-screen, three additional rule-pieces in their starting
hand (this requires a piece-hand rule, which is a separate feature; if
not available, the pieces are already on the board in a queue area).
White places NOT and WALL onto squares — say, b3 and d3 — and slides
their existing SUBJECT(KNIGHT) tile (which... white doesn't own; the
parser doesn't care about ownership) to bracket them.

OK, this puzzle requires more setup. Simpler version:

```
 . [SUBJECT KNIGHT] [IS] [NOT] [WALL] . .
 . . . . . . . . .
 . [SUBJECT KNIGHT] [IS] [WALL] . . .
 . . . . . . . . .
 . . N . . . . . k
```

Two clauses for knights. NOT wins. Knight is not walled. Knight can
move freely; deliver mate.

The pedagogical point: the player must spot the NOT-clause and recognize
that the positive clause is overridden, so the knight is movable. Easy
to miss; that's the puzzle.

## Where it shines

- Override puzzles where a hostile rule must be defeated, not deleted.
- Composing predicates where some apply and some don't.
- Late-game positions where multiple clauses overlap and the player must
  parse the residual effects.

## Where it's awkward

- **Double-NOT.** Forbidden in v1, but tempting. `NOT NOT WALL = WALL`
  is grammatically pleasant but semantically a tautology (it's just
  `WALL` with two extra tokens). v1 keeps it forbidden; no benefit.
- **NOT on a category.** `LEAPER IS NOT WALL` strips wall from any
  leaper. If `KNIGHT IS WALL` is active, and knights are in the LEAPER
  category, do the knights remain walled? Per the override-wins rule:
  NOT-clauses win pairwise per (subject, predicate). But the subjects
  are different — `KNIGHT` and `LEAPER`. v1: category and specific are
  treated as distinct subjects for override resolution. So `KNIGHT IS
  WALL` and `LEAPER IS NOT WALL` produce: knights are walled (via the
  KNIGHT clause), and leapers other than knights are not walled (via
  the LEAPER clause's atomized expansion). The KNIGHT-specific clause
  "wins" by being more specific. This contradicts the
  positionless-NOT-wins rule; document the exception clearly.

  Actually, on reflection: v1 should just say **specific overrides
  category-NOT only if there's no specific NOT.** Resolution:

  1. If `KNIGHT IS NOT WALL` exists (specific NOT): knights not walled.
  2. Else if `KNIGHT IS WALL` exists (specific positive): knights
     walled.
  3. Else if `LEAPER IS NOT WALL` exists (category NOT): knights not
     walled.
  4. Else if `LEAPER IS WALL` exists (category positive): knights
     walled.
  5. Else: default behavior.

  Specific-NOT > specific-positive > category-NOT > category-positive
  > default. This is the v1 precedence.

- **NOT inside a complex pred-group.** Parsing `WALL AND NOT WALL` is
  pedantically valid — two atoms, one positive one negative, override
  wins, predicate off. Useless but consistent.

## Engine dependencies

- `VariantId::Baba`.
- The rule-effect registry must distinguish positive and negative atoms
  and apply the override-wins rule.
- The atomizer (subj-group × pred-group expansion) must propagate the
  NOT flag onto each generated atom.

## New features required

- **`Modifier::Not`** in the RulePiece enum.
- **Atom = (subject, predicate, polarity)** type.
- **Override-wins resolver** in the rule-effect registry. Specific-NOT
  > specific-positive > category-NOT > category-positive precedence.

## FEN encoding

```
(R=NOT)
```

No payload.

## Open questions

1. **NOT on the subj-group side.** Forbidden in v1 (NOT cannot negate a
   noun). Future extension: `NOT KNIGHT IS WALL` could mean "everything
   that is not a knight is a wall," which is enormously powerful and
   probably belongs in a follow-up batch.
2. **Multiple NOT-clauses with different predicates.** Independent and
   stackable. No interaction.
3. **NOT and category-membership chains.** Per the precedence above, but
   tested in edge cases like "knight IS YOU, LEAPER IS NOT YOU" — which
   wins for knights? Specific (KNIGHT) wins, knights are royal. Document
   thoroughly.
4. **NOT capture.** Capturing the NOT-token immediately reverts its
   negation: the underlying positive clause (if any) re-activates. This
   is the offensive move "take the NOT to re-enable my paralysis clause
   on your knight." Tactically expressive.
