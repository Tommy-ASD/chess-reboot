/// Let's add The Monkey
/// Has a range of 1 (moves like a king), can't take
/// If there's a piece directly next to it, it can jump over
/// If there's a piece next to the new location, it can jump over that, as well
/// If there's a piece where the monkey ends up, it takes
///
/// Example moves (M = Monkey, P = generic piece, X = possible move, . = empty):
/// X M X . . . . .
/// X P X . . . . .
/// . X P X . . . .
/// . . . . . . . .
///
/// This should be done recursively, to allow for multiple jumps in a single move
use crate::{
    board::{Board, Coord, GameMove},
    pieces::{Color, Piece},
};

#[derive(Clone, PartialEq, Debug)]
pub struct Monkey {
    pub color: Color,
}
impl Piece for Monkey {
    fn name(&self) -> &str {
        "Monkey"
    }
    fn color(&self) -> Color {
        self.color
    }
    fn initial_moves(&self, board: &Board, from: &Coord) -> Vec<GameMove> {
        let mut moves = Vec::new();
        let directions: [(isize, isize); 8] = [
            (1, 0),
            (1, 1),
            (0, 1),
            (-1, 1),
            (-1, 0),
            (-1, -1),
            (0, -1),
            (1, -1),
        ];

        // first, handle normal one-square moves
        for (df, dr) in &directions {
            let new_file = from.file as isize + df;
            let new_rank = from.rank as isize + dr;
            if new_file >= 0 && new_file < 8 && new_rank >= 0 && new_rank < 8 {
                let coord = Coord {
                    file: new_file as u8,
                    rank: new_rank as u8,
                };
                if board.square_is_empty(&coord) {
                    let game_move = GameMove {
                        from: from.clone(),
                        to: coord.clone(),
                    };
                    moves.push(game_move);
                }
            }
        }
        // now, handle jump moves
        let mut visited = Vec::new();
        self.find_jump_moves(board, from, &mut visited, &mut moves);
        moves
    }
    fn symbol(&self) -> String {
        match self.color {
            Color::White => 'M'.to_string(),
            Color::Black => 'm'.to_string(),
        }
    }

    fn clone_box(&self) -> Box<dyn Piece> {
        Box::new(self.clone())
    }
}

impl Monkey {
    /// Recursive function to find jump moves
    /// `current_coord` is the current position of the monkey during the jump sequence
    /// `visited` keeps track of coordinates already jumped to in this sequence
    /// `moves` accumulates the valid jump moves
    fn find_jump_moves(
        &self,
        board: &Board,
        current_coord: &Coord,
        visited: &mut Vec<Coord>,
        moves: &mut Vec<GameMove>,
    ) {
        let directions: [(isize, isize); 8] = [
            (1, 0),
            (1, 1),
            (0, 1),
            (-1, 1),
            (-1, 0),
            (-1, -1),
            (0, -1),
            (1, -1),
        ];

        for (df, dr) in &directions {
            let adj_file = current_coord.file as isize + df;
            let adj_rank = current_coord.rank as isize + dr;
            let jump_file = adj_file + df;
            let jump_rank = adj_rank + dr;

            if adj_file >= 0
                && adj_file < 8
                && adj_rank >= 0
                && adj_rank < 8
                && jump_file >= 0
                && jump_file < 8
                && jump_rank >= 0
                && jump_rank < 8
            {
                let adj_coord = Coord {
                    file: adj_file as u8,
                    rank: adj_rank as u8,
                };
                let jump_coord = Coord {
                    file: jump_file as u8,
                    rank: jump_rank as u8,
                };

                if visited.contains(&jump_coord) {
                    continue;
                };

                if let Some(adj_square) = board.get_square_at(&adj_coord) {
                    if adj_square.piece.is_some() {
                        if let Some(jump_square) = board.get_square_at(&jump_coord) {
                            if jump_square.piece.is_none() {
                                let game_move = GameMove {
                                    from: current_coord.clone(),
                                    to: jump_coord.clone(),
                                };
                                moves.push(game_move);
                                visited.push(jump_coord.clone());
                                self.find_jump_moves(board, &jump_coord, visited, moves);
                            }
                            if jump_square.has_piece_of_color(self.color.opposite()) {
                                let game_move = GameMove {
                                    from: current_coord.clone(),
                                    to: jump_coord.clone(),
                                };
                                moves.push(game_move);
                                visited.push(jump_coord.clone());
                            }
                        }
                    }
                }
            }
        }
    }
}
