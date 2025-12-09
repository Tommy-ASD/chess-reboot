use crate::{
    board::{Board, Coord, GameMove},
    movement::glider::{OMNI_DIRS, generate_glider_moves},
    pieces::{Color, Piece},
};

#[derive(Clone, PartialEq, Debug)]
pub struct Queen {
    pub color: Color,
}
impl Piece for Queen {
    fn name(&self) -> &str {
        "Queen"
    }
    fn color(&self) -> Color {
        self.color
    }
    fn initial_moves(&self, board: &Board, from: &Coord) -> Vec<GameMove> {
        generate_glider_moves(board, from, &OMNI_DIRS, usize::MAX)
    }
    fn symbol(&self) -> String {
        match self.color {
            Color::White => 'Q'.to_string(),
            Color::Black => 'q'.to_string(),
        }
    }

    fn clone_box(&self) -> Box<dyn Piece> {
        Box::new(self.clone())
    }
}
