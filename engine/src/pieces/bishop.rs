use crate::{Board, pieces::{Color, Piece}};


#[derive(Clone, PartialEq, Debug)]
pub struct Bishop { pub color: Color }
impl Piece for Bishop {
    fn name(&self) -> &str { "Bishop" }
    fn color(&self) -> Color { self.color }
    fn legal_moves(&self, board: &Board, from: (usize, usize)) -> Vec<(usize, usize)> {
        todo!()
    }
    fn symbol(&self) -> String { match self.color { Color::White => 'B'.to_string(),
Color::Black => 'b'.to_string()} }
    
    fn clone_box(&self) -> Box<dyn Piece> {
        Box::new(self.clone())
    }
}