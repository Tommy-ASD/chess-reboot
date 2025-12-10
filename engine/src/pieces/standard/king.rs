use crate::{
    board::{Board, Coord, GameMove},
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
    fn initial_moves(&self, board: &Board, from: &Coord) -> Vec<GameMove> {
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

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
