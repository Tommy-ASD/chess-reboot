# IS (the copula)

> The verb-tile. Binds a SUBJECT to a PREDICATE and turns a row of tokens
> into a rule.

## Role in the grammar

IS is the **VERB**. It is the linchpin of every clause. A clause is valid
iff exactly one IS sits between a `subj-group` and a `pred-group`. The IS
is what *makes* the clause a clause; remove it, and the surrounding tokens
become inert decoration.

In v1, IS is the only verb in the grammar. Future verbs (HAS, MAKE, EATS)
would slot into the same VERB position, with different semantics for what
they do with the bound noun and predicate.

The parser identifies IS by its `token_kind == Verb`. There is no payload —
all IS tiles are interchangeable.

## Inspiration

Baba Is You's IS tile is the entire game's central mechanic. Every rule
the player creates passes through an IS. The chess version preserves this:
removing the IS from a board removes every active rule simultaneously, a
single-move grammar bomb.

## Mechanic

### Placement

An IS tile occupies a square on the board. It is colored Neutral — neither
player owns it — and thus cannot be moved by either side as a normal piece
move. Instead, IS is **immobile**. It is relocated only by:

- **Being pushed.** A chess piece moving onto IS's square instead slides
  IS one square further in the same direction (if that destination square
  is empty; otherwise the move is illegal). The pusher does not occupy
  IS's old square — IS moves out of the way, and the pusher takes the
  intermediate square... wait, see below.
- **Being captured.** A piece may capture IS by moving onto its square in
  a capture-context (e.g. a pawn diagonal capture, a knight leap to an
  attacked square). Capture removes IS from the board.

Pushing semantics (clarified): when piece P moves to IS's square via a
non-capture move, IS slides one square in the direction of P's motion, and
P occupies IS's former square. If the slide-destination is blocked (by a
piece, board edge, or another rule-token), the original move is illegal.

This makes IS feel like a heavy boulder: you can shove it around with
slow, deliberate moves, but you can also just kill it with a knight.

### Parser interaction

When the parser scans a run of tokens and encounters an IS, it splits the
run into two halves: everything before IS (must parse as `subj-group`),
everything after IS (must parse as `[NOT] pred-group`). If both halves
parse, the clause is valid and registers its effects.

A run containing **two** IS tiles is **invalid** — the grammar admits
exactly one verb per clause. Both halves' rules are silently discarded
until one IS is removed.

### When IS is captured

Capturing IS removes the tile, immediately invalidating every clause it
was holding together. The rule-effects from those clauses are unregistered
at the start of the next move-gen pass.

This is the **grammar bomb**: a single IS may anchor multiple rules (one
clause running horizontally, one vertically through the same IS — see open
questions). Capturing the shared IS collapses both clauses at once.

### When IS is pushed off the line

If a piece pushes IS one square sideways, the IS is no longer in the
original clause's run (it's in an adjacent row/column). The original
clause loses its verb and goes inert. A *new* clause may form along
whichever row/column IS now sits in — if the parser finds a valid
SUBJECT-IS-PREDICATE arrangement there.

This is the puzzle-richness lever: one push, two simultaneous grammar
changes.

## Composition rules

- **One IS per clause.** Hard constraint. Two IS in a single orthogonal
  run produces no valid clause from any subspan.
- **IS at multiple intersections.** A single IS can anchor up to two
  clauses (one horizontal, one vertical) because each orthogonal direction
  parses independently. The two clauses share the IS but are otherwise
  independent.
- **Adjacent IS tiles.** Two IS tiles next to each other — `[IS][IS]` — is
  not a valid clause and both are inert. Even with a SUBJECT and PREDICATE
  around them: `SUBJECT IS IS WALL` doesn't parse.
- **IS at the end of a run.** `SUBJECT(KNIGHT) IS` with nothing to the
  right of IS is invalid (no `pred-group`). Inert.
- **IS at the start of a run.** `IS WALL` with nothing to the left is
  invalid (no `subj-group`). Inert.
- **Pushing during a turn.** Pushing IS happens *during* the piece move
  half-turn. The parser re-runs *after* the move applies, with IS in its
  new position. Both the loss-of-old-clause and gain-of-new-clause happen
  at the same parser invocation; there is no half-state.

## Why it's interesting

IS is the **central lever** of the entire rule-piece category. Most
puzzle-solving moves in this variant will be:

- Pushing IS into a new line to bind a different SUBJECT or PREDICATE.
- Capturing IS to nuke an unfavorable rule entirely.
- Defending IS so the opponent cannot break a rule favorable to you.

The asymmetry between "push IS one square" (slow, costs a move, both
players see it coming) and "capture IS" (fast, decisive, ends the rule
immediately) creates the offense/defense rhythm.

## Example sentences

```
[SUBJECT KNIGHT] [IS] [WALL]
  → knights are impassable terrain.

[SUBJECT QUEEN] [IS] [YOU]
  → the queen is the win condition.
    Capture the queen, win.

[SUBJECT PAWN] [AND] [SUBJECT KNIGHT] [IS] [WALL] [AND] [YOU]
  → pawns and knights are both walls and royal. Stalemate-tactic territory.
```

## Example puzzle

A 7-square horizontal slice:

```
 . . [SUBJECT KNIGHT] [IS] [WALL] . .
       a3              b3   c3
 . N . . . . .
 . . . . . . k
```

White has a knight on b1, black king on g4, the clause "KNIGHT IS WALL"
sitting on rank 3. The knight is paralyzed.

White also has a bishop on, say, h6 (not shown). Knight cannot move
(walled), bishop can.

The puzzle: bishop captures IS on b3 (assume the diagonal works out).
Clause dissolves, knight unfrozen, knight delivers mate on f6 next turn.

The grammar-puzzle is: which token to capture? IS is the most powerful
target because removing it collapses every clause. SUBJECT or PREDICATE
would only break this clause. (And black's defense, similarly, is to
threaten the bishop or push IS out of bishop range.)

## Where it shines

The single most expressive piece in the category. Almost every "grammar
move" in a Baba puzzle is doing something to an IS.

Puzzles where capturing IS is *not* the right move — because doing so
collapses a clause that was helping you — are especially good. The
player has to recognize that they want the rule to *stay*.

## Where it's awkward

- **Pushing semantics.** Pushing is borrowed from Baba Is You but doesn't
  generalize cleanly to chess piece movement. A knight that "lands on" IS
  hasn't pushed in a direction (knights leap). v1: knights and other
  leapers cannot push IS — they can only capture it. Pushing requires an
  adjacent-square move (king, pawn-forward, single-step moves).
- **Multiple IS on the board.** Permitted, and useful for multi-clause
  positions. But the parser's "exactly one IS per clause" rule means a
  player can sabotage their own rule by sliding a *second* IS into the
  line. Surprising but consistent.
- **IS adjacency with non-rule pieces.** A standard chess piece between
  two rule-tokens breaks the run. So a SUBJECT-PIECE-IS-PREDICATE
  arrangement isn't a clause; the piece sitting between SUBJECT and IS
  blocks parsing.
- **Capturable Neutral piece.** IS being color-Neutral but capturable is
  unusual. Either player can capture it on their turn. Fine; mirrors the
  duck-or-not asymmetry from plan 11.

## Engine dependencies

- `VariantId::Baba` active.
- The grammar parser at `engine/src/baba/parser.rs`.
- A push-resolution modifier in the movement stack (plan 10) — when a piece
  moves onto a rule-token's square via a non-capture move, the move
  resolves to "piece moves to that square, token slides one further in the
  same direction, if legal."
- Color::Neutral support (already present).

## New features required

- **`RulePiece::Is` variant.** No payload.
- **Push-resolution movement modifier.** Order 100–199 band per plan 10.
- **Clause invalidation on token-removal.** The rule-effect registry needs
  to recompute when any rule-piece moves or dies.

## FEN encoding

```
(R=IS)
```

No payload. Trivial.

## Open questions

1. **Pushing a chain.** Can a piece push IS, which pushes another rule-token,
   which pushes another? v1: no — pushing is one-deep. The destination of
   the slide must be empty. Cascading pushes are a Baba Is You feature but
   they explode the move-validation complexity for marginal benefit.
2. **Two clauses sharing an IS in different orientations.** As described
   above, allowed. Worth a test: a `+` shape of tokens with IS at the
   center forms two clauses simultaneously, both active.
3. **What does "the same clause" mean for rule-deduplication?** If a board
   has two identical clauses in two different rows, do they double-apply?
   v1: no — rules are de-duplicated at the (subject, predicate) atomic
   level. Two `KNIGHT IS WALL` clauses produce one rule, not two.
4. **Capturing IS with a non-pushable piece.** Knights, bishops, rooks
   capture IS normally (it's just a piece on a square). The "pushable
   only by adjacent-step pieces" rule is for non-capture moves only.
   Capture is unconditional.
5. **IS in promotion squares.** A pawn promotes by moving to the back rank.
   If that square holds IS, does the pawn push IS or capture it? v1:
   capture (a promotion is a piece-creating move that requires the square
   be free of *pieces*; IS is a token, not a piece, but for promotion
   purposes treat it as capture-only).
