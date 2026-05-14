# piece_ideas/

Design-doc files for every fairy chess piece that's been proposed for this
project but not yet built. One file per piece. Organized into folders by
the inspiration / design tradition that produced the idea, not by the
mechanic it lands on (mechanics cross-cut categories; inspiration is the
cleanest cut).

Each piece file follows roughly this skeleton — *Inspiration*, *Mechanic*,
*Why it's interesting*, *Example scenarios*, *Where it shines*, *Where
it's awkward*, *Engine dependencies*, *New features required*, *FEN
encoding*, *Open questions*. Some categories add specialty sections
(folk_horror has *Character*, retrograde has *The deduction it enables*,
weird_physics has *Determinism notes*, classical_composition has *A
worked problem*, etc.).

Nothing in here is committed engineering. These are options. Read this
folder when you want to pick what to build next, or when you want a
catalog of ideas to mine for new combinations.

## Index

| Folder | Pieces | Inspiration / through-line |
|--|--|--|
| [`claude_first_brainstorm/`](claude_first_brainstorm/README.md) | 11 | The opening pitch — pieces grouped by which engine system they hook into (Block tiles, Track tiles, signal substrate, carrier passenger lists, environment reactions). |
| [`meme_brain/`](meme_brain/README.md) | 8 | TikTok / Gen Z internet meme culture. Loud names, serious mechanics underneath. Skibidi's lineage. |
| [`combo_theorist/`](combo_theorist/README.md) | 8 | MTG / Slay the Spire / Inscryption. Pieces as nodes in a synergy graph; each one names a combo + a counter. |
| [`spatial_puzzle/`](spatial_puzzle/README.md) | 8 | Baba Is You / Into the Breach / Patrick's Parabox. Geometry-first; pieces are localized rules that bend the board. |
| [`folk_horror/`](folk_horror/README.md) | 8 | Folk-horror fairytale (Hilda, Over the Garden Wall, original Grimm). Each piece is a character first; mechanics serve character. The quiet-atmospheric opposite of meme_brain. |
| [`weird_physics/`](weird_physics/README.md) | 8 | Hard speculative fiction (Tenet, Greg Egan, Three Body Problem). Each piece deliberately breaks one law: time, identity, locality, dimensionality, conservation, causality, information. |
| [`retrograde/`](retrograde/README.md) | 8 | Smullyan's *Chess Mysteries of Sherlock Holmes*. Pieces leave history visible; the puzzle is forensic reconstruction. State-as-evidence design. |
| [`into_the_breach/`](into_the_breach/README.md) | 9 | Into the Breach / Hoplite / 868-HACK. Single-turn perfect-information puzzles; enemies telegraph next-turn action, finite tool kit, "solve this turn." |
| [`rube_goldberg/`](rube_goldberg/README.md) | 8 | Zachtronics / Rube Goldberg cartoons. The board IS a machine; place one piece, watch the deterministic cascade. Extends the engine's existing signal substrate. |
| [`baba_is_you/`](baba_is_you/README.md) | 8 | Literal Baba Is You. Pieces ARE grammatical rule-tokens that compose into clauses. Almost certainly needs its own `VariantId::Baba` flag. |
| [`classical_composition/`](classical_composition/README.md) | 8 | Sam Loyd / T.R. Dawson tradition. Pieces that enable beautiful chess problems: zugzwang, switchback, helpmate, selfmate, unique-key, series-mover. |
| [`original_ideas_txt/`](original_ideas_txt/README.md) | 5 | The user's own opening brainstorm in `engine/src/pieces/ideas.txt`. Goblin and Skibidi already shipped from this same document; these five are the un-built half. |

97 pieces total.

## Name collisions across categories

Convergent design surfaced several name clashes. They are different pieces
— same word, different mechanics — kept under their inspiration's folder.

- **Echo** — [combo_theorist/echo.md](combo_theorist/echo.md) is a poison
  pill that compels its capturer to replay the same move next turn.
  [spatial_puzzle/echo.md](spatial_puzzle/echo.md) records the last move
  played anywhere and applies that delta to any friendly piece.
- **Recursion** — [spatial_puzzle/recursion.md](spatial_puzzle/recursion.md)
  grants a half-range bonus move to anything ending its move adjacent.
  [classical_composition/recursion.md](classical_composition/recursion.md)
  copies an adjacent piece's identity permanently for series-mover
  problems.
- **Domino** — [into_the_breach/domino.md](into_the_breach/domino.md) is
  a stationary enemy that topples in a telegraphed direction when
  disturbed. [rube_goldberg/domino.md](rube_goldberg/domino.md) is a
  signal-receiver that slides one square and forwards the signal to
  adjacent Dominoes.
- **Anchor** — [spatial_puzzle/anchor.md](spatial_puzzle/anchor.md)
  names a specific enemy and mirrors their moves.
  [into_the_breach/anchor_flag.md](into_the_breach/anchor_flag.md) is
  a one-charge stun tile (different name in file, same conceptual
  family).

If any of these go on to actually ship, the survivor takes the simple
name and the other gets a disambiguating prefix in the piece-symbol
table.

## Cross-category themes

Patterns the brainstorms produced more than once, in case any of these
feel like "build this category, not that piece":

- **Counter-state pieces** (a piece carrying a strictly-increasing or
  decreasing integer) appear across retrograde (Chainwalker), classical
  (Tithe), Rube Goldberg (Hourpetal), combo_theorist (Hourglass),
  meme_brain (Sigma, Mewing). The "piece with a counter" is the most
  fertile single shape this brainstorm produced. Cheap to implement,
  expressive at the design level.
- **Rewind-on-capture** appears twice (combo's Hourglass, physics's
  Eternal Return). Both want the same engine machinery — an undo stack.
  Build it once.
- **Reflection / redirection** primitives recur (Rube Goldberg's
  Mirror-Coil, Into the Breach's Mirror Plate, plus the signal-substrate
  itself in plan 08). Routing-as-puzzle is clearly a load-bearing motif.
- **Telegraphed-future state** (Into the Breach pieces' visible
  next-turn intent) overlaps with weird_physics's Prophet and Paradox.
  They're solving different problems with the same shape — a sealed FEN
  slot the player can see but not yet act on.
- **Terrain-painters** (Architect, Engineer, Dog, Jackhammer, Trellis)
  all want a paint-an-adjacent-square-with-a-special-tile primitive.
  Same hook, different products.

## How to use this folder

- Picking what to build next: start at the category README, then click
  through to individual pieces. Each piece file has *Where it shines*
  and *Where it's awkward* sections to triangulate fit.
- Designing a new variant: the [`baba_is_you/`](baba_is_you/) and
  [`into_the_breach/`](into_the_breach/) categories both define
  variant-shaped piece systems; they're the most direct templates for
  building a self-contained variant rather than a single piece.
- Hunting for combos: the [`combo_theorist/`](combo_theorist/) files
  each name 2+ synergies with existing implemented pieces (Goblin,
  Skibidi, Bus, Monkey, Locomotive, Carriage). Useful as a meta-survey.
- Bug-fishing the engine: many *New features required* sections name
  infrastructure the engine doesn't have yet. Aggregating these would
  produce a plan-shaped list of "what concrete primitives would unlock
  the most pieces."

## What's *not* here

- The implemented pieces (standard chess + Goblin, Skibidi, Bus, Monkey,
  Locomotive, Carriage). Those are in the codebase under
  `engine/src/pieces/`.
- The build plans for new mechanisms (plan 10 movement stack, plan 11
  Duck Chess + variants, plan 12 Block tile). Those are in `plans/`.
- Any piece I have *not* personally proposed in our conversations. This
  folder is closed over the brainstorms in our chat history; if more
  pieces show up, add them and update the table here.
