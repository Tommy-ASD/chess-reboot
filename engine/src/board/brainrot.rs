use tracing::trace;

use crate::{
    board::{Board, Coord, square::SquareCondition},
    pieces::{Color, piecetype::PieceType},
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

    /// Plan 04: distinguish a "win by Brainrot" from an ordinary
    /// stalemate. Called only from `Board::status()`'s no-legal-moves /
    /// not-in-check branch. Returns true when `to_move` is paralysed
    /// *because of* an opponent's Brainrot:
    ///
    /// 1. `to_move` has at least one (top-level) piece on the board, and
    /// 2. every one of those pieces sits on a `Brainrot` square — so the
    ///    side is genuinely frozen, not merely pinned or blocked, and
    /// 3. an *opposing* Skibidi is actively radiating (`phase > 1`) to
    ///    take credit for the win.
    ///
    /// Deliberately approximate (plan 04 documents this): a side with
    /// one brainrot-frozen piece and one independently-pinned piece
    /// reads as `Stalemate`, not a brainrot win. Self-inflicted brainrot
    /// (only `to_move`'s own Skibidi radiates) also falls through to
    /// `Stalemate` — condition 3 requires an opposing Skibidi. A king
    /// riding a Neutral cart is not counted as a top-level piece of
    /// `to_move`, so that nested edge case also resolves to `Stalemate`
    /// rather than risk a false win.
    pub(crate) fn is_brainrot_win(&self, to_move: Color) -> bool {
        let mut saw_own_piece = false;
        for (coord, piece) in self.iter_pieces() {
            if piece.get_color() != to_move {
                continue;
            }
            saw_own_piece = true;
            let on_brainrot = self
                .get_square_at(&coord)
                .is_some_and(|sq| sq.conditions.contains(&SquareCondition::Brainrot));
            if !on_brainrot {
                return false;
            }
        }
        if !saw_own_piece {
            return false;
        }
        // An opposing Skibidi must be radiating to claim the win. Use the
        // opponent colour explicitly (not `!= to_move`): with three Color
        // variants, `!=` would also match a *Neutral* Skibidi, crediting
        // the opponent a win a neutral/unaligned aura caused. Mirrors
        // `is_brainrot_lockout`'s opponent-only discrimination; a Neutral
        // aura falls through to `Stalemate`.
        let opponent = to_move.opposite();
        self.iter_pieces().any(|(_, p)| {
            matches!(p, PieceType::Skibidi(sk) if sk.color == opponent && sk.phase > 1)
        })
    }

    /// Plan 04 (Skibidi spec, line 15): "If your Skibidi is captured
    /// while your opponent's Skibidi is in phase 4, there is nothing you
    /// can do." Detected positionally: `loser` has no Skibidi anywhere on
    /// the board while the opponent holds at least one *radiating*
    /// (top-level) phase-4 Skibidi. Reaching phase 4 requires some
    /// other-colour Skibidi to be present (`make_move` caps PhaseShift at
    /// 3 otherwise — and that cap treats a Neutral Skibidi as "other-
    /// colour" too), so this positionally *approximates* the spec's "your
    /// Skibidi was captured": it fires whenever `loser` has no Skibidi
    /// while a top-level *enemy* Skibidi sits at phase 4 — including the
    /// corner case where `loser` never owned a Skibidi at all.
    ///
    /// A Skibidi riding inside a carrier still belongs to `loser` (it
    /// wasn't captured), so the descent below counts it and suppresses
    /// the lockout. The opponent's phase-4 check stays top-level only: a
    /// carried Skibidi does not radiate (`recalc_brainrot` scans
    /// top-level pieces), so it can't be the unstoppable aura.
    ///
    /// Deliberately a positional heuristic with no lookahead: it declares
    /// the loss even if `loser` could in principle capture the phase-4
    /// Skibidi or deliver mate first. This matches the spec's literal
    /// "there is nothing you can do" and the project's keep-it-
    /// approximate-and-document convention — `status()` reports the
    /// current position, it does not search. Restricted to the actual
    /// opponent colour, so a Neutral phase-4 Skibidi never triggers it.
    pub(crate) fn is_brainrot_lockout(&self, loser: Color) -> bool {
        let opponent = loser.opposite();
        let mut loser_has_skibidi = false;
        let mut opponent_phase4 = false;
        for (_, p) in self.iter_pieces() {
            if let PieceType::Skibidi(sk) = p {
                if sk.color == loser {
                    loser_has_skibidi = true;
                } else if sk.color == opponent && sk.phase == 4 {
                    opponent_phase4 = true;
                }
            }
            // A carried Skibidi of the loser's colour isn't captured.
            if p.passengers().is_some_and(|passengers| {
                passengers
                    .iter()
                    .any(|q| matches!(q, PieceType::Skibidi(s) if s.color == loser))
            }) {
                loser_has_skibidi = true;
            }
        }
        !loser_has_skibidi && opponent_phase4
    }
}
