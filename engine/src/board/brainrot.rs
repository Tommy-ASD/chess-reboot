use crate::{
    board::{Board, Coord, square::SquareCondition},
    pieces::piecetype::PieceType,
};

impl Board {
    pub fn recalc_brainrot(&mut self) {
        // Step 1: Remove all existing brainrot
        for row in &mut self.grid {
            for sq in row {
                sq.conditions
                    .retain(|c| !matches!(c, SquareCondition::Brainrot));
            }
        }

        // Step 2: Reapply brainrot from ALL Skibidis on board
        for (coord, piece) in self.all_pieces() {
            if let PieceType::Skibidi(sk) = piece {
                self.apply_skibidi_brainrot(&coord, sk.phase);
            }
        }
    }

    fn apply_skibidi_brainrot(&mut self, center: &Coord, phase: u8) {
        let radius = match phase {
            1 => 0, // no effect
            2 => 1, // orthogonals only
            3 => 2,
            4 => 3,
            _ => return,
        };

        for dx in -(radius as isize)..=(radius as isize) {
            for dy in -(radius as isize)..=(radius as isize) {
                // don't brainrot the skibidi
                if dx == 0 && dy == 0 {
                    continue;
                };
                let f = center.file as isize + dx;
                let r = center.rank as isize + dy;
                if self.in_bounds(f, r) {
                    let sq = self
                        .get_square_mut(
                            &(Coord {
                                file: f as u8,
                                rank: r as u8,
                            }),
                        )
                        .unwrap();
                    sq.conditions.push(SquareCondition::Brainrot);
                    println!("{f}-{r} brainrotted")
                }
            }
        }
    }
}
