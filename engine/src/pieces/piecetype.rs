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

    /// Does this piece block the path of other pieces?
    /// Currently, all pieces block paths, but maybe we add ghosts or whatnot later.
    /// This will likely also take in more parameters in the future.
    pub fn blocks_path(&self) -> bool {
        true
    }

    pub fn get_color(&self) -> Color {
        match self {
            PieceType::Pawn(p) => p.color(),
            PieceType::Rook(r) => r.color(),
            PieceType::Knight(n) => n.color(),
            PieceType::Bishop(b) => b.color(),
            PieceType::Queen(q) => q.color(),
            PieceType::King(k) => k.color(),
            PieceType::Custom(p) => p.color(),
        }
    }

    pub fn get_moves(
        &self,
        board: &crate::board::Board,
        from: &crate::board::Coord,
    ) -> Vec<crate::board::GameMove> {
        let mut moves = match self {
            PieceType::Pawn(p) => p.initial_moves(board, from),
            PieceType::Rook(r) => r.initial_moves(board, from),
            PieceType::Knight(n) => n.initial_moves(board, from),
            PieceType::Bishop(b) => b.initial_moves(board, from),
            PieceType::Queen(q) => q.initial_moves(board, from),
            PieceType::King(k) => k.initial_moves(board, from),
            PieceType::Custom(p) => p.initial_moves(board, from),
        };

        // if the square has a piece of the same color, filter it out
        moves.retain(|game_move| {
            if let Some(target_square) = board.get_square_at(&game_move.to) {
                if let Some(target_piece) = &target_square.piece {
                    target_piece.get_color() != self.get_color()
                } else {
                    true
                }
            } else {
                false
            }
        });

        moves
    }
}
