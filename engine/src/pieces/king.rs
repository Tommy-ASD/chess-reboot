use crate::{
    board::{Board, Coord},
    movement::glider::{OMNI_DIRS, generate_glider_moves},
    pieces::{Color, Piece},
};

#[derive(Clone, PartialEq, Debug)]
pub struct King {
    pub color: Color,
}
impl Piece for King {
    fn name(&self) -> &str {
        "King"
    }
    fn color(&self) -> Color {
        self.color
    }
    fn legal_moves(&self, board: &Board, from: &Coord) -> Vec<Coord> {
        generate_glider_moves(board, from, &OMNI_DIRS, 1)
    }
    fn symbol(&self) -> String {
        match self.color {
            Color::White => 'K'.to_string(),
            Color::Black => 'k'.to_string(),
        }
    }

    fn clone_box(&self) -> Box<dyn Piece> {
        Box::new(self.clone())
    }
}
