use crate::{
    board::{Board, Coord, GameMove},
    movement::glider::{STRAIGHT_DIRS, generate_glider_moves},
    pieces::{Color, Piece},
};

#[derive(Clone, PartialEq, Debug)]
pub struct Rook {
    pub color: Color,
}
impl Piece for Rook {
    fn name(&self) -> &str {
        "Rook"
    }
    fn color(&self) -> Color {
        self.color
    }
    fn set_color(&mut self, color: Color) {
        self.color = color;
    }
    fn initial_moves(&self, board: &Board, from: &Coord) -> Vec<GameMove> {
        generate_glider_moves(board, from, &STRAIGHT_DIRS, usize::MAX)
    }
    fn symbol(&self) -> String {
        match self.color {
            Color::White => 'R'.to_string(),
            Color::Black => 'r'.to_string(),
        }
    }

    fn clone_box(&self) -> Box<dyn Piece> {
        Box::new(self.clone())
    }

    fn post_move_effects(
        &mut self,
        _board_before: &Board,
        board_after: &mut Board,
        game_move: &GameMove,
    ) {
        // A rook leaving its starting corner permanently disables castling on
        // that side. Rooks that promoted onto the board started life on a
        // non-corner square, so the starting-square test correctly leaves the
        // flags alone for them.
        let from = &game_move.from;
        match self.color {
            Color::White => {
                if from.rank == 7 {
                    if from.file == 0 {
                        board_after.flags.white_can_castle_queenside = false;
                    } else if from.file == 7 {
                        board_after.flags.white_can_castle_kingside = false;
                    }
                }
            }
            Color::Black => {
                if from.rank == 0 {
                    if from.file == 0 {
                        board_after.flags.black_can_castle_queenside = false;
                    } else if from.file == 7 {
                        board_after.flags.black_can_castle_kingside = false;
                    }
                }
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
