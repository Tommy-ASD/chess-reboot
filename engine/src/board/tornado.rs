use crate::board::Board;
use crate::board::square::SquareCondition;

impl Board {
    /// Tick every `SquareCondition::Tornado` on the board down by one,
    /// removing the condition from any square whose counter reaches 0.
    ///
    /// **Frozen suspends the countdown.** A square that is also
    /// `Frozen` does not tick — consistent with Frozen halting
    /// condition/telegraph progression elsewhere (the Clock
    /// precedent). A frozen tornado is suspended, not extended past
    /// its number; it resumes counting when the freeze lifts.
    ///
    /// Each `Tornado` on a square decrements independently (two
    /// `Tornado`s on one square — FEN-constructible — both tick; ones
    /// reaching 0 are dropped, survivors stay).
    ///
    /// Called from `apply_environment_reactions` via
    /// `TornadoTickHandler` at `EnvPhase::PostTick`, once per applied
    /// move (per ply), not per `TrainTickRate`.
    pub fn tick_tornadoes(&mut self) {
        // Audit R1/B3: read-only fast-path so a tornado-free board (the
        // overwhelmingly common case) pays one short-circuiting scan
        // instead of a full `iter_mut` grid walk every ply. Mirrors the
        // sibling handlers' perf gating (`TornadoCompulsionFilter`'s
        // `any_tornado`, `BrainrotRecalcHandler`'s `train_ticked`) —
        // shares the one `any_tornado` definition to avoid drift.
        if !crate::movement::stack::tornado::any_tornado(self) {
            return;
        }
        for row in self.grid.iter_mut() {
            for sq in row.iter_mut() {
                // Frozen suspends the countdown for this square.
                if sq.conditions.contains(&SquareCondition::Frozen) {
                    continue;
                }
                let mut ticked = false;
                for c in sq.conditions.iter_mut() {
                    if let SquareCondition::Tornado { remaining } = c {
                        *remaining = remaining.saturating_sub(1);
                        ticked = true;
                    }
                }
                if ticked {
                    sq.conditions.retain(
                        |c| !matches!(c, SquareCondition::Tornado { remaining: 0 }),
                    );
                }
            }
        }
    }
}
