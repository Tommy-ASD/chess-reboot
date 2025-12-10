use crate::{
    board::{Board, Coord, GameMove},
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
            if new_file >= 0 && new_file < 8 && new_rank >= 0 && new_rank < 8 {
                let coord = Coord {
                    file: new_file as u8,
                    rank: new_rank as u8,
                };
                let game_move = GameMove {
                    from: from.clone(),
                    to: coord.clone(),
                };
                moves.push(game_move);
            }
        }
        moves
    }
    fn symbol(&self) -> String {
        match self.color {
            Color::White => 'N'.to_string(),
            Color::Black => 'n'.to_string(),
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
