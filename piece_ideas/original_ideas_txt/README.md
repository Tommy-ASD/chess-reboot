# original_ideas_txt — the user's own brainstorm

This folder is the design-doc-ified version of the half of
`engine/src/pieces/ideas.txt:1` that *didn't get built*.

That source file is the user's first brainstorm — a casual, slightly
chaotic list of fairy-piece ideas, scrawled before any of the
implementation lift happened. From that document, two pieces have
since shipped to the engine:

- **Goblin** — the kidnap-and-convert piece (ideas.txt:3-8).
- **Skibidi** — the brainrot-radius stunner (ideas.txt:24-46).

Both have full engine implementations, FEN payloads, and test
coverage today. The rest of the file — five other ideas, some only
two words long — never made it past the brainstorm stage. This
folder catches them up.

## Why "design-doc-ify" the un-built ones?

Same reason design docs exist anywhere: forcing a vague idea
("Mind control piece") into precise specification surfaces every
choice that was implicit. The originals are not bad ideas — they're
just under-specified. Goblin and Skibidi got specified-by-doing
(via implementation). These five get specified-by-doc.

The originals' tone is preserved where it matters (notably for Dog,
which is a joke that should *stay* a joke, mechanically supported).
The docs themselves are rigorous: state encoding, FEN format, edge
cases, open questions.

## Index

| File | Source line(s) | One-liner |
|------|---------------|-----------|
| [enemy_mover.md](enemy_mover.md) | ideas.txt:1 | A piece whose moves are the union of all enemy pieces' moves from its square. |
| [mind_control.md](mind_control.md) | ideas.txt:11 | Spends a turn marking an enemy piece; controls that piece's next move. |
| [blender.md](blender.md) | ideas.txt:13-16 | Banks the point-value of captures; spends those points to transform into any piece at-or-below tier. |
| [jackhammer.md](jackhammer.md) | ideas.txt:18-19 | Single-use capture turns the captured square into a permanent `Block`. |
| [dog.md](dog.md) | ideas.txt:21-22 | Every other move, deposits a slippery tile that causes pieces to slide one extra square. |

(In ideas.txt order. Mind Control and Blender are next to each
other in the source; the rest are scattered.)

## What these have in common

All five are reactive or transformative — they don't just *move*,
they *change something* about the game state when they act:

- Enemy Mover changes its own movement based on the enemy army.
- Mind Control changes whose turn the next move belongs to.
- Blender changes its own piece type.
- Jackhammer changes the terrain on capture.
- Dog changes the terrain by walking.

The two implemented pieces from this source (Goblin, Skibidi) are
also transformative — Goblin converts captured pieces, Skibidi
stuns. The brainstorm has a clear thematic spine: **pieces that
do more than capture**.

## Honesty notes

- **Dog** is the silliest one and the design doc says so. The
  mechanic underneath (slip-tile trail) is reusable and
  interesting; the naming carries the joke. Both layers are
  documented honestly.
- **Mind Control** is the most under-specified original (two
  words). The doc proposes a full mechanic and labels every
  decision as "from scratch."
- **Jackhammer** has two reasonable trigger variants. The source
  itself was uncertain ("when taken (or taking another piece?)").
  Both are documented; one is recommended.
- **Blender** has an open question in the source ("maybe add a
  limit?"). That question is answered in the doc (yes — saturate
  at 9).
- **Enemy Mover**'s one-line source has an interpretation
  ambiguity (moves *from* where, or *to* where). Both readings
  are presented; one is recommended.

## Relationship to the rest of `piece_ideas/`

Other folders in `piece_ideas/` are agent-generated brainstorms.
This folder is the *user's* original brainstorm, retroactively
elevated to the same documentation standard. The Goblin and
Skibidi entries that *did* ship live in the engine source — see
`engine/src/pieces/fairy/` for their implementations. This folder
documents the rest of the same lineage.

If any of these five are picked up for implementation, the doc
here is the spec to start from. Open questions in each doc are
the dials to make policy on before coding.
