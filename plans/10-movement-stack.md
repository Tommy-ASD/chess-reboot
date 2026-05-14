# Plan 10: Movement Stack

A unified pipeline for the "this thing modifies that thing" rules that
currently live as ad-hoc conditionals scattered across pieces and
square types. The seed idea is the doc at [`003-Movement stack.md`](../003-Movement%20stack.md):
inspired by *Binding of Isaac: Four Souls*, each modifier sits as a
stack frame that transforms the move (or threat) set produced by the
frames below it.

This plan does *not* propose ripping out the current logic in one
commit. It defines the abstraction, lists which existing rules each
layer would absorb, and prescribes a migration sequence small enough
to land one layer at a time.

## Why now

The trigger is plan 09 (trains) — each new train interaction (same-
train cart self-collision, locomotive wrap-around, collision-handler
hooks) has added a one-off conditional to `Piece::attacks` or to
`is_attacked_by`. The pattern generalises:

- **Movement gates**: brainrot, frozen, gate (closed), turret, vent —
  each lives as a `get_moves` or piece-level walkability check today.
- **Capture filters**: train carts (any train — round-4 broadened
  this from same-train only), future "stunned" / "phased" pieces —
  each shows up as a return-true tweak in some piece's `attacks`
  or in `make_move_unchecked`.
- **Auto-mechanics that shape threats**: trains contribute next-tick
  tiles; future "weather" / aura modifiers will want the same hook.

Without an abstraction these accrete in O(N²) directions — every new
modifier has to know about every existing one. The movement stack
inverts the relationship: each modifier declares what it does in
terms of a generic event/effect protocol, and the pipeline orders +
applies them.

## Concept

Three core ideas:

1. **Movement events** are immutable proposals. The base case is the
   piece's raw `initial_moves`. As the pipeline runs, each modifier
   may emit *new* events (e.g. "this turret blocks the e-file") or
   transform an existing one.
2. **Modifiers** are registered globally and queried in priority
   order. Each modifier is a `(predicate, transform)` pair: the
   predicate decides whether it applies to a given event; the
   transform returns the modified set.
3. **The stack** is the ordered list of modifiers, evaluated bottom-
   up: piece-intrinsic rules at the bottom, board-state effects in
   the middle, global / configuration effects at the top.

The output is a final `Vec<GameMove>` (for legality) or a final
`AttackSet` (for king-safety). Same machinery, two consumers.

## Types

```rust
/// The smallest unit the pipeline operates on. A "move candidate" or
/// a "threat candidate" — discriminated so modifiers can target one
/// or the other.
#[derive(Clone, Debug)]
pub enum MovementEvent {
    Candidate {
        mover: Coord,
        game_move: GameMove,
    },
    Threat {
        attacker: Coord,
        attacker_piece: PieceType,
        target: Coord,
    },
}

/// What a modifier returns after seeing an event.
#[derive(Clone, Debug)]
pub enum MovementEffect {
    /// Don't change the event.
    Keep,
    /// Drop the event from the stack.
    Drop,
    /// Replace the event with new ones (zero, one, or many).
    Replace(Vec<MovementEvent>),
    /// Keep the event and add more.
    Augment(Vec<MovementEvent>),
}

/// One layer of the stack. Modifiers are stateless functions of
/// `(board, event)` — all state lives on the board. Side effects
/// (firing signals, queuing post-move hooks) go through the
/// existing `Board` mutators after the stack resolves.
pub trait MovementModifier {
    /// Stable identifier so the registry can dedupe, log, and order
    /// modifiers without name collisions.
    fn id(&self) -> &'static str;

    /// Priority. Lower numbers run first. Convention: piece-
    /// intrinsic at 0..99, square-type at 100..199, board-wide /
    /// global at 200+. Trains and trains-derived modifiers sit at
    /// the square-type / board-wide layer.
    fn priority(&self) -> u32;

    /// Apply to a single event. Returning `Keep` is the no-op fast
    /// path the registry uses to skip irrelevant modifiers cheaply.
    fn apply(&self, board: &Board, event: &MovementEvent) -> MovementEffect;
}

/// The stack itself. The registry is built at engine init; modifiers
/// don't change at runtime.
pub struct MovementStack {
    modifiers: Vec<Box<dyn MovementModifier>>,
}

impl MovementStack {
    pub fn resolve_moves(&self, board: &Board, from: &Coord) -> Vec<GameMove> { /* ... */ }
    pub fn resolve_threats(&self, board: &Board, target: &Coord) -> Vec<Coord> { /* ... */ }
}
```

## What each layer absorbs

| Priority band | Modifier kind | Replaces today's |
|--|--|--|
| 0–99 | Piece-intrinsic move generation | `Piece::initial_moves` per type |
| 0–99 | Piece-intrinsic capture rules | The `would_capture_at` predicate (introduced in this iteration) |
| 100–199 | Square walkability | `SquareType::is_walkable` checks scattered in piece generators |
| 100–199 | Square conditions | Brainrot / Frozen short-circuits in `Board::get_moves` |
| 100–199 | Square type effects | Switch's extra `ThrowSwitch`, junction's branch dispatch |
| 200–299 | Train geometry | Locomotive's next-tick tile contribution to threats |
| 200–299 | King-safety filter | The clone-and-apply check in `validate_move` / `legal_moves` |
| 300+ | Future global modifiers | Weather, aura ranges, scenario rules |

Each row is one or more `MovementModifier`s. The registry sorts and
applies them in priority order, so a square's "frozen" effect (mid-
band) overrides a piece's "I can move here" (low band), but a
board-wide "no diagonal moves on Fridays" (high band) overrides both.

## What the new train layer looks like

Once the stack is real, plan 09's accumulated conditionals collapse
into three small modifiers:

```rust
struct TrainHeadCrushModifier;
impl MovementModifier for TrainHeadCrushModifier {
    fn priority(&self) -> u32 { 210 }
    fn apply(&self, board: &Board, event: &MovementEvent) -> MovementEffect {
        // For each loco, emit a Threat event on its `next_train_step` tile.
        // Pure additive — Augment, never Drop.
    }
}

struct TrainCartCaptureFilter;
impl MovementModifier for TrainCartCaptureFilter {
    fn priority(&self) -> u32 { 211 }
    fn apply(&self, board: &Board, event: &MovementEvent) -> MovementEffect {
        // Drop Threat events whose target tile holds *any* cart —
        // same-train (chain-follow) or foreign (stop short). Neither
        // is a real capture, so king-safety queries on a king parked
        // in a cart at the train's next-tile should not see a threat.
        // (Round-4 broadening; was: same-train carts only.)
    }
}

struct TwoTrainCollisionFilter;
impl MovementModifier for TwoTrainCollisionFilter {
    fn priority(&self) -> u32 { 212 }
    fn apply(&self, board: &Board, event: &MovementEvent) -> MovementEffect {
        // Drop Threat events where two locomotives share the same
        // next-tick tile — they Stop, neither captures.
    }
}
```

No conditional fan-out in `Piece::attacks`. Adding a new train rule
("locomotive that lays track behind it", say) is a new modifier; no
existing modifier needs to know about it.

## Migration sequence

Eight commits, in this order. Each leaves the engine fully working
with all existing tests passing.

1. **Type scaffolding.** Land `MovementEvent`, `MovementEffect`,
   `MovementModifier`, `MovementStack`. No production callers yet —
   just unit tests proving the pipeline runs.
2. **Threat resolution shim.** Reimplement `Board::is_attacked_by`
   on top of the stack's `resolve_threats`. The stack starts with a
   single modifier that wraps each existing `Piece::attacks`. No
   behavior change.

   **Status (partial — landed in plan-09 audit):** the
   `Piece::would_capture_at` predicate already centralises the
   per-piece "is this reachable tile a real capture?" filter that
   step 2's shim was going to host. When step 2 lands, the predicate
   collapses into a low-priority modifier and the trait method goes
   away. Until then, the predicate carries the same load.
3. **Piece-intrinsic attacks migrated.** Each piece type becomes its
   own modifier in the 0–99 band. `Piece::attacks` is gone; the trait
   no longer has the method.
4. **Square gates migrated.** Walkability, brainrot, frozen — all
   become 100–199 modifiers consuming `Candidate` events.
5. **Train-head crush modifier.** Replaces `Locomotive::attacks`'
   next-tick contribution. Plan-09 `next_train_step` stays as the
   geometry primitive.
6. **Train self-collision + two-train filters.** Replace the
   per-piece conditionals from plan 09.
7. **Move-generation migrated.** `Board::get_moves` becomes a thin
   call to `resolve_moves`. Piece-intrinsic moves are 0–99
   modifiers (mirroring step 3 for threats).
8. **King-safety filter migrated.** The clone-and-apply check in
   `validate_move` / `legal_moves` becomes a high-priority modifier
   that drops `Candidate` events whose post-apply state leaves the
   mover's king in `resolve_threats`.

Step 8 is the payoff: king-safety becomes one entry in the registry,
not a special-cased method on `Board`. The simulation/conservative
attack-set tradeoff debated in plan 09's threat handling becomes a
modifier choice — swap one modifier for another and the entire
engine adopts the new policy.

## Things to be careful about

- **Performance.** The naive implementation calls every modifier
  against every event. For the common case (8×8 board, no
  modifiers active), this is O(M·E) with small constants. The
  registry should fast-path modifiers whose `apply` is `Keep` for
  the event kind they don't care about (`Threat` vs `Candidate`).
- **Ordering subtlety.** Same-priority modifiers run in registration
  order; tests should pin the order explicitly so a future
  rearrangement doesn't silently flip semantics. Resist the urge to
  add ties.
- **Cycles.** A modifier that `Augment`s with events that another
  modifier `Augment`s back could loop. The pipeline must bound
  iterations (a simple "no event can be added more than once"
  invariant should suffice for v1).
- **Side-effect-free.** Modifiers must not mutate the board. All
  state mutation happens after the stack returns, through the
  existing `make_move_unchecked` path. This is enforced by the
  `&Board` (not `&mut Board`) arg in `apply`.
- **Stack vs. dispatch.** The stack is for *generation* (what moves
  exist, what threats exist). It does not replace `make_move`'s
  dispatch on `MoveType` — that stays as a match. Modifiers can emit
  any `MoveType` they like; the dispatch handles each.
- **Discoverability of effects.** With effects scattered as
  modifiers, "why is this move illegal?" gets harder to answer. The
  registry should retain a debug-trace mode that returns the list of
  modifiers that touched a given event, so editor / API consumers
  can surface "blocked by frozen tile" rather than just "no such
  move."

## Open questions

1. **Composite events.** A `MoveIntoCarrier` rewriting a `MoveTo` is
   today handled inside the piece-type filter. Is that a single
   modifier (rewrite via `Replace`), or two modifiers (drop the
   `MoveTo`, augment with a `MoveIntoCarrier`)? The two-modifier
   form is more orthogonal but more verbose. Recommend: single
   `Replace` per logical rewrite, even if it touches multiple
   variants.
2. **Modifier authoring.** The current style — each modifier its
   own struct + impl — is verbose. A macro / builder might be worth
   it once the registry has ten-plus entries. Defer.
3. **Cross-cell modifiers.** A "this tile beams light five squares
   north" effect needs the stack to know about *future* events
   (events on tiles five squares north). Current sketch handles
   single-event apply; cross-cell needs a second pass or a
   pre-computed "global effects" map. Out of scope for v1.
4. **Per-color modifiers.** Some effects only apply to one side
   (e.g. a turret aligned to white captures only black pieces).
   `MovementEvent::Threat` already carries the attacker piece;
   modifiers can read its color. No special-case needed.
5. **Serialization.** Modifiers as code aren't FEN-serializable.
   The board-state mutations they read from *are*. So the stack is
   a property of the engine binary, not the position. A position
   played with engine v1.2 may resolve differently under v1.3 if
   the modifier set changed. Document this and accept it as the
   cost of having modifiers live in code rather than config.
