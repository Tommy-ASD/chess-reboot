use std::rc::Rc;

use crate::pieces::{
    Color, Piece,
    fairy::goblin::Goblin,
    standard::{bishop::Bishop, king::King, knight::Knight, pawn::Pawn, queen::Queen, rook::Rook},
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

    Monkey(crate::pieces::chess2::monkey::Monkey),
    Goblin(crate::pieces::fairy::goblin::Goblin),

    Custom(Box<dyn Piece>),
}

impl From<Rc<PieceType>> for PieceType {
    fn from(rc: Rc<PieceType>) -> Self {
        rc.as_ref().clone()
    }
}

impl PieceType {
    pub fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        match self {
            PieceType::Pawn(piece) => piece.as_any_mut(),
            PieceType::Rook(piece) => piece.as_any_mut(),
            PieceType::Knight(piece) => piece.as_any_mut(),
            PieceType::Bishop(piece) => piece.as_any_mut(),
            PieceType::Queen(piece) => piece.as_any_mut(),
            PieceType::King(piece) => piece.as_any_mut(),

            PieceType::Monkey(piece) => piece.as_any_mut(),
            PieceType::Goblin(piece) => piece.as_any_mut(),

            PieceType::Custom(piece) => piece.as_any_mut(),
        }
    }

    pub fn symbol(&self) -> String {
        match self {
            PieceType::Pawn(piece) => piece.symbol(),
            PieceType::Rook(piece) => piece.symbol(),
            PieceType::Knight(piece) => piece.symbol(),
            PieceType::Bishop(piece) => piece.symbol(),
            PieceType::Queen(piece) => piece.symbol(),
            PieceType::King(piece) => piece.symbol(),

            PieceType::Monkey(piece) => piece.symbol(),
            PieceType::Goblin(piece) => piece.symbol(),

            PieceType::Custom(piece) => piece.symbol(),
        }
    }

    pub fn symbol_to_piece(symbol: &str) -> Option<PieceType> {
        // get initial symbol (before first bracket, if any)
        // can't just be first character, as some symbols may be multiple characters
        let sym = symbol.split('(').next().unwrap();

        // next, lower case to match both colors
        let sym_lower = sym.to_lowercase();

        // println!("Parsing piece from symbol: {}", symbol);
        // println!("  -> base symbol: {}", sym);

        // match symbol to create piece
        // give full symbol to piece constructors
        match sym_lower.as_str() {
            "p" => Some(PieceType::new_pawn(if sym == "P" {
                Color::White
            } else {
                Color::Black
            })),
            "r" => Some(PieceType::new_rook(if sym == "R" {
                Color::White
            } else {
                Color::Black
            })),
            "n" => Some(PieceType::new_knight(if sym == "N" {
                Color::White
            } else {
                Color::Black
            })),
            "b" => Some(PieceType::new_bishop(if sym == "B" {
                Color::White
            } else {
                Color::Black
            })),
            "q" => Some(PieceType::new_queen(if sym == "Q" {
                Color::White
            } else {
                Color::Black
            })),
            "k" => Some(PieceType::new_king(if sym == "K" {
                Color::White
            } else {
                Color::Black
            })),

            "g" => Goblin::from_symbol(symbol),

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
            PieceType::Pawn(piece) => piece.color(),
            PieceType::Rook(piece) => piece.color(),
            PieceType::Knight(piece) => piece.color(),
            PieceType::Bishop(piece) => piece.color(),
            PieceType::Queen(piece) => piece.color(),
            PieceType::King(piece) => piece.color(),
            PieceType::Monkey(piece) => piece.color(),
            PieceType::Goblin(piece) => piece.color(),
            PieceType::Custom(piece) => piece.color(),
        }
    }

    pub fn set_color(&mut self, color: Color) {
        match self {
            PieceType::Pawn(piece) => piece.set_color(color),
            PieceType::Rook(piece) => piece.set_color(color),
            PieceType::Knight(piece) => piece.set_color(color),
            PieceType::Bishop(piece) => piece.set_color(color),
            PieceType::Queen(piece) => piece.set_color(color),
            PieceType::King(piece) => piece.set_color(color),
            PieceType::Monkey(piece) => piece.set_color(color),
            PieceType::Goblin(piece) => piece.set_color(color),
            PieceType::Custom(piece) => piece.set_color(color),
        }
    }

    pub fn get_moves(
        &self,
        board: &crate::board::Board,
        from: &crate::board::Coord,
    ) -> Vec<crate::board::GameMove> {
        let mut moves = match self {
            PieceType::Pawn(piece) => piece.initial_moves(board, from),
            PieceType::Rook(piece) => piece.initial_moves(board, from),
            PieceType::Knight(piece) => piece.initial_moves(board, from),
            PieceType::Bishop(piece) => piece.initial_moves(board, from),
            PieceType::Queen(piece) => piece.initial_moves(board, from),
            PieceType::King(piece) => piece.initial_moves(board, from),
            PieceType::Monkey(piece) => piece.initial_moves(board, from),
            PieceType::Goblin(piece) => piece.initial_moves(board, from),
            PieceType::Custom(piece) => piece.initial_moves(board, from),
        };

        // if the square has a piece of the same color, filter it out
        moves.retain(|game_move| {
            let target = match &game_move.move_type {
                crate::board::MoveType::MoveTo(coord) => coord,
                _ => {
                    todo!("Handle other move types in get_moves filtering");
                }
            };
            if let Some(target_square) = board.get_square_at(&target) {
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

    pub fn post_move_effects(
        &self,
        board_before: &crate::board::Board,
        board_after: &mut crate::board::Board,
        from: &crate::board::Coord,
        to: &crate::board::Coord,
    ) {
        match self {
            PieceType::Pawn(piece) => piece.post_move_effects(board_before, board_after, from, to),
            PieceType::Rook(piece) => piece.post_move_effects(board_before, board_after, from, to),
            PieceType::Knight(piece) => {
                piece.post_move_effects(board_before, board_after, from, to)
            }
            PieceType::Bishop(piece) => {
                piece.post_move_effects(board_before, board_after, from, to)
            }
            PieceType::Queen(piece) => piece.post_move_effects(board_before, board_after, from, to),
            PieceType::King(piece) => piece.post_move_effects(board_before, board_after, from, to),
            PieceType::Monkey(piece) => {
                piece.post_move_effects(board_before, board_after, from, to)
            }
            PieceType::Goblin(piece) => {
                piece.post_move_effects(board_before, board_after, from, to)
            }
            PieceType::Custom(piece) => {
                piece.post_move_effects(board_before, board_after, from, to)
            }
        }
    }
}
