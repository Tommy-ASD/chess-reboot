#![allow(unused)]

use std::fmt::{Debug, Formatter, Result};
/// ------------- Pieces -------------

trait Piece {
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
enum Color { White, Black }

#[derive(Clone, PartialEq, Debug)]
struct Pawn { color: Color }
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

#[derive(Clone, PartialEq, Debug)]
struct Rook { color: Color }
impl Piece for Rook {
    fn name(&self) -> &str { "Rook" }
    fn color(&self) -> Color { self.color }
    fn legal_moves(&self, board: &Board, from: (usize, usize)) -> Vec<(usize, usize)> {
        todo!()
    }
    fn symbol(&self) -> String { match self.color { Color::White => 'R'.to_string(),
Color::Black => 'r'.to_string()} }
    
    fn clone_box(&self) -> Box<dyn Piece> {
        Box::new(self.clone())
    }
}

#[derive(Clone, PartialEq, Debug)]
struct Knight { color: Color }
impl Piece for Knight {
    fn name(&self) -> &str { "Knight" }
    fn color(&self) -> Color { self.color }
    fn legal_moves(&self, board: &Board, from: (usize, usize)) -> Vec<(usize, usize)> {
        todo!()
    }
    fn symbol(&self) -> String { match self.color { Color::White => 'N'.to_string(),
Color::Black => 'n'.to_string()} }
    
    fn clone_box(&self) -> Box<dyn Piece> {
        Box::new(self.clone())
    }
}

#[derive(Clone, PartialEq, Debug)]
struct Bishop { color: Color }
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

#[derive(Clone, PartialEq, Debug)]
struct Queen { color: Color }
impl Piece for Queen {
    fn name(&self) -> &str { "Queen" }
    fn color(&self) -> Color { self.color }
    fn legal_moves(&self, board: &Board, from: (usize, usize)) -> Vec<(usize, usize)> {
        todo!()
    }
    fn symbol(&self) -> String { match self.color { Color::White => 'Q'.to_string(),
Color::Black => 'q'.to_string()} }
    
    fn clone_box(&self) -> Box<dyn Piece> {
        Box::new(self.clone())
    }
}

#[derive(Clone, PartialEq, Debug)]
struct King { color: Color }
impl Piece for King {
    fn name(&self) -> &str { "King" }
    fn color(&self) -> Color { self.color }
    fn legal_moves(&self, board: &Board, from: (usize, usize)) -> Vec<(usize, usize)> {
        todo!()
    }
    fn symbol(&self) -> String { match self.color { Color::White => 'K'.to_string(),
Color::Black => 'k'.to_string()} }
    
    fn clone_box(&self) -> Box<dyn Piece> {
        Box::new(self.clone())
    }
}

// the rest
#[derive(Clone, PartialEq, Debug)]
enum PieceKind {
    Pawn(Pawn),
    Rook(Rook),
    Knight(Knight),
    Bishop(Bishop),
    Queen(Queen),
    King(King),
    Custom(Box<dyn Piece>),
}

impl PieceKind {
    fn symbol(&self) -> String {
       match self {
            PieceKind::Pawn(p) => p.symbol().to_string(),
            PieceKind::Rook(r) => r.symbol().to_string(),
            PieceKind::Knight(n) => n.symbol().to_string(),
            PieceKind::Bishop(b) => b.symbol().to_string(),
            PieceKind::Queen(q) => q.symbol().to_string(),
            PieceKind::King(k) => k.symbol().to_string(),
            PieceKind::Custom(p) => p.symbol(),
        }
    }

    fn symbol_to_piece(symbol: char) -> Option<PieceKind> {
        match symbol {
            'P' => Some(PieceKind::Pawn(Pawn { color: Color::White })),
            'R' => Some(PieceKind::Rook(Rook { color: Color::White })),
            'N' => Some(PieceKind::Knight(Knight { color: Color::White })),
            'B' => Some(PieceKind::Bishop(Bishop { color: Color::White })),
            'Q' => Some(PieceKind::Queen(Queen { color: Color::White })),
            'K' => Some(PieceKind::King(King { color: Color::White })),
            'p' => Some(PieceKind::Pawn(Pawn { color: Color::Black })),
            'r' => Some(PieceKind::Rook(Rook { color: Color::Black })),
            'n' => Some(PieceKind::Knight(Knight { color: Color::Black })),
            'b' => Some(PieceKind::Bishop(Bishop { color: Color::Black })),
            'q' => Some(PieceKind::Queen(Queen { color: Color::Black })),
            'k' => Some(PieceKind::King(King { color: Color::Black })),
            _ => None,
        }
    }

    fn new_pawn(color: Color) -> PieceKind {
        Self::Pawn(Pawn { color })
    }

    fn new_rook(color: Color) -> PieceKind {
        Self::Rook(Rook { color })
    }

    fn new_knight(color: Color) -> PieceKind {
        Self::Knight(Knight { color })
    }

    fn new_bishop(color: Color) -> PieceKind {
        Self::Bishop(Bishop { color })
    }

    fn new_queen(color: Color) -> PieceKind {
        Self::Queen(Queen { color })
    }

    fn new_king(color: Color) -> PieceKind {
        Self::King(King { color })
    }
}
/// ------------- End Pieces -------------

/// ------------- Square types -------------
#[derive(PartialEq, Debug, Clone)]
enum SquareType {
    Standard,
    Turret,
    Vent,
    // adding more later on
}

impl SquareType {
    fn as_str(&self) -> &'static str {
        match self {
            SquareType::Standard => "STANDARD",
            SquareType::Turret => "TURRET",
            SquareType::Vent => "VENT",
        }
    }
}
/// ------------- End Square types -------------


/// ------------- Square conditions -------------
#[derive(PartialEq, Debug, Clone)]
enum SquareCondition {
    Frozen,
    // adding more later on
}


impl SquareCondition {
    fn as_str(&self) -> &'static str {
        match self {
            SquareCondition::Frozen => "FROZEN",
        }
    }
}
/// ------------- End Square conditions -------------

/// We use this so there's no confusion with which index is 
#[derive(PartialEq, Debug)]
struct BoardIndex {
    x: usize,
    y: usize
}

#[derive(PartialEq, Debug)]
struct BoardFlags {
    white_can_castle_kingside: bool,
    white_can_castle_queenside: bool,
    black_can_castle_kingside: bool,
    black_can_castle_queenside: bool,
    en_passant_target: Option<BoardIndex>,
    // more fields we can figure out later
}

/// ------------- Square logic -------------
#[derive(Clone, PartialEq, Debug)]
struct Square {
    piece: Option<PieceKind>,
    square_type: SquareType,
    conditions: Vec<SquareCondition>,
}

impl Square {
    fn new() -> Self {
        Self {
            piece: None,
            square_type: SquareType::Standard,
            conditions: vec![]
        }
    }
    fn set_piece(mut self, piece: PieceKind) -> Self {
        self.piece = Some(piece);
        self
    }
    fn remove_piece(mut self) -> Self {
        self.piece = None;
        self
    }
    fn set_square_type(mut self, square_type: SquareType) -> Self {
        self.square_type = square_type;
        self
    }
    fn add_square_condition(mut self, square_condition: SquareCondition) -> Self {
        self.conditions.push(square_condition);
        self
    }
}

fn square_to_fen(square: &Square) -> String {
    let piece_symbol = square.piece.as_ref().map(|p| p.symbol()).unwrap_or("".to_string());
    let is_standard_square = matches!(square.square_type, SquareType::Standard) && square.conditions.is_empty();

    if piece_symbol.len() == 1 && is_standard_square {
        return piece_symbol; // e.g., "P" or "r"
    }

    // Non-standard notation
    let mut parts = vec![];

    if !piece_symbol.is_empty() {
        parts.push(format!("P={}", piece_symbol));
    }

    if !matches!(square.square_type, SquareType::Standard) {
        parts.push(format!("T={}", square.square_type.as_str()));
    }

    for cond in &square.conditions {
        parts.push(format!("C={}", cond.as_str()));
    }

    format!("({})", parts.join(","))
}

fn fen_to_square(fen: &str) -> Square {
    // Standard empty square
    if fen.is_empty() || fen == "()" {
        return Square {
            piece: None,
            square_type: SquareType::Standard,
            conditions: vec![],
        };
    }

    // Extended format (P=...,T=...,C=...)
    if fen.starts_with('(') && fen.ends_with(')') {
        let inner = &fen[1..fen.len()-1];
        let mut piece = None;
        let mut square_type = SquareType::Standard;
        let mut conditions = vec![];

        for part in inner.split(',') {
            let kv: Vec<&str> = part.split('=').collect();
            if kv.len() != 2 { continue; }

            match kv[0] {
                "P" => {
                    let sym = kv[1];
                    if let Some(p) = PieceKind::symbol_to_piece(sym.chars().next().unwrap()) {
                        piece = Some(p);
                    } else {
                        println!("Unknown piece!! {sym}");
                    }
                },
                "T" => {
                    square_type = {
                    let sqty = kv[1];    
                    match sqty {
                        "TURRET" => SquareType::Turret,
                        "VENT" => SquareType::Vent,
                        _ => {
                            println!("Unknown square type!! {sqty}");
                            SquareType::Standard
                        },
                    }}
                },
                "C" => {
                    let sqcon = kv[1];
                    match sqcon {
                        "FROZEN" => conditions.push(SquareCondition::Frozen),
                        _ => {
                            println!("Unknown square condition!! {sqcon}");
                        }
                    }
                },
                _ => {}
            }
        }

        return Square {
            piece,
            square_type,
            conditions,
        };
    }

    // Standard single-character piece
    let piece = PieceKind::symbol_to_piece(fen.chars().next().unwrap());
    Square {
        piece,
        square_type: SquareType::Standard,
        conditions: vec![],
    }
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

fn fen_to_board(fen: &str) -> Board {
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
struct Board {
    grid: Vec<Vec<Square>>,
    flags: BoardFlags,
}

fn board_to_fen(board: &Board) -> String {
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
    
    let rook_vent_test_square = Square::new().set_piece(PieceKind::new_rook(Color::White)).set_square_type(SquareType::Vent);

    board.grid[0][0] = rook_vent_test_square;

    let fen = board_to_fen(&board);
    println!("{}", fen);

    let b2 = fen_to_board(&fen);
    assert_eq!(b2, board);
    println!("Success!!");
    println!("{board:?}");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_board_fen() {
        let board = Board {
            grid: vec![vec![Square::new(); 8]; 8],
            flags: BoardFlags {
                white_can_castle_kingside: true,
                white_can_castle_queenside: true,
                black_can_castle_kingside: true,
                black_can_castle_queenside: true,
                en_passant_target: None,
            },
        };

        let fen = board_to_fen(&board);
        assert_eq!(fen, "8/8/8/8/8/8/8/8");

        let board2 = fen_to_board(&fen);
        assert_eq!(board2, board);
    }

    #[test]
    fn test_standard_pieces_fen() {
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

        board.grid[0][0] = Square::new().set_piece(PieceKind::new_rook(Color::White));
        board.grid[7][7] = Square::new().set_piece(PieceKind::new_king(Color::Black));

        let fen = board_to_fen(&board);
        assert_eq!(fen, "R7/8/8/8/8/8/8/7k");

        let board2 = fen_to_board(&fen);
        assert_eq!(board2, board);
    }

    #[test]
    fn test_extended_square_fen() {
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

        // Place a white rook on a vent square
        board.grid[0][0] = Square::new()
            .set_piece(PieceKind::new_rook(Color::White))
            .set_square_type(SquareType::Vent);

        let fen = board_to_fen(&board);
        assert_eq!(fen, "(P=R,T=VENT)7/8/8/8/8/8/8/8");

        let board2 = fen_to_board(&fen);
        assert_eq!(board2, board);
    }

    #[test]
    fn test_square_with_conditions_fen() {
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

        board.grid[1][1] = Square::new()
            .set_piece(PieceKind::new_knight(Color::Black))
            .add_square_condition(SquareCondition::Frozen);

        let fen = board_to_fen(&board);
        assert_eq!(fen, "8/1(P=n,C=FROZEN)6/8/8/8/8/8/8");

        let board2 = fen_to_board(&fen);
        assert_eq!(board2, board);
    }

    #[test]
    fn test_square_with_conditions_and_types_fen() {
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

        board.grid[1][1] = Square::new()
            .set_piece(PieceKind::new_knight(Color::Black))
            .add_square_condition(SquareCondition::Frozen)
            .set_square_type(SquareType::Vent);

        let fen = board_to_fen(&board);
        assert_eq!(fen, "8/1(P=n,T=VENT,C=FROZEN)6/8/8/8/8/8/8");

        let board2 = fen_to_board(&fen);
        assert_eq!(board2, board);
    }

    #[test]
    fn test_fen_roundtrip() {
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

        // Mix of standard and extended squares
        board.grid[0][0] = Square::new().set_piece(PieceKind::new_rook(Color::White));
        board.grid[0][1] = Square::new()
            .set_piece(PieceKind::new_knight(Color::Black))
            .set_square_type(SquareType::Turret)
            .add_square_condition(SquareCondition::Frozen);

        let fen = board_to_fen(&board);
        let board2 = fen_to_board(&fen);
        assert_eq!(board2, board);
    }
}
