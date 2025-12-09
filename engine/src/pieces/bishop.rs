use crate::{
    board::{Board, Coord},
    movement::glider::{DIAGONAL_DIRS, generate_glider_moves},
    pieces::{Color, Piece},
};

#[derive(Clone, PartialEq, Debug)]
pub struct Bishop {
    pub color: Color,
}
impl Piece for Bishop {
    fn name(&self) -> &str {
        "Bishop"
    }
    fn color(&self) -> Color {
        self.color
    }
    fn initial_moves(&self, board: &Board, from: &Coord) -> Vec<Coord> {
        generate_glider_moves(board, from, &DIAGONAL_DIRS, usize::MAX)
    }
    fn symbol(&self) -> String {
        match self.color {
            Color::White => 'B'.to_string(),
            Color::Black => 'b'.to_string(),
        }
    }

    fn clone_box(&self) -> Box<dyn Piece> {
        Box::new(self.clone())
    }
}
