use crate::{Board, pieces::{Color, Piece}};

#[derive(Clone, PartialEq, Debug)]
pub struct Pawn { pub color: Color }
impl Piece for Pawn {
    fn name(&self) -> &str { "Pawn" }
    fn color(&self) -> Color { self.color }
    fn legal_moves(&self, board: &Board, from: (usize, usize)) -> Vec<(usize, usize)> {
        todo!()
    }
    fn symbol(&self) -> String { match self.color { Color::White => 'P'.to_string(),
Color::Black => 'p'.to_string()} }

    fn clone_box(&self) -> Box<dyn Piece> {
        Box::new(self.clone())
    }
}