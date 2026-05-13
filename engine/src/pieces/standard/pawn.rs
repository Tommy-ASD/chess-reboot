use tracing::trace;

use crate::{
    board::{Board, Coord, GameMove, MoveType, PromotionTarget},
    pieces::{Color, Piece},
};

#[derive(Clone, PartialEq, Debug)]
pub struct Pawn {
    pub color: Color,
}

impl Pawn {
    /// Furthest rank from the home side — promotion fires when a pawn
    /// lands here. Black promotes at the bottom (`height - 1`); white
    /// promotes at the top (rank 0). Height comes from the board so this
    /// is correct for boards taller or shorter than 8.
    fn promotion_rank(&self, board: &Board) -> u8 {
        match self.color {
            Color::White => 0,
            Color::Black => board.height().saturating_sub(1),
        }
    }

    /// Rank a pawn occupies before its first move — the row eligible for
    /// the optional double-push. White starts one row up from the back
    /// (`height - 2`); black starts on rank 1 (one row down from rank 0).
    fn starting_rank(&self, board: &Board) -> u8 {
        match self.color {
            Color::White => board.height().saturating_sub(2),
            Color::Black => 1,
        }
    }

    /// Push either a `MoveTo` or the four `Promotion` variants — same target,
    /// four piece choices — depending on whether the destination is the
    /// promotion rank.
    fn push_advance_or_promotion(
        &self,
        target: Coord,
        from: &Coord,
        board: &Board,
        out: &mut Vec<GameMove>,
    ) {
        if target.rank == self.promotion_rank(board) {
            for into in [
                PromotionTarget::Queen,
                PromotionTarget::Rook,
                PromotionTarget::Bishop,
                PromotionTarget::Knight,
            ] {
                out.push(GameMove {
                    from: from.clone(),
                    move_type: MoveType::Promotion {
                        target: target.clone(),
                        into,
                    },
                });
            }
        } else {
            out.push(GameMove {
                from: from.clone(),
                move_type: MoveType::MoveTo(target),
            });
        }
    }
}

impl Piece for Pawn {
    fn name(&self) -> &str {
        "Pawn"
    }
    fn color(&self) -> Color {
        self.color
    }
    fn set_color(&mut self, color: Color) {
        self.color = color;
    }
    fn initial_moves(&self, board: &Board, from: &Coord) -> Vec<GameMove> {
        let mut moves = Vec::new();
        let direction: isize = match self.color {
            Color::White => -1,
            Color::Black => 1,
        };

        trace!(direction, "pawn move direction");

        // One square forward
        let new_rank = from.rank as isize + direction;
        if board.in_bounds(from.file as isize, new_rank) {
            trace!("one-square forward in bounds");
            let forward_coord = Coord {
                file: from.file,
                rank: new_rank as u8,
            };
            if let Some(square) = board.get_square_at(&forward_coord) {
                trace!(?square, ?forward_coord, "forward square");
                if square.piece.is_none() {
                    trace!("forward square empty, pushing move");
                    self.push_advance_or_promotion(forward_coord.clone(), from, board, &mut moves);

                    // Two squares forward from starting position. Double push
                    // never coincides with a promotion rank, so no promotion
                    // handling is needed here.
                    let starting_rank = self.starting_rank(board);
                    if from.rank == starting_rank {
                        let two_forward_coord = Coord {
                            file: from.file,
                            rank: (new_rank + direction) as u8,
                        };
                        if let Some(two_square) = board.get_square_at(&two_forward_coord) {
                            if two_square.piece.is_none() {
                                let game_move = GameMove {
                                    from: from.clone(),
                                    move_type: MoveType::MoveTo(two_forward_coord.clone()),
                                };
                                moves.push(game_move);
                            }
                        }
                    }
                }
            }
        }

        trace!(?moves, "after forward push");

        // Captures (regular + en passant)
        for df in &[-1, 1] {
            let new_file = from.file as isize + df;
            if !board.in_bounds(new_file, new_rank) {
                continue;
            }
            let capture_coord = Coord {
                file: new_file as u8,
                rank: new_rank as u8,
            };
            if let Some(square) = board.get_square_at(&capture_coord) {
                // Ordinary diagonal capture: enemy piece sitting on the square.
                if let Some(piece) = &square.piece {
                    if piece.get_color() != self.color {
                        self.push_advance_or_promotion(
                            capture_coord.clone(),
                            from,
                            board,
                            &mut moves,
                        );
                    }
                } else if let Some(ep) = &board.flags.en_passant_target {
                    // En passant: the destination matches the recorded ep
                    // target. The captured pawn sits on (target.file,
                    // from.rank) — same file as our diagonal target, same
                    // rank as we currently are on.
                    if ep == &capture_coord {
                        let captured = Coord {
                            file: capture_coord.file,
                            rank: from.rank,
                        };
                        moves.push(GameMove {
                            from: from.clone(),
                            move_type: MoveType::EnPassant {
                                target: capture_coord.clone(),
                                captured,
                            },
                        });
                    }
                }
            }
        }

        trace!(?moves, "after captures");

        moves
    }
    fn symbol(&self) -> String {
        match self.color {
            Color::White => 'P'.to_string(),
            Color::Black => 'p'.to_string(),
        }
    }

    fn clone_box(&self) -> Box<dyn Piece> {
        Box::new(self.clone())
    }

    /// A pawn attacks only its two forward diagonals — not the forward push.
    /// The diagonals are threatened regardless of whether they're currently
    /// occupied (king-safety needs the "hypothetical capture" view).
    fn attacks(&self, board: &Board, from: &Coord) -> Vec<Coord> {
        let direction: isize = match self.color {
            Color::White => -1,
            Color::Black => 1,
        };
        let new_rank = from.rank as isize + direction;
        let mut out = Vec::new();
        for df in &[-1isize, 1] {
            let nf = from.file as isize + df;
            if board.in_bounds(nf, new_rank) {
                out.push(Coord {
                    file: nf as u8,
                    rank: new_rank as u8,
                });
            }
        }
        out
    }

    fn post_move_effects(
        &mut self,
        _board_before: &Board,
        board_after: &mut Board,
        game_move: &GameMove,
    ) {
        match &game_move.move_type {
            MoveType::MoveTo(target) => {
                // Double push: set en-passant target to the square between
                // from and target. handle_post_move_effects resets
                // en_passant_target to None *before* calling this, so any
                // non-double-push correctly leaves it cleared.
                let rank_diff =
                    (target.rank as i32 - game_move.from.rank as i32).abs();
                if rank_diff == 2 {
                    let ep_rank =
                        (target.rank as u16 + game_move.from.rank as u16) / 2;
                    board_after.flags.en_passant_target = Some(Coord {
                        file: target.file,
                        rank: ep_rank as u8,
                    });
                }
            }
            _ => {}
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
