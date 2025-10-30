use std::fmt::{Formatter, Debug, Result};

use crate::board::Board;

pub mod bishop;pub mod queen;pub mod rook;pub mod pawn;pub mod knight;pub mod king;
pub mod piecekind;

/// ------------- Pieces -------------

pub(crate) trait Piece {
    fn name(&self) -> &str;
    fn color(&self) -> Color;
    fn legal_moves(&self, board: &Board, from: (usize, usize)) -> Vec<(usize, usize)>;
    fn symbol(&self) -> String;

    fn clone_box(&self) -> Box<dyn Piece>;
}

impl PartialEq for dyn Piece {
    fn eq(&self, other: &Self) -> bool {
        self.symbol() == other.symbol()
    }
}

impl Debug for dyn Piece {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self.symbol())
    }
}

impl Clone for Box<dyn Piece> {
    fn clone(&self) -> Box<dyn Piece> {
        self.clone_box()
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Color { White, Black }

