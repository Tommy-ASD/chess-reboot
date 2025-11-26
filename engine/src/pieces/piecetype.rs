use crate::pieces::{
    Color, Piece, bishop::Bishop, king::King, knight::Knight, pawn::Pawn, queen::Queen, rook::Rook,
};

// the rest
#[derive(Clone, PartialEq, Debug)]
pub enum PieceType {
    Pawn(Pawn),
    Rook(Rook),
    Knight(Knight),
    Bishop(Bishop),
    Queen(Queen),
    King(King),
    Custom(Box<dyn Piece>),
}

impl PieceType {
    pub fn symbol(&self) -> String {
        match self {
            PieceType::Pawn(p) => p.symbol().to_string(),
            PieceType::Rook(r) => r.symbol().to_string(),
            PieceType::Knight(n) => n.symbol().to_string(),
            PieceType::Bishop(b) => b.symbol().to_string(),
            PieceType::Queen(q) => q.symbol().to_string(),
            PieceType::King(k) => k.symbol().to_string(),
            PieceType::Custom(p) => p.symbol(),
        }
    }

    pub fn symbol_to_piece(symbol: char) -> Option<PieceType> {
        match symbol {
            'P' => Some(PieceType::Pawn(Pawn {
                color: Color::White,
            })),
            'R' => Some(PieceType::Rook(Rook {
                color: Color::White,
            })),
            'N' => Some(PieceType::Knight(Knight {
                color: Color::White,
            })),
            'B' => Some(PieceType::Bishop(Bishop {
                color: Color::White,
            })),
            'Q' => Some(PieceType::Queen(Queen {
                color: Color::White,
            })),
            'K' => Some(PieceType::King(King {
                color: Color::White,
            })),
            'p' => Some(PieceType::Pawn(Pawn {
                color: Color::Black,
            })),
            'r' => Some(PieceType::Rook(Rook {
                color: Color::Black,
            })),
            'n' => Some(PieceType::Knight(Knight {
                color: Color::Black,
            })),
            'b' => Some(PieceType::Bishop(Bishop {
                color: Color::Black,
            })),
            'q' => Some(PieceType::Queen(Queen {
                color: Color::Black,
            })),
            'k' => Some(PieceType::King(King {
                color: Color::Black,
            })),
            _ => None,
        }
    }

    pub fn new_pawn(color: Color) -> PieceType {
        Self::Pawn(Pawn { color })
    }

    pub fn new_rook(color: Color) -> PieceType {
        Self::Rook(Rook { color })
    }

    pub fn new_knight(color: Color) -> PieceType {
        Self::Knight(Knight { color })
    }

    pub fn new_bishop(color: Color) -> PieceType {
        Self::Bishop(Bishop { color })
    }

    pub fn new_queen(color: Color) -> PieceType {
        Self::Queen(Queen { color })
    }

    pub fn new_king(color: Color) -> PieceType {
        Self::King(King { color })
    }
}
