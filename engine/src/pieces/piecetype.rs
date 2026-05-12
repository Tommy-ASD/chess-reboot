use std::rc::Rc;

use crate::{
    board::GameMove,
    pieces::{
        Color, Piece,
        fairy::{bus::Bus, goblin::Goblin, skibidi::Skibidi},
        standard::{
            bishop::Bishop, king::King, knight::Knight, pawn::Pawn, queen::Queen, rook::Rook,
        },
    },
};

/// Dispatches a method call to whichever piece variant is held.
/// Every `PieceType` variant wraps something that implements `Piece`,
/// so the body works uniformly across them.
macro_rules! dispatch {
    ($self:expr, $piece:ident => $body:expr) => {
        match $self {
            PieceType::Pawn($piece) => $body,
            PieceType::Rook($piece) => $body,
            PieceType::Knight($piece) => $body,
            PieceType::Bishop($piece) => $body,
            PieceType::Queen($piece) => $body,
            PieceType::King($piece) => $body,
            PieceType::Monkey($piece) => $body,
            PieceType::Goblin($piece) => $body,
            PieceType::Skibidi($piece) => $body,
            PieceType::Bus($piece) => $body,
            PieceType::Custom($piece) => $body,
        }
    };
}

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
    Skibidi(crate::pieces::fairy::skibidi::Skibidi),
    Bus(crate::pieces::fairy::bus::Bus),

    Custom(Box<dyn Piece>),
}

impl From<Rc<PieceType>> for PieceType {
    fn from(rc: Rc<PieceType>) -> Self {
        rc.as_ref().clone()
    }
}

impl PieceType {
    pub fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        dispatch!(self, p => p.as_any_mut())
    }

    pub fn symbol(&self) -> String {
        dispatch!(self, p => p.symbol())
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
            "s" => Skibidi::from_symbol(symbol),
            "bus" => Bus::from_symbol(symbol),

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

    fn can_carry_piece(&self) -> bool {
        dispatch!(self, p => p.can_carry_piece())
    }

    pub fn get_color(&self) -> Color {
        dispatch!(self, p => p.color())
    }

    pub fn set_color(&mut self, color: Color) {
        dispatch!(self, p => p.set_color(color))
    }

    pub fn get_moves(
        &self,
        board: &crate::board::Board,
        from: &crate::board::Coord,
    ) -> Vec<crate::board::GameMove> {
        let mut moves = dispatch!(self, p => p.initial_moves(board, from));

        // if the square has a piece of the same color, filter it out
        moves.retain_mut(|game_move| {
            let target = match &game_move.move_type {
                crate::board::MoveType::MoveTo(coord) => coord,
                crate::board::MoveType::PieceInCarrier {
                    piece_index: idx,
                    move_type: inner_move_type,
                } => match inner_move_type.as_ref() {
                    crate::board::MoveType::MoveTo(coord) => coord,
                    crate::board::MoveType::MoveIntoCarrier(coord) => coord,
                    _ => {
                        println!("Unmatched type {inner_move_type:?}");
                        todo!();
                        return false;
                    }
                },
                crate::board::MoveType::PhaseShift => return true,
                _ => {
                    todo!("Handle other move types in get_moves filtering");
                }
            };
            if let Some(target_square) = board.get_square_at(&target) {
                if let Some(target_piece) = &target_square.piece {
                    // if target piece is friendly carrying piece
                    if target_piece.can_carry_piece()
                        && target_piece.get_color() == self.get_color()
                    {
                        game_move.move_type =
                            crate::board::MoveType::MoveIntoCarrier(target.clone());
                        true
                    } else {
                        target_piece.get_color() != self.get_color()
                    }
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
        &mut self,
        board_before: &crate::board::Board,
        board_after: &mut crate::board::Board,
        game_move: &GameMove,
    ) {
        dispatch!(self, p => p.post_move_effects(board_before, board_after, game_move))
    }
}
