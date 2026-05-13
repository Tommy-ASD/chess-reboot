use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::{
    board::square::{Square, SquareCondition, SquareType},
    pieces::{Color, piecetype::PieceType},
};

pub mod brainrot;
pub mod fen;
pub mod make_move;
pub mod signal;
pub mod square;
mod tests;

pub type File = u8; // 0–7 for default boards
pub type Rank = u8; // 0–7 for default boards

/// Opaque identifier wiring a signal emitter (e.g. `SquareType::Switch`) to
/// one or more receivers (e.g. `SquareType::Junction`, `SquareType::Gate`).
/// Many-to-many: a switch can target several IDs, and several switches can
/// share a target ID. IDs are arbitrary u32 — the editor allocates them.
pub type SignalId = u32;

/// We use this so there's no confusion with which index is which.
#[derive(PartialEq, Debug, Clone, Deserialize, Serialize)]
pub struct Coord {
    pub file: File,
    pub rank: Rank,
}

impl std::fmt::Display for Coord {
    /// `(file, rank)` index notation. Algebraic ("e4") needs the board
    /// height to be correct, which `Coord` doesn't have access to — use
    /// `Board::format_coord` when you need algebraic for a real board.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.file, self.rank)
    }
}

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub enum PromotionTarget {
    Queen,
    Rook,
    Bishop,
    Knight,
}

impl std::fmt::Display for PromotionTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            PromotionTarget::Queen => "queen",
            PromotionTarget::Rook => "rook",
            PromotionTarget::Bishop => "bishop",
            PromotionTarget::Knight => "knight",
        })
    }
}

#[derive(PartialEq, Debug, Clone, Copy, Serialize, Deserialize)]
pub enum CastleSide {
    Kingside,
    Queenside,
}

impl std::fmt::Display for CastleSide {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            CastleSide::Kingside => "kingside",
            CastleSide::Queenside => "queenside",
        })
    }
}

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", content = "target")]
pub enum MoveType {
    MoveTo(Coord),
    MoveIntoCarrier(Coord),
    PieceInCarrier {
        piece_index: u8,
        move_type: Arc<MoveType>,
    },
    PhaseShift,
    Promotion {
        target: Coord,
        into: PromotionTarget,
    },
    Castle {
        side: CastleSide,
    },
    EnPassant {
        target: Coord,
        captured: Coord,
    },
    /// Plan 08: throw the Switch tile this piece is standing on. The piece
    /// stays put; the signal pulse fires at the Switch's `targets`. The
    /// `switch` coord is technically redundant with `GameMove.from` today
    /// (the piece on the switch tile is throwing it) but storing it
    /// explicitly leaves room for a future "throw an adjacent switch"
    /// mechanic without breaking the move shape.
    ThrowSwitch {
        switch: Coord,
    },
}

impl std::fmt::Display for MoveType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MoveType::MoveTo(c) => write!(f, "move to {c}"),
            MoveType::MoveIntoCarrier(c) => write!(f, "board the carrier at {c}"),
            MoveType::PieceInCarrier {
                piece_index,
                move_type,
            } => write!(f, "passenger #{piece_index} → {move_type}"),
            MoveType::PhaseShift => f.write_str("phase shift"),
            MoveType::Promotion { target, into } => {
                write!(f, "promote to {into} at {target}")
            }
            MoveType::Castle { side } => write!(f, "castle {side}"),
            MoveType::EnPassant { target, captured } => {
                write!(f, "en-passant to {target} (capturing {captured})")
            }
            MoveType::ThrowSwitch { switch } => write!(f, "throw switch at {switch}"),
        }
    }
}

/// Represents a move from one coordinate to another.
/// Will likely be expanded later with more info.
#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct GameMove {
    pub from: Coord,
    pub move_type: MoveType,
}

pub type Direction = (isize, isize);

#[derive(PartialEq, Debug, Clone)]
pub struct BoardFlags {
    pub side_to_move: Color,
    pub white_can_castle_kingside: bool,
    pub white_can_castle_queenside: bool,
    pub black_can_castle_kingside: bool,
    pub black_can_castle_queenside: bool,
    pub en_passant_target: Option<Coord>,
    // more fields we can figure out later
}

#[derive(PartialEq, Debug, Clone)]
pub enum GameStatus {
    Ongoing,
    Check { side_to_move: Color },
    Checkmate { winner: Color },
    Stalemate,
}

/// Helper used by `Board::find_king` and tests. Lives at module scope so
/// the closure inside `find_king` doesn't need to capture anything.
fn king_of_color(piece: &PieceType, color: Color) -> bool {
    matches!(piece, PieceType::King(k) if k.color == color)
}

/// Structured failure reasons for `Board::make_move` /
/// `Board::validate_move`. Surfaces enough context that an API consumer
/// can render a useful error message without re-deriving state from the
/// FEN. Plan 06 will likely flow these straight into HTTP error bodies.
#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "code", rename_all = "snake_case")]
pub enum MoveError {
    /// `from` is outside the board grid.
    NoSourceSquare { from: Coord },
    /// `from` is in bounds but the square is empty.
    NoPieceAtSource { from: Coord },
    /// A piece is at `from` but it's the other side's turn.
    WrongTurn {
        from: Coord,
        piece_symbol: String,
        piece_color: Color,
        side_to_move: Color,
    },
    /// The attempted move isn't in the piece's raw move set at all —
    /// e.g. trying to move a rook diagonally, or trying to land on a
    /// friendly piece. `candidate_alternatives` lists the move-shapes
    /// the piece actually offers from `from`. These are the *raw*
    /// candidates (pre-king-safety filter), so some may themselves be
    /// illegal because they'd leave the mover's king in check. Treat
    /// them as "did you mean…" hints, not as guaranteed-legal moves.
    PieceCannotMakeMove {
        from: Coord,
        piece_symbol: String,
        piece_color: Color,
        attempted: MoveType,
        candidate_alternatives: Vec<MoveType>,
    },
    /// The move is geometrically valid but applying it would leave the
    /// mover's own king in check (illegal pin / discovered check / king
    /// walking into attack).
    WouldLeaveKingInCheck {
        from: Coord,
        piece_symbol: String,
        piece_color: Color,
        attempted: MoveType,
    },
    /// `make_move_unchecked` returned `Err` after `validate_move` already
    /// accepted the move. In practice this is unreachable from a normal
    /// `make_move` call — `validate_move` runs the same apply path on a
    /// clone first, so any error would have surfaced there. The variant
    /// exists for defence-in-depth and to keep the `Result` chain honest;
    /// if you ever observe it in the wild, treat it as an engine bug and
    /// include the `reason` field in the bug report.
    ApplyFailed {
        from: Coord,
        attempted: MoveType,
        reason: String,
    },
}

impl MoveError {
    /// Human-readable message — what the API used to return as a bare
    /// string. Always derivable from the structured fields, so clients
    /// that want richer rendering can ignore this and consume the enum
    /// directly. Uses `Display` rather than `Debug` formatting so it's
    /// safe to surface verbatim in a UI alert.
    pub fn message(&self) -> String {
        match self {
            MoveError::NoSourceSquare { from } => format!(
                "Source square {from} is out of bounds."
            ),
            MoveError::NoPieceAtSource { from } => format!(
                "Source square {from} is empty — there is no piece to move."
            ),
            MoveError::WrongTurn {
                from,
                piece_symbol,
                piece_color,
                side_to_move,
            } => format!(
                "It is {side_to_move}'s turn, but the piece at {from} ('{piece_symbol}') is {piece_color}."
            ),
            MoveError::PieceCannotMakeMove {
                from,
                piece_symbol,
                piece_color,
                attempted,
                candidate_alternatives,
            } => {
                let n = candidate_alternatives.len();
                let noun = if n == 1 { "candidate" } else { "candidates" };
                format!(
                    "The {piece_color} '{piece_symbol}' at {from} cannot {attempted}. \
                     {n} {noun} available from this square (some may leave the king in check)."
                )
            }
            MoveError::WouldLeaveKingInCheck {
                from,
                piece_symbol,
                piece_color,
                attempted,
            } => format!(
                "{piece_color} '{piece_symbol}' at {from} cannot {attempted}: \
                 that move would leave the {piece_color} king in check."
            ),
            MoveError::ApplyFailed {
                from,
                attempted,
                reason,
            } => format!(
                "Internal error applying '{attempted}' from {from} after validation passed — {reason}"
            ),
        }
    }
}

impl std::fmt::Display for MoveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message())
    }
}

impl std::error::Error for MoveError {}

#[derive(PartialEq, Debug, Clone)]
pub struct Board {
    pub grid: Vec<Vec<Square>>,
    pub flags: BoardFlags,
}

impl Board {
    /// Get an immutable reference to the square at `coord`, if within bounds.
    pub fn get_square_at(&self, coord: &Coord) -> Option<&Square> {
        self.grid
            .get(coord.rank as usize)
            .and_then(|row| row.get(coord.file as usize))
    }
    /// Get a mutable reference to the square at `coord`, if within bounds.
    pub fn get_square_mut(&mut self, coord: &Coord) -> Option<&mut Square> {
        self.grid
            .get_mut(coord.rank as usize)
            .and_then(|row| row.get_mut(coord.file as usize))
    }

    pub fn set_piece_at(&mut self, coord: &Coord, piece: PieceType) {
        if let Some(square) = self.get_square_mut(coord) {
            square.piece = Some(piece);
        }
    }

    pub fn square_is_empty(&self, coord: &Coord) -> bool {
        if let Some(square) = self.get_square_at(coord) {
            square.square_type.is_walkable() && square.piece.is_none()
        } else {
            false
        }
    }

    /// Get all possible moves for the piece at `from`.
    pub fn get_moves(&self, from: &Coord) -> Vec<GameMove> {
        let Some(square) = self.get_square_at(from) else {
            return vec![];
        };
        if square.conditions.contains(&SquareCondition::Brainrot)
            || square.conditions.contains(&SquareCondition::Frozen)
        {
            return vec![];
        }
        let Some(piece) = &square.piece else {
            return vec![];
        };

        let mut moves = piece.get_moves(self, from);

        // Square-driven additions: a piece standing on a Switch tile can
        // throw that switch. This is independent of the piece's own
        // movement, so we add it after the piece-level move generation.
        // `Piece::can_throw_switch()` lets specific pieces opt out (the
        // default is `true`).
        if matches!(square.square_type, SquareType::Switch { .. }) && piece.can_throw_switch() {
            moves.push(GameMove {
                from: from.clone(),
                move_type: MoveType::ThrowSwitch {
                    switch: from.clone(),
                },
            });
        }

        moves
    }

    /// Takes a from and to coordinate and returns true if the move is valid.
    /// Thin wrapper over `validate_move` — use that directly when you need
    /// to know *why* a move is invalid.
    pub fn is_valid_move(&self, game_move: &GameMove) -> bool {
        self.validate_move(game_move).is_ok()
    }

    /// Single-pass legality check that produces a structured `MoveError`
    /// instead of a bool. The order of checks is deliberate so the most
    /// specific reason wins:
    ///
    /// 1. Source square exists.
    /// 2. Source has a piece.
    /// 3. Piece is the side to move.
    /// 4. Move is in the piece's raw move set (`get_moves`).
    /// 5. Applying the move doesn't leave the mover's own king in check.
    pub fn validate_move(&self, game_move: &GameMove) -> Result<(), MoveError> {
        let Some(square) = self.get_square_at(&game_move.from) else {
            return Err(MoveError::NoSourceSquare {
                from: game_move.from.clone(),
            });
        };
        let Some(piece) = square.piece.as_ref() else {
            return Err(MoveError::NoPieceAtSource {
                from: game_move.from.clone(),
            });
        };
        let piece_color = piece.get_color();
        let piece_symbol = piece.symbol();
        let side_to_move = self.flags.side_to_move;
        if piece_color != side_to_move {
            return Err(MoveError::WrongTurn {
                from: game_move.from.clone(),
                piece_symbol,
                piece_color,
                side_to_move,
            });
        }

        let raw_moves = self.get_moves(&game_move.from);
        if !raw_moves.iter().any(|m| m == game_move) {
            return Err(MoveError::PieceCannotMakeMove {
                from: game_move.from.clone(),
                piece_symbol,
                piece_color,
                attempted: game_move.move_type.clone(),
                candidate_alternatives: raw_moves.into_iter().map(|m| m.move_type).collect(),
            });
        }

        let mut hypothetical = self.clone();
        match hypothetical.make_move_unchecked(game_move.clone()) {
            Ok(()) => {
                if hypothetical.is_in_check(piece_color) {
                    return Err(MoveError::WouldLeaveKingInCheck {
                        from: game_move.from.clone(),
                        piece_symbol,
                        piece_color,
                        attempted: game_move.move_type.clone(),
                    });
                }
                Ok(())
            }
            Err(reason) => Err(MoveError::ApplyFailed {
                from: game_move.from.clone(),
                attempted: game_move.move_type.clone(),
                reason,
            }),
        }
    }

    /// Is `target` attacked by any piece of `attacker`? Used for check
    /// detection (`target = king square`, `attacker = enemy color`) and
    /// castle-path safety.
    ///
    /// Implementation is O(N·M) — for each piece of `attacker`, ask the
    /// piece what squares it threatens, then look for `target` in the set.
    /// At 8×8 this is fine; revisit if board sizes grow.
    pub fn is_attacked_by(&self, target: &Coord, attacker: Color) -> bool {
        for (coord, piece) in self.all_pieces() {
            if piece.get_color() != attacker {
                continue;
            }
            for c in piece.attacks(self, &coord) {
                if &c == target {
                    return true;
                }
            }
        }
        false
    }

    /// Locate the king of `color`, if one exists on the board. Returns
    /// `None` for setups where the king is missing (tests, partial boards) —
    /// callers should treat the absence as "not in check" rather than panic.
    ///
    /// Descends into `Bus` passengers: a passenger king's effective square
    /// is the Bus's square, since capturing the Bus also captures every
    /// piece inside. Without this, `is_in_check(color)` silently returns
    /// `false` for a king parked inside a Bus and the game can never end.
    pub fn find_king(&self, color: Color) -> Option<Coord> {
        for (coord, piece) in self.all_pieces() {
            if king_of_color(&piece, color) {
                return Some(coord);
            }
            if let PieceType::Bus(bus) = &piece {
                if bus.pieces.iter().any(|p| king_of_color(p, color)) {
                    return Some(coord);
                }
            }
        }
        None
    }

    /// Is the `color` king currently under attack? Defensively returns
    /// `false` when no king of that colour exists on the board.
    pub fn is_in_check(&self, color: Color) -> bool {
        match self.find_king(color) {
            Some(king_coord) => self.is_attacked_by(&king_coord, color.opposite()),
            None => false,
        }
    }

    /// Subset of `get_moves(from)` after dropping any move that would leave
    /// the moving side's own king in check (or fail apply for any other
    /// reason). Implements pin/discovered-check filtering by clone-and-try
    /// per candidate move; correct but not fast — see plan 02's notes.
    pub fn legal_moves(&self, from: &Coord) -> Vec<GameMove> {
        let raw = self.get_moves(from);
        let moving_color = match self.get_square_at(from).and_then(|s| s.piece.as_ref()) {
            Some(p) => p.get_color(),
            None => return Vec::new(),
        };
        raw.into_iter()
            .filter(|m| {
                let mut hypothetical = self.clone();
                match hypothetical.make_move_unchecked(m.clone()) {
                    Ok(()) => !hypothetical.is_in_check(moving_color),
                    Err(_) => false,
                }
            })
            .collect()
    }

    /// Overall status from the perspective of `side_to_move`. `BrainrotWin`
    /// is intentionally absent — plan 04 will fold that in once the
    /// distinguish-stalemate-from-brainrot heuristic lands.
    pub fn status(&self) -> GameStatus {
        let to_move = self.flags.side_to_move;
        let any_legal = self
            .all_pieces()
            .iter()
            .filter(|(_, p)| p.get_color() == to_move)
            .any(|(coord, _)| !self.legal_moves(coord).is_empty());

        if any_legal {
            if self.is_in_check(to_move) {
                return GameStatus::Check {
                    side_to_move: to_move,
                };
            }
            return GameStatus::Ongoing;
        }
        if self.is_in_check(to_move) {
            GameStatus::Checkmate {
                winner: to_move.opposite(),
            }
        } else {
            GameStatus::Stalemate
        }
    }

    pub fn all_pieces(&self) -> Vec<(Coord, PieceType)> {
        let mut out = Vec::new();

        for (rank, row) in self.grid.iter().enumerate() {
            for (file, square) in row.iter().enumerate() {
                if let Some(piece) = &square.piece {
                    out.push((
                        Coord {
                            file: file as u8,
                            rank: rank as u8,
                        },
                        piece.clone(),
                    ));
                }
            }
        }

        out
    }

    /// Returns true if (file, rank) is inside the board grid.
    pub fn in_bounds(&self, file: isize, rank: isize) -> bool {
        rank >= 0
            && file >= 0
            && (rank as usize) < self.grid.len()
            && (file as usize) < self.grid[rank as usize].len()
    }

    /// Number of ranks (rows) in the board. `height() - 1` is the bottom rank.
    pub fn height(&self) -> u8 {
        self.grid.len() as u8
    }

    /// Number of files (columns) on the board, measured by the first row.
    /// Rows are expected to be uniform in length; FEN parsing enforces this.
    pub fn width(&self) -> u8 {
        self.grid.first().map(|row| row.len() as u8).unwrap_or(0)
    }

    /// Algebraic notation ("e4") for a Coord, using this board's height to
    /// invert the rank. For 8-tall boards this is the same as the chess
    /// convention; for taller boards the rank counts up from 1 at the
    /// bottom-most row.
    pub fn format_coord(&self, c: &Coord) -> String {
        let file_letter = (b'a' + c.file) as char;
        let algebraic_rank = self.height().saturating_sub(c.rank);
        format!("{file_letter}{algebraic_rank}")
    }
}
