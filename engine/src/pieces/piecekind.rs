use crate::pieces::{Color, Piece, bishop::Bishop, king::King, knight::Knight, pawn::Pawn, queen::Queen, rook::Rook};

// the rest
#[derive(Clone, PartialEq, Debug)]
pub enum PieceKind {
    Pawn(Pawn),
    Rook(Rook),
    Knight(Knight),
    Bishop(Bishop),
    Queen(Queen),
    King(King),
    Custom(Box<dyn Piece>),
}

impl PieceKind {
    pub fn symbol(&self) -> String {
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

    pub fn symbol_to_piece(symbol: char) -> Option<PieceKind> {
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

    pub fn new_pawn(color: Color) -> PieceKind {
        Self::Pawn(Pawn { color })
    }

    pub fn new_rook(color: Color) -> PieceKind {
        Self::Rook(Rook { color })
    }

    pub fn new_knight(color: Color) -> PieceKind {
        Self::Knight(Knight { color })
    }

    pub fn new_bishop(color: Color) -> PieceKind {
        Self::Bishop(Bishop { color })
    }

    pub fn new_queen(color: Color) -> PieceKind {
        Self::Queen(Queen { color })
    }

    pub fn new_king(color: Color) -> PieceKind {
        Self::King(King { color })
    }
}