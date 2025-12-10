use serde::{Deserialize, Serialize};

use crate::{
    board::{
        fen::{board_to_fen, fen_to_board, fen_to_square, square_to_fen},
        square::{Square, SquareType},
    },
    pieces::piecetype::PieceType,
};

pub mod fen;
pub mod make_move;
pub mod square;
mod tests;

pub type File = u8; // 0–7 for default boards
pub type Rank = u8; // 0–7 for default boards

/// We use this so there's no confusion with which index is
#[derive(PartialEq, Debug, Clone, Deserialize, Serialize)]
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

/// Represents a move from one coordinate to another.
/// Will likely be expanded later with more info.
#[derive(PartialEq, Debug)]
pub struct GameMove {
    pub from: Coord,
    pub to: Coord,
}

pub type Direction = (isize, isize);

#[derive(PartialEq, Debug, Clone)]
pub struct BoardFlags {
    pub white_can_castle_kingside: bool,
    pub white_can_castle_queenside: bool,
    pub black_can_castle_kingside: bool,
    pub black_can_castle_queenside: bool,
    pub en_passant_target: Option<Coord>,
    // more fields we can figure out later
}

#[derive(PartialEq, Debug, Clone)]
pub struct Board {
    pub grid: Vec<Vec<Square>>,
    pub flags: BoardFlags,
}

impl Board {
    /// Get an immutable reference to the square at `coord`, if within bounds.
    pub fn get_square_at(&self, coord: &Coord) -> Option<&Square> {
        self.grid
            .get(coord.rank as usize)
            .and_then(|row| row.get(coord.file as usize))
    }
    /// Get a mutable reference to the square at `coord`, if within bounds.
    pub fn get_square_mut(&mut self, coord: &Coord) -> Option<&mut Square> {
        self.grid
            .get_mut(coord.rank as usize)
            .and_then(|row| row.get_mut(coord.file as usize))
    }

    pub fn set_piece_at(&mut self, coord: &Coord, piece: PieceType) {
        if let Some(square) = self.get_square_mut(coord) {
            square.piece = Some(piece);
        }
    }

    pub fn square_is_empty(&self, coord: &Coord) -> bool {
        if let Some(square) = self.get_square_at(coord) {
            square.square_type == SquareType::Standard && square.piece.is_none()
        } else {
            false
        }
    }

    /// Get all possible moves for the piece at `from`.
    pub fn get_moves(&self, from: &Coord) -> Vec<GameMove> {
        if let Some(square) = self.get_square_at(from) {
            if let Some(piece) = &square.piece {
                piece.get_moves(self, from)
            } else {
                vec![]
            }
        } else {
            vec![]
        }
    }

    /// Takes a from and to coordinate and returns true if the move is valid.
    pub fn is_valid_move(&self, from: &Coord, to: &Coord) -> bool {
        let possible_moves = self.get_moves(from);
        possible_moves.iter().any(|m| &m.to == to)
    }
}
