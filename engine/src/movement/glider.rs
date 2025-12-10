use std::collections::HashSet;

use crate::board::{Board, Coord, Direction, File, GameMove, MoveType, Rank, Sq};
use crate::board::{
    fen::{fen_to_square, square_to_fen},
    square::{Square, SquareType},
};

/// Directions for glider movement
pub const STRAIGHT_DIRS: &[Direction] = &[(1, 0), (-1, 0), (0, 1), (0, -1)];
pub const DIAGONAL_DIRS: &[Direction] = &[(1, 1), (1, -1), (-1, 1), (-1, -1)];
pub const OMNI_DIRS: &[Direction] = &[
    (1, 0),
    (-1, 0),
    (0, 1),
    (0, -1),
    (1, 1),
    (1, -1),
    (-1, 1),
    (-1, -1),
];

/// Generate all reachable coordinates from `from` by moving along each
/// vector in `directions`, up to `max_range` squares. This function
/// consider blockers and captures.
///
/// - board: used to sanity-check grid shape (assumes 8x8 default), check if the path is blocked
/// - from: starting Coord (file, rank)
/// - directions: slice of (dx, dy) where dx/dy are signed offsets (file, rank)
/// - max_range: maximum number of squares to step in a direction; use
///              usize::MAX to indicate "unbounded" (slide to edge)
pub fn generate_glider_moves(
    board: &Board,
    from: &Coord,
    directions: &[Direction],
    max_range: usize,
) -> Vec<GameMove> {
    let files = board.grid[0].len() as isize;
    let ranks = board.grid.len() as isize;

    let mut moves = Vec::new();

    let start_file = from.file as isize;
    let start_rank = from.rank as isize;

    for &(dx, dy) in directions {
        let unbounded = max_range == usize::MAX;

        let mut step = 1usize;
        loop {
            let f = start_file + dx * step as isize;
            let r = start_rank + dy * step as isize;

            // out of bounds
            if f < 0 || r < 0 || f >= files || r >= ranks {
                break;
            }

            let coord = Coord {
                file: f as File,
                rank: r as Rank,
            };

            moves.push(GameMove {
                from: from.clone(),
                move_type: MoveType::MoveTo(coord.clone()),
            });

            // stop if encountering a piece that blocks
            if let Some(sq) = board.get_square_at(&coord) {
                if let Some(piece) = &sq.piece
                    && piece.blocks_path()
                {
                    break;
                }
            }

            // stop if bounded and reached max range
            if !unbounded && step >= max_range {
                break;
            }

            step += 1;
        }
    }

    moves
}
