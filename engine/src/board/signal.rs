use tracing::{debug, trace, warn};

use crate::board::{
    Board, Coord, SignalId,
    square::{PressureTrigger, SquareType},
};

impl Board {
    /// Fire a signal pulse at the given target IDs. Each matching receiver
    /// is activated once. **Bounded propagation:** activations only mutate
    /// the receiver's own state — they never themselves trigger further
    /// emitters during this call. This forbids cascades (and the cycles
    /// they would enable) in v1; an explicit "delayed-fire" emitter type
    /// can be added later if chain triggers become desirable.
    ///
    /// Dangling targets (an ID with no matching receiver on the board)
    /// are silently no-op — by design. The editor surfaces dangling
    /// references at design time; runtime treats them as inert.
    pub fn fire_signal(&mut self, targets: &[SignalId]) {
        debug!(?targets, "firing signal");
        for target_id in targets {
            self.activate_receiver(*target_id);
        }
    }

    /// Plan 08 step 4: if `coord` is a `PressurePlate` whose `fires_for`
    /// matches the piece currently standing on it, fire its targets.
    /// Called from `apply_piece_post_effects` once per landing square of
    /// the moving piece(s) — see `make_move.rs::collect_landings`.
    ///
    /// Empty plates and color-mismatched plates are silent no-ops.
    /// Bounded propagation applies as it does for `fire_signal`:
    /// downstream receivers cannot themselves emit during this call.
    pub fn maybe_fire_pressure_plate(&mut self, coord: &Coord) {
        // Snapshot the plate's targets + trigger out from under the
        // grid borrow so `fire_signal` can take `&mut self` cleanly.
        let plate_info = self.get_square_at(coord).and_then(|sq| {
            if let SquareType::PressurePlate { targets, fires_for } = &sq.square_type {
                Some((targets.clone(), fires_for.clone()))
            } else {
                None
            }
        });
        let Some((targets, fires_for)) = plate_info else {
            return;
        };
        if !self.piece_matches_trigger(coord, &fires_for) {
            trace!(?coord, ?fires_for, "plate trigger not satisfied; no fire");
            return;
        }
        debug!(?coord, ?targets, "pressure plate fires");
        self.fire_signal(&targets);
    }

    /// Does the piece (if any) at `coord` satisfy this plate's trigger?
    /// `AnyPiece` matches anything; `OnlyColor(c)` matches only pieces of
    /// that color. An empty square never satisfies a trigger.
    fn piece_matches_trigger(&self, coord: &Coord, trigger: &PressureTrigger) -> bool {
        let Some(piece) = self.get_square_at(coord).and_then(|s| s.piece.as_ref()) else {
            return false;
        };
        match trigger {
            PressureTrigger::AnyPiece => true,
            PressureTrigger::OnlyColor(c) => piece.get_color() == *c,
        }
    }

    /// Linear scan over the grid, activating every receiver whose ID
    /// matches. At 8×8 this is fine; if board sizes grow significantly,
    /// cache a `HashMap<SignalId, Vec<Coord>>` on `Board` rebuilt by
    /// `recalc_signal_index()` after FEN load + after editor mutations.
    ///
    /// Plan 08 explicitly allows the same numeric ID across different
    /// receiver kinds (Junction id=3 and Gate id=3 coexist) — both fire.
    ///
    /// **Brainrot does NOT silence receivers.** Brainrot (plan 04) scopes
    /// to *piece movement* — a piece on a brainrotted square cannot move
    /// or throw a switch. Signals are infrastructure-level pulses with no
    /// piece-agency, so the receiver activates regardless of its square's
    /// conditions. This keeps the model consistent: a PressurePlate fires
    /// (step 4) regardless of the standing piece's brainrot state, and
    /// the receivers downstream of that pulse should too.
    fn activate_receiver(&mut self, id: SignalId) {
        for row in &mut self.grid {
            for sq in row {
                match &mut sq.square_type {
                    SquareType::Junction {
                        id: jid,
                        state,
                        branches,
                    } if *jid == id => {
                        // Defensive: a Junction with no branches would
                        // panic on modulo-by-zero. The editor should
                        // reject such squares, but a hand-edited FEN can
                        // sneak one in — log and skip.
                        if branches.is_empty() {
                            warn!(id, "junction has no branches; signal ignored");
                        } else {
                            // Compute in `usize` and cast back; using
                            // `branches.len() as u8` for the modulus
                            // would wrap to `len % 256` for >255-branch
                            // junctions and could even produce a zero
                            // modulus (panic in debug). The cast at the
                            // end is bounded by `branches.len()` which
                            // we keep ≤ 255 (see fen.rs parse).
                            let next = ((*state as usize).wrapping_add(1)) % branches.len();
                            let new_state = next as u8;
                            trace!(
                                id,
                                old = *state,
                                new = new_state,
                                "junction advanced"
                            );
                            *state = new_state;
                        }
                    }
                    SquareType::Gate { id: gid, open } if *gid == id => {
                        trace!(id, was_open = *open, "gate toggled");
                        *open = !*open;
                    }
                    _ => {}
                }
            }
        }
    }
}
