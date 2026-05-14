# Classical Composition

Eight pieces designed in the spirit of Sam Loyd, T. R. Dawson, and the
fairy-chess problemist tradition. They are not balanced for tournament
play. Each piece is a small machine for producing one specific kind of
**problem**.

## Design philosophy

A problem composer asks a different question than a player. The player
wants pieces whose value is legible across many positions. The composer
wants pieces whose constraints, *in exactly one position*, produce
exactly one solution — and reject every near-miss with elegance.

Composition pieces are evaluated by three criteria:

1. **Unambiguous mechanic.** A composer cannot work with a piece whose
   movement depends on hidden judgement calls. Every rule in this
   directory is fully deterministic and FEN-serializable.
2. **Solution uniqueness.** A piece should make the *intended*
   solution the *only* solution. Tries — moves that look correct but
   fail to a specific defence — are how composers verify uniqueness.
3. **Motif specificity.** A great composition piece is built for one
   problem-class. Asking "what is this piece *for*?" should yield a
   one-sentence answer (zugzwang, switchback, helpmate, selfmate,
   etc.).

The eight pieces here each target a different motif. None overlap.

## Glossary

- **Helpmate.** A problem where Black moves first (or per the convention)
  and both sides cooperate to checkmate the Black king in exactly N
  moves. The "help" is unusual: both players choose the same goal —
  mating Black.
- **Selfmate.** White moves first and forces Black to checkmate
  *White's own* king in exactly N moves. White's moves are all
  legal chess moves; the constraint is that they leave Black with only
  moves that produce the forced mate.
- **Series-mover.** One side makes N consecutive moves; the other side
  does not move. The Nth move delivers mate. Intermediate checks are
  usually illegal (the moving side cannot pass the king through check).
- **Zugzwang.** A position where the side to move would prefer to pass.
  Every legal move worsens their position. The composer's gold standard
  is *mutual zugzwang* — both sides are in zugzwang and only the side-
  to-move loses.
- **Switchback.** A piece's trajectory that returns it to its starting
  square. The most elegant problems use a switchback as the
  *solution's* signature: the piece moves out, does its work, and
  returns home.
- **Key.** The first move of the solution. A "good key" is non-obvious;
  a "thematic key" embodies the problem's theme; a "bad key" is
  capture, check, or restricts Black's moves trivially.
- **Try.** A near-solution that fails to one specific Black defence.
  Tries are part of the composition's content — solvers explore them,
  rule them out, and arrive at the unique solution.
- **Cook.** An unintended alternative solution. A cooked problem is
  invalid and discarded.
- **Stipulation.** The exact specification: "mate in N," "helpmate in
  N," "selfmate in N," "series-mover in N," with any auxiliary
  conditions ("with declared switchback," "without intermediate
  check," etc.).

## The pieces

- **[Helix](helix.md)** — A rook with a rotation counter. Switchback
  problems where exactly 4 Helix moves return it to the starting
  rotation-state.
- **[Tithe](tithe.md)** — A king that donates one square of its
  movement to an adjacent enemy per move. Selfmates that write
  themselves: White feeds Black the exact mating-piece power.
- **[Kaddish](kaddish.md)** — A bishop that may only land where enemy
  pieces have died. The key move is a sacrifice three moves earlier
  whose purpose is to create the landing-pad.
- **[Solstice](solstice.md)** — A king that may move once per game as
  a queen. Helpmates where the queen-move's *spend-position* in the
  move-order is the puzzle's whole content.
- **[Eclipse](eclipse.md)** — A bishop with colour-and-distance-parity
  threats. Pure zugzwang via invisible-attack squares.
- **[Recursion](recursion.md)** — A piece that copies any adjacent
  piece's type and moves as it, permanently becoming that type.
  Series-movers as identity-chains: P → N → B → R → Q.
- **[Antiphon](antiphon.md)** — Two paired pieces, each moves only to
  the 180° rotation of its twin's current square. Symmetric helpmates
  where cooperation manifests as mirror-positioning.
- **[Pilgrim](pilgrim.md)** — A knight constrained to land on
  alphabet-by-file order (a → b → … → h → a → …). Unique-key problems
  where most knight-checks are illegal because of the file counter.

## Index by motif

- Switchback: **Helix**
- Selfmate: **Tithe**, **Eclipse** (as part of mutual-zugzwang
  selfmates)
- Deep-key sacrifice: **Kaddish**
- Helpmate: **Solstice**, **Antiphon**
- Zugzwang: **Eclipse**
- Series-mover: **Recursion**
- Unique-key: **Pilgrim**

## A note on playability

None of these pieces is balanced for symmetric variant play. The Tithe
*feeds the enemy*. The Solstice's queen-move is one-shot. The Kaddish
cannot move at game start. The Pilgrim's attack set is one file wide.

This is intentional. Composition pieces buy puzzle-richness with
play-balance. To use them in a variant, pair them with auxiliary
mechanics (additional Solstices, regenerating Kaddish landing-pads,
file-counter resets for Pilgrims, etc.) and accept that the variant
will play *as a puzzle*, not as a tactical game.

## Format

Each piece file follows the structure documented in
`piece_ideas/classical_composition/README.md` (this file's parent
template). The "worked problem" section is the heart of each file —
composers should be able to read it and immediately see what the piece
*makes possible*.
