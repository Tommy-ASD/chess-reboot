use crate::{
    board::Board,
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
    fn legal_moves(&self, board: &Board, from: (usize, usize)) -> Vec<(usize, usize)> {
        todo!()
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
