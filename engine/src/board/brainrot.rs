use tracing::trace;

use crate::{
    board::{Board, Coord, square::SquareCondition},
    pieces::piecetype::PieceType,
};

/// Manhattan-disk radius for each Skibidi phase per the doc on `Skibidi`.
fn phase_to_radius(phase: u8) -> isize {
    match phase {
        2 => 1,
        3 => 2,
        4 => 3,
        _ => 0, // phase 1 (and anything out of range) has no aura
    }
}

impl Board {
    /// Recompute the board's Brainrot square-conditions from current
    /// Skibidi positions, and apply the spec's neutralization rule.
    ///
    /// **Permanently mutates Skibidi phases.** If Skibidi A's aura
    /// covers Skibidi B's tile, A is reset to phase 1 *on the board*
    /// (not just for this call). The reset persists across future
    /// recalcs — it is not recovered by A's aura disappearing or by
    /// B being captured. The only way to raise A back above phase 1
    /// is for A's owner to spend a move on a `PhaseShift`.
    ///
    /// Called from `apply_piece_post_effects` after every player move,
    /// and from `apply_environment_reactions` after a train tick that
    /// actually fired (since a tick could have captured a Skibidi).
    /// The function is idempotent under repeated calls when no
    /// neutralization is pending.
    pub fn recalc_brainrot(&mut self) {
        // Step 1: clear all existing Brainrot conditions.
        for row in &mut self.grid {
            for sq in row {
                sq.conditions
                    .retain(|c| !matches!(c, SquareCondition::Brainrot));
            }
        }

        // Step 2: collect every Skibidi's coord + current phase.
        let mut skibidis: Vec<(Coord, u8)> = self
            .iter_pieces()
            .filter_map(|(coord, piece)| match piece {
                PieceType::Skibidi(sk) => Some((coord, sk.phase)),
                _ => None,
            })
            .collect();

        // Step 3: neutralization — any Skibidi sitting inside another
        // Skibidi's aura resets that radiating Skibidi back to phase 1.
        // (Same rule applies regardless of colour, per spec.)
        let mut neutralized = vec![false; skibidis.len()];
        for i in 0..skibidis.len() {
            let radius = phase_to_radius(skibidis[i].1);
            if radius == 0 {
                continue;
            }
            for j in 0..skibidis.len() {
                if i == j {
                    continue;
                }
                let dx = (skibidis[j].0.file as isize - skibidis[i].0.file as isize).abs();
                let dy = (skibidis[j].0.rank as isize - skibidis[i].0.rank as isize).abs();
                if dx + dy <= radius {
                    neutralized[i] = true;
                    break;
                }
            }
        }

        // Step 4: write the neutralized phase back to the board (and the
        // local copy, so step 5 paints the correct auras).
        for i in 0..skibidis.len() {
            if neutralized[i] {
                let coord = skibidis[i].0.clone();
                if let Some(sq) = self.get_square_mut(&coord) {
                    if let Some(PieceType::Skibidi(sk)) = &mut sq.piece {
                        sk.phase = 1;
                    }
                }
                skibidis[i].1 = 1;
            }
        }

        // Step 5: re-apply brainrot from each Skibidi's (possibly updated) phase.
        for (coord, phase) in skibidis.clone() {
            self.apply_skibidi_brainrot(&coord, phase);
        }
    }

    fn apply_skibidi_brainrot(&mut self, center: &Coord, phase: u8) {
        let radius = phase_to_radius(phase);
        if radius == 0 {
            return;
        }

        for dx in -radius..=radius {
            for dy in -radius..=radius {
                if dx == 0 && dy == 0 {
                    continue;
                }
                // Manhattan disk: |dx| + |dy| <= radius
                if dx.abs() + dy.abs() > radius {
                    continue;
                }
                let f = center.file as isize + dx;
                let r = center.rank as isize + dy;
                if !self.in_bounds(f, r) {
                    continue;
                }
                let coord = Coord {
                    file: f as u8,
                    rank: r as u8,
                };
                if let Some(sq) = self.get_square_mut(&coord) {
                    if !sq.conditions.contains(&SquareCondition::Brainrot) {
                        sq.conditions.push(SquareCondition::Brainrot);
                        trace!(f, r, "brainrot applied");
                    }
                }
            }
        }
    }
}
