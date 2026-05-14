# Baba Is You — Rules-as-Pieces

> The board is the rulebook. Rearrange the grammar, rewrite the game.

## The manifesto

In every other fairy variant in this engine, a piece is a *noun*. It has a
location, a color, a movement pattern, maybe a payload. It does what it does
because the engine has hard-coded what it does.

Baba Is You pieces are different. A Baba Is You piece is a *token*. By
itself it is inert: a SUBJECT tile sitting alone on the board does nothing.
But arranged into a *clause* — a horizontal or vertical run of tokens that
parses as a grammatical sentence — those same tiles rewrite the engine's
rules for the duration of the position.

```
[SUBJECT KNIGHT] [IS] [WALL]
```

While that clause sits intact on the board, every knight (of either color)
is **impassable terrain**. Capture the IS off the line and the clause
dissolves; the knights become knights again. Push a different SUBJECT into
the line and now bishops are walls instead.

The puzzle isn't *checkmate the king*. The puzzle is *find the sequence of
piece moves that rearranges the sentence into something solvable, without
the opponent rewriting the sentence away from you first*.

## The grammar specification

The parser scans the board for **clauses**. A clause is a maximal run of
rule-tokens in either of the eight king-radii — but only the four orthogonal
ones in v1, because diagonal sentences invite ambiguity (which diagonal? read
which way?). Diagonals are deferred.

The grammar:

```
clause   := subj-group VERB pred-group
subj-group := SUBJECT (AND SUBJECT)*
pred-group := [NOT] PREDICATE (AND [NOT] PREDICATE)*
```

`SUBJECT`, `VERB`, `PREDICATE`, `NOT`, `AND` are token-kinds, not specific
tiles. Each rule-piece carries a `token_kind` field telling the parser which
slot it fills.

Currently exactly one VERB exists (IS), so VERB is effectively a literal.
Future verbs (HAS, MAKE, EATS) would slot in identically.

A clause is **valid** iff it parses against the grammar above with no leftover
tokens. An invalid run (e.g. two IS in a line, or no SUBJECT) is **discarded
silently** — the tiles sit there doing nothing until rearranged. No errors;
no penalties.

### Parsing direction

For a horizontal run, parsing is left-to-right. For vertical, top-to-bottom.
The board has no "front" — both halves of a clause read the same direction —
so a SUBJECT on the left becomes the subject of the sentence regardless of
whose turn it is.

### Token adjacency

"Adjacent" means **directly orthogonal, no empty square or non-token piece
between**. A chess piece or empty square breaks the run. Two clauses on the
same row separated by a knight are two separate clauses.

A rule-token *is* a piece — it occupies a square, moves like a piece (slow,
one-step orthogonal for SUBJECT/PREDICATE; the verb IS is immobile and only
relocates by being shoved), can be captured. Capture removes the token, which
breaks any clause it participated in.

## The variant flag

These pieces require `VariantId::Baba` in the position's `variants` list.

This is not negotiable. The rule-piece machinery is a wholesale grammar
parser running between every half-move and rewriting the engine's behavior
based on what it finds. Standard chess positions cannot tolerate this
overhead, and a SUBJECT-KNIGHT tile in a non-Baba position would have
nowhere to dispatch to.

When `VariantId::Baba` is **active**:

- The grid-parser runs after every applied move.
- The set of valid clauses is recomputed.
- Each clause registers a **rule-effect** (e.g. "knights are walls") for
  the duration of the next legal-move generation.
- Effects from invalid clauses are discarded; effects from valid clauses
  are applied in **priority order** (see Composition rules below).

When the flag is **absent**:

- Rule-pieces are illegal in the FEN parse — reject the position.
- (Future: a "Baba-lite" flag could allow rule-pieces but treat them as
  inert decoration. Out of scope for v1.)

### Default behavior when no IS exists

A board with rule-pieces but no IS-token has no valid clauses. Every clause
needs a verb. With no clauses, the engine **falls back to standard chess
rules**: knights are knights, pawns capture diagonally, the king is YOU.

This is the puzzle's safety net. A solver who deletes every IS from the
board returns to a normal chess position (modulo whatever board geometry
the Baba flag also brings).

## Composition rules

Multiple valid clauses can apply to the same SUBJECT. The resolution order:

1. **Collect all clauses.** For each clause, expand the AND-conjunctions
   into independent SUBJECT-PREDICATE pairs. `KNIGHT AND BISHOP IS WALL AND
   ROYAL` becomes four atomic rules: knight-IS-wall, knight-IS-royal,
   bishop-IS-wall, bishop-IS-royal.
2. **Apply NOT.** Each atomic rule is either positive (`KNIGHT IS WALL`) or
   negative (`KNIGHT IS NOT WALL`). NOT applies to the predicate immediately
   to its right.
3. **Resolve contradictions.** If both `KNIGHT IS WALL` and `KNIGHT IS NOT
   WALL` are present, **NOT wins**. This is the inverse of Baba Is You's
   in-game rule, but the engine version is more useful for puzzles: NOT is
   an *override*, an explicit veto.
4. **Mutual exclusivity.** Some predicates are mutually exclusive (a piece
   can't be both WALL and YOU in a meaningful sense). v1: both apply
   simultaneously where compatible; conflicts are documented per-predicate.
   WALL+YOU = an immovable royal target, which is a degenerate but legal
   state.
5. **Default fallback.** SUBJECTs with no clause keep their default behavior
   (knights move like knights). YOU defaults to KING for the side whose
   king it is.

## The token index

The eight rule-pieces in this category, all sharing `VariantId::Baba`:

- [subject.md](subject.md) — the noun-token, parameterized by piece type.
- [copula_is.md](copula_is.md) — the verb tile, the engine of binding.
- [predicate_wall.md](predicate_wall.md) — "is impassable terrain."
- [predicate_you.md](predicate_you.md) — "is the win-condition holder."
- [modifier_not.md](modifier_not.md) — the negation token.
- [conjunction_and.md](conjunction_and.md) — joins SUBJECTs or PREDICATEs.
- [quote_bracket.md](quote_bracket.md) — paired tiles that turn a contained
  sentence into a literal piece description.
- [global_local_toggle.md](global_local_toggle.md) — flips rules between
  applying everywhere and applying only near their SUBJECT-tile.

Eight tokens is the minimum interesting grammar. Future expansions
(predicate STOP, predicate MOVE, conjunction OR, verb HAS) compose with
these without changing the parser shape.

## FEN summary

All rule-pieces use the `R=` per-square prefix to distinguish from chess
pieces:

- `(R=SUBJECT:KNIGHT)` — a SUBJECT token bound to the KNIGHT category.
- `(R=IS)` — the verb tile.
- `(R=WALL)` — a predicate.
- `(R=YOU)` — a predicate.
- `(R=NOT)` — a modifier.
- `(R=AND)` — a conjunction.
- `(R=QUOTE)` — one half of a bracket pair (both occurrences identical;
  parser pairs them by proximity).
- `(R=TOGGLE:GLOBAL)` or `(R=TOGGLE:LOCAL)` — the local/global flipper.

Board-level flag: `variants=baba` joins the existing comma-separated list.
