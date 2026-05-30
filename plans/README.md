# Plans

Where the project is and where it's going. Each numbered file is a focused
plan for one chunk of work. Read this overview first, then jump to whichever
plan you want to act on.

## Current state (after the audit + fix passes)

- **Engine** (`engine/`): move generation, FEN serialization, move execution,
  and brainrot recalculation work for standard pieces + the custom set
  (Goblin, Skibidi, Bus, Monkey) plus train carts (Locomotive, Carriage).
  Engine test suite at 280+ tests across lib + integration (perft,
  properties, fairy scenarios, standard game), 0 compile warnings.
  Property tests cover both standard chess and an active train loop
  (the train property's helper now descends into Neutral carts so
  king-passenger scenarios are exercised under random play).
  Recent commits collapsed match boilerplate, migrated logging to
  `tracing`, fixed the audit-surfaced critical bugs (Monkey chain `from`,
  PieceInCarrier envelope, Bus direction, Skibidi captures, Brainrot
  shape/neutralization), and converted the reachable `panic!`/`todo!`
  paths to `Err` returns.
- **API** (`api/`): axum service with two endpoint families — stateless
  `/board/*` (`moves`, `new_state`, `status`; client owns the FEN) and
  stateful `/games/*` (server-owned games with move history + resignation,
  plan 06 step 4). Structured `400 Bad Request` JSON on illegal moves.
  See [api/001-API architecture.md](../api/001-API%20architecture.md).
- **Frontend** (`frontend/vite-dev/`): exists, mostly an editor /
  rendering surface. Out of scope of this engine-focused project work
  unless the API contract changes.

## What's shipped

- **Plan 01 — turn system**: `side_to_move` flag, side-flip in
  `apply_environment_reactions`, ply counter, `WrongTurn` error variant.
- **Plan 02 — king safety**: `is_in_check`, `legal_moves`,
  `WouldLeaveKingInCheck`, `would_capture_at` phantom-attack filter,
  `find_king` descent into carriers, `apply_move_for_validation`
  validate variant that skips the train tick.
- **Plan 03 — standard-chess completeness**: promotion (Q/R/B/N),
  castling (king-/queen-side), en passant (set + clear + capture).
- **Plan 08 — signal substrate**: Switch, Junction, Gate, PressurePlate
  tile types; `fire_signal` dispatcher; FEN round-trip; `ThrowSwitch`
  move type.
- **Plan 09 — trains** (v1): Locomotive + Carriage, Track + Junction
  traversal, two-train collision, foreign-cart filter, head-swap
  detection, three-phase commit, train-tick rate flags (`tr=full|ply|Nply`).
  v1 explicitly defers: per-piece collision hooks, carriage detaching,
  heading reversal, boarding-from-adjacent.
- **Plan 10 — movement stack**: generic modifier pipeline; `get_moves`,
  `is_attacked_by`, and `legal_moves` all resolve through an ordered
  registry (`movement::stack`). Brainrot/frozen/walkability, train threats,
  and the king-safety filter are modifiers now, not scattered conditionals.
- **Plan 12 — Block square**: payload-free, semantics-free impassable
  tile (`T=BLOCK`). `is_walkable()` returns `false`; FEN round-trips;
  frontend brush + brick-pattern SVG + `.type-block` CSS shipped.
- **Plan 06 — API evolution** (steps 1/2/4): serde on the whole `Board`;
  `GameStatus` in responses + `/board/status`; stateful `/games/*` with
  history + resignation. Step 3 (structured `FenError` JSON) waits on
  plan 05.

## What's still missing

In rough priority order:

1. **Custom-piece spec gaps** — Skibidi win-by-brainrot, passenger Pawn
   double-push semantics, a few smaller items.
   → [04-custom-piece-spec-gaps.md](04-custom-piece-spec-gaps.md)
2. **FEN parser hardening** — most paths now warn loudly on malformed
   input; remaining gaps tracked in the plan.
   → [05-fen-hardening.md](05-fen-hardening.md)
3. **API evolution** — steps 1/2/4 shipped (see above); step 3 (structured
   `FenError` JSON) is all that remains, blocked on plan 05.
   → [06-api-evolution.md](06-api-evolution.md)
4. **Test strategy** — 280+ lib tests + perft + property + integration,
   but coverage is still uneven.
   → [07-testing-strategy.md](07-testing-strategy.md)
5. **Duck Chess + variant infrastructure** — first true rule-variant,
   plus the per-position `variants` flag future variants hook into. With
   plan 10 landed, its variant gates can be written directly as
   movement-stack modifiers.
   → [11-duck-chess.md](11-duck-chess.md)
6. **Trains v2** — the deferred items from plan 09 (collision-hook
   chain, carriage detaching, heading reversal, boarding-from-adjacent).
   → [09-trains.md](09-trains.md)

## Suggested sequence

Plans **04 / 05** can proceed in parallel; **06** has only step 3 left
(blocked on **05**). Plan **10** (movement stack) has landed, so plan
**11** (Duck Chess + variant infra) can write its variant gates directly
as movement-stack modifiers rather than as conditionals to migrate later.
Trains v2 (plan 09's deferred items) is the natural follow-up to plan 10.

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

## FEN extensions reference

The engine extends standard FEN with named-field payloads inside
parens. Format: `(KEY=value,KEY=value,...)`. Keys are uppercase
single-or-multi-letter; values are unquoted. `find_matching_paren`
handles nested payloads (e.g. `LOCO(...,P=(K,R))`).

### Per-square keys

Missing-field defaults are noted in parens. Note: `STATE` and
`BRANCHES` are normalized at parse time — `STATE` is reduced
mod `BRANCHES.len()`, and >255-branch lists are truncated with a warn.

| Key | Meaning | Example |
|-----|---------|---------|
| `P`  | Piece occupying the square | `P=K`, `P=BUS(P=(K))` |
| `T`  | Square type (default `STANDARD`) | `T=SWITCH`, `T=PLATE`, `T=GATE`, `T=JUNCTION`, `T=TRACK`, `T=VENT`, `T=TURRET`, `T=BLOCK`, `T=STANDARD` |
| `C`  | Condition (repeatable) | `C=FROZEN`, `C=BRAINROT` |
| `ID` | Signal ID for Junction/Gate/Switch/Plate (default `0`) | `ID=3` |
| `STATE` | Current branch index of a Junction (default `0`) | `STATE=0` |
| `BRANCHES` | Branch direction list of a Junction (default `()`) | `BRANCHES=(N,E,S,W)` |
| `TARGETS` | Signal target list for Switch/Plate (default `()`) | `TARGETS=(3,7)` |
| `OPEN` | Gate state (default open; `OPEN=garbage` parses as closed) | `OPEN=1` |
| `FIRES` | Pressure plate trigger (default `ANY`) | `FIRES=ANY`, `FIRES=W`, `FIRES=B`, `FIRES=N` |
| `D`  | Track exit direction (default `E`) | `D=N`, `D=E`, `D=S`, `D=W` |
| `DUCK` | Duck on this square (plan 11 — **planned, not yet parsed**; value-less flag) | `(DUCK)` |

### Piece-payload keys (inside `P=...` for carriers)
| Key | Meaning | Example |
|-----|---------|---------|
| `P` (Bus) | Passengers list | `BUS(P=(K,N))` |
| `P` (Goblin) | Kidnapped piece — single symbol, no parens | `G(H=4-0,P=n)` |
| `PHASE` (Skibidi) | Brainrot phase 1..=4 (omitted when 1) | `S(PHASE=3)` |
| `H` (Goblin / Locomotive) | Goblin home square / Loco heading | `G(H=0-0)`, `LOCO(H=F)` |
| `ID` (Locomotive / Carriage) | Train ID | `LOCO(ID=1,H=F)` |
| `I` (Carriage) | Chain index (1..255; 0 reserved for the loco) | `CART(ID=1,I=2)` |
| `L` (Locomotive) | Last-entered direction (round-trip hint) | `LOCO(...,L=N)` |
| `P` (Locomotive / Carriage) | Passenger list | `LOCO(ID=1,H=F,P=(K))` |

### Board-flag keys (after the grid + side + castle + ep)
| Token | Meaning | Example |
|-------|---------|---------|
| `tr=full` / `tr=ply` / `tr=<n>ply` | Train tick rate | `tr=full`, `tr=2ply` |
| `p=<n>` | Plies elapsed (for `EveryNPly` gate alignment) | `p=42` |
| `variants=<id>,<id>,…` | Active rule variants (plan 11 — **planned, not yet parsed**; default empty = standard chess) | `variants=duck_chess` |
| `duck_phase=piece` / `duck_phase=placing` | Duck Chess half-turn (plan 11 — **planned, not yet parsed**; default `piece`) | `duck_phase=placing` |
| `lm=(C=…,F=…,K=…[,T=…][,V=…],P=…)` | Last-move snapshot (plan 10; default absent = no prior move). `C` is mover color (W/B/N), `F` is from coord, `K` is move kind (MOVE / MIC / PROMO / CASTLE / EP / PS / TS / PIC), `T` is to coord (omitted for ThrowSwitch / PhaseShift), `V` is captured-piece symbol (omitted on non-captures), `P` is primary piece symbol (post-promotion for Promote moves) | `lm=(C=W,F=4-6,K=MOVE,T=4-5,P=P)` |

Canonical implementer: `engine/src/board/fen.rs`. Frontend parser:
`frontend/vite-dev/src/fen.ts`.
