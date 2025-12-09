use crate::{
    board::{Board, Coord},
    movement::glider::{STRAIGHT_DIRS, generate_glider_moves},
    pieces::{Color, Piece},
};

#[derive(Clone, PartialEq, Debug)]
pub struct Rook {
    pub color: Color,
}
impl Piece for Rook {
    fn name(&self) -> &str {
        "Rook"
    }
    fn color(&self) -> Color {
        self.color
    }
    fn initial_moves(&self, board: &Board, from: &Coord) -> Vec<Coord> {
        generate_glider_moves(board, from, &STRAIGHT_DIRS, usize::MAX)
    }
    fn symbol(&self) -> String {
        match self.color {
            Color::White => 'R'.to_string(),
            Color::Black => 'r'.to_string(),
        }
    }

    fn clone_box(&self) -> Box<dyn Piece> {
        Box::new(self.clone())
    }
}
