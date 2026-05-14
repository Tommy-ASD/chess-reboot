# Spatial Puzzle Pieces

> Pieces are rules. The board is the puzzle.

## Manifesto

The pieces in this directory do not behave like chess pieces. They behave
like **tiles in a spatial puzzle game** — Baba Is You's word-rules, Into
the Breach's forced-relocation telegraphs, Patrick's Parabox's recursive
containment, Stephen's Sausage Roll's grid-aligned causality.

A classical chess piece *is an actor*: it has a turn, it moves, it
captures, it is captured. A spatial puzzle piece *is a local rule*: it
sits in one square and bends the geometry of every other piece's move
around it. Some never move at all. Some don't even occupy a single
square. Capturing one is sometimes the whole point — and sometimes
impossible until you've rearranged the board around it.

## Why "rules as pieces" matters

Three things fall out of treating pieces as localized geometric rules:

1. **Composition.** Stacking two rule-pieces produces emergent behaviour
   the designer didn't have to encode. Two Pivots overlapping cancel.
   Three Pivots flip. A Fold inside a Tessera's footprint creates a
   moving topological seam. Combinatorial expressiveness from a small
   rule alphabet — Baba Is You's lesson.
2. **Puzzle-friendliness.** A position with these pieces is a *spatial
   problem* in the cartographic sense. The solver may need pencil and
   paper. The board state is small (FEN-serializable, deterministic),
   but its reachable-position graph is structurally rich. Composers
   can craft "mate in 4 if you fold the c-file" problems.
3. **Engine honesty.** Every rule is local, deterministic, and serializes
   to FEN. No hidden state, no random tables. The engine doesn't need
   physics — it needs careful resolution order. That order is the
   interesting part of the implementation.

## Design constraints (all pieces obey)

- **Deterministic.** No randomness anywhere.
- **FEN-serializable.** Every bit of state — pivot axis, anchor target,
  fold line, recorded delta — lives in the parenthesized payload.
- **Local rule, global consequence.** Each piece states a small
  predicate about its neighbourhood; the consequences may be felt
  across the board, but the rule itself fits on a line.
- **No emojis, no narrative.** These are geometric primitives.

## Index

| Piece | One-line essence |
|---|---|
| [pivot](pivot.md) | Rotates the move directions of nearby pieces 90° around a chosen axis. |
| [fold](fold.md) | Creases the board along its rank or file; pieces crossing the crease re-emerge on the mirror side. |
| [anchor](anchor.md) | Leashes one named enemy piece — every enemy move is matched by an equal-and-opposite move the Anchor's owner chooses. |
| [tessera](tessera.md) | Occupies a 2×2 footprint. Slides as a block. Pushes whatever's in its path Sokoban-style. |
| [echo](echo.md) | Records the last move played on the board. On its turn, replays that delta onto any friendly piece. |
| [lens](lens.md) | Compacts a chosen row or column — adjacent same-colour pieces fuse into stacks. |
| [tide](tide.md) | Every fourth move, all pieces on dark squares shift one square toward the nearest Tide; next cycle, light squares shift away. |
| [recursion](recursion.md) | Any piece ending its move adjacent to a Recursion immediately gets a second half-range move. Chains. |

## Recommended reading order

Start with **pivot** (simplest local geometric rule) and **fold**
(simplest topological one). Then **tessera** for footprint pieces.
**Recursion** and **echo** introduce move-as-resource. **Anchor** and
**tide** introduce inter-turn constraints. **Lens** is the wildest.

## Engine surface area

Several of these pieces require new resolution-order machinery —
specifically a **rule-evaluation phase** that runs after move legality
but before move application, where local rules (Pivot, Fold, Anchor,
Recursion) get to mutate or veto the proposed move. Each piece doc
calls out exactly what new phase it needs. Designing the resolution
order up-front, across all eight, is the real implementation work.
