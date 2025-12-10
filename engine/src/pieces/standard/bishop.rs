use crate::{
    board::{Board, Coord, GameMove},
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
    fn initial_moves(&self, board: &Board, from: &Coord) -> Vec<GameMove> {
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

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
