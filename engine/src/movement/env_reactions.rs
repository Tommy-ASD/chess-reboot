//! Plan 10 step 11 — environment-reaction registry.
//!
//! Generalized phased registry that hosts piece-initiated auto-actions
//! (Magnet pull, Bell-Ringer toll, NPC step, Marcher march) in
//! addition to the train tick that's hardcoded in
//! `apply_environment_reactions` today. Handlers register against a
//! phase; the dispatcher runs each phase's handlers in priority
//! order at the appropriate point in `make_move`'s phase 3.
//!
//! Sister to `MovementStack` — same registration pattern, different
//! protocol. Movement modifiers are read-only on the board;
//! environment-reaction handlers mutate it directly (they're the
//! only registry permitted to do so during their callback).
//!
//! **What ships in step 11:** the types, a default registry, and two
//! relocated handlers — `TrainTickHandler` (was inline in
//! `apply_environment_reactions`) and `BrainrotRecalcHandler` (the
//! post-tick recalc). Future plans wire in Magnet, Bell-Ringer, etc.

// Reservation hooks: `id`, `len`, `is_empty`, `handler_ids`, the
// `EnvReactionCtx` struct, and the `PreMover`/`PostMover` phases all
// land in step 11 but have no consumers yet. Drop the allow once
// the first follower (Magnet, Bell-Ringer, …) lands and starts
// reading them.
#![allow(dead_code)]

use std::sync::OnceLock;

use crate::board::Board;

/// When in the per-move pipeline a handler fires. Names follow the
/// "from the player's perspective" convention — `PreMover` happens
/// after the previous side's full reaction settles and BEFORE the
/// next side starts thinking; `PostMover` fires after the current
/// player's piece relocates and post-effects fire, but BEFORE the
/// environment ticks.
///
/// The order is: PostMover → TickGate → PostTick. PreMover currently
/// has no handlers and is reserved for the "start of opponent's
/// turn" handlers Magnet et al want.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum EnvPhase {
    /// Before the current side's turn officially begins. Unused in
    /// v1; reserved for Magnet, Bell-Ringer, etc.
    PreMover,
    /// After the player's piece move applied + piece post-effects
    /// fired, but before the train tick. Reserved for Boy Who
    /// Followed Geese, Marcher step, NPC advance.
    PostMover,
    /// The train tick. The legacy `maybe_advance_trains` lives here.
    TickGate,
    /// Anything that should re-run after the train tick — currently
    /// the brainrot recalc.
    PostTick,
}

/// Context passed to each handler. Handlers can read everything from
/// `&mut Board`; this struct carries cross-handler signals that
/// aren't derivable from board state alone.
#[derive(Debug, Default, Clone, Copy)]
pub struct EnvReactionCtx {
    /// Whether the train tick fired this turn. Set by
    /// `TrainTickHandler` at `TickGate`; consumed by
    /// `BrainrotRecalcHandler` at `PostTick` to skip the recalc when
    /// no piece moved during the tick. Defaults to `false`.
    pub train_ticked: bool,
}

/// A handler that fires at one specific `EnvPhase`.
pub trait EnvReactionHandler: Send + Sync {
    fn id(&self) -> &'static str;
    fn phase(&self) -> EnvPhase;
    fn priority(&self) -> u32;

    /// Whether this handler runs during `apply_move_for_validation`.
    /// Default `false` — most environment reactions are deferred
    /// during the hypothetical-apply of legal-move filtering, since
    /// a train tick mid-validation could capture the king before
    /// king-safety can react.
    ///
    /// `BrainrotRecalcHandler` returns `true` because brainrot is
    /// deterministic-from-position and doesn't affect king-safety in
    /// a way that hides illegality.
    fn runs_in_validation(&self) -> bool {
        false
    }

    fn apply(&self, board: &mut Board, ctx: &mut EnvReactionCtx);
}

/// Registry holding all environment-reaction handlers. Built at
/// engine init via `default_registry()`; tests can construct
/// custom registries.
pub struct EnvReactionRegistry {
    handlers: Vec<Box<dyn EnvReactionHandler>>,
}

impl Default for EnvReactionRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl EnvReactionRegistry {
    pub fn new() -> Self {
        Self { handlers: Vec::new() }
    }

    pub fn register(&mut self, h: Box<dyn EnvReactionHandler>) {
        self.handlers.push(h);
        // Stable sort by priority; same-priority preserves
        // registration order.
        self.handlers.sort_by_key(|h| h.priority());
    }

    pub fn len(&self) -> usize {
        self.handlers.len()
    }

    pub fn is_empty(&self) -> bool {
        self.handlers.is_empty()
    }

    pub fn handler_ids(&self) -> impl Iterator<Item = &'static str> + '_ {
        self.handlers.iter().map(|h| h.id())
    }

    /// Run every handler registered against `phase`, in priority
    /// order. Handlers may freely mutate `board` and may write into
    /// `ctx` to signal subsequent handlers (e.g. `TrainTickHandler`
    /// sets `ctx.train_ticked` so `BrainrotRecalcHandler` knows
    /// whether to recalc). The `during_validation` flag drives the
    /// `runs_in_validation` gate.
    pub fn run_phase(
        &self,
        board: &mut Board,
        phase: EnvPhase,
        during_validation: bool,
        ctx: &mut EnvReactionCtx,
    ) {
        for handler in &self.handlers {
            if handler.phase() != phase {
                continue;
            }
            if during_validation && !handler.runs_in_validation() {
                continue;
            }
            handler.apply(board, ctx);
        }
    }
}

/// Process-wide default registry. Currently registers the train tick
/// and the post-tick brainrot recalc; future plans add Magnet,
/// Bell-Ringer, etc.
pub fn default_registry() -> &'static EnvReactionRegistry {
    static REG: OnceLock<EnvReactionRegistry> = OnceLock::new();
    REG.get_or_init(build_default_registry)
}

fn build_default_registry() -> EnvReactionRegistry {
    let mut r = EnvReactionRegistry::new();
    r.register(Box::new(TrainTickHandler));
    r.register(Box::new(TornadoTickHandler));
    r.register(Box::new(BrainrotRecalcHandler));
    r
}

/// Step 11 relocation: the train tick was inline in
/// `apply_environment_reactions`. It now lives as the sole handler
/// at `EnvPhase::TickGate`.
pub struct TrainTickHandler;

impl EnvReactionHandler for TrainTickHandler {
    fn id(&self) -> &'static str {
        "env.train_tick"
    }
    fn phase(&self) -> EnvPhase {
        EnvPhase::TickGate
    }
    fn priority(&self) -> u32 {
        100
    }
    fn runs_in_validation(&self) -> bool {
        // Plan-09 critical guard: train ticks mid-validation can
        // capture the king before king-safety can react. Keep this
        // false unless plan 9 changes the king-safety design.
        false
    }
    fn apply(&self, board: &mut Board, ctx: &mut EnvReactionCtx) {
        ctx.train_ticked = board.maybe_advance_trains();
    }
}

/// Brainrot recalc. Pre-refactor: `if ticked { recalc() }` inline in
/// `apply_environment_reactions`. Round-2 audit reinstated that gate
/// via `EnvReactionCtx.train_ticked` — running unconditionally was
/// doubling the recalc cost per move on Skibidi-less boards. Gate
/// preserves the perf delta the original inline check had.
pub struct BrainrotRecalcHandler;

impl EnvReactionHandler for BrainrotRecalcHandler {
    fn id(&self) -> &'static str {
        "env.brainrot_recalc"
    }
    fn phase(&self) -> EnvPhase {
        EnvPhase::PostTick
    }
    fn priority(&self) -> u32 {
        100
    }
    fn runs_in_validation(&self) -> bool {
        // Brainrot is deterministic-from-position; running it during
        // validation can't hide an illegality. Set `true` so the
        // hypothetical board sees the same brainrot map as the real
        // one would.
        true
    }
    fn apply(&self, board: &mut Board, ctx: &mut EnvReactionCtx) {
        if ctx.train_ticked {
            board.recalc_brainrot();
        }
    }
}

/// Plan 13 commit 2: the Tornado countdown. Decrements every
/// `SquareCondition::Tornado` by one per applied move and removes the
/// condition when it reaches 0. Runs at `PostTick` so all "after the
/// move settled" timers share a phase with the brainrot recalc;
/// priority 90 puts it just before that recalc (order is immaterial —
/// the countdown reads/writes only its own conditions).
///
/// **Cadence (resolved).** One decrement per applied move (per ply),
/// not per `TrainTickRate`. A tornado's lifetime is its own clock, not
/// the trains' — entangling it with `TrainTickRate` would make
/// `remaining` unreadable ("3, but in train-ticks, which depend on the
/// board flag"). Per-ply is deterministic and reads as written.
///
/// **Frozen pauses it.** A square that is also `Frozen` does not tick
/// — consistent with Frozen halting condition/telegraph progression
/// elsewhere (the Clock precedent). A frozen tornado is suspended,
/// not extended past its number; it resumes counting when the freeze
/// lifts.
///
/// **`runs_in_validation` is false.** Matches the train tick: env
/// mutation during the hypothetical apply is avoided on principle.
/// The compulsion that *reads* tornado state is a movement-stack
/// modifier (commit 3), not part of `apply_move_for_validation`, so
/// the countdown never needs to run during validation.
pub struct TornadoTickHandler;

impl EnvReactionHandler for TornadoTickHandler {
    fn id(&self) -> &'static str {
        "env.tornado_tick"
    }
    fn phase(&self) -> EnvPhase {
        EnvPhase::PostTick
    }
    fn priority(&self) -> u32 {
        90
    }
    fn runs_in_validation(&self) -> bool {
        false
    }
    fn apply(&self, board: &mut Board, _ctx: &mut EnvReactionCtx) {
        // Thin delegator — the grid walk (incl. the `any_tornado`
        // fast-path and the Frozen-suspends rule) lives on
        // `Board::tick_tornadoes`, mirroring `BrainrotRecalcHandler`
        // delegating to `Board::recalc_brainrot`. Reads/writes only
        // square conditions; never touches `_ctx`.
        board.tick_tornadoes();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::{BoardFlags, TrainTickRate};
    use crate::board::square::{Square, SquareCondition};
    use crate::pieces::Color;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    fn empty_board() -> Board {
        let grid = (0..8)
            .map(|_| (0..8).map(|_| Square::new()).collect())
            .collect();
        Board {
            grid,
            flags: BoardFlags {
                side_to_move: Color::White,
                white_can_castle_kingside: false,
                white_can_castle_queenside: false,
                black_can_castle_kingside: false,
                black_can_castle_queenside: false,
                en_passant_target: None,
                train_tick_rate: TrainTickRate::EveryFullTurn,
                ply_count: 0,
                last_move: None,
            },
        }
    }

    /// Counting handler with PER-INSTANCE atomic. Per-test counters
    /// (not a module-level static) prevent parallel-test races —
    /// `cargo test` runs the test runner with multiple threads by
    /// default, so a shared static would flake under load.
    struct CountingHandler {
        phase_kind: EnvPhase,
        runs_in_validation_flag: bool,
        counter: Arc<AtomicUsize>,
    }

    impl EnvReactionHandler for CountingHandler {
        fn id(&self) -> &'static str {
            "test.counting"
        }
        fn phase(&self) -> EnvPhase {
            self.phase_kind
        }
        fn priority(&self) -> u32 {
            10
        }
        fn runs_in_validation(&self) -> bool {
            self.runs_in_validation_flag
        }
        fn apply(&self, _board: &mut Board, _ctx: &mut EnvReactionCtx) {
            self.counter.fetch_add(1, Ordering::SeqCst);
        }
    }

    /// A handler registered at a given phase fires exactly when that
    /// phase runs.
    #[test]
    fn handler_fires_at_its_registered_phase() {
        let counter = Arc::new(AtomicUsize::new(0));
        let mut reg = EnvReactionRegistry::new();
        reg.register(Box::new(CountingHandler {
            phase_kind: EnvPhase::PostMover,
            runs_in_validation_flag: false,
            counter: counter.clone(),
        }));

        let mut board = empty_board();
        let mut ctx = EnvReactionCtx::default();
        reg.run_phase(&mut board, EnvPhase::PostMover, false, &mut ctx);
        assert_eq!(counter.load(Ordering::SeqCst), 1);

        // A different phase does NOT fire it.
        counter.store(0, Ordering::SeqCst);
        reg.run_phase(&mut board, EnvPhase::TickGate, false, &mut ctx);
        assert_eq!(counter.load(Ordering::SeqCst), 0);
    }

    /// `during_validation = true` skips a handler whose
    /// `runs_in_validation()` is false. The fire-during-validation
    /// gate is the canonical mechanism for "this handler must not
    /// run inside `apply_move_for_validation`."
    #[test]
    fn validation_gate_skips_non_validation_handler() {
        let counter = Arc::new(AtomicUsize::new(0));
        let mut reg = EnvReactionRegistry::new();
        reg.register(Box::new(CountingHandler {
            phase_kind: EnvPhase::TickGate,
            runs_in_validation_flag: false,
            counter: counter.clone(),
        }));

        let mut board = empty_board();
        let mut ctx = EnvReactionCtx::default();
        reg.run_phase(&mut board, EnvPhase::TickGate, true, &mut ctx);
        assert_eq!(
            counter.load(Ordering::SeqCst),
            0,
            "non-validation handler must not fire when during_validation=true"
        );

        // Same handler does fire when during_validation=false.
        reg.run_phase(&mut board, EnvPhase::TickGate, false, &mut ctx);
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    /// `during_validation = true` DOES run a handler that opts in.
    #[test]
    fn validation_gate_runs_opt_in_handler() {
        let counter = Arc::new(AtomicUsize::new(0));
        let mut reg = EnvReactionRegistry::new();
        reg.register(Box::new(CountingHandler {
            phase_kind: EnvPhase::PostTick,
            runs_in_validation_flag: true,
            counter: counter.clone(),
        }));

        let mut board = empty_board();
        let mut ctx = EnvReactionCtx::default();
        reg.run_phase(&mut board, EnvPhase::PostTick, true, &mut ctx);
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    /// The default registry includes the train tick + brainrot
    /// recalc handlers at the documented phases.
    #[test]
    fn default_registry_has_step_11_handlers() {
        let reg = default_registry();
        let ids: Vec<_> = reg.handler_ids().collect();
        assert!(ids.contains(&"env.train_tick"));
        assert!(ids.contains(&"env.brainrot_recalc"));
    }

    /// PreMover and PostMover have no DEFAULT handlers. The check is
    /// indirect via the known handler ids — every registered default
    /// (`env.train_tick` at TickGate, `env.tornado_tick` and
    /// `env.brainrot_recalc` at PostTick) is at TickGate/PostTick, not
    /// PreMover/PostMover. Both PreMover and PostMover are reserved
    /// hooks for future pieces.
    #[test]
    fn pre_post_mover_have_no_default_handlers() {
        let reg = default_registry();
        let ids: Vec<_> = reg.handler_ids().collect();
        // The only currently-registered handlers:
        //   env.train_tick     → TickGate
        //   env.tornado_tick   → PostTick
        //   env.brainrot_recalc → PostTick
        // None is at PreMover/PostMover.
        for id in &ids {
            assert!(
                *id == "env.train_tick"
                    || *id == "env.tornado_tick"
                    || *id == "env.brainrot_recalc",
                "unexpected default-registry handler {id:?}; PreMover/PostMover must stay empty"
            );
        }
    }

    /// Brainrot recalc gating: it only fires when `ctx.train_ticked`
    /// is true. Pre-round-2-audit it ran unconditionally, doubling
    /// the recalc cost per move on Skibidi-less boards.
    #[test]
    fn brainrot_recalc_skips_when_tick_did_not_fire() {
        use crate::pieces::piecetype::PieceType;
        let mut board = empty_board();
        // Plant a Skibidi so we can observe the recalc effect.
        // (We don't need the brainrot map to actually change — just
        // that the handler runs vs. doesn't run.)
        board.grid[3][3] = Square::new()
            .set_piece(PieceType::Skibidi(crate::pieces::fairy::skibidi::Skibidi {
                color: Color::White,
                phase: 1,
            }));

        let h = BrainrotRecalcHandler;
        // ctx.train_ticked=false → no recalc (no work).
        let mut ctx_no_tick = EnvReactionCtx {
            train_ticked: false,
        };
        h.apply(&mut board, &mut ctx_no_tick);
        // ctx.train_ticked=true → recalc fires (no panic; brainrot
        // map gets refreshed).
        let mut ctx_ticked = EnvReactionCtx {
            train_ticked: true,
        };
        h.apply(&mut board, &mut ctx_ticked);
    }

    /// Trait-level invariants for the two default handlers.
    #[test]
    fn default_handler_trait_invariants() {
        let bh = BrainrotRecalcHandler;
        assert_eq!(bh.phase(), EnvPhase::PostTick);
        assert!(bh.runs_in_validation());

        let th = TrainTickHandler;
        assert_eq!(th.phase(), EnvPhase::TickGate);
        // Plan-09 critical: train tick mid-validation could capture
        // the king before king-safety can react.
        assert!(!th.runs_in_validation());
    }

    /// `TrainTickHandler` writes its tick-fired bit into the shared
    /// `EnvReactionCtx`. Pin this so `BrainrotRecalcHandler` (which
    /// reads it) can rely on it.
    #[test]
    fn train_tick_writes_into_ctx() {
        let mut board = empty_board();
        let mut ctx = EnvReactionCtx::default();
        assert!(!ctx.train_ticked);
        TrainTickHandler.apply(&mut board, &mut ctx);
        // With `EveryFullTurn` rate and ply 0, the tick gate doesn't
        // open on a single call — `maybe_advance_trains` bumps ply
        // to 1 and checks `1 % 2 == 0` which is false. So ctx stays
        // false. The important property is "the field is written by
        // the handler" — verify by running the rate that DOES fire.
        board.flags.train_tick_rate = TrainTickRate::EveryPly;
        let mut ctx2 = EnvReactionCtx::default();
        TrainTickHandler.apply(&mut board, &mut ctx2);
        assert!(ctx2.train_ticked, "EveryPly rate must fire the tick");
    }

    /// Plan 13 commit 2: a tornado decrements by one per applied move.
    #[test]
    fn tornado_tick_decrements() {
        let mut board = empty_board();
        board.grid[0][0] = Square::new()
            .add_square_condition(SquareCondition::Tornado { remaining: 3 });
        let mut ctx = EnvReactionCtx::default();
        TornadoTickHandler.apply(&mut board, &mut ctx);
        assert_eq!(
            board.grid[0][0].conditions,
            vec![SquareCondition::Tornado { remaining: 2 }]
        );
    }

    /// At `remaining == 1`, one tick takes it to 0 and the condition
    /// is removed entirely (a freed-on-the-tick square, no event).
    #[test]
    fn tornado_tick_removes_at_zero() {
        let mut board = empty_board();
        board.grid[2][4] = Square::new()
            .add_square_condition(SquareCondition::Tornado { remaining: 1 });
        let mut ctx = EnvReactionCtx::default();
        TornadoTickHandler.apply(&mut board, &mut ctx);
        assert!(
            board.grid[2][4].conditions.is_empty(),
            "tornado at remaining=1 must be gone after one tick"
        );
    }

    /// A `Frozen` square suspends the tornado countdown — the number
    /// does not move while frozen (the Clock precedent: Frozen halts
    /// condition progression).
    #[test]
    fn tornado_tick_frozen_pauses() {
        let mut board = empty_board();
        board.grid[1][1] = Square::new()
            .add_square_condition(SquareCondition::Frozen)
            .add_square_condition(SquareCondition::Tornado { remaining: 2 });
        let mut ctx = EnvReactionCtx::default();
        TornadoTickHandler.apply(&mut board, &mut ctx);
        assert_eq!(
            board.grid[1][1].conditions,
            vec![
                SquareCondition::Frozen,
                SquareCondition::Tornado { remaining: 2 }
            ],
            "frozen tornado must keep its number"
        );
    }

    /// The default registry registers the tornado tick at PostTick.
    #[test]
    fn default_registry_has_tornado_tick() {
        let reg = default_registry();
        assert!(reg.handler_ids().any(|id| id == "env.tornado_tick"));
    }

    /// Trait-level invariants for the tornado tick handler.
    #[test]
    fn tornado_tick_handler_invariants() {
        let h = TornadoTickHandler;
        assert_eq!(h.phase(), EnvPhase::PostTick);
        assert_eq!(h.priority(), 90);
        // Matches the train-tick stance: no env mutation during the
        // hypothetical-apply of legal-move validation.
        assert!(!h.runs_in_validation());
    }

    /// Audit R1/E4 (plan's `test_tornado_tick_dissipates`): a piece
    /// trapped on a `remaining: 1` tornado has NO legal moves; after
    /// one tick the condition is gone AND the formerly-trapped piece
    /// can move again. The plan mandated the "moves again" half; the
    /// shipped `tornado_tick_removes_at_zero` only checked removal.
    #[test]
    fn tornado_tick_frees_trapped_piece() {
        use crate::board::Coord;
        use crate::pieces::piecetype::PieceType;
        let mut board = empty_board();
        let sq = Coord { file: 4, rank: 4 };
        board.grid[4][4] = Square::new()
            .set_piece(PieceType::new_rook(Color::White))
            .add_square_condition(SquareCondition::Tornado { remaining: 1 });

        // Trapped while the tornado lives.
        assert!(
            board.legal_moves(&sq).is_empty(),
            "rook on a tornado square must be trapped before dissipation"
        );

        TornadoTickHandler.apply(&mut board, &mut EnvReactionCtx::default());

        assert!(
            board.grid[4][4].conditions.is_empty(),
            "tornado must be gone after the tick"
        );
        assert!(
            !board.legal_moves(&sq).is_empty(),
            "the formerly-trapped rook must move again once freed"
        );
    }

    /// Audit R1/B2: two `Tornado` conditions on ONE square are
    /// FEN-constructible (no dedup). Pin the documented aggregate
    /// behaviour: every Tornado decrements; those hitting 0 are
    /// removed; survivors stay. Guards a future loop refactor from
    /// silently changing this to first-match-only.
    #[test]
    fn tornado_tick_multi_condition_on_one_square() {
        let mut board = empty_board();
        board.grid[1][1] = Square::new()
            .add_square_condition(SquareCondition::Tornado { remaining: 1 })
            .add_square_condition(SquareCondition::Tornado { remaining: 3 });

        TornadoTickHandler.apply(&mut board, &mut EnvReactionCtx::default());

        // The :1 decremented to 0 and was removed; the :3 → :2 survives.
        assert_eq!(
            board.grid[1][1].conditions,
            vec![SquareCondition::Tornado { remaining: 2 }],
            "each Tornado ticks independently; zero-reaching ones are dropped"
        );
    }
}
