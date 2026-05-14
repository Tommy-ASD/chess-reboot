# Into the Breach — Piece Family

Telegraphed enemies and finite-charge tools. The chessboard becomes a
single-turn perfect-information puzzle: every threat is visible on the
board, every solution is a finite combination of player verbs. No
hidden info, no randomness, no exploration — just "solve this turn."

## Design philosophy

Borrowed wholesale from Into the Breach, Hoplite, 868-HACK, and
Imbroglio. Those games share one trick: **enemies show their next move
before they take it.** The player reads the board, sees three Vek
about to punch three buildings, and has exactly the tools on hand to
prevent all three. The puzzle is mechanical — find the sequence of
verbs that defuses every telegraph before the enemy turn fires.

Chess already has telegraphs in a weak sense: a queen on d1 is
"threatening" every diagonal. But standard chess threats are
**conditional** — they activate only if the player moves into them.
Into-the-Breach threats are **unconditional** — they fire next turn
regardless, and the player's job is to redirect, block, push, or
neutralize.

That distinction is the whole category.

## Why the engine can host this

The fairy engine already has every primitive we need:

- **`Color::Neutral`** — telegraphed enemies aren't aligned with the
  player; they're aligned with the puzzle. Neutral color lets us drop
  them into positions without breaking the two-sided turn order.
- **Signal substrate** — telegraph state is just signals + payloads.
  A Siege Engine's "loaded" flag, a Clock's countdown, a Marcher's
  arrow — all serialize through the existing `(K=V,...)` syntax.
- **Square types and conditions** — Frozen, Brainrot, Block already
  modify piece behaviour mid-turn. Telegraphed effects (beams,
  yanks, explosions) reuse the same hook machinery.
- **Variable boards** — puzzle positions are rarely 8×8. The engine
  already handles arbitrary dimensions.
- **FEN parenthesized payloads** — every telegraph state round-trips.

What's missing is one new orchestration concept: a **telegraph
resolution phase** between the player's turn and the start of the
next player turn. Enemies don't get a "turn" in the chess sense —
they get a resolution step where all their queued telegraphs fire
simultaneously, in a documented order. Most of these pieces depend on
that phase existing.

## Classification

Each piece is tagged:

- **[ENEMY]** — Neutral-colored, telegraphs an action, fires during
  the enemy resolution phase. Player neutralizes by capturing,
  pushing, blocking, or redirecting.
- **[TOOL]** — Player-owned piece or placeable. Usually charge-based:
  one or two uses per puzzle. The player's verbs.
- **[NEUTRAL]** — Board topology. Affects how telegraphs route but
  takes no action itself. Both sides can interact with it.

## Index

| Piece | Class | One-liner |
|-------|-------|-----------|
| [Marcher](marcher.md) | ENEMY | Steps + rotates 90° clockwise each turn; arrow on the piece shows next direction. |
| [Siege Engine](siege_engine.md) | ENEMY | Loads one turn, fires a line beam the next. |
| [Latcher](latcher.md) | ENEMY | Yanks its nearest enemy one square closer each turn. |
| [The Clock](the_clock.md) | ENEMY | 3-2-1 countdown then 3×3 explosion. Push it, don't kill it. |
| [Domino](domino.md) | ENEMY | Stationary until poked; then falls and re-points. Cascades. |
| [Shover](shover.md) | TOOL | Knight-leap that pushes an adjacent piece away. One charge. |
| [Mirror Plate](mirror_plate.md) | TOOL | Placeable tile; reflects beams/leashes/telegraphs 90°. |
| [Anchor Flag](anchor_flag.md) | TOOL | Placeable; adjacent enemies skip their next action. |
| [Conduit](conduit.md) | NEUTRAL | Any telegraph entering one Conduit emerges from every other. |

## What this category isn't

Not standard chess. These pieces assume a puzzle-mode variant where
the player has a goal-state ("survive 3 turns," "reach square X,"
"detonate all Clocks safely") and a finite tool inventory. They are
not balanced for free-play 1v1.
