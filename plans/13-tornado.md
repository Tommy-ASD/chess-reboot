# Plan 13: Tornado

**Status (engine complete — commits 1–4 of 5 landed).** Condition
type + FEN (`C=TORNADO:<n>`), env-reaction countdown,
`TornadoCompulsionFilter` (priority 305, recursion-guarded probe),
and the `Stormcaller` + `MoveType::PlaceTornado` placer are all
shipped and tested (full engine suite green). Resolved en route:
cadence is **per-ply** (not `TrainTickRate`-coupled — open question 2
below); placer name is **Stormcaller** (confirmed). Open: the
same-turn-tick interaction means a freshly placed `dur=3` reads as 2
on the opponent's turn (documented in `stormcaller.rs`; duration cap
is open question 1). Commit 5 (frontend brush + countdown overlay)
is deferred — engine scope, API contract unchanged.

A timed square condition that **compels destination**. While a Tornado
condition sits on a square, any side that *could* legally move a piece
onto that square *must* — every move that doesn't land on a tornado
square is illegal for that turn. A piece that enters is trapped there
(cannot leave) until the tornado dissipates; an enemy that can reach it
is then forced to capture it. The Tornado is not a piece. It is a
condition placed by a **dedicated piece** (this plan ships one;
more placers come later).

Depends on the movement stack (plan 10, shipped) and the environment-
reaction registry (`engine/src/movement/env_reactions.rs`). Reuses the
`SquareCondition` surface (plan 08 lineage, the same one Frozen /
Brainrot ride) and the `post_move_effects` hook Skibidi uses to stamp
state into `board_after`.

## Why now

The catalog (`piece_ideas/`) has terrain-painters (Architect, Engineer,
Jackhammer), capture-as-telegraph (Echo), and a denial primitive
(Lien — locks a square *out*). Nothing yet does the attractive dual:
**you must move *something* *here*.** Echo compels a vector; a pin
*forbids* a piece's move. Tornado compels a *destination*, board-wide,
for both sides. It is a new primitive, and every part of it has an
existing precedent — the move filter is the shape of the plan-10
square filters, the countdown is the shape of the train tick, the
placement is the shape of `ThrowSwitch`.

## Concept

A square may carry a `Tornado { remaining: u8 }` condition. While any
square on the board carries it:

1. **Destination compulsion.** On a side's turn, take its legal move
   set `L` (already king-safety-filtered). If any move in `L` lands on
   a tornado square, `L` is restricted to exactly those moves. If none
   does, `L` is unchanged. This is a **set intersection over already-
   legal moves** — it never produces an illegal or self-checking move,
   and it never manufactures a move that wasn't already legal.
2. **Trap in place.** A piece standing on a tornado square cannot
   generate any move out of it. It is immobilized until the condition
   dissipates. (Mechanically identical to the Frozen short-circuit;
   different only in that it expires.)
3. **Forced execution.** Because the compulsion applies to both sides,
   a piece trapped on a tornado square is a square the *enemy* is
   compelled to capture into the first turn a capture there is among
   their legal moves. No special case — a capture onto a tornado
   square is just a move whose destination is the tornado square.
4. **King is fully exempt.** A king is never forced toward a tornado,
   never trapped by one, never triggers or satisfies the compulsion,
   and the compulsion never restricts a king move. This is a coherence
   requirement, not a balance dial — but **not** for the reason "you
   can't force a king into check." The filter-on-legal-set framing
   (Concept 1) already excludes king-into-check moves for free, and a
   king is never *captured* anyway, so the compulsion half is a
   non-issue. The real reason is the **trap**:
   - **Forced self-immobilization.** A side whose only tornado-reaching
     legal move is a king step onto a safe (unattacked) tornado square
     gets all other moves dropped — it is force-marched, via
     individually-legal moves, into gluing its own king to that square,
     then mated at leisure. No move into check ever occurs.
   - **Flight-square loss under later check.** A king on a tornado
     square that is *then* checked cannot leave (trapped), losing every
     escape square — manufacturing mates the checked side never walked
     into.
   - **Terminal-state corruption.** A trapped king that is the side's
     only mover yields zero legal moves; `status()` reads checkmate /
     stalemate, but not because of check — a distortion of game-end
     detection that is hard to reason about.

   None of these involve moving into check, so the legal-set framing
   does not rescue them — hence the explicit exemption.
5. **Timed.** `remaining` decrements once per the engine's
   environment-reaction tick. At 0 the condition is removed and any
   trapped piece is freed on that tick — no event fires.
6. **Deterministic, FEN-serializable, no hidden state.** "Can side S
   legally move onto square X" is a pure function of the position — a
   sub-query the engine already runs.

## Naming

The **condition** is `Tornado` (the user's word; FEN tag `TORNADO`).

The **placer piece** is **`Stormcaller`** (user-confirmed) — it calls
the storm; reads cleanly next to the condition; one word, the existing
convention (Goblin, Skibidi, Bus). The name appears only in the new
piece module, its `PieceType` arm, and its FEN symbol; a future rename
would be mechanical (cf. plan 12's `Block`/`Wall`), but the decision
is settled.

## Types

### `engine/src/board/square.rs`

`SquareCondition` becomes the engine's first payload-carrying
condition. Today it is value-less (`Frozen`, `Brainrot`, ~line 152):

```rust
pub enum SquareCondition {
    Frozen,
    Brainrot,
    Tornado { remaining: u8 },   // NEW — payload-carrying, timed
}

impl SquareCondition {
    pub fn as_str(&self) -> &'static str {
        match self {
            SquareCondition::Frozen => "FROZEN",
            SquareCondition::Brainrot => "BRAINROT",
            SquareCondition::Tornado { .. } => "TORNADO",
        }
    }

    /// Full FEN value form, including payload. The serialize loop in
    /// `fen.rs` switches from `as_str()` to this so Tornado can carry
    /// its countdown. Value-less conditions are unchanged.
    pub fn to_fen(&self) -> String {
        match self {
            SquareCondition::Tornado { remaining } => format!("TORNADO:{remaining}"),
            other => other.as_str().to_string(),
        }
    }
}
```

`as_str()` stays (other callers use it for the bare tag); only the FEN
serialize site moves to `to_fen()`.

### `engine/src/board/mod.rs` — new `MoveType` arm

The placer spends its turn stamping a tornado at range — the same
shape as `ThrowSwitch { switch }` (`MoveType`, ~line 79):

```rust
pub enum MoveType {
    MoveTo(Coord),
    // ... existing arms ...
    ThrowSwitch { switch: Coord },
    PlaceTornado { target: Coord },   // NEW
}
```

`Display`, the `MoveKind` discriminant, and the `make_move` dispatch
match each get one arm (the compiler's exhaustiveness check lists
every site — same discipline as plan 12).

### `engine/src/movement/stack/square_filters.rs` — the compulsion

The compulsion is a **turn-level** rule: whether a candidate is
dropped depends on whether *some other piece of the same side* can
reach a tornado square. A plan-10 modifier's `apply` sees one event
at a time, but the deciding fact is a pure board predicate, so the
modifier recomputes it:

```rust
/// Priority 305 — runs *after* king-safety (300) so the probe and
/// the final set are both over legal moves. Touches CANDIDATE.
pub struct TornadoCompulsionFilter;

impl MovementModifier for TornadoCompulsionFilter {
    fn id(&self) -> &'static str { "square.tornado_compulsion" }
    fn priority(&self) -> u32 { 305 }
    fn touches(&self) -> EventKindMask { EventKindMask::CANDIDATE }

    fn apply(&self, board: &Board, event: &MovementEvent) -> MovementEffect {
        let MovementEvent::Candidate { mover, game_move } = event else {
            return MovementEffect::Keep;
        };
        // (4) King moves are exempt — never trapped, never compelled.
        if board.piece_at_is_king(mover) {
            return MovementEffect::Keep;
        }
        // (2) Trapped: the mover stands on a tornado square. Drop all
        //     of its candidates. (King already returned above.)
        if board.square_has_tornado(mover) {
            return MovementEffect::Drop;
        }
        // (1) Compulsion: if the side to move can reach ANY tornado
        //     square with a king-safe move, every non-tornado-landing
        //     candidate is illegal this turn.
        if board.side_can_reach_tornado(board.flags.side_to_move)
            && !destination_is_tornado(board, game_move)
        {
            return MovementEffect::Drop;
        }
        MovementEffect::Keep
    }
}
```

`side_can_reach_tornado` is the subtle part — see the next section.

### `engine/src/movement/env_reactions.rs` — the tick

A new reaction handler in the registry (sibling of `env.train_tick`),
running in the post-tick phase: for every square, if it carries
`Tornado { remaining }`, decrement; remove the condition at 0. No
side effect, no signal, idempotent.

### Placer piece — `engine/src/pieces/fairy/stormcaller.rs`

A new fairy piece following the Skibidi template (`Piece` trait:
`name`/`color`/`set_color`/`initial_moves`/`symbol`/`clone_box`/
`attacks`/`as_any`/`post_move_effects`, plus a `from_symbol`).
Registered in `pieces/fairy/mod.rs`, `PieceType` (the macro + enum arm
in `pieces/piecetype.rs`), and the `symbol_to_piece` dispatch.

- **Movement.** King-step (one square, eight directions) so it can
  reposition. Cannot capture (a placer, not a fighter) — mirrors
  Skibidi's "cannot take other pieces" arm.
- **Place action.** `initial_moves` additionally emits one
  `PlaceTornado { target }` per square within placement range (v1:
  king-radius of the Stormcaller; tunable). Placement onto an
  occupied square is **legal and central to the design** — that is
  the "trap their queen" play.
- **Resolution.** `make_move`'s `PlaceTornado` arm adds
  `SquareCondition::Tornado { remaining: TORNADO_DURATION }` to the
  target square. Constant duration set at placement (v1: `3`). The
  stamp goes through the same `board_after` write path Skibidi's
  `post_move_effects` uses for its phase reset.

## The compulsion predicate (the correctness heart)

`side_can_reach_tornado(side)` must answer "does `side` have at least
one king-safe move landing on a tornado square" — which means running
move generation. If it ran the *full* stack it would re-enter
`TornadoCompulsionFilter` → infinite recursion.

Break it with the stack's existing `max_priority` cap. King-safety is
priority 300; the compulsion filter is 305. The probe runs the
pipeline with `max_priority = Some(304)`: it sees king-safe moves
(300 ≤ 304) but **not** the compulsion filter (305 > 304). So the
probe asks "is a tornado square reachable by an otherwise-legal move,"
which is exactly the predicate, with no recursion.

The probe is O(side pieces × moves) per query and is invoked once per
candidate. Memoize it per `(board pointer, side)` for the duration of
one `resolve_legal_moves` pass — the board is `&` (immutable) through
the stack, so a pass-scoped cache is sound. Flag in "things to be
careful about."

Registration: add to `register_default_modifiers` in
`engine/src/movement/stack.rs`, after the king-safety modifier.
`resolve_legal_moves` already runs the full registry (no cap);
`resolve_moves` caps at 299 and so never sees the compulsion — correct,
because compulsion is a *legality* rule, not a raw-move rule, exactly
like king-safety.

`validate_move` (`board/mod.rs`) needs no change: it calls
`get_moves` for the raw-set membership check and then clone-applies for
king-safety. The compulsion lives in `legal_moves`/`resolve_legal_moves`
and in `status()` (checkmate/stalemate aggregation already iterates the
side's pieces through `legal_moves`), so a non-tornado move correctly
fails the `legal_moves` membership the GUI/API enforce. If a stricter
single-call rejection is wanted, add one `legal_moves` cross-check to
`validate_move`; recommend deferring (the existing layering covers the
real callers).

## FEN encoding

New condition value form, colon-suffixed payload inside the existing
repeatable `C=` key:

```
(C=TORNADO:3)            tornado, 3 ticks remaining
(C=TORNADO:1)            about to dissipate
(C=FROZEN,C=TORNADO:2)   conditions stack as before
```

Two edits in `engine/src/board/fen.rs`:

1. **Serialize** (~line 662, the `for cond in &square.conditions`
   loop): `format!("C={}", cond.as_str())` → `format!("C={}",
   cond.to_fen())`. Value-less conditions are byte-identical; only
   Tornado gains the `:n` suffix.
2. **Parse** (~line 901, the `"C"` arm): split the value on `:`.
   `"FROZEN"`/`"BRAINROT"` unchanged. `"TORNADO"` → parse the suffix
   as `u8`; absent or unparseable → default `3` with a `warn!`;
   clamp to `1..=255` (0 is meaningless — the tick would remove it
   the same turn; warn + default to `1`, mirroring Skibidi's phase
   clamp).

Round-trip is symmetric. An old FEN never emits `TORNADO`; an unknown
condition value already `warn!`s and is skipped (no behavioural
regression — same lenient posture as plan 12).

Stormcaller symbol: `W`/`w` (Stormcaller — pick a free letter at
implementation; verify against `symbol_to_piece` in
`pieces/piecetype.rs`). No piece payload needed — duration lives on
the *condition*, not the placer.

### Reference update (`plans/README.md`)

- `C` row, per-square keys table:
  `C=FROZEN`, `C=BRAINROT`, `C=TORNADO:<n>` — note Tornado is the
  first condition with a payload.
- Board/move note: add `PlaceTornado` to the `K=` move-kind list in
  the `lm=` row (last-move snapshot), alongside `TS` (ThrowSwitch).
- Add `Stormcaller` to the piece roster line in the overview.

## Frontend (out of scope, enumerated)

Mirrors plan 12's frontend follow-up. After the engine commit:

- `frontend/vite-dev/src/variables.ts` — add `"TORNADO"` to the
  condition union; the value carries `:n`.
- `frontend/vite-dev/src/fen.ts` — parse `C=TORNADO:n`.
- `frontend/vite-dev/src/editor_page.ts` — condition brush + a
  countdown numeral overlay (the Clock-style "3/2/1" read; the
  compelled-destination outline is a nice-to-have, not required for
  correctness).
- Stormcaller piece sprite + palette entry.

Ship as a follow-up commit or fold into the same PR — decoupled
either way, same as plan 12.

## Sequencing

Five commits, each leaving the engine green:

1. **Condition + FEN.** `SquareCondition::Tornado`, `to_fen()`,
   serialize/parse edits, round-trip test. No behaviour yet (nothing
   places it; the filter isn't registered).
2. **Env tick.** The countdown handler in `env_reactions.rs` +
   removal-at-0 test. Still inert without a placer/filter.
3. **Compulsion filter.** `TornadoCompulsionFilter` + the
   `max_priority`-capped probe + memoization, registered in
   `register_default_modifiers`. Trap + compulsion + king-exempt
   tests. This is the load-bearing commit.
4. **Stormcaller + `PlaceTornado`.** New `MoveType` arm, dispatch,
   the piece module, registration. End-to-end test: place → trap →
   forced capture → dissipate.
5. **Frontend.** Brush, FEN, sprite, countdown overlay.

Commits 1–2 can land independently; 3 depends on 1; 4 depends on 1+3.

## Tests to add

`engine/src/board/tests.rs` and the stack module tests, mirroring
plan 12's specificity:

- `test_tornado_fen_roundtrip` — `(C=TORNADO:3)` survives
  `board_to_fen → fen_to_board → board_to_fen`. Also `C=TORNADO`
  (no suffix) → defaults to 3; `C=TORNADO:0` → warns, becomes 1.
- `test_tornado_compulsion_restricts_set` — side has one move onto a
  tornado square and several elsewhere; `legal_moves` aggregated over
  the side yields only the tornado-landing move(s).
- `test_tornado_no_reachable_mover_is_noop` — tornado square exists
  but no piece of the side can reach it; full normal move set returns.
- `test_tornado_traps_occupant` — piece on a tornado square has zero
  legal moves; an adjacent friendly moves freely.
- `test_tornado_forces_enemy_capture` — enemy piece trapped on a
  tornado square, a captor in range: the side's legal set is exactly
  the capture(s) onto that square.
- `test_tornado_king_exempt` — king adjacent to a reachable tornado
  square with the side having a tornado-landing move elsewhere: the
  king's own moves are *not* restricted; a king standing on a tornado
  square (editor-placed) is *not* trapped.
- `test_tornado_compulsion_no_recursion` — a position with a tornado
  and the filter registered; `resolve_legal_moves` terminates and the
  `max_priority`-capped probe is exercised (assert via the stack's
  trace mode that the compulsion modifier did not re-enter itself).
- `test_tornado_tick_dissipates` — `remaining: 1`, run one env tick,
  condition gone, formerly-trapped piece moves again.
- `test_tornado_compulsion_intersects_check` — side in check; only
  check-resolving moves are in `L` *before* the filter; if one of
  them lands on the tornado, restricted to it; if none does, normal
  check-evasion set (the filter never forces an illegal escape).

## Things to be careful about

- **The compulsion is a set rule, not a per-piece rule.** It is
  correct only because the deciding fact (`side_can_reach_tornado`) is
  a pure board predicate. Do not try to express it without the
  whole-board probe; a naive per-event check that only looks at the
  current `from` square will under-restrict (it'll allow a non-tornado
  move when a *different* piece could have reached the tornado).
- **Recursion guard is load-bearing.** The probe *must* run capped
  below priority 305. A future refactor that changes the king-safety
  priority (300) or the cap convention has to keep `probe_cap <
  compulsion_priority ≤ everything that consumes the result`. Pin the
  priorities in a test comment.
- **Check interaction is free.** Phrasing the rule as "intersect with
  the already-king-safety-filtered set" means the scary case ("must I
  walk into check to obey the tornado?") cannot arise — check
  resolution is baked into `L` at priority 300 before the filter at
  305 runs. No special case; the `test_tornado_compulsion_intersects_check`
  test guards it.
- **Multi-tornado: satisfy one.** If several tornado squares are
  reachable, the side must move onto *some* one of them; one
  discharges the turn. The filter already does this (it keeps every
  candidate whose destination is *any* tornado square).
- **Frozen pauses Tornado.** A square that is both `Frozen` and
  `Tornado` should not tick down — consistent with Frozen halting
  condition/telegraph progression elsewhere (the Clock precedent in
  `piece_ideas/into_the_breach/the_clock.md`). Cheap: the env-tick
  handler skips squares whose conditions also contain `Frozen`.
  Pleasant interaction, worth keeping.
- **Tornado-zugzwang is a feature.** A position where every legal
  move dumps a piece into the tornado is a forced bleed. Intended —
  it is the composition use. The brakes are the countdown and the
  tempo the Stormcaller spent placing it, not a softening of the rule.
- **Memoize the probe.** One uncached probe per candidate is
  O(pieces × moves) inside an O(pieces × moves) aggregation — square
  cost. Pass-scoped memo keyed by side (board is immutable through the
  stack). Without it, large positions get slow; with it, it's one
  extra full move-gen per legal-move query.
- **`piece_at_is_king` / `square_has_tornado` / `side_can_reach_tornado`
  are new `Board` helpers.** Keep them small and pure; they are the
  only new surface on `Board` itself.

## Open questions

1. **Competitive duration cap.** v1 places `remaining = 3`. The Clock
   keeps its fuse short for the same reason; competitive variants may
   want `≤ 3` hard-capped while composition allows longer. Defer to a
   variant flag; v1 ships the constant.
2. **Tick cadence.** Per-ply or per-full-turn? The env-reaction
   registry runs per the existing tick cadence; recommend matching the
   train tick (whatever `TrainTickRate` resolves to for the position)
   so all timed board state shares one clock. Confirm.
3. **Placement onto a king's square.** King is exempt, so a tornado
   stamped under a king is inert until the king leaves. Recommend
   allow-but-inert (consistent, no extra rule) over forbidding it.
4. **Should the trap-source clause merge into `SquareConditionFilter`?**
   Frozen and a Tornado-occupied source both "drop all candidates from
   this square." Tempting to unify. Recommend keeping
   `TornadoCompulsionFilter` separate: it also owns the destination
   compulsion and the king-exemption, which Frozen has no analog for,
   and one modifier owning the whole Tornado story is easier to reason
   about than a behaviour split across two filters at different
   priorities.
5. **Multiple Stormcallers / multiple tornadoes interacting.** Each
   tornado is an independent condition; the compulsion is satisfied by
   reaching *any* of them. No interaction beyond that. The "must I
   feed two at once" question is answered by "no — one discharges the
   turn." Confirmed by the set semantics; no code needed.

## Relationship to plan 10

Tornado *is* a plan-10 modifier — the movement stack is shipped
(`default_stack()`, `resolve_legal_moves`, the 300-band king-safety
modifier, the `max_priority` cap all exist today). This plan adds one
modifier at priority 305 and one env-reaction handler; it does not
touch the pipeline shape. The only plan-10 mechanism it leans on that
no shipped modifier uses yet is the `max_priority` cap *for a probe
inside a modifier* — a clean, intended use of the existing cycle
guard, not a new capability.
</invoke>
