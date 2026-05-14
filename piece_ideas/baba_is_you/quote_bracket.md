# QUOTE (bracket pair)

> Paired tokens. The sentence between them is interpreted as a piece
> description and spawns a new ghost-piece at the bracket's midpoint.

## Role in the grammar

QUOTE is a **bracket** token. It does not occupy a slot in the normal
clause grammar; instead, the parser scans for **QUOTE-pair regions** and
treats their interior as **descriptive grammar** — a sentence that
describes a piece rather than rewriting an existing one.

Two QUOTE tokens on the same orthogonal run (with rule-tokens between
them, no chess pieces between them) form a paired region. Inside the
region, the parser interprets the contained tokens as a piece-description
and spawns a piece at the midpoint of the region.

The parser identifies QUOTE by `token_kind == Bracket, payload == Quote`.

## Inspiration

Baba Is You's TEXT-as-objects: in some levels, the text tiles themselves
become objects in the world. The conceptual leap "the rule *is* the
thing" is the inspiration. The chess version implements the most literal
form of this idea: a region of the board reads as a piece description
and *makes that piece*.

It is also a nod to Lisp's quote: data treated as code, code treated as
data.

## Mechanic

### Placement and movement

QUOTE tiles are placed in pairs. Each QUOTE is an individual movable
rule-token (Color::Neutral, one-step orthogonal). They are paired by
proximity: any two QUOTE tiles on the same orthogonal run with no other
QUOTE between them and no chess piece between them count as a pair.

A board may have multiple QUOTE pairs simultaneously, on different lines.
Three QUOTEs on one line pair the leftmost two; the third is unpaired
until its partner moves into reach.

### Parser interaction

For each detected QUOTE pair, the parser:

1. Extracts the tokens between the two QUOTEs.
2. Parses them as a **descriptive clause** with grammar:

   ```
   desc := SUBJECT (AND SUBJECT)*
   ```

   The descriptive clause is a list of SUBJECTs joined by ANDs. No VERB,
   no PREDICATE, no NOT. Just nouns.

3. The parser computes the **midpoint square** of the bracket region. If
   the QUOTEs are at squares A and B on a row, the midpoint is the square
   exactly between them (rounded toward A if the distance is even).
4. If the midpoint square is empty (no piece, no rule-token), the engine
   **spawns** a ghost-piece there. The ghost-piece's type is determined
   by the descriptive clause:

   - Single SUBJECT: the ghost is a piece of that type.
   - Multi-SUBJECT via AND: the ghost is a **compound piece** that
     inherits the movement-union of all listed types. `KNIGHT AND ROOK`
     produces a knight-rook hybrid (an empress, in fairy-chess terms).

5. The ghost-piece is Color::Neutral. It behaves as a piece (can be
   captured, blocks lines, is subject to predicate clauses) but cannot
   be moved by either player — it only exists by the QUOTE's grace.

### Removing the bracket

If either QUOTE is captured or moved out of pair-range, the bracket
dissolves and the ghost-piece **disappears at the next parser pass**.
Removed from the board.

If the ghost-piece had captured pieces while alive, those captures are
not reversed — only the ghost itself vanishes.

### Re-spawning

If a QUOTE pair re-forms in the same configuration, a new ghost-piece
spawns. The previous ghost is not "remembered" — each parser pass freshly
creates ghosts from current QUOTE pairs.

If the midpoint square is **occupied** at parse time (a real piece has
moved there, or another ghost is there from a different QUOTE pair), no
ghost spawns. The QUOTE pair is valid but dormant. As soon as the square
clears, the ghost respawns.

## Composition rules

- **Descriptive grammar is strict.** Only SUBJECT-AND-SUBJECT inside a
  QUOTE pair. Putting an IS, PREDICATE, NOT, etc. inside the brackets
  produces an unparseable description; no ghost spawns.
- **Multiple QUOTE pairs.** Independent. Each pair spawns at its own
  midpoint.
- **Nested QUOTE pairs.** v1 forbids nesting. If a QUOTE appears between
  two other QUOTEs, the parser is confused; v1 rule: the innermost pair
  is processed first, and outer QUOTEs that lose their pair-mate to the
  inner pair are unpaired. Avoid nesting in v1 puzzles.
- **Ghost-piece is subject to rule-pieces.** A ghost knight is bound by
  `KNIGHT IS WALL` clauses just like a real knight. This is the
  "spawn-then-paralyze" combination: a QUOTE bracket spawns a piece, and
  another active clause immediately walls it.
- **Ghost-piece's color.** Neutral. Implies: captureable by both sides,
  not movable by either, not in either side's move-gen, never YOU
  (default). To make a ghost royal, declare `SUBJECT(KNIGHT) IS YOU` and
  the ghost-knight inherits royalty along with the real ones — which
  has weird consequences (a Neutral royal piece is essentially a piece
  *both* sides must capture, with the first to reach it winning).
- **Compound movement of AND-ghosts.** `KNIGHT AND ROOK` ghost moves like
  the union of knight + rook moves. The engine's piece-type union logic
  is a new feature.

## Why it's interesting

QUOTE is the **piece factory**. It introduces:

- **Spawned pieces from rule-piece configurations.** Without QUOTE, all
  pieces come from the FEN at game-start (modulo promotion). QUOTE makes
  the rule-pieces themselves a source of new pieces.
- **Compound pieces on demand.** A puzzle can require the solver to
  *construct* a centaur (knight + bishop) by arranging the right
  QUOTE-bracketed sentence — and place the centaur in a specific square
  by choosing the QUOTE positions to determine the midpoint.
- **Disposable pieces.** The ghost vanishes when the bracket breaks. A
  player can build a temporary attacker, use it, then dissolve it (e.g.
  by moving a piece to capture one QUOTE).

This is the most chaotic rule-piece. Puzzles using QUOTE will tend
toward the highly constructed — players are building things, not just
moving things.

## Example sentences

```
[QUOTE] [SUBJECT KNIGHT] [QUOTE]
  → ghost knight spawns at the midpoint.

[QUOTE] [SUBJECT ROOK] [AND] [SUBJECT KNIGHT] [QUOTE]
  → ghost empress (rook-knight compound) spawns at midpoint.

[QUOTE] [SUBJECT QUEEN] [AND] [SUBJECT GOBLIN] [QUOTE]
  → ghost queen-goblin hybrid. Composite of queen-movement and
    goblin-payload.
```

## Example puzzle

```
 . . . . k . . .
 . . . . . . . .
 . . . . . . . .
 . [QUOTE] [SUBJECT BISHOP] . . [QUOTE] .
 . . . . . . . .
 . . . . . . . .
 . . . . . . . .
 . . . . K . . .
```

White king e1, black king e7. A QUOTE pair on rank 5 with SUBJECT(BISHOP)
between them. Midpoint of QUOTE-at-b5 and QUOTE-at-g5 is around d5/e5.

The clause spawns a ghost-bishop at the midpoint.

The puzzle: arrange for the ghost-bishop to deliver mate, by choosing
where it spawns. The player moves their pieces such that one of the
QUOTEs shifts position, changing the midpoint, until the ghost-bishop
appears on the diagonal it needs.

E.g. capture one QUOTE with a pawn, then in subsequent moves re-spawn
the ghost-bishop at a specific square by re-placing a QUOTE — well, the
player can't place pieces from a hand in v1, but the player can *slide*
remaining QUOTE tokens to change the geometry.

The puzzle becomes: geometric calculation of "where does the midpoint
land if I slide QUOTE one square?" and "where do I need the bishop to
deliver mate from?"

## Where it shines

- Construction puzzles. Build a piece, position it, attack with it.
- Composite-piece puzzles using AND inside QUOTE.
- Spawn-paralyze combos: QUOTE creates a piece, another clause walls
  it, an opponent must navigate around the new obstacle.

## Where it's awkward

- **Midpoint computation is geometric.** When two QUOTEs are equidistant
  (even-length region), pick one direction by convention (engine-side:
  always round toward the lower-coordinate QUOTE). Document this
  carefully because puzzles depend on exact midpoint.
- **Empty descriptions.** QUOTE-QUOTE with nothing between them produces
  no description. No ghost. Dormant pair, just two tokens facing each
  other.
- **Massive ghosts.** `QUOTE ... QUEEN AND ROOK AND BISHOP AND KNIGHT
  ... QUOTE` produces a ghost that moves like the union of all four —
  effectively a chancellor-empress-queen mega-piece. v1 allows it; the
  movement union is well-defined. Puzzle designers manage tempo.
- **Pieces blocking the description.** A chess piece sitting in the
  QUOTE-region breaks the run; the bracket is dormant until the piece
  moves. Useful for puzzles where a piece blocks the spawn.
- **Two pairs sharing a QUOTE.** Three QUOTEs on a line: the parser
  pairs by proximity (leftmost two), the third is unpaired. If the
  middle QUOTE moves, repair occurs. Document the rule clearly so
  puzzles don't accidentally trigger pair-flipping.

## Engine dependencies

- `VariantId::Baba`.
- The parser's QUOTE-pair detection routine.
- The descriptive grammar parser (a subset of the main grammar).
- A **piece-type union** mechanism that produces a movement set from
  multiple piece-types — needed for compound ghosts. New feature.
- **Ghost-piece registry** in the rule-effect output, separate from the
  main grid (or as a parallel `is_ghost: bool` field on Square — see
  open questions).

## New features required

- **`Bracket::Quote`** in the RulePiece enum.
- **Descriptive-grammar sub-parser.**
- **Ghost-piece spawning hook** in the parser pass. The ghost is
  materialized into the grid at the midpoint each pass; if the previous
  pass had a ghost there and the current pass's midpoint matches, the
  ghost is treated as continuous (no respawn animation, no state
  change).
- **Piece-type union for movement.** A new fairy mechanism. Could share
  code with future fairy compound pieces.

## FEN encoding

```
(R=QUOTE)
```

No payload. Both halves of a pair are identical tiles; the parser pairs
them by spatial reasoning.

For ghost-pieces (which exist by virtue of QUOTE pairs but are
*temporary*), v1 does not serialize them in FEN — they're recomputed
from the rule-tokens at FEN-load. This keeps the FEN canonical: only
the rule-tokens are serialized; the ghost-state is derived.

## Open questions

1. **Is the ghost actually on the grid?** Two options: (a) materialize
   the ghost into `Square.piece` so all existing move-gen / threat code
   sees it; (b) keep a separate ghost-registry the relevant code
   consults. v1 leans (a): materialize into the grid for compatibility,
   mark with `Square.ghost_origin: Option<(Coord, Coord)>` pointing at
   the QUOTE pair that spawned it.
2. **Midpoint tie-breaking.** Round toward the lower-coordinate QUOTE.
   For a 4-square region between QUOTEs at file a and file d, the
   midpoint is b or c — pick b (closer to a). Documented.
3. **Promotion of ghosts.** A ghost-pawn (created via `QUOTE PAWN
   QUOTE`) cannot move, so cannot reach promotion rank by itself. But
   captures of ghosts during normal play: irrelevant, the ghost just
   disappears. Document that ghost-pawns never promote.
4. **Composite ghost interactions with category SUBJECTs.** Is a
   knight-rook ghost a member of the LEAPER category (because of the
   knight half) or RANGED (because of the rook half) or both? v1: both.
   Multi-category membership.
5. **Color-qualified ghosts.** `QUOTE WHITE_KNIGHT QUOTE` — does it spawn
   a white knight (not Neutral)? Useful for puzzles. v1: yes, color-
   qualified descriptions spawn colored ghosts. The "Neutral default"
   applies only when no color is specified.
6. **Composite ghosts as YOU.** If `KNIGHT IS YOU` is active and a
   QUOTE pair spawns a ghost knight, the ghost is YOU. Capturing it
   ends the game. This is wildly exploitable in puzzles — design with
   care.
