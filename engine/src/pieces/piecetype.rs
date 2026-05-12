use std::rc::Rc;

use tracing::error;

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

    pub fn can_carry_piece(&self) -> bool {
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

        // Each candidate move is either a plain `MoveTo` or a `PieceInCarrier`
        // wrapper around a `MoveTo` / `MoveIntoCarrier`. We extract the target
        // coord, decide whether to keep the move based on the target's
        // contents, and — if it lands on a friendly carrier — rewrite the
        // *inner* `MoveTo` to `MoveIntoCarrier` while preserving the outer
        // wrapper (so `piece_index` survives for the consumer).
        use crate::board::MoveType;

        moves.retain_mut(|game_move| {
            // Extract target + remember whether we're inside a PieceInCarrier.
            // The carrier_index is `Some(idx)` iff the outer move is a
            // PieceInCarrier whose inner is itself a MoveTo (the only case
            // that's meaningful to rewrite to MoveIntoCarrier).
            let (target, carrier_index) = match &game_move.move_type {
                MoveType::MoveTo(coord) => (coord.clone(), None),
                MoveType::PieceInCarrier {
                    piece_index,
                    move_type: inner,
                } => match inner.as_ref() {
                    MoveType::MoveTo(coord) => (coord.clone(), Some(*piece_index)),
                    MoveType::MoveIntoCarrier(coord) => (coord.clone(), Some(*piece_index)),
                    _ => {
                        error!(?inner, "unmatched inner MoveType in get_moves filter");
                        todo!();
                    }
                },
                MoveType::PhaseShift => return true,
                MoveType::MoveIntoCarrier(_) => {
                    // Pieces never produce a top-level MoveIntoCarrier from
                    // `initial_moves`; the filter is the sole producer.
                    todo!("top-level MoveIntoCarrier reached the filter");
                }
            };

            let Some(target_square) = board.get_square_at(&target) else {
                return false;
            };

            let Some(target_piece) = &target_square.piece else {
                // empty target — keep as-is
                return true;
            };

            if target_piece.can_carry_piece()
                && target_piece.get_color() == self.get_color()
            {
                // Capacity check — Bus holds at most 5 (per spec).
                let at_capacity = match target_piece {
                    PieceType::Bus(bus) => bus.pieces.len() >= 5,
                    _ => false,
                };
                if at_capacity {
                    return false;
                }
                // Landing on a friendly carrier: swap the *inner* move (or the
                // top-level move) to MoveIntoCarrier, preserving any
                // PieceInCarrier wrapper.
                let into = MoveType::MoveIntoCarrier(target);
                game_move.move_type = match carrier_index {
                    None => into,
                    Some(idx) => MoveType::PieceInCarrier {
                        piece_index: idx,
                        move_type: std::sync::Arc::new(into),
                    },
                };
                true
            } else {
                target_piece.get_color() != self.get_color()
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
