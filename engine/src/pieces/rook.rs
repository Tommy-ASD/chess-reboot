use crate::{
    board::Board,
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
    fn legal_moves(&self, board: &Board, from: (usize, usize)) -> Vec<(usize, usize)> {
        todo!()
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
