# Plans

Where the project is and where it's going. Each numbered file is a focused
plan for one chunk of work. Read this overview first, then jump to whichever
plan you want to act on.

## Current state (after the audit + fix passes)

- **Engine** (`engine/`): move generation, FEN serialization, move execution,
  and brainrot recalculation work for standard pieces + the custom set
  (Goblin, Skibidi, Bus, Monkey, Stormcaller) plus train carts
  (Locomotive, Carriage). Move/threat generation, square-condition gates,
  train geometry, king-safety, and captures all run through the plan-10
  movement-stack modifier pipeline (`movement::stack`).
  Engine test suite at 340+ tests across lib + integration (perft,
  properties, fairy scenarios, standard game) + doctests, 0 compile
  warnings.
  Property tests cover both standard chess and an active train loop
  (the train property's helper now descends into Neutral carts so
  king-passenger scenarios are exercised under random play).
  Recent commits collapsed match boilerplate, migrated logging to
  `tracing`, fixed the audit-surfaced critical bugs (Monkey chain `from`,
  PieceInCarrier envelope, Bus direction, Skibidi captures, Brainrot
  shape/neutralization), and converted the reachable `panic!`/`todo!`
  paths to `Err` returns.
- **API** (`api/`): axum service with three endpoints
  (`POST /board/moves`, `POST /board/new_state`, `POST /board/status`).
  Returns `400 Bad Request` on illegal moves and a structured 400 on
  malformed FEN. `/new_state` and `/status` carry the adjacently-tagged
  `GameStatus`.
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
- **Plan 04 — custom-piece spec gaps**: Skibidi win-by-brainrot
  (`GameStatus::BrainrotWin`) and the phase-4 lockout
  (`GameStatus::BrainrotLockout` — "your Skibidi captured while the
  enemy's is phase 4 = nothing you can do"), both folded into `status()`
  behind documented-approximate heuristics (`is_brainrot_win` /
  `is_brainrot_lockout`). Goblin "the taker moves again" (mechanic 2)
  as flip-suppression in `apply_environment_reactions_with`
  (`move_grants_extra_turn`) — no FEN/state needed, the captor's side
  simply stays to move. Passenger-Pawn semantics documented on `Bus`
  move-gen (a carried pawn double-pushes off the *Bus's* rank). Goblin
  drop-on-capture (mechanic 1) + the Monkey-chain reading already
  shipped earlier; Bus capacity counting stays documented dead code.
- **Plan 05 — FEN hardening**: `FenError` type; `fen_to_board` /
  `fen_to_square` / `fen_row_to_squares` are now fallible. Hard
  structural errors (empty input, ragged board, unknown piece glyph,
  unbalanced parens) abort the parse; field-level slips stay lenient
  (`warn!` + default). `api/` returns a structured 400 on bad FEN.
  Surfaced + fixed a latent silent-garbage bug: the Monkey (`M`/`m`)
  had no `symbol_to_piece` arm and round-tripped to an empty square
  (fairy perft constants re-pinned to the correct board).
- **Plan 06 — API evolution** (steps 1–3): serde `Serialize`/
  `Deserialize` swept across the whole `Board`/`grid`/`flags`/piece
  graph (23 types; JSON round-trip smoke-tested). `GameStatus`
  (adjacently tagged) folded into `POST /board/new_state` and exposed
  via new `POST /board/status`; structured-400-on-bad-FEN (plan 05)
  already wired. Step 4 (stateful redesign with game IDs) deferred —
  no clock / multiplayer / persistence. The JSON board format (vs FEN
  on the wire) also still deferred; the unblocking derives have landed.
- **Plan 08 — signal substrate**: Switch, Junction, Gate, PressurePlate
  tile types; `fire_signal` dispatcher; FEN round-trip; `ThrowSwitch`
  move type.
- **Plan 09 — trains** (v1): Locomotive + Carriage, Track + Junction
  traversal, two-train collision, foreign-cart filter, head-swap
  detection, three-phase commit, train-tick rate flags (`tr=full|ply|Nply`).
  v1 explicitly defers: per-piece collision hooks, carriage detaching,
  heading reversal, boarding-from-adjacent.
- **Plan 10 — movement stack**: the unified modifier pipeline is the
  live path — `Board::get_moves` / `legal_moves` / `is_attacked_by` and
  the capture path all delegate to `movement::stack::default_stack()`.
  Registered modifiers: `PieceMovesModifier` (piece-intrinsic moves),
  per-piece `piece_attacks` threat modifiers (one per piece type +
  Neutral-carrier passengers), square gates (`SquareConditionFilter` /
  `WalkabilityFilter` / `SwitchTileAugment`), train geometry (head-crush
  + cart-capture + two-train collision), `KingSafetyFilter` (priority
  300), and the plan-13 `TornadoCompulsionFilter` (305); the capture
  pipeline lives in `movement::stack::capture`. The one sketch item
  deliberately not taken: removing `Piece::attacks` / `initial_moves`
  from the trait — they're kept as the per-piece primitives the
  modifiers compose (see `piece_attacks.rs`).
- **Plan 12 — Block square**: payload-free, semantics-free impassable
  tile (`T=BLOCK`). `is_walkable()` returns `false`; FEN round-trips;
  frontend brush + brick-pattern SVG + `.type-block` CSS shipped.
- **Plan 13 — Tornado** (engine, commits 1–4 of 5): timed
  `SquareCondition::Tornado { remaining }` (`C=TORNADO:<n>`, the
  engine's first payload-carrying condition). `TornadoTickHandler`
  (env-reaction, PostTick) counts it down per ply, Frozen pauses it.
  `TornadoCompulsionFilter` (movement stack, priority 305): the side
  to move must move onto a tornado square if it can; a non-king piece
  standing on one is trapped; king fully exempt; recursion-guarded
  reachability probe (capped at 304). `Stormcaller` fairy piece
  (`W`/`w`) + `MoveType::PlaceTornado` stamp tornadoes in-game.
  Commit 5 (frontend brush + countdown overlay) deferred — engine
  scope, API contract unchanged.

## What's still missing

In rough priority order:

1. **Test strategy** — 340+ tests now (328 lib + perft + property +
   integration + doctests), but coverage is still uneven.
   → [07-testing-strategy.md](07-testing-strategy.md)
2. **Duck Chess + variant infrastructure** — first true rule-variant,
   plus the per-position `variants` flag future variants hook into.
   The movement stack (plan 10) is shipped, so the chokepoint
   conditionals it adds can land directly as modifiers.
   → [11-duck-chess.md](11-duck-chess.md)
3. **Trains v2** — the deferred items from plan 09 (collision-hook
   chain, carriage detaching, heading reversal, boarding-from-adjacent).
   → [09-trains.md](09-trains.md)

## Suggested sequence

Plan **07** (test coverage) can proceed now (plans **04**, **05**, **06**,
**10** shipped). With the movement stack landed, **Plan 11** (Duck Chess
+ variant infra) is the natural next structural piece — its chokepoint
conditionals land directly as modifiers. Trains v2 (plan 09's deferred
items) is the other follow-up, composing as train modifiers in the stack.

## Open questions

A few things the spec doesn't pin down and the project will have to
decide at some point:

- **Win-by-brainrot vs stalemate** — *resolved (plan 04)*: when the
  side to move has no legal move and isn't in check, `status()`
  distinguishes `BrainrotWin` (all its pieces frozen on Brainrot squares
  with an opposing Skibidi radiating) from an ordinary `Stalemate`, via
  a documented-approximate heuristic. The separate phase-4 case is a
  `BrainrotLockout` that fires even with legal moves available.
- **Passenger Pawn semantics** — *resolved (plan 04)*: a Bus passenger
  moves from the *Bus's* coordinate, so a carried pawn double-pushes
  only off the Bus's rank, not its pre-boarding rank. Documented as a
  deliberate choice on `Bus` move-gen; the other reading (retain
  origin-rank rights) would need a `has_moved` field — plan 04-bis.
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
| `C`  | Condition (repeatable). `TORNADO` carries a `:<n>` countdown payload (plan 13; the only payload-carrying condition); bare `TORNADO` defaults to 3 | `C=FROZEN`, `C=BRAINROT`, `C=TORNADO:3` |
| `ID` | Signal ID for Junction/Gate/Switch/Plate (default `0`) | `ID=3` |
| `STATE` | Current branch index of a Junction (default `0`) | `STATE=0` |
| `BRANCHES` | Branch direction list of a Junction (default `()`) | `BRANCHES=(N,E,S,W)` |
| `TARGETS` | Signal target list for Switch/Plate (default `()`) | `TARGETS=(3,7)` |
| `OPEN` | Gate state (default open; `OPEN=garbage` parses as closed) | `OPEN=1` |
| `FIRES` | Pressure plate trigger (default `ANY`) | `FIRES=ANY`, `FIRES=W`, `FIRES=B`, `FIRES=N` |
| `D`  | Track exit direction (default `E`) | `D=N`, `D=E`, `D=S`, `D=W` |
| `DUCK` | Duck on this square (plan 11; value-less flag). Mutually exclusive with `P` — a piece+duck square is a parse error | `(DUCK)` |

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
| `variants=<id>,<id>,…` | Active rule variants (plan 11; default empty = standard chess). Omitted when empty | `variants=duck_chess` |
| `duck_phase=piece` / `duck_phase=placing` | Duck Chess half-turn (plan 11; default `piece`, only meaningful under `duck_chess`). Emitted only for the non-default `placing` | `duck_phase=placing` |
| `lm=(C=…,F=…,K=…[,T=…][,V=…],P=…)` | Last-move snapshot (plan 10; default absent = no prior move). `C` is mover color (W/B/N), `F` is from coord, `K` is move kind (MOVE / MIC / PROMO / CASTLE / EP / PS / TS / PIC / PT), `T` is to coord (omitted for ThrowSwitch / PhaseShift / PlaceTornado), `V` is captured-piece symbol (omitted on non-captures), `P` is primary piece symbol (post-promotion for Promote moves) | `lm=(C=W,F=4-6,K=MOVE,T=4-5,P=P)` |

Canonical implementer: `engine/src/board/fen.rs`. Frontend parser:
`frontend/vite-dev/src/fen.ts`.
