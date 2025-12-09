use std::fmt::{Debug, Formatter, Result};

use crate::board::{Board, Coord, GameMove};

pub mod bishop;
pub mod king;
pub mod knight;
pub mod pawn;
pub mod piecetype;
pub mod queen;
pub mod rook;

/// ------------- Pieces -------------

pub(crate) trait Piece {
    fn name(&self) -> &str;
    fn color(&self) -> Color;
    fn initial_moves(&self, board: &Board, from: &Coord) -> Vec<GameMove>;
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
pub enum Color {
    White,
    Black,
}
