use crate::board::square::{Square, SquareType, fen_to_square, square_to_fen};

pub mod fen;
pub mod square;
mod tests;

pub type File = u8; // 0–7 for default boards
pub type Rank = u8; // 0–7 for default boards

/// We use this so there's no confusion with which index is
#[derive(PartialEq, Debug)]
pub struct Coord {
    pub file: File,
    pub rank: Rank,
}

pub type Sq = u8; // 0..63 on a standard board
fn coord_to_sq(c: Coord) -> Sq {
    c.rank * 8 + c.file
}
fn sq_to_coord(sq: Sq) -> Coord {
    Coord {
        file: sq % 8,
        rank: sq / 8,
    }
}

#[derive(PartialEq, Debug)]
pub struct BoardFlags {
    pub white_can_castle_kingside: bool,
    pub white_can_castle_queenside: bool,
    pub black_can_castle_kingside: bool,
    pub black_can_castle_queenside: bool,
    pub en_passant_target: Option<Coord>,
    // more fields we can figure out later
}

#[derive(PartialEq, Debug)]
pub struct Board {
    pub grid: Vec<Vec<Square>>,
    pub flags: BoardFlags,
}
