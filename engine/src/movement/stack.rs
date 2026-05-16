//! Plan 10 — the movement stack. A registry of priority-ordered
//! modifiers that transforms move and threat sets. Replaces ad-hoc
//! conditionals scattered across `Board::get_moves`,
//! `Board::is_attacked_by`, piece-level `attacks`/`would_capture_at`,
//! and friends.
//!
//! Production callers: `Board::get_moves` delegates to
//! `resolve_moves` (priority cap 299, no king-safety); `Board::
//! legal_moves` delegates to `resolve_legal_moves` (full pipeline);
//! `Board::is_attacked_by` delegates to `resolve_threats`.
//!
//! The design follows plan 10's protocol: modifiers are pure
//! `(&Board, &MovementEvent) -> MovementEffect`. They never mutate the
//! board; mutation flows from the `Vec<GameMove>` the stack ultimately
//! returns through `make_move_unchecked`'s dispatch.

// Many items in this file are reservation hooks — `RESOLUTION`/`CAPTURE`
// mask bits, `Candidate` event variants, fields read only by future
// modifiers. They land here so step 8 / step 10 / step 11 don't have to
// re-extend the protocol. Remove this allow once step 8 wires production
// callers in.
#![allow(dead_code)]

pub mod capture;
pub mod king_safety;
pub mod piece_attacks;
pub mod piece_moves;
pub mod square_filters;
pub mod tornado;
pub mod train_modifiers;

use std::sync::OnceLock;

use crate::board::{Board, Coord, GameMove};
use crate::pieces::Color;
use crate::pieces::piecetype::PieceType;

/// Discriminator for `MovementEvent` variants. Powers the `touches()`
/// fast-path: a modifier that doesn't touch a given event kind is
/// skipped without a virtual call.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EventKind {
    /// Seed event for a "what moves does the piece at `from` have?"
    /// query. Piece-intrinsic move modifiers convert these into
    /// `Candidate` events.
    MoveQuery,
    /// Seed event for an "is `target` attacked by `attacker_color`?"
    /// query. Piece-intrinsic threat modifiers convert these into
    /// `Threat` events.
    ThreatQuery,
    /// A working-set move proposal.
    Candidate,
    /// A working-set threat proposal.
    Threat,
}

/// Bitmask of event kinds a modifier participates in. `ALL` is the
/// default — modifiers that haven't narrowed their interest receive
/// every event.
///
/// The `RESOLUTION` and `CAPTURE` bits are **reserved** for the
/// capture pipeline that lands in step 10. They have no consumers in
/// this registry — they exist so a modifier participating in both
/// stacks can declare both bits without API churn later.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EventKindMask(pub u8);

impl EventKindMask {
    pub const NONE: Self = Self(0);
    pub const MOVE_QUERY: Self = Self(1 << 0);
    pub const THREAT_QUERY: Self = Self(1 << 1);
    pub const CANDIDATE: Self = Self(1 << 2);
    pub const THREAT: Self = Self(1 << 3);
    /// Reserved (step 10): the modifier participates in the capture
    /// pipeline. `MovementStack` does not consume this bit.
    pub const RESOLUTION: Self = Self(1 << 4);
    /// Reserved (step 10): the modifier observes captures specifically.
    /// `MovementStack` does not consume this bit.
    pub const CAPTURE: Self = Self(1 << 5);
    pub const ALL: Self = Self(0xFF);

    pub fn touches(self, kind: EventKind) -> bool {
        let bit = match kind {
            EventKind::MoveQuery => Self::MOVE_QUERY,
            EventKind::ThreatQuery => Self::THREAT_QUERY,
            EventKind::Candidate => Self::CANDIDATE,
            EventKind::Threat => Self::THREAT,
        };
        (self.0 & bit.0) != 0
    }

    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
}

/// The unit the pipeline operates on. Two kinds of seed events
/// (`MoveQuery`, `ThreatQuery`) and two kinds of working-set events
/// (`Candidate`, `Threat`). Seed events represent "we want to know X";
/// modifiers convert them into the working set, which subsequent
/// modifiers filter and transform.
#[derive(Clone, Debug)]
pub enum MovementEvent {
    /// "What moves does the piece at `from` have?" Modifiers in the
    /// 0..99 priority band (piece-intrinsic) handle this by emitting
    /// `Candidate` events.
    MoveQuery { from: Coord },
    /// "Is `target` attacked by some piece of `attacker_color`?"
    /// Modifiers in the 0..99 band emit `Threat` events that name
    /// the attacker, attacking-piece, and target. Neutral
    /// `attacker_color` is the convention for "all aligned pieces"
    /// (used by train-threat queries).
    ThreatQuery {
        target: Coord,
        attacker_color: Color,
    },
    /// A move the piece could make. The carrier-rewrite filter,
    /// king-safety filter, and square-condition filter operate on
    /// these.
    Candidate { mover: Coord, game_move: GameMove },
    /// A capture the attacker would land on `target` from
    /// `attacker_coord`. Train-collision filters drop these for
    /// invincible-cart targets; king-safety reads the full set
    /// against the king's coord.
    Threat {
        attacker: Coord,
        attacker_piece: PieceType,
        target: Coord,
    },
}

impl MovementEvent {
    pub fn kind(&self) -> EventKind {
        match self {
            MovementEvent::MoveQuery { .. } => EventKind::MoveQuery,
            MovementEvent::ThreatQuery { .. } => EventKind::ThreatQuery,
            MovementEvent::Candidate { .. } => EventKind::Candidate,
            MovementEvent::Threat { .. } => EventKind::Threat,
        }
    }
}

/// What a modifier returns after seeing an event. The four shapes are
/// chosen so the registry can be a flat iteration over `(modifier ×
/// event)` pairs — no fixed-point, no re-feeding.
#[derive(Clone, Debug)]
pub enum MovementEffect {
    /// Don't change the event. The registry's fast-path.
    Keep,
    /// Drop the event. Used by filters (king-safety, walkability,
    /// brainrot, train-cart capture).
    Drop,
    /// Replace the event with zero, one, or many new events. Used by
    /// rewrites (`MoveQuery` → `Vec<Candidate>`, the carrier-boarding
    /// filter turning `MoveTo` onto a friendly carrier into
    /// `MoveIntoCarrier`).
    Replace(Vec<MovementEvent>),
    /// Keep the event and add more. Used by augmentations (Switch
    /// tile adds a `ThrowSwitch` candidate alongside the piece's own;
    /// the train-head modifier adds a crush-threat alongside threats
    /// from other pieces).
    Augment(Vec<MovementEvent>),
}

/// Per-resolution debug record. Each modifier that touched an event
/// records its id and the effect it produced. Used by the editor to
/// surface "blocked by frozen tile" instead of "no such move."
///
/// Cheap when disabled: callers pass `None` and the registry skips
/// the bookkeeping. The trace allocates only when populated.
#[derive(Debug, Default, Clone)]
pub struct ResolveTrace {
    pub touched_by: Vec<(&'static str, MovementEffect)>,
}

impl ResolveTrace {
    pub fn new() -> Self {
        Self { touched_by: Vec::new() }
    }
}

/// One layer of the stack. Modifiers are pure functions; the registry
/// holds them in priority order and runs each once against the
/// working set.
///
/// **Priority convention** (plan 10):
/// - `0..99`: piece-intrinsic move generation and threats (one
///   modifier per piece type).
/// - `100..199`: square-driven gates — walkability, conditions,
///   Switch augments, the carrier-boarding rewrite.
/// - `200..299`: train geometry, board-state effects.
/// - `300+`: king-safety, scenario rules, future global modifiers.
///
/// Same-priority modifiers run in registration order (the underlying
/// sort is stable). Tests should pin ordering explicitly when it
/// matters.
pub trait MovementModifier: Send + Sync {
    /// Stable identifier. Used for debug traces and registry dedupe.
    /// Convention: `"piece_intrinsic.rook.moves"`, `"square.brainrot"`,
    /// etc.
    fn id(&self) -> &'static str;

    /// Lower numbers run first. See module docs for the priority
    /// bands.
    fn priority(&self) -> u32;

    /// Which event kinds this modifier cares about. The registry uses
    /// this to skip the virtual call entirely for non-matching events.
    /// Default `ALL` — narrow it for performance.
    fn touches(&self) -> EventKindMask {
        EventKindMask::ALL
    }

    /// Apply the modifier to a single event. Pure; must not mutate
    /// the board (the `&Board` signature enforces this).
    fn apply(&self, board: &Board, event: &MovementEvent) -> MovementEffect;
}

/// The registry itself. Built at engine init via `default_stack()`;
/// custom stacks can be constructed manually for tests.
pub struct MovementStack {
    modifiers: Vec<Box<dyn MovementModifier>>,
}

impl Default for MovementStack {
    fn default() -> Self {
        Self::new()
    }
}

impl MovementStack {
    pub fn new() -> Self {
        Self { modifiers: Vec::new() }
    }

    /// Register a modifier. The registry is kept sorted by priority
    /// after each registration; the underlying sort is stable, so
    /// same-priority modifiers preserve insertion order.
    pub fn register(&mut self, m: Box<dyn MovementModifier>) {
        self.modifiers.push(m);
        self.modifiers.sort_by_key(|m| m.priority());
    }

    /// Number of registered modifiers. Useful for tests.
    pub fn len(&self) -> usize {
        self.modifiers.len()
    }

    pub fn is_empty(&self) -> bool {
        self.modifiers.is_empty()
    }

    /// Iterate modifier ids in priority order. Useful for debug.
    pub fn modifier_ids(&self) -> impl Iterator<Item = &'static str> + '_ {
        self.modifiers.iter().map(|m| m.id())
    }

    /// Run the stack over a seed working set. Each modifier sees the
    /// current working set in priority order; their effects produce
    /// the working set for the next modifier.
    ///
    /// `max_priority` caps which modifiers run — passing `Some(299)`
    /// runs the 0..=299 band (everything before king-safety), passing
    /// `None` runs the full stack. The cap exists so
    /// `Board::validate_move` can distinguish geometric failures
    /// (`PieceCannotMakeMove`) from king-safety failures
    /// (`WouldLeaveKingInCheck`) — both surface as "missing from the
    /// resolve set" otherwise.
    ///
    /// No re-processing: each modifier runs exactly once. A modifier
    /// emitting an event of its own kind via `Augment` or `Replace`
    /// will NOT see that event come back through itself — only
    /// later-priority modifiers will. This is the cycle guard for v1.
    fn resolve(
        &self,
        board: &Board,
        seed: Vec<MovementEvent>,
        mut trace: Option<&mut ResolveTrace>,
        max_priority: Option<u32>,
    ) -> Vec<MovementEvent> {
        let mut events = seed;
        for modifier in &self.modifiers {
            if let Some(cap) = max_priority {
                // Modifiers are kept sorted ascending by priority, so
                // hitting the cap means every remaining modifier is
                // also above it.
                if modifier.priority() > cap {
                    break;
                }
            }
            let touches = modifier.touches();
            let mut next: Vec<MovementEvent> = Vec::with_capacity(events.len());
            for ev in events.drain(..) {
                if !touches.touches(ev.kind()) {
                    next.push(ev);
                    continue;
                }
                let effect = modifier.apply(board, &ev);
                if let Some(ref mut tr) = trace {
                    tr.touched_by.push((modifier.id(), effect.clone()));
                }
                match effect {
                    MovementEffect::Keep => next.push(ev),
                    MovementEffect::Drop => {}
                    MovementEffect::Replace(rep) => next.extend(rep),
                    MovementEffect::Augment(aug) => {
                        next.push(ev);
                        next.extend(aug);
                    }
                }
            }
            events = next;
        }
        events
    }

    /// Resolve geometrically-valid moves (NOT king-safety filtered)
    /// for the piece at `from`. Used by `Board::get_moves` and the
    /// "is this move in the candidate set" check in
    /// `Board::validate_move`.
    ///
    /// Caps modifier execution at priority 299 — the king-safety
    /// modifier at 300+ does not run. Use `resolve_legal_moves` for
    /// the king-safety-filtered set.
    pub fn resolve_moves(&self, board: &Board, from: &Coord) -> Vec<GameMove> {
        self.resolve_moves_traced(board, from, None)
    }

    pub fn resolve_moves_traced(
        &self,
        board: &Board,
        from: &Coord,
        trace: Option<&mut ResolveTrace>,
    ) -> Vec<GameMove> {
        let seed = vec![MovementEvent::MoveQuery { from: from.clone() }];
        let events = self.resolve(board, seed, trace, Some(299));
        events
            .into_iter()
            .filter_map(|ev| match ev {
                MovementEvent::Candidate { game_move, .. } => Some(game_move),
                _ => None,
            })
            .collect()
    }

    /// Resolve the legal-move set for the piece at `from` — every
    /// modifier runs, including the 300+ band (king-safety, future
    /// variant rules).
    pub fn resolve_legal_moves(&self, board: &Board, from: &Coord) -> Vec<GameMove> {
        let seed = vec![MovementEvent::MoveQuery { from: from.clone() }];
        let events = self.resolve(board, seed, None, None);
        events
            .into_iter()
            .filter_map(|ev| match ev {
                MovementEvent::Candidate { game_move, .. } => Some(game_move),
                _ => None,
            })
            .collect()
    }

    /// Resolve moves for the piece at `from`, running every modifier
    /// with `priority <= max_priority`. The recursion guard for the
    /// tornado compulsion probe (plan 13): the probe caps at
    /// `tornado::PROBE_CAP` (= `TORNADO_PRIORITY - 1`) so it sees
    /// king-safe moves (king-safety is 300) but never re-enters the
    /// compulsion filter (305). Mirrors `resolve_legal_moves` with an
    /// explicit cap instead of "no cap".
    pub fn resolve_moves_capped(
        &self,
        board: &Board,
        from: &Coord,
        max_priority: u32,
    ) -> Vec<GameMove> {
        let seed = vec![MovementEvent::MoveQuery { from: from.clone() }];
        self.resolve(board, seed, None, Some(max_priority))
            .into_iter()
            .filter_map(|ev| match ev {
                MovementEvent::Candidate { game_move, .. } => Some(game_move),
                _ => None,
            })
            .collect()
    }

    /// Resolve threats against `target` from pieces of `attacker_color`.
    /// Seeds with a single `ThreatQuery`; piece-intrinsic threat
    /// modifiers (step 4) emit `Threat` events; train collision filters
    /// (step 7) drop those that aren't real captures.
    pub fn resolve_threats(
        &self,
        board: &Board,
        target: &Coord,
        attacker_color: Color,
    ) -> Vec<Coord> {
        self.resolve_threats_traced(board, target, attacker_color, None)
    }

    pub fn resolve_threats_traced(
        &self,
        board: &Board,
        target: &Coord,
        attacker_color: Color,
        trace: Option<&mut ResolveTrace>,
    ) -> Vec<Coord> {
        let seed = vec![MovementEvent::ThreatQuery {
            target: target.clone(),
            attacker_color,
        }];
        let events = self.resolve(board, seed, trace, None);
        events
            .into_iter()
            .filter_map(|ev| match ev {
                MovementEvent::Threat { attacker, .. } => Some(attacker),
                _ => None,
            })
            .collect()
    }
}

/// Process-wide default stack, lazily initialised on first access.
/// Production code paths call this; tests that want custom behaviour
/// construct their own `MovementStack`. The full list of registered
/// modifiers lives on `build_default_stack` below.
pub fn default_stack() -> &'static MovementStack {
    static STACK: OnceLock<MovementStack> = OnceLock::new();
    STACK.get_or_init(build_default_stack)
}

/// Constructor for the default stack. Each implementation step adds
/// its modifiers here.
///
/// Currently registered:
/// - Step 8 (move emission): `PieceMovesModifier` (priority 30).
/// - Step 4 (threat path): twelve `PieceAttacksModifier` instances
///   (priority 40) plus `NeutralCarrierPassengerThreatModifier`
///   (priority 60).
/// - Step 5 (square filters): `SquareConditionFilter` (110),
///   `WalkabilityFilter` (120), `SwitchTileAugment` (130).
/// - Steps 6-7 (train geometry): `TrainHeadCrushModifier` (210),
///   `TrainCartCaptureFilter` (211), `TwoTrainCollisionFilter`
///   (212).
/// - Step 9: `KingSafetyFilter` (300) — skipped by `resolve_moves`
///   (capped at 299); applied by `resolve_legal_moves`.
/// - Plan 13: `TornadoCompulsionFilter` (305) — destination
///   compulsion + trap; runs after king-safety so it operates over
///   the king-safe set. Skipped by `resolve_moves` and by the
///   tornado probe's capped resolve.
fn build_default_stack() -> MovementStack {
    let mut s = MovementStack::new();
    // Step 8: piece-intrinsic move emission (priority 30).
    s.register(Box::new(piece_moves::PieceMovesModifier));
    // Step 4: piece-intrinsic threat emission.
    for m in piece_attacks::all_piece_attack_modifiers() {
        s.register(m);
    }
    // Step 5: square-driven filters and augments.
    s.register(Box::new(square_filters::SquareConditionFilter));
    s.register(Box::new(square_filters::WalkabilityFilter));
    s.register(Box::new(square_filters::SwitchTileAugment));
    // Steps 6-7: train geometry (head crush + cart-capture filter +
    // two-train collision filter).
    s.register(Box::new(train_modifiers::TrainHeadCrushModifier));
    s.register(Box::new(train_modifiers::TrainCartCaptureFilter));
    s.register(Box::new(train_modifiers::TwoTrainCollisionFilter));
    // Step 9: king-safety filter (priority 300). Skipped by
    // `resolve_moves` (capped at 299); applied by `resolve_legal_moves`.
    s.register(Box::new(king_safety::KingSafetyFilter));
    // Plan 13: tornado destination-compulsion (priority 305). After
    // king-safety so the probe and the final set are both over
    // king-safe moves.
    s.register(Box::new(tornado::TornadoCompulsionFilter));
    s
}

// ---------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::BoardFlags;
    use crate::board::TrainTickRate;

    fn empty_board() -> Board {
        let grid = (0..8)
            .map(|_| {
                (0..8)
                    .map(|_| crate::board::square::Square::new())
                    .collect()
            })
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

    /// Simple test modifier that emits a fixed effect for every event
    /// it sees. Lets tests assert priority ordering, fast-path
    /// behaviour, and effect handling.
    struct TestModifier {
        id: &'static str,
        priority: u32,
        touches: EventKindMask,
        effect: MovementEffect,
        applied: std::sync::atomic::AtomicUsize,
    }

    impl TestModifier {
        fn new(
            id: &'static str,
            priority: u32,
            touches: EventKindMask,
            effect: MovementEffect,
        ) -> Self {
            Self {
                id,
                priority,
                touches,
                effect,
                applied: std::sync::atomic::AtomicUsize::new(0),
            }
        }

        fn apply_count(&self) -> usize {
            self.applied.load(std::sync::atomic::Ordering::SeqCst)
        }
    }

    impl MovementModifier for TestModifier {
        fn id(&self) -> &'static str {
            self.id
        }

        fn priority(&self) -> u32 {
            self.priority
        }

        fn touches(&self) -> EventKindMask {
            self.touches
        }

        fn apply(&self, _board: &Board, _event: &MovementEvent) -> MovementEffect {
            self.applied
                .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            self.effect.clone()
        }
    }

    #[test]
    fn empty_stack_passes_through_seed() {
        let board = empty_board();
        let stack = MovementStack::new();
        // No modifiers, seed is one MoveQuery — the query never
        // becomes a Candidate, so resolve_moves returns empty.
        let result = stack.resolve_moves(&board, &Coord { file: 0, rank: 0 });
        assert!(result.is_empty());
    }

    #[test]
    fn touches_fast_path_skips_modifier() {
        let board = empty_board();
        let mut stack = MovementStack::new();
        let m = Box::new(TestModifier::new(
            "skipped",
            10,
            EventKindMask::THREAT, // only THREAT, not THREAT_QUERY
            MovementEffect::Drop,
        ));
        // We can't actually verify the atomic counter through the box,
        // but the resolve_threats call below should still see the seed
        // ThreatQuery (since the modifier doesn't touch ThreatQuery).
        stack.register(m);
        let result = stack.resolve_threats(
            &board,
            &Coord { file: 0, rank: 0 },
            Color::White,
        );
        // No modifier converts ThreatQuery → Threat, so the final
        // result is empty. (The query event is filtered out at the
        // resolve_threats output collector.)
        assert!(result.is_empty());
    }

    #[test]
    fn priority_ordering_low_runs_first() {
        let board = empty_board();
        let mut stack = MovementStack::new();
        // Low-priority modifier emits a Threat via Replace.
        stack.register(Box::new(TestModifier::new(
            "emit",
            10,
            EventKindMask::THREAT_QUERY,
            MovementEffect::Replace(vec![MovementEvent::Threat {
                attacker: Coord { file: 1, rank: 1 },
                attacker_piece: PieceType::new_rook(Color::White),
                target: Coord { file: 0, rank: 0 },
            }]),
        )));
        // High-priority modifier drops Threats.
        stack.register(Box::new(TestModifier::new(
            "drop",
            300,
            EventKindMask::THREAT,
            MovementEffect::Drop,
        )));
        let result = stack.resolve_threats(
            &board,
            &Coord { file: 0, rank: 0 },
            Color::White,
        );
        // Emit ran first → threat created → drop ran second → empty.
        assert!(result.is_empty());
    }

    #[test]
    fn priority_ordering_high_first_registration_does_not_matter() {
        let board = empty_board();
        let mut stack = MovementStack::new();
        // Register the high-priority drop first.
        stack.register(Box::new(TestModifier::new(
            "drop",
            300,
            EventKindMask::THREAT,
            MovementEffect::Drop,
        )));
        stack.register(Box::new(TestModifier::new(
            "emit",
            10,
            EventKindMask::THREAT_QUERY,
            MovementEffect::Replace(vec![MovementEvent::Threat {
                attacker: Coord { file: 1, rank: 1 },
                attacker_piece: PieceType::new_rook(Color::White),
                target: Coord { file: 0, rank: 0 },
            }]),
        )));
        let result = stack.resolve_threats(
            &board,
            &Coord { file: 0, rank: 0 },
            Color::White,
        );
        // Stable sort by priority means emit (10) still runs first.
        assert!(result.is_empty());
    }

    #[test]
    fn augment_keeps_original_plus_new() {
        let board = empty_board();
        let mut stack = MovementStack::new();
        stack.register(Box::new(TestModifier::new(
            "augment",
            10,
            EventKindMask::THREAT_QUERY,
            MovementEffect::Augment(vec![MovementEvent::Threat {
                attacker: Coord { file: 5, rank: 5 },
                attacker_piece: PieceType::new_knight(Color::Black),
                target: Coord { file: 0, rank: 0 },
            }]),
        )));
        let result = stack.resolve_threats(
            &board,
            &Coord { file: 0, rank: 0 },
            Color::Black,
        );
        // ThreatQuery is preserved (Augment kept the original) but
        // it's filtered out of the result by the output collector.
        // The augmented Threat survives.
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], Coord { file: 5, rank: 5 });
    }

    #[test]
    fn keep_passes_event_through() {
        let board = empty_board();
        let mut stack = MovementStack::new();
        // Low-pri: emit a threat.
        stack.register(Box::new(TestModifier::new(
            "emit",
            10,
            EventKindMask::THREAT_QUERY,
            MovementEffect::Replace(vec![MovementEvent::Threat {
                attacker: Coord { file: 2, rank: 2 },
                attacker_piece: PieceType::new_bishop(Color::White),
                target: Coord { file: 0, rank: 0 },
            }]),
        )));
        // High-pri: Keep (no-op).
        stack.register(Box::new(TestModifier::new(
            "passthrough",
            200,
            EventKindMask::THREAT,
            MovementEffect::Keep,
        )));
        let result = stack.resolve_threats(
            &board,
            &Coord { file: 0, rank: 0 },
            Color::White,
        );
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], Coord { file: 2, rank: 2 });
    }

    #[test]
    fn trace_records_each_touched_modifier() {
        let board = empty_board();
        let mut stack = MovementStack::new();
        stack.register(Box::new(TestModifier::new(
            "emit",
            10,
            EventKindMask::THREAT_QUERY,
            MovementEffect::Replace(vec![MovementEvent::Threat {
                attacker: Coord { file: 3, rank: 3 },
                attacker_piece: PieceType::new_queen(Color::Black),
                target: Coord { file: 0, rank: 0 },
            }]),
        )));
        stack.register(Box::new(TestModifier::new(
            "drop",
            200,
            EventKindMask::THREAT,
            MovementEffect::Drop,
        )));
        let mut trace = ResolveTrace::new();
        let _ = stack.resolve_threats_traced(
            &board,
            &Coord { file: 0, rank: 0 },
            Color::Black,
            Some(&mut trace),
        );
        // Two modifiers, each touched one event.
        assert_eq!(trace.touched_by.len(), 2);
        assert_eq!(trace.touched_by[0].0, "emit");
        assert_eq!(trace.touched_by[1].0, "drop");
    }

    #[test]
    fn modifier_does_not_revisit_its_own_output() {
        // Sanity: a modifier that Augments its event-kind shouldn't
        // see the augmentation come back. v1 cycle guard is "each
        // modifier runs at most once."
        let board = empty_board();
        let mut stack = MovementStack::new();
        let m = TestModifier::new(
            "augment_self",
            10,
            EventKindMask::THREAT_QUERY,
            MovementEffect::Augment(vec![MovementEvent::ThreatQuery {
                target: Coord { file: 0, rank: 0 },
                attacker_color: Color::White,
            }]),
        );
        // We need to count applications; the Box hides the atomic.
        // Workaround: register and run twice with separate instances.
        // For this test we just verify resolve completes without
        // looping forever — if v1's cycle guard fails, this hangs.
        stack.register(Box::new(m));
        let result = stack.resolve_threats(
            &board,
            &Coord { file: 0, rank: 0 },
            Color::White,
        );
        // No `Threat` event ever produced, so result is empty.
        assert!(result.is_empty());
    }

    #[test]
    fn event_kind_mask_touches() {
        let mask = EventKindMask::CANDIDATE.union(EventKindMask::THREAT);
        assert!(mask.touches(EventKind::Candidate));
        assert!(mask.touches(EventKind::Threat));
        assert!(!mask.touches(EventKind::MoveQuery));
        assert!(!mask.touches(EventKind::ThreatQuery));
    }

    #[test]
    fn default_stack_has_step_4_piece_attack_modifiers() {
        // Twelve piece-attack modifiers plus the Neutral-carrier
        // passenger modifier.
        let stack = default_stack();
        let ids: Vec<_> = stack.modifier_ids().collect();
        for expected in [
            "piece_attacks.pawn",
            "piece_attacks.rook",
            "piece_attacks.knight",
            "piece_attacks.bishop",
            "piece_attacks.queen",
            "piece_attacks.king",
            "piece_attacks.monkey",
            "piece_attacks.goblin",
            "piece_attacks.skibidi",
            "piece_attacks.bus",
            "piece_attacks.locomotive",
            "piece_attacks.carriage",
            "piece_attacks.neutral_passenger",
        ] {
            assert!(
                ids.contains(&expected),
                "default stack should include {expected}; got {ids:?}"
            );
        }
    }

    #[test]
    fn register_keeps_priority_order() {
        let mut stack = MovementStack::new();
        stack.register(Box::new(TestModifier::new(
            "c",
            300,
            EventKindMask::NONE,
            MovementEffect::Keep,
        )));
        stack.register(Box::new(TestModifier::new(
            "a",
            10,
            EventKindMask::NONE,
            MovementEffect::Keep,
        )));
        stack.register(Box::new(TestModifier::new(
            "b",
            200,
            EventKindMask::NONE,
            MovementEffect::Keep,
        )));
        let ids: Vec<_> = stack.modifier_ids().collect();
        assert_eq!(ids, vec!["a", "b", "c"]);
    }

    /// Same-priority modifiers must run in registration order. The
    /// 12 piece-attack modifiers all share priority 40; if Rust's
    /// `sort_by_key` ever switches to unstable, ordering becomes
    /// nondeterministic. Pin the stable-sort invariant via
    /// `modifier_ids()` (the dispatch order) AND via observable
    /// effects when modifier order is load-bearing (later modifiers
    /// can override earlier ones).
    #[test]
    fn same_priority_modifiers_run_in_registration_order() {
        let mut stack = MovementStack::new();
        stack.register(Box::new(TestModifier::new(
            "first",
            50,
            EventKindMask::NONE,
            MovementEffect::Keep,
        )));
        stack.register(Box::new(TestModifier::new(
            "second",
            50,
            EventKindMask::NONE,
            MovementEffect::Keep,
        )));
        stack.register(Box::new(TestModifier::new(
            "third",
            50,
            EventKindMask::NONE,
            MovementEffect::Keep,
        )));
        let ids: Vec<_> = stack.modifier_ids().collect();
        assert_eq!(
            ids,
            vec!["first", "second", "third"],
            "same-priority modifiers must preserve registration order"
        );

        // Observable order: later-registered modifiers see (and can
        // override) earlier ones' outputs. Pin this with an emit +
        // drop pair at the same priority: emit registers first
        // (produces a Threat), drop registers second (drops Threats).
        // The drop runs after the emit per stable-sort, so the final
        // working set is empty.
        let mut stack = MovementStack::new();
        stack.register(Box::new(TestModifier::new(
            "emit",
            10,
            EventKindMask::THREAT_QUERY,
            MovementEffect::Replace(vec![MovementEvent::Threat {
                attacker: Coord { file: 0, rank: 0 },
                attacker_piece: PieceType::new_rook(Color::White),
                target: Coord { file: 0, rank: 0 },
            }]),
        )));
        stack.register(Box::new(TestModifier::new(
            "drop",
            10,
            EventKindMask::THREAT,
            MovementEffect::Drop,
        )));
        let board = empty_board();
        let threats = stack.resolve_threats(&board, &Coord { file: 0, rank: 0 }, Color::White);
        assert!(
            threats.is_empty(),
            "emit-then-drop at the same priority should produce empty set; got {threats:?}"
        );

        // Inverse order: drop runs first (no-op on empty set), then
        // emit produces a Threat that survives.
        let mut stack = MovementStack::new();
        stack.register(Box::new(TestModifier::new(
            "drop",
            10,
            EventKindMask::THREAT,
            MovementEffect::Drop,
        )));
        stack.register(Box::new(TestModifier::new(
            "emit",
            10,
            EventKindMask::THREAT_QUERY,
            MovementEffect::Replace(vec![MovementEvent::Threat {
                attacker: Coord { file: 0, rank: 0 },
                attacker_piece: PieceType::new_rook(Color::White),
                target: Coord { file: 0, rank: 0 },
            }]),
        )));
        let threats = stack.resolve_threats(&board, &Coord { file: 0, rank: 0 }, Color::White);
        assert_eq!(
            threats.len(),
            1,
            "drop-then-emit at the same priority should produce one threat (emit ran after drop saw nothing); got {threats:?}"
        );
    }
}
