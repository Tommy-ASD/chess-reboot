use crate::board::square::{Square, SquareType, fen_to_square, square_to_fen};

pub mod square;
mod tests;

/// We use this so there's no confusion with which index is 
#[derive(PartialEq, Debug)]
pub struct BoardIndex {
   pub x: usize,
   pub y: usize
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

fn fen_row_to_squares(row: &str) -> Vec<Square> {
    let mut squares = vec![];
    let mut chars = row.chars().peekable();

    while let Some(&ch) = chars.peek() {
        if ch.is_digit(10) {
            let count = ch.to_digit(10).unwrap();
            for _ in 0..count {
                squares.push(Square { piece: None, square_type: SquareType::Standard, conditions: vec![] });
            }
            chars.next();
        } else if ch == '(' {
            // Extended square: find the closing ')'
            let mut fen_piece = String::new();
            let mut depth = 0;
            while let Some(c) = chars.next() {
                fen_piece.push(c);
                if c == '(' { depth += 1; }
                if c == ')' { depth -= 1; if depth == 0 { break; } }
            }
            squares.push(fen_to_square(&fen_piece));
        } else {
            // Normal piece
            squares.push(fen_to_square(&ch.to_string()));
            chars.next();
        }
    }

    squares
}

pub fn fen_to_board(fen: &str) -> Board {
    let rows: Vec<&str> = fen.split('/').collect();
    let mut grid = vec![];

    for row in rows {
        grid.push(fen_row_to_squares(row));
    }

    // Default flags for now
    let flags = BoardFlags {
        white_can_castle_kingside: true,
        white_can_castle_queenside: true,
        black_can_castle_kingside: true,
        black_can_castle_queenside: true,
        en_passant_target: None,
    };

    Board { grid, flags }
}

#[derive(PartialEq, Debug)]
pub struct Board {
    pub grid: Vec<Vec<Square>>,
    pub flags: BoardFlags,
}

pub fn board_to_fen(board: &Board) -> String {
    let mut rows = vec![];

    for row in &board.grid {
        let mut fen_row = String::new();
        let mut empty_count = 0;

        for square in row {
            let fen = square_to_fen(square);

            if fen.is_empty() || fen == "()" {
                empty_count += 1;
            } else {
                if empty_count > 0 {
                    fen_row.push_str(&empty_count.to_string());
                    empty_count = 0;
                }
                fen_row.push_str(&fen);
            }
        }

        if empty_count > 0 {
            fen_row.push_str(&empty_count.to_string());
        }

        rows.push(fen_row);
    }

    rows.join("/")
}