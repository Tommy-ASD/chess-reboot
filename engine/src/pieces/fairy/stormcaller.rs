//! Plan 13 commit 4 — the Stormcaller.
//!
//! A king-step piece that cannot capture. Its job is `PlaceTornado`:
//! spend the turn stamping a `Tornado` condition on an in-bounds
//! adjacent square (king-radius range in v1). The countdown lives on
//! the square condition, so the Stormcaller carries no per-piece
//! state beyond colour — its symbol is a bare letter, like a standard
//! piece.
//!
//! The dedicated placer is the v1 surface for tornadoes in live play
//! (commits 1-3 made FEN-authored tornadoes work; this lets a game
//! produce them). More placers can follow later without touching the
//! condition or the compulsion filter.

use crate::{
    board::{Board, Coord, GameMove, MoveType},
    pieces::{Color, Piece, piecetype::PieceType},
};

/// `remaining` a freshly-placed tornado is stamped with.
///
/// NOTE: `TornadoTickHandler` (env-reaction, PostTick) also runs at
/// the end of the placing move, so the first decrement lands on the
/// placing turn itself — effective compulsion is felt for roughly
/// `TORNADO_DURATION - 1` opponent-facing turns. v1 ships this
/// constant; a duration cap / tuning is plan 13 open question 1.
pub const TORNADO_DURATION: u8 = 3;

const KING_STEPS: [(isize, isize); 8] = [
    (1, 0),
    (1, 1),
    (0, 1),
    (-1, 1),
    (-1, 0),
    (-1, -1),
    (0, -1),
    (1, -1),
];

#[derive(Clone, PartialEq, Debug, serde::Serialize, serde::Deserialize)]
pub struct Stormcaller {
    pub color: Color,
}

impl Stormcaller {
    pub fn new(color: Color) -> Self {
        Stormcaller { color }
    }

    /// Stateless beyond colour → a bare-letter symbol, parsed like a
    /// standard piece (no `(...)` payload).
    pub fn from_symbol(symbol: &str) -> Option<PieceType> {
        let first = symbol.chars().next()?;
        let color = if first.is_lowercase() {
            Color::Black
        } else {
            Color::White
        };
        Some(PieceType::Stormcaller(Stormcaller { color }))
    }
}

impl Piece for Stormcaller {
    fn name(&self) -> &str {
        "Stormcaller"
    }
    fn color(&self) -> Color {
        self.color
    }
    fn set_color(&mut self, color: Color) {
        self.color = color;
    }
    fn initial_moves(&self, board: &Board, from: &Coord) -> Vec<GameMove> {
        // Plan 09 convention: Neutral non-train pieces yield no moves.
        if self.color == Color::Neutral {
            return Vec::new();
        }
        let mut moves = Vec::new();
        for (df, dr) in &KING_STEPS {
            let nf = from.file as isize + df;
            let nr = from.rank as isize + dr;
            if !board.in_bounds(nf, nr) {
                continue;
            }
            let coord = Coord {
                file: nf as u8,
                rank: nr as u8,
            };
            let Some(sq) = board.get_square_at(&coord) else {
                continue;
            };
            // Reposition: step onto an EMPTY, walkable adjacent
            // square. Stormcaller cannot capture (a placer, not a
            // fighter) — occupied squares are not movement targets.
            if sq.square_type.is_walkable() && sq.piece.is_none() {
                moves.push(GameMove {
                    from: from.clone(),
                    move_type: MoveType::MoveTo(coord.clone()),
                });
            }
            // Place: stamp a tornado on any in-bounds adjacent square,
            // occupied or not — placing onto an enemy piece (trapping
            // it) is the central play. Walkability is irrelevant: the
            // condition rides the square, no piece relocates here.
            moves.push(GameMove {
                from: from.clone(),
                move_type: MoveType::PlaceTornado { target: coord },
            });
        }
        moves
    }
    fn symbol(&self) -> String {
        match self.color {
            Color::White => "W".to_string(),
            Color::Black => "w".to_string(),
            // Mirror Skibidi: a Neutral instance still renders the
            // uppercase letter.
            Color::Neutral => "W".to_string(),
        }
    }
    fn clone_box(&self) -> Box<dyn Piece> {
        Box::new(self.clone())
    }
    /// Stormcaller never captures, so it threatens nothing. Overrides
    /// the default `attacks` (which would project king-step move
    /// squares as phantom threats and let the Stormcaller "give
    /// check").
    fn attacks(&self, _board: &Board, _from: &Coord) -> Vec<Coord> {
        Vec::new()
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::square::{Square, SquareCondition};
    use crate::board::{BoardFlags, TrainTickRate};

    fn board8() -> Board {
        let grid = (0..8)
            .map(|_| (0..8).map(|_| Square::new()).collect())
            .collect();
        Board {
            grid,
            flags: BoardFlags {
                side_to_move: Color::White,
                white_can_castle_kingside: false,
                white_can_castle_queenside: false,
                black_can_castle_kingside: false,
                black_can_castle_queenside: false,
                en_passant_target: None,
                train_tick_rate: TrainTickRate::EveryFullTurn,
                ply_count: 0,
                last_move: None,
            },
        }
    }

    fn cc(file: u8, rank: u8) -> Coord {
        Coord { file, rank }
    }

    /// On an open board a Stormcaller emits a step + a place for each
    /// of its 8 king-neighbours: 16 candidates.
    #[test]
    fn emits_step_and_place_for_each_neighbour() {
        let b = board8();
        let s = Stormcaller::new(Color::White);
        let moves = s.initial_moves(&b, &cc(4, 4));
        let steps = moves
            .iter()
            .filter(|m| matches!(m.move_type, MoveType::MoveTo(_)))
            .count();
        let places = moves
            .iter()
            .filter(|m| matches!(m.move_type, MoveType::PlaceTornado { .. }))
            .count();
        assert_eq!(steps, 8, "8 empty walkable neighbours → 8 steps");
        assert_eq!(places, 8, "8 in-bounds neighbours → 8 place actions");
    }

    /// Stormcaller cannot capture: an enemy-occupied neighbour yields
    /// no step there, but the place action onto it is still offered
    /// (trapping it is the point).
    #[test]
    fn cannot_step_onto_enemy_but_can_place_on_it() {
        let mut b = board8();
        b.grid[4][5] = Square::new().set_piece(PieceType::new_rook(Color::Black));
        let s = Stormcaller::new(Color::White);
        let moves = s.initial_moves(&b, &cc(4, 4));
        assert!(
            !moves
                .iter()
                .any(|m| m.move_type == MoveType::MoveTo(cc(5, 4))),
            "must not step onto (capture) the enemy"
        );
        assert!(
            moves.iter().any(|m| m.move_type
                == MoveType::PlaceTornado { target: cc(5, 4) }),
            "must still offer placing the tornado onto the enemy"
        );
    }

    /// Neutral non-train pieces yield no moves (plan 09 convention).
    #[test]
    fn neutral_has_no_moves() {
        let b = board8();
        let s = Stormcaller::new(Color::Neutral);
        assert!(s.initial_moves(&b, &cc(4, 4)).is_empty());
    }

    /// Symbol round-trips through `from_symbol`; the piece is
    /// stateless so the symbol is a bare letter.
    #[test]
    fn symbol_roundtrips() {
        assert_eq!(Stormcaller::new(Color::White).symbol(), "W");
        assert_eq!(Stormcaller::new(Color::Black).symbol(), "w");
        assert!(matches!(
            Stormcaller::from_symbol("W"),
            Some(PieceType::Stormcaller(s)) if s.color == Color::White
        ));
        assert!(matches!(
            Stormcaller::from_symbol("w"),
            Some(PieceType::Stormcaller(s)) if s.color == Color::Black
        ));
    }

    /// End-to-end: a PlaceTornado move through `make_move` stamps the
    /// condition and the same-turn env tick decrements it once
    /// (`TORNADO_DURATION` → `TORNADO_DURATION - 1`). The placer does
    /// not move; the turn flips.
    #[test]
    fn place_tornado_via_make_move() {
        let mut b = board8();
        b.grid[4][4] = Square::new().set_piece(PieceType::Stormcaller(
            Stormcaller::new(Color::White),
        ));
        b.make_move(GameMove {
            from: cc(4, 4),
            move_type: MoveType::PlaceTornado { target: cc(4, 3) },
        })
        .expect("PlaceTornado onto an in-range empty square is legal");

        assert_eq!(
            b.grid[3][4].conditions,
            vec![SquareCondition::Tornado {
                remaining: TORNADO_DURATION - 1
            }],
            "placed at full duration, decremented once by the same-turn tick"
        );
        assert!(
            matches!(
                b.grid[4][4].piece,
                Some(PieceType::Stormcaller(_))
            ),
            "placer stays put"
        );
        assert_eq!(b.flags.side_to_move, Color::Black, "turn flips");
    }

    /// End-to-end: placing onto an enemy traps it — on the enemy's
    /// turn it has no legal moves while the tornado lives.
    #[test]
    fn place_tornado_onto_enemy_traps_it() {
        let mut b = board8();
        b.grid[4][4] = Square::new().set_piece(PieceType::Stormcaller(
            Stormcaller::new(Color::White),
        ));
        b.grid[4][5] = Square::new().set_piece(PieceType::new_rook(Color::Black));

        b.make_move(GameMove {
            from: cc(4, 4),
            move_type: MoveType::PlaceTornado { target: cc(5, 4) },
        })
        .expect("PlaceTornado onto the adjacent enemy is legal");

        // Tornado is on the rook's square; it's now Black to move.
        assert!(b.grid[4][5]
            .conditions
            .iter()
            .any(|c| matches!(c, SquareCondition::Tornado { .. })));
        assert_eq!(b.flags.side_to_move, Color::Black);
        assert!(
            b.legal_moves(&cc(5, 4)).is_empty(),
            "the trapped enemy rook has no legal moves"
        );
    }
}
