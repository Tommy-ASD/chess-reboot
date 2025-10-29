#![allow(unused)]
/// ------------- Pieces -------------

trait Piece {
    fn name(&self) -> &str;
    fn color(&self) -> Color;
    fn legal_moves(&self, board: &Board, from: (usize, usize)) -> Vec<(usize, usize)>;
    fn symbol(&self) -> String;

    fn clone_box(&self) -> Box<dyn Piece>;
}
impl Clone for Box<dyn Piece> {
    fn clone(&self) -> Box<dyn Piece> {
        self.clone_box()
    }
}

#[derive(Clone, Copy)]
enum Color { White, Black }

#[derive(Clone)]
struct Pawn { color: Color }
impl Piece for Pawn {
    fn name(&self) -> &str { "Pawn" }
    fn color(&self) -> Color { self.color }
    fn legal_moves(&self, board: &Board, from: (usize, usize)) -> Vec<(usize, usize)> {
        todo!()
    }
    fn symbol(&self) -> String { 'P'.to_string() }

    fn clone_box(&self) -> Box<dyn Piece> {
        Box::new(self.clone())
    }
}

#[derive(Clone)]
struct Rook { color: Color }
impl Piece for Rook {
    fn name(&self) -> &str { "Rook" }
    fn color(&self) -> Color { self.color }
    fn legal_moves(&self, board: &Board, from: (usize, usize)) -> Vec<(usize, usize)> {
        todo!()
    }
    fn symbol(&self) -> String { 'R'.to_string() }
    
    fn clone_box(&self) -> Box<dyn Piece> {
        Box::new(self.clone())
    }
}

#[derive(Clone)]
struct Knight { color: Color }
impl Piece for Knight {
    fn name(&self) -> &str { "Knight" }
    fn color(&self) -> Color { self.color }
    fn legal_moves(&self, board: &Board, from: (usize, usize)) -> Vec<(usize, usize)> {
        todo!()
    }
    fn symbol(&self) -> String { 'N'.to_string() }
    
    fn clone_box(&self) -> Box<dyn Piece> {
        Box::new(self.clone())
    }
}

#[derive(Clone)]
struct Bishop { color: Color }
impl Piece for Bishop {
    fn name(&self) -> &str { "Bishop" }
    fn color(&self) -> Color { self.color }
    fn legal_moves(&self, board: &Board, from: (usize, usize)) -> Vec<(usize, usize)> {
        todo!()
    }
    fn symbol(&self) -> String { 'B'.to_string() }

    fn clone_box(&self) -> Box<dyn Piece> {
        Box::new(self.clone())
    }
}

#[derive(Clone)]
struct Queen { color: Color }
impl Piece for Queen {
    fn name(&self) -> &str { "Queen" }
    fn color(&self) -> Color { self.color }
    fn legal_moves(&self, board: &Board, from: (usize, usize)) -> Vec<(usize, usize)> {
        todo!()
    }
    fn symbol(&self) -> String { 'Q'.to_string() }
    
    fn clone_box(&self) -> Box<dyn Piece> {
        Box::new(self.clone())
    }
}

#[derive(Clone)]
struct King { color: Color }
impl Piece for King {
    fn name(&self) -> &str { "King" }
    fn color(&self) -> Color { self.color }
    fn legal_moves(&self, board: &Board, from: (usize, usize)) -> Vec<(usize, usize)> {
        todo!()
    }
    fn symbol(&self) -> String { 'K'.to_string() }
    
    fn clone_box(&self) -> Box<dyn Piece> {
        Box::new(self.clone())
    }
}

// the rest
#[derive(Clone)]
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
}

/// ------------- End Pieces -------------

/// ------------- Square types -------------
#[derive(PartialEq, Clone)]
enum SquareType {
    Standard,
    Vent,
    // adding more later on
}

impl SquareType {
    fn as_str(&self) -> &'static str {
        match self {
            SquareType::Standard => "STANDARD",
            SquareType::Vent => "VENT",
        }
    }
}
/// ------------- End Square types -------------


/// ------------- Square conditions -------------
#[derive(PartialEq, Clone)]
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
struct BoardIndex {
    x: usize,
    y: usize
}

struct BoardFlags {
    white_can_castle_kingside: bool,
    white_can_castle_queenside: bool,
    black_can_castle_kingside: bool,
    black_can_castle_queenside: bool,
    en_passant_target: Option<BoardIndex>,
    // more fields we can figure out later
}

/// ------------- Square logic -------------
#[derive(Clone)]
struct Square {
    piece: Option<PieceKind>,
    square_type: SquareType,
    conditions: Vec<SquareCondition>,
}

fn square_to_fen(square: &Square) -> String {
    let piece_symbol = square.piece.as_ref().map(|p| p.symbol()).unwrap_or("".to_string());
    let is_standard_square = matches!(square.square_type, SquareType::Standard) && square.conditions.is_empty();

    // if the square can be represented by a single character, return that character
    if piece_symbol.len() == 1 && is_standard_square {
        return piece_symbol;
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

/// ------------- End Square logic -------------


/// ------------- Board logic -------------
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

            if fen.is_empty() {
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

/// ------------- End Board logic -------------

fn main() {
    // Example: 1 white custom piece on A1, rest empty
    let mut board = Board {
        grid: vec![vec![Square { piece: Some(PieceKind::Rook(Rook{ color: Color::White })), square_type: SquareType::Vent, conditions: vec![SquareCondition::Frozen] }; 8]; 8],
        flags: BoardFlags {
            white_can_castle_kingside: true,
            white_can_castle_queenside: true,
            black_can_castle_kingside: true,
            black_can_castle_queenside: true,
            en_passant_target: None,
        },
    };

    let fen = board_to_fen(&board);
    println!("{}", fen);
}
