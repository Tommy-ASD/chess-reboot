use crate::{
    board::{Board, CastleSide, Coord, GameMove, MoveType},
    movement::glider::{OMNI_DIRS, generate_glider_moves},
    pieces::{Color, Piece, piecetype::PieceType},
};

#[derive(Clone, PartialEq, Debug)]
pub struct King {
    pub color: Color,
}
impl King {
    fn back_rank(&self) -> u8 {
        match self.color {
            Color::White => 7,
            Color::Black => 0,
        }
    }

    /// Generate castle candidates that satisfy every standard-chess
    /// precondition: the relevant castle right, king on its starting square,
    /// rook of the right colour on its starting square, empty path, and the
    /// king's start + intermediate + destination squares not under attack.
    fn castle_moves(&self, board: &Board, from: &Coord) -> Vec<GameMove> {
        let mut moves = Vec::new();
        let back_rank = self.back_rank();
        if from.file != 4 || from.rank != back_rank {
            return moves;
        }
        if board.is_in_check(self.color) {
            return moves;
        }
        let opp = self.color.opposite();

        let (can_ks, can_qs) = match self.color {
            Color::White => (
                board.flags.white_can_castle_kingside,
                board.flags.white_can_castle_queenside,
            ),
            Color::Black => (
                board.flags.black_can_castle_kingside,
                board.flags.black_can_castle_queenside,
            ),
        };

        let rook_is_friendly =
            |board: &Board, sq: &Coord| -> bool {
                matches!(
                    board.get_square_at(sq).and_then(|s| s.piece.as_ref()),
                    Some(PieceType::Rook(r)) if r.color == self.color
                )
            };
        let empty = |board: &Board, sq: &Coord| -> bool {
            matches!(board.get_square_at(sq), Some(s) if s.piece.is_none())
        };

        if can_ks {
            let p5 = Coord {
                file: 5,
                rank: back_rank,
            };
            let p6 = Coord {
                file: 6,
                rank: back_rank,
            };
            let p7 = Coord {
                file: 7,
                rank: back_rank,
            };
            if empty(board, &p5)
                && empty(board, &p6)
                && rook_is_friendly(board, &p7)
                && !board.is_attacked_by(&p5, opp)
                && !board.is_attacked_by(&p6, opp)
            {
                moves.push(GameMove {
                    from: from.clone(),
                    move_type: MoveType::Castle {
                        side: CastleSide::Kingside,
                    },
                });
            }
        }

        if can_qs {
            let p1 = Coord {
                file: 1,
                rank: back_rank,
            };
            let p2 = Coord {
                file: 2,
                rank: back_rank,
            };
            let p3 = Coord {
                file: 3,
                rank: back_rank,
            };
            let p0 = Coord {
                file: 0,
                rank: back_rank,
            };
            if empty(board, &p1)
                && empty(board, &p2)
                && empty(board, &p3)
                && rook_is_friendly(board, &p0)
                && !board.is_attacked_by(&p3, opp)
                && !board.is_attacked_by(&p2, opp)
            {
                moves.push(GameMove {
                    from: from.clone(),
                    move_type: MoveType::Castle {
                        side: CastleSide::Queenside,
                    },
                });
            }
        }

        moves
    }
}
impl Piece for King {
    fn name(&self) -> &str {
        "King"
    }
    fn color(&self) -> Color {
        self.color
    }
    fn set_color(&mut self, color: Color) {
        self.color = color;
    }
    fn initial_moves(&self, board: &Board, from: &Coord) -> Vec<GameMove> {
        let mut moves = generate_glider_moves(board, from, &OMNI_DIRS, 1);
        moves.extend(self.castle_moves(board, from));
        moves
    }
    /// The king attacks only the 8 squares adjacent to it. Castle moves are
    /// deliberately excluded — `Board::is_attacked_by` calls into `attacks`,
    /// and `castle_moves` itself calls `is_attacked_by` for path-safety
    /// checks. Including castle-target squares in `attacks` would cause
    /// `is_in_check` to recurse infinitely the moment a king sits on its
    /// starting square.
    fn attacks(&self, board: &Board, from: &Coord) -> Vec<Coord> {
        let mut out = Vec::new();
        for &(df, dr) in OMNI_DIRS {
            let nf = from.file as isize + df;
            let nr = from.rank as isize + dr;
            if board.in_bounds(nf, nr) {
                out.push(Coord {
                    file: nf as u8,
                    rank: nr as u8,
                });
            }
        }
        out
    }
    fn symbol(&self) -> String {
        match self.color {
            Color::White => 'K'.to_string(),
            Color::Black => 'k'.to_string(),
        }
    }

    fn clone_box(&self) -> Box<dyn Piece> {
        Box::new(self.clone())
    }

    fn post_move_effects(
        &mut self,
        _board_before: &Board,
        board_after: &mut Board,
        _game_move: &GameMove,
    ) {
        // Once the king has moved, castling on either side is no longer
        // available for that colour — covers ordinary moves and castles.
        match self.color {
            Color::White => {
                board_after.flags.white_can_castle_kingside = false;
                board_after.flags.white_can_castle_queenside = false;
            }
            Color::Black => {
                board_after.flags.black_can_castle_kingside = false;
                board_after.flags.black_can_castle_queenside = false;
            }
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
