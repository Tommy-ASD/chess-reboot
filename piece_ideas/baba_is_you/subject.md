# SUBJECT

> A noun-token. Names the piece type or category that a rule applies to.

## Role in the grammar

SUBJECT is the **noun** slot. Every well-formed clause begins with one or
more SUBJECTs (joined by ANDs). The SUBJECT names the piece-type — or
piece-category — that the predicate of the clause will rewrite.

Without a SUBJECT, there is no clause. A predicate floating alone on the
board does nothing; predicates need a subject to attach to.

A SUBJECT tile carries a **noun payload**: the specific piece-type or
category it names. `SUBJECT(KNIGHT)` and `SUBJECT(PAWN)` are distinct tiles
on the board, distinguished by their payload in FEN and by glyph in
rendering.

## Inspiration

Baba Is You's noun tiles — BABA, ROCK, FLAG, WALL — each name a class of
in-game object. The rule-engine identifies them by their `NOUN` token-kind
and reads the payload (BABA vs ROCK) to find the matching world-objects.

The chess version is identical: the parser sees a SUBJECT token, reads its
payload, and the predicate of the same clause rewrites the behavior of
every chess piece of that type on the board (regardless of color, unless
the payload itself encodes a color).

## Mechanic

### Placement

A SUBJECT tile is placed on the grid the same way any chess piece is —
during board setup (FEN) or by being captured into a hand and dropped (if
the variant allows). It occupies a square. It blocks line-of-sight for
gliders. It can be captured by a normal piece move.

### Movement

The SUBJECT tile can be moved by a player as their half-move, but **only
one step orthogonally** to an empty adjacent square. This makes
rearrangements slow and visible — the player has to spend turns to shift the
grammar. (Standard pieces remain free to move at full speed; this
asymmetry is the budget the puzzle designer trades against.)

### The noun payload

The payload `noun: NounRef` is either:

- A specific piece-type: `KNIGHT`, `PAWN`, `ROOK`, `BISHOP`, `QUEEN`,
  `KING`, `GOBLIN`, `SKIBIDI`, etc. The rule binds only to pieces of that
  exact type.
- A category: `ROYAL` (any piece currently flagged YOU), `RANGED` (gliders),
  `LEAPER` (knight-class), `MINOR`, `MAJOR`. The category set is a finite,
  engine-defined enum. Categories let rules reference dynamically-changing
  groups.
- A color-qualified type: `WHITE_KNIGHT`, `BLACK_PAWN`. Letting a clause
  affect only one side's pieces.

The payload is **fixed at placement time** and travels with the tile. It
does not mutate — a SUBJECT(KNIGHT) is always SUBJECT(KNIGHT) until
captured off the board.

### Parser interaction

The parser scans every orthogonal run of rule-tokens on the board. When it
sees a SUBJECT in the first position (or after an AND that itself follows a
SUBJECT), it accumulates it into the clause's `subj-group`. The clause is
valid only if a VERB follows the (possibly multi-SUBJECT) group.

A SUBJECT not followed by a VERB on the same orthogonal run is **inert**.
No effect; not an error.

## Composition rules

- **Multiple SUBJECTs in one clause** (via AND): the predicate applies to
  the union of the named types. `KNIGHT AND BISHOP IS WALL` makes both
  knights and bishops impassable.
- **Multiple clauses naming the same SUBJECT**: each clause's effects stack.
  `KNIGHT IS WALL` and `KNIGHT IS YOU` together produce knights that are
  both impassable *and* royal — capturing any knight ends the game and the
  knights can never move (a fun lockout).
- **Category vs specific**: when both `KNIGHT IS WALL` and `LEAPER IS NOT
  WALL` are active, the NOT clause wins (see modifier_not.md). Category
  rules and specific rules are not ordered by specificity — NOT is the only
  override mechanism.
- **Color-qualified vs unqualified**: independent. `KNIGHT IS WALL` plus
  `WHITE_KNIGHT IS NOT WALL` leaves white knights movable and black
  knights impassable.

## Why it's interesting

The SUBJECT tile is the **anchor** of every rule. It is what makes the
grammar visible: the player can see, on the board, that "knights are
affected" because there is a SUBJECT(KNIGHT) tile sitting in a clause. The
mapping from rule-text to board-position is literal.

Slow movement (one step orthogonal) means changing the subject of a clause
is a multi-turn commitment. The player has to plan: "I want bishops to
become walls instead of knights, so I need to spend three turns sliding
SUBJECT(BISHOP) into the clause line, while my opponent does what?"

## Example sentences

```
[SUBJECT KNIGHT] [IS] [WALL]
  → every knight is impassable terrain.

[SUBJECT PAWN] [AND] [SUBJECT KNIGHT] [IS] [YOU]
  → every pawn and every knight is a win-condition piece.
    Capture any one of them, the side with that piece loses.

[SUBJECT ROYAL] [IS] [NOT] [YOU]
  → whichever piece is currently YOU is no longer YOU.
    Strips win-condition status (degenerate — no royalty means
    the engine falls back to king-by-default? See open questions).
```

## Example puzzle

A minimal 5×5 corner of the board:

```
 . . . . k
 . R . . .
 . . . . .
 . [SUBJECT KNIGHT] [IS] [WALL] .
 K N . . .
```

White to move. White's king is on a1, white knight on b1, black king on e5.
The clause "KNIGHT IS WALL" makes the knight on b1 a wall — it cannot move.

The puzzle: rearrange the clause to free the knight, then deliver mate.

Solution: slide SUBJECT(KNIGHT) one square (replacing it with, say, a
SUBJECT(ROOK) that was off-screen — or simply destroying it via the white
rook). With the clause broken, the knight moves to c3, and the rook on b4
delivers mate on the back rank. The point: the chess problem and the
grammar problem are the same problem.

## Where it shines

Puzzles where the player must *unlock* their own pieces by deleting or
displacing a SUBJECT that was paralyzing them. Or, conversely, puzzles
where the player must *bind* an opponent's piece by sliding a SUBJECT into
position.

## Where it's awkward

- **Payload combinatorics.** Every piece type the engine knows about needs
  a corresponding SUBJECT tile glyph and FEN encoding. With 6 standard + 5
  fairy pieces, that's 11 distinct SUBJECT tiles — plus the categories.
  The FEN gets verbose. Manageable; just verbose.
- **"What is a knight?"** Once `KNIGHT IS WALL` and `BISHOP IS KNIGHT` are
  both active, what is a piece-type? v1 punts: predicates that reassign
  type are explicit (and currently the only one, KNIGHT-class transform,
  isn't in this batch). SUBJECT matches the *original* piece-type at the
  moment of binding.
- **Self-referential SUBJECT.** `SUBJECT(SUBJECT) IS WALL` — would the
  SUBJECT tile itself become a wall? v1 forbids meta-SUBJECTs; the payload
  must be a chess-piece type or category, not another token-kind.

## Engine dependencies

- `VariantId::Baba` must be active.
- The grammar parser (see README.md) runs after every move.
- A piece-type registry the SUBJECT payload can name.
- A category registry (ROYAL, RANGED, LEAPER, …) — engine-defined enum.

## New features required

- **`RulePiece` enum** with `Subject { noun: NounRef }` as one variant.
- **`NounRef` enum** covering specific types, categories, and color-
  qualified types.
- **Grammar parser** that scans the board and produces a list of valid
  clauses. Lives at `engine/src/baba/parser.rs` (proposed).
- **Rule-effect registry** the parser writes into, that the rest of the
  engine reads when generating moves / resolving captures.
- **SUBJECT movement modifier** — one-step orthogonal as a single move-gen
  case. Reuse king movement for the geometry.

## FEN encoding

```
(R=SUBJECT:KNIGHT)
(R=SUBJECT:WHITE_PAWN)
(R=SUBJECT:LEAPER)
```

Payload after the colon, uppercase, matches the engine's piece-type and
category enum names.

## Open questions

1. **Should SUBJECT be capturable by opponents?** Yes in v1 — captures are
   the main way to break unfavorable clauses. But this means the player
   placing a SUBJECT must defend it.
2. **What if the noun payload names a piece type that doesn't exist on the
   board?** Clause is valid but has no effect (no pieces to bind). Document
   as "vacuous truth."
3. **Promotion interaction.** A pawn promotes to a knight while `KNIGHT IS
   WALL` is active. Does the new knight become a wall? v1: yes — the rule
   is evaluated continuously, not at piece-creation time.
4. **Categories vs piece-type after rewrites.** If `KNIGHT IS WALL` is
   active and another clause says `WALL IS YOU`, do walled-knights become
   YOU? The chain is real and probably desirable; v1 says yes, document as
   "transitive predicate evaluation." See conjunction_and.md for how this
   composes with multi-predicate clauses.
