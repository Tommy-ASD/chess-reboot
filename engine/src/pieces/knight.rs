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
    fn legal_moves(&self, board: &Board, from: &Coord) -> Vec<Coord> {
        todo!()
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
