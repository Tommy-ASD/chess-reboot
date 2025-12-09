use crate::{
    board::{Board, Coord},
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
    fn initial_moves(&self, board: &Board, from: &Coord) -> Vec<Coord> {
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
                moves.push(Coord {
                    file: new_file as u8,
                    rank: new_rank as u8,
                });
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
}
