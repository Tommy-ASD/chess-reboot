# Plans

Where the project is and where it's going. Each numbered file is a focused
plan for one chunk of work. Read this overview first, then jump to whichever
plan you want to act on.

## Current state (after the audit + fix passes)

- **Engine** (`engine/`): move generation, FEN serialization, move execution,
  and brainrot recalculation work for standard pieces + the custom set
  (Goblin, Skibidi, Bus, Monkey). 37 tests pass, 0 compile warnings.
  Recent commits collapsed match boilerplate, migrated logging to
  `tracing`, fixed the audit-surfaced critical bugs (Monkey chain `from`,
  PieceInCarrier envelope, Bus direction, Skibidi captures, Brainrot
  shape/neutralization), and converted the reachable `panic!`/`todo!`
  paths to `Err` returns.
- **API** (`api/`): basic axum service with two endpoints
  (`POST /board/moves`, `POST /board/new_state`). Now returns
  `400 Bad Request` on illegal moves instead of silently echoing.
- **Frontend** (`frontend/vite-dev/`): exists, mostly an editor /
  rendering surface. Out of scope of this engine-focused project work
  unless the API contract changes.

## What's still missing

In rough priority order:

1. **Turn order** — `BoardFlags` has no `turn` field. Either color can move
   at any time. Everything below depends on this. → [01-turn-system.md](01-turn-system.md)
2. **King safety** — no check / checkmate / pinned-piece filtering. King
   can walk into attacked squares. → [02-king-safety.md](02-king-safety.md)
3. **Standard-chess completeness** — promotion, castling, and en passant
   all still missing despite `BoardFlags` having the bookkeeping fields.
   → [03-standard-chess-completeness.md](03-standard-chess-completeness.md)
4. **Custom-piece spec gaps** — Goblin captured-while-kidnapping, Skibidi
   win-by-brainrot, passenger Pawn double-push semantics, and a few
   smaller items. Each needs the turn system before it can land properly.
   → [04-custom-piece-spec-gaps.md](04-custom-piece-spec-gaps.md)
5. **FEN parser hardening** — silently produces garbage on malformed
   input. Two doctests are `ignore`d because their assertions are wrong.
   → [05-fen-hardening.md](05-fen-hardening.md)
6. **API evolution** — current API is stateless and tiny. As the engine
   grows, the API needs game-state endpoints, error shapes, and probably
   serde JSON instead of FEN-only. → [06-api-evolution.md](06-api-evolution.md)
7. **Test strategy** — 37 tests is a good start; missing perft,
   property tests, and integration coverage of full game scenarios.
   → [07-testing-strategy.md](07-testing-strategy.md)
8. **Signal substrate** — generic event/wiring layer for switches,
   junctions, gates, pressure plates. Underpins trains and a lot of
   potential future composability features. Independent of the
   bug-fixing plans above.
   → [08-signal-substrate.md](08-signal-substrate.md)
9. **Trains** — buildable closed-loop tracks that auto-move neutral
   trains carrying pieces. Carts are invincible; their occupants are
   not. Depends on plans 01, 02, and 08.
   → [09-trains.md](09-trains.md)

## Suggested sequence

Plans **01 → 02 → 03** are the natural chain — each blocks the next.
Plan **04** unblocks once **01** lands. Plans **05**, **06**, **07** can
proceed in parallel with the engine work.

Plans **08 → 09** are a feature track rather than a bug-fix track —
they add new mechanics on top of a working baseline. **08** can land
any time after **01**; **09** needs **01**, **02**, and **08**.

If picking one thing to do next: **plan 01**. Everything that relates to
"whose turn is it" or "can this player move here" depends on it.

## Open questions

A few things the spec doesn't pin down and the project will have to
decide at some point:

- **Win-by-brainrot vs stalemate**: spec mentions both but the rules
  for which fires when aren't explicit. See plan 02 and 04.
- **Passenger Pawn semantics**: should a pawn carried in a Bus retain
  its "starting position" rights? Spec is silent. See plan 04.
- **Nested carriers**: currently forbidden (plan 04 keeps this). If
  Buses-inside-Buses is ever wanted, the capacity-5 invariant needs
  a recursive count.
- **Promotion target**: standard chess lets the player choose Q/R/B/N.
  Does Chess 2 / Fairy mode allow promoting to custom pieces?
  See plan 03.
