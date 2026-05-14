use crate::{
    board::{Board, Coord, GameMove, MoveType},
    pieces::{Color, Piece},
};

#[derive(Clone, PartialEq, Debug)]
pub struct Knight {
    pub color: Color,
}
impl Piece for Knight {
    fn name(&self) -> &str {
        "Knight"
    }
    fn color(&self) -> Color {
        self.color
    }
    fn set_color(&mut self, color: Color) {
        self.color = color;
    }
    fn initial_moves(&self, board: &Board, from: &Coord) -> Vec<GameMove> {
        // Plan 09: only train carts are Neutral; a Neutral non-train
        // piece would be flagged as a threat to *both* sides by
        // `is_attacked_by`'s "include Neutral" rule, which is wrong
        // for non-train pieces. Yield no moves so a stray Neutral
        // knight (e.g. from a hand-rolled FEN) can't act.
        if self.color == Color::Neutral {
            return Vec::new();
        }
        let knight_moves: [(isize, isize); 8] = [
            (2, 1),
            (1, 2),
            (-1, 2),
            (-2, 1),
            (-2, -1),
            (-1, -2),
            (1, -2),
            (2, -1),
        ];

        let mut moves = Vec::new();
        for (df, dr) in &knight_moves {
            let new_file = from.file as isize + df;
            let new_rank = from.rank as isize + dr;
            if !board.in_bounds(new_file, new_rank) {
                continue;
            }
            let coord = Coord {
                file: new_file as u8,
                rank: new_rank as u8,
            };
            // Plan 08: skip non-walkable destinations (closed Gate / Turret / Vent).
            // The general filter handles same-color targets later.
            if !board
                .get_square_at(&coord)
                .map(|s| s.square_type.is_walkable())
                .unwrap_or(false)
            {
                continue;
            }
            moves.push(GameMove {
                from: from.clone(),
                move_type: MoveType::MoveTo(coord),
            });
        }
        moves
    }
    fn symbol(&self) -> String {
        match self.color {
            Color::White => 'N'.to_string(),
            Color::Black => 'n'.to_string(),
            Color::Neutral => 'N'.to_string(),
        }
    }

    fn clone_box(&self) -> Box<dyn Piece> {
        Box::new(self.clone())
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
