use std::collections::HashSet;

use crate::board::square::{Square, SquareType, fen_to_square, square_to_fen};
use crate::board::{Board, Coord, Direction, File, Rank, Sq};

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
    from: Coord,
    directions: &[Direction],
    max_range: usize,
) -> Vec<Coord> {
    // Board size assumptions (standard 8x8). If your board may vary,
    // compute from board.grid dimensions instead.
    let files = board.grid.get(0).map(|r| r.len()).unwrap_or(8) as isize;
    let ranks = board.grid.len() as isize;

    let mut moves = Vec::new();

    let start_file = from.file as isize;
    let start_rank = from.rank as isize;

    for &(dx, dy) in directions {
        let mut step = 1isize;
        loop {
            let f = start_file + dx * step;
            let r = start_rank + dy * step;

            if f < 0 || r < 0 || f >= files || r >= ranks {
                break;
            }

            let coord = Coord {
                file: f as File,
                rank: r as Rank,
            };

            moves.push(coord.clone());

            // check for blockers
            if let Some(sq) = board.get_square_at(coord) {
                if let Some(piece) = &sq.piece
                    && piece.blocks_path()
                {
                    break; // blocked by piece
                }
            }

            step += 1;
            if step > max_range as isize {
                break;
            }
        }
    }

    moves
}
