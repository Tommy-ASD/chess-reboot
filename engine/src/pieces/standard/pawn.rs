use crate::{
    board::{Board, Coord, GameMove, MoveType},
    pieces::{Color, Piece},
};

#[derive(Clone, PartialEq, Debug)]
pub struct Pawn {
    pub color: Color,
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

        dbg!(&direction);

        // One square forward
        let new_rank = from.rank as isize + direction;
        if new_rank >= 0 && new_rank < 8 {
            dbg!();
            let forward_coord = Coord {
                file: from.file,
                rank: new_rank as u8,
            };
            if let Some(square) = board.get_square_at(&forward_coord) {
                dbg!(&square, &forward_coord);
                if square.piece.is_none() {
                    dbg!();
                    let game_move = GameMove {
                        from: from.clone(),
                        move_type: MoveType::MoveTo(forward_coord.clone()),
                    };
                    moves.push(game_move);

                    // Two squares forward from starting position
                    let starting_rank = match self.color {
                        Color::White => 6,
                        Color::Black => 1,
                    };
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

        dbg!(&moves);

        // Captures
        for df in &[-1, 1] {
            let new_file = from.file as isize + df;
            if new_file >= 0 && new_file < 8 && new_rank >= 0 && new_rank < 8 {
                let capture_coord = Coord {
                    file: new_file as u8,
                    rank: new_rank as u8,
                };
                if let Some(square) = board.get_square_at(&capture_coord) {
                    if let Some(piece) = &square.piece {
                        if piece.get_color() != self.color {
                            let game_move = GameMove {
                                from: from.clone(),
                                move_type: MoveType::MoveTo(capture_coord.clone()),
                            };
                            moves.push(game_move);
                        }
                    }
                }
            }
        }

        dbg!(&moves);

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

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
