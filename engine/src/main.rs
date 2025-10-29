/// ------------- Pieces -------------

trait Piece {
    fn name(&self) -> &str;
    fn color(&self) -> Color;
    fn legal_moves(&self, board: &Board, from: (usize, usize)) -> Vec<(usize, usize)>;
    fn symbol(&self) -> char;
}

#[derive(Clone, Copy)]
enum Color { White, Black }

struct Pawn { color: Color }
impl Piece for Pawn {
    fn name(&self) -> &str { "Pawn" }
    fn color(&self) -> Color { self.color }
    fn legal_moves(&self, board: &Board, from: (usize, usize)) -> Vec<(usize, usize)> {
        todo!()
    }
    fn symbol(&self) -> char { 'P' }
}

struct Rook { color: Color }
impl Piece for Rook {
    fn name(&self) -> &str { "Rook" }
    fn color(&self) -> Color { self.color }
    fn legal_moves(&self, board: &Board, from: (usize, usize)) -> Vec<(usize, usize)> {
        todo!()
    }
    fn symbol(&self) -> char { 'R' }
}

struct Knight { color: Color }
impl Piece for Knight {
    fn name(&self) -> &str { "Knight" }
    fn color(&self) -> Color { self.color }
    fn legal_moves(&self, board: &Board, from: (usize, usize)) -> Vec<(usize, usize)> {
        todo!()
    }
    fn symbol(&self) -> char { 'N' }
}

struct Bishop { color: Color }
impl Piece for Bishop {
    fn name(&self) -> &str { "Bishop" }
    fn color(&self) -> Color { self.color }
    fn legal_moves(&self, board: &Board, from: (usize, usize)) -> Vec<(usize, usize)> {
        todo!()
    }
    fn symbol(&self) -> char { 'B' }
}

struct Queen { color: Color }
impl Piece for Queen {
    fn name(&self) -> &str { "Queen" }
    fn color(&self) -> Color { self.color }
    fn legal_moves(&self, board: &Board, from: (usize, usize)) -> Vec<(usize, usize)> {
        todo!()
    }
    fn symbol(&self) -> char { 'Q' }
}

struct King { color: Color }
impl Piece for King {
    fn name(&self) -> &str { "King" }
    fn color(&self) -> Color { self.color }
    fn legal_moves(&self, board: &Board, from: (usize, usize)) -> Vec<(usize, usize)> {
        todo!()
    }
    fn symbol(&self) -> char { 'K' }
}

// the rest

enum PieceKind {
    Pawn(Pawn),
    Rook(Rook),
    Knight(Knight),
    Bishop(Bishop),
    Queen(Queen),
    King(King),
    Custom(Box<dyn Piece>),
}

/// ------------- End Pieces -------------

/// ------------- Square types -------------
#[derive(PartialEq)]
enum SquareType {
    Standard,
    Turret,
    Vent,
    // adding more later on
}
/// ------------- End Square types -------------


/// ------------- Square conditions -------------
#[derive(PartialEq)]
enum SquareCondition {
    Frozen,
    // adding more later on
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
struct Square {
    piece: Option<PieceKind>,
    square_type: SquareType,
    conditions: Vec<SquareCondition>,
}

fn square_to_fen(square: &Square) -> String {
    let mut parts = vec![];

    // Piece
    if let Some(piece_kind) = &square.piece {
        match piece_kind {
            PieceKind::Custom(p) => parts.push(format!("P={}", p.symbol())),
            PieceKind::Pawn(p) => if square.square_type != SquareType::Standard || !square.conditions.is_empty() {
                parts.push(format!("P={}", p.symbol()));
            },
            PieceKind::Rook(p) => if square.square_type != SquareType::Standard || !square.conditions.is_empty() {
                parts.push(format!("P={}", p.symbol()));
            },
            _ => {}
        }
    }

    // Square type
    if square.square_type != SquareType::Standard {
        parts.push(format!("T={:?}", square.square_type).to_uppercase());
    }

    // Square conditions
    for cond in &square.conditions {
        parts.push(format!("C={:?}", cond).to_uppercase());
    }

    if parts.is_empty() {
        // Standard square with standard piece
        match &square.piece {
            Some(piece) => match piece {
                PieceKind::Pawn(p) => p.symbol().to_string(),
                PieceKind::Rook(p) => p.symbol().to_string(),
                PieceKind::Knight(p) => p.symbol().to_string(),
                PieceKind::Bishop(p) => p.symbol().to_string(),
                PieceKind::Queen(p) => p.symbol().to_string(),
                PieceKind::King(p) => p.symbol().to_string(),
                PieceKind::Custom(p) => format!("({})", p.symbol()),
            },
            None => "1".to_string(), // empty square
        }
    } else {
        format!("({})", parts.join(","))
    }
}


struct Board {
    grid: Vec<Vec<Square>>,
    flags: BoardFlags,
}

fn main() {

}

