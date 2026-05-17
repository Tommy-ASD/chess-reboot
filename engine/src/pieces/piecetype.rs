use std::sync::Arc;

use crate::{
    board::GameMove,
    pieces::{
        Color, Piece,
        chess2::monkey::Monkey,
        fairy::{
            bus::Bus, carriage::Carriage, goblin::Goblin, locomotive::Locomotive,
            skibidi::Skibidi, stormcaller::Stormcaller,
        },
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
            PieceType::Locomotive($piece) => $body,
            PieceType::Carriage($piece) => $body,
            PieceType::Stormcaller($piece) => $body,
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
    Locomotive(crate::pieces::fairy::locomotive::Locomotive),
    Carriage(crate::pieces::fairy::carriage::Carriage),
    Stormcaller(crate::pieces::fairy::stormcaller::Stormcaller),
}

impl PieceType {
    /// Plan 09: is this piece a train cart (locomotive or carriage)?
    /// Used by `advance_trains` to identify carts quickly, and by
    /// `find_king` to know which carriers to descend into.
    pub fn is_train_cart(&self) -> bool {
        matches!(self, PieceType::Locomotive(_) | PieceType::Carriage(_))
    }

    /// Read-only view of a carrier's passenger list. Returns `Some(&[])`
    /// for an empty carrier and `None` for non-carriers. Dispatches
    /// to `Piece::passengers`, so any future custom carrier piece
    /// only needs to override the trait method — no `PieceType` arm
    /// to update.
    pub fn passengers(&self) -> Option<&[PieceType]> {
        dispatch!(self, p => p.passengers())
    }

    /// Mutable handle on a carrier's passenger list. Dispatches to
    /// `Piece::passengers_mut`. Used by make_move when a passenger
    /// enters, exits, or is captured by an enemy boarding.
    pub fn passengers_mut(&mut self) -> Option<&mut Vec<PieceType>> {
        dispatch!(self, p => p.passengers_mut())
    }
}

impl From<Arc<PieceType>> for PieceType {
    fn from(arc: Arc<PieceType>) -> Self {
        arc.as_ref().clone()
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

            // Monkey serializes as `M`/`m` but had no parse arm — it
            // silently round-tripped to an empty square (plan 05). A
            // Neutral monkey serializes as `M` and parses back as White,
            // the same Neutral asymmetry the standard pieces carry.
            "m" => Some(PieceType::new_monkey(if sym == "M" {
                Color::White
            } else {
                Color::Black
            })),
            "g" => Goblin::from_symbol(symbol),
            "s" => Skibidi::from_symbol(symbol),
            "w" => Stormcaller::from_symbol(symbol),
            "bus" => Bus::from_symbol(symbol),
            "loco" => Locomotive::from_symbol(symbol),
            "cart" => Carriage::from_symbol(symbol),

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

    pub fn new_monkey(color: Color) -> PieceType {
        Self::Monkey(Monkey { color })
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

    pub fn can_throw_switch(&self) -> bool {
        dispatch!(self, p => p.can_throw_switch())
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
            //
            // `is_promotion` short-circuits the carrier rewrite: a promoted
            // pawn doesn't *enter* a friendly bus on the back rank, it
            // promotes to the chosen piece on that square (or, if the
            // square is enemy-occupied, captures).
            let (target, carrier_index, is_promotion) = match &game_move.move_type {
                MoveType::MoveTo(coord) => (coord.clone(), None, false),
                MoveType::Promotion { target, .. } => (target.clone(), None, true),
                MoveType::PieceInCarrier {
                    piece_index,
                    move_type: inner,
                } => match inner.as_ref() {
                    MoveType::MoveTo(coord) => {
                        (coord.clone(), Some(*piece_index), false)
                    }
                    MoveType::MoveIntoCarrier(coord) => {
                        (coord.clone(), Some(*piece_index), false)
                    }
                    // Passenger-level PhaseShift or nested PieceInCarrier
                    // aren't supported through the carrier today — drop the
                    // move rather than crash. The passenger can still take
                    // such moves once it has exited the carrier.
                    _ => return false,
                },
                MoveType::PhaseShift => return true,
                // Castle's preconditions are fully validated by
                // King::castle_moves before we ever see the move here, so
                // there is nothing for the generic filter to check.
                MoveType::Castle { .. } => return true,
                // EnPassant emits onto an empty square by construction
                // (the ep target is the square the double-pushing pawn
                // jumped over) — the filter's empty-target check would
                // pass, but skipping it explicitly avoids any temptation
                // to interpret the capture-coord as a target.
                MoveType::EnPassant { .. } => return true,
                // ThrowSwitch doesn't move the piece — the filter's
                // target-occupancy logic doesn't apply. Whether the piece
                // is even *allowed* to throw is checked at move-generation
                // time in `Board::get_moves` via `can_throw_switch()`.
                MoveType::ThrowSwitch { .. } => return true,
                // PlaceTornado doesn't move the piece — like
                // ThrowSwitch, the target-occupancy/carrier-rewrite
                // logic below doesn't apply. The placer stays put and
                // the target square gets a condition, not a piece.
                MoveType::PlaceTornado { .. } => return true,
                MoveType::MoveIntoCarrier(_) => {
                    // No piece's `initial_moves` produces a top-level
                    // MoveIntoCarrier today — the filter is the sole
                    // producer. Drop defensively instead of panicking.
                    return false;
                }
            };

            let Some(target_square) = board.get_square_at(&target) else {
                return false;
            };

            let Some(target_piece) = &target_square.piece else {
                // empty target — keep as-is
                return true;
            };

            // Same-color carrier → friendly board. Neutral carrier (a
            // train cart) → board-by-capture: the cart is invincible, so
            // moving onto its tile boards it rather than capturing it.
            // When the cart is Neutral and the boarder isn't, the
            // make_move MoveIntoCarrier handler removes opposite-
            // colour passengers (plan 09's "passenger captured by
            // enemy entering cart" rule). Same-colour boarding of a
            // colour-matched carrier (Bus) is purely additive — no
            // passenger capture.
            let target_is_boardable = target_piece.can_carry_piece()
                && (target_piece.get_color() == self.get_color()
                    || target_piece.get_color() == Color::Neutral);
            // Promotion onto a Neutral cart would let `relocate_pieces`
            // overwrite the cart with the promoted piece, breaking the
            // trains-invincible invariant. The pawn has no way to
            // "promote inside the cart" through the current pipeline,
            // so drop the move — the pawn can still push or capture
            // onto a non-cart promotion square.
            if is_promotion && target_piece.get_color() == Color::Neutral {
                return false;
            }
            if !is_promotion && target_is_boardable {
                // Capacity check — Bus holds at most 5 (per spec). Trains
                // have no cap in v1.
                let at_capacity = match target_piece {
                    PieceType::Bus(bus) => bus.pieces.len() >= 5,
                    _ => false,
                };
                if at_capacity {
                    return false;
                }
                // Forbid nested carriers — a carrier inside a carrier
                // would obscure passenger accounting and break the
                // "topmost piece is the cart" invariant for trains.
                let entering_is_carrier = match carrier_index {
                    None => self.can_carry_piece(),
                    Some(idx) => self
                        .passengers()
                        .and_then(|ps| ps.get(idx as usize))
                        .map(|p| p.can_carry_piece())
                        .unwrap_or(false),
                };
                if entering_is_carrier {
                    return false;
                }
                // Rewrite the move so make_move treats this as boarding
                // (cart preserved), not relocation (cart destroyed).
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
        &self,
        board_before: &crate::board::Board,
        board_after: &mut crate::board::Board,
        game_move: &GameMove,
    ) {
        dispatch!(self, p => p.post_move_effects(board_before, board_after, game_move))
    }

    /// See `Piece::attacks` for semantics.
    pub fn attacks(
        &self,
        board: &crate::board::Board,
        from: &crate::board::Coord,
    ) -> Vec<crate::board::Coord> {
        dispatch!(self, p => p.attacks(board, from))
    }

    /// See `Piece::would_capture_at` for semantics. Drives the
    /// "phantom-attack" guard in `Board::is_attacked_by`: a piece
    /// may *reach* a tile via its attack set without that constituting
    /// a capture. Default-true for everything except train carts.
    pub fn would_capture_at(
        &self,
        board: &crate::board::Board,
        from: &crate::board::Coord,
        target: &crate::board::Coord,
    ) -> bool {
        dispatch!(self, p => p.would_capture_at(board, from, target))
    }
}
