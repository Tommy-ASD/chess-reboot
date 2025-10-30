use crate::board::square::{Square, SquareType, fen_to_square, square_to_fen};

pub mod fen;
pub mod square;
mod tests;

/// We use this so there's no confusion with which index is
#[derive(PartialEq, Debug)]
pub struct BoardIndex {
    pub x: usize,
    pub y: usize,
}

#[derive(PartialEq, Debug)]
pub struct BoardFlags {
    pub white_can_castle_kingside: bool,
    pub white_can_castle_queenside: bool,
    pub black_can_castle_kingside: bool,
    pub black_can_castle_queenside: bool,
    pub en_passant_target: Option<BoardIndex>,
    // more fields we can figure out later
}

#[derive(PartialEq, Debug)]
pub struct Board {
    pub grid: Vec<Vec<Square>>,
    pub flags: BoardFlags,
}
