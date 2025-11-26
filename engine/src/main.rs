#![allow(unused)]

use std::fmt::{Debug, Formatter, Result};

use crate::{
    board::{
        Board, BoardFlags,
        fen::{board_to_fen, fen_to_board},
        square::{Square, SquareType},
    },
    pieces::{Color, piecetype::PieceType},
};

mod board;
mod movement;
mod pieces;

fn main() {
    let mut board = Board {
        grid: vec![vec![Square::new(); 8]; 8],
        flags: BoardFlags {
            white_can_castle_kingside: true,
            white_can_castle_queenside: true,
            black_can_castle_kingside: true,
            black_can_castle_queenside: true,
            en_passant_target: None,
        },
    };

    let rook_vent_test_square = Square::new()
        .set_piece(PieceType::new_rook(Color::White))
        .set_square_type(SquareType::Vent);

    board.grid[0][0] = rook_vent_test_square;

    let fen = board_to_fen(&board);
    println!("{}", fen);

    let b2 = fen_to_board(&fen);
    assert_eq!(b2, board);
    println!("Success!!");
    println!("{board:?}");

    // see if moves work
    let from = crate::board::Coord { file: 0, rank: 0 };
    let moves = board.get_moves(&from);
    println!("Moves from {from:?}: {moves:?}");
}
