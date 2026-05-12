#[cfg(test)]
mod tests {
    use crate::{
        board::{
            Board, BoardFlags, Coord, GameMove, MoveType,
            fen::{board_to_fen, fen_to_board},
            square::{Square, SquareCondition, SquareType},
        },
        pieces::{
            Color,
            chess2::monkey::Monkey,
            fairy::{
                bus::Bus,
                goblin::{Goblin, GoblinState},
                skibidi::Skibidi,
            },
            piecetype::PieceType,
        },
    };

    use super::*;

    /// Helper: blank 8×8 board with default flags.
    fn empty_board() -> Board {
        Board {
            grid: vec![vec![Square::new(); 8]; 8],
            flags: BoardFlags {
                white_can_castle_kingside: true,
                white_can_castle_queenside: true,
                black_can_castle_kingside: true,
                black_can_castle_queenside: true,
                en_passant_target: None,
            },
        }
    }

    #[test]
    fn test_empty_board_fen() {
        let board = Board {
            grid: vec![vec![Square::new(); 8]; 8],
            flags: BoardFlags {
                white_can_castle_kingside: true,
                white_can_castle_queenside: true,
                black_can_castle_kingside: true,
                black_can_castle_queenside: true,
                en_passant_target: None,
            },
        };

        let fen = board_to_fen(&board);
        assert_eq!(fen, "8/8/8/8/8/8/8/8");

        let board2 = fen_to_board(&fen);
        assert_eq!(board2, board);
    }

    #[test]
    fn test_standard_pieces_fen() {
        let mut board = Board {
            grid: vec![vec![Square::new(); 8]; 8],
            flags: BoardFlags {
                white_can_castle_kingside: true,
                white_can_castle_queenside: true,
                black_can_castle_kingside: true,
                black_can_castle_queenside: true,
                en_passant_target: None,
            },
        };

        board.grid[0][0] = Square::new().set_piece(PieceType::new_rook(Color::White));
        board.grid[7][7] = Square::new().set_piece(PieceType::new_king(Color::Black));

        let fen = board_to_fen(&board);
        assert_eq!(fen, "R7/8/8/8/8/8/8/7k");

        let board2 = fen_to_board(&fen);
        assert_eq!(board2, board);
    }

    #[test]
    fn test_extended_square_fen() {
        let mut board = Board {
            grid: vec![vec![Square::new(); 8]; 8],
            flags: BoardFlags {
                white_can_castle_kingside: true,
                white_can_castle_queenside: true,
                black_can_castle_kingside: true,
                black_can_castle_queenside: true,
                en_passant_target: None,
            },
        };

        // Place a white rook on a vent square
        board.grid[0][0] = Square::new()
            .set_piece(PieceType::new_rook(Color::White))
            .set_square_type(SquareType::Vent);

        let fen = board_to_fen(&board);
        assert_eq!(fen, "(P=R,T=VENT)7/8/8/8/8/8/8/8");

        let board2 = fen_to_board(&fen);
        assert_eq!(board2, board);
    }

    #[test]
    fn test_square_with_conditions_fen() {
        let mut board = Board {
            grid: vec![vec![Square::new(); 8]; 8],
            flags: BoardFlags {
                white_can_castle_kingside: true,
                white_can_castle_queenside: true,
                black_can_castle_kingside: true,
                black_can_castle_queenside: true,
                en_passant_target: None,
            },
        };

        board.grid[1][1] = Square::new()
            .set_piece(PieceType::new_knight(Color::Black))
            .add_square_condition(SquareCondition::Frozen);

        let fen = board_to_fen(&board);
        assert_eq!(fen, "8/1(P=n,C=FROZEN)6/8/8/8/8/8/8");

        let board2 = fen_to_board(&fen);
        assert_eq!(board2, board);
    }

    #[test]
    fn test_square_with_conditions_and_types_fen() {
        let mut board = Board {
            grid: vec![vec![Square::new(); 8]; 8],
            flags: BoardFlags {
                white_can_castle_kingside: true,
                white_can_castle_queenside: true,
                black_can_castle_kingside: true,
                black_can_castle_queenside: true,
                en_passant_target: None,
            },
        };

        board.grid[1][1] = Square::new()
            .set_piece(PieceType::new_knight(Color::Black))
            .add_square_condition(SquareCondition::Frozen)
            .set_square_type(SquareType::Vent);

        let fen = board_to_fen(&board);
        assert_eq!(fen, "8/1(P=n,T=VENT,C=FROZEN)6/8/8/8/8/8/8");

        let board2 = fen_to_board(&fen);
        assert_eq!(board2, board);
    }

    #[test]
    fn test_fen_roundtrip() {
        let mut board = Board {
            grid: vec![vec![Square::new(); 8]; 8],
            flags: BoardFlags {
                white_can_castle_kingside: true,
                white_can_castle_queenside: true,
                black_can_castle_kingside: true,
                black_can_castle_queenside: true,
                en_passant_target: None,
            },
        };

        // Mix of standard and extended squares
        board.grid[0][0] = Square::new().set_piece(PieceType::new_rook(Color::White));
        board.grid[0][1] = Square::new()
            .set_piece(PieceType::new_knight(Color::Black))
            .set_square_type(SquareType::Turret)
            .add_square_condition(SquareCondition::Frozen);

        let fen = board_to_fen(&board);
        let board2 = fen_to_board(&fen);
        assert_eq!(board2, board);
    }

    // ============================================================
    // Pass 1 regression tests — critical-bug coverage
    // ============================================================

    /// A multi-jump Monkey move must report `from` as the original square,
    /// not the intermediate landing square. Otherwise `make_move` clears the
    /// wrong square and corrupts the board.
    #[test]
    fn test_monkey_chain_from_is_original_square() {
        let mut board = empty_board();
        board.grid[0][0] = Square::new().set_piece(PieceType::Monkey(Monkey { color: Color::White }));
        board.grid[1][1] = Square::new().set_piece(PieceType::new_pawn(Color::Black));
        board.grid[3][3] = Square::new().set_piece(PieceType::new_pawn(Color::Black));

        let from = Coord { file: 0, rank: 0 };
        let moves = board.get_moves(&from);

        // Find the second-hop move: lands on (4,4) after jumping (1,1) and (3,3).
        let chain = moves.iter().find(|m| match &m.move_type {
            MoveType::MoveTo(c) => c.file == 4 && c.rank == 4,
            _ => false,
        });
        let chain = chain.expect("expected a chain-jump move landing on (4,4)");
        assert_eq!(
            chain.from,
            Coord { file: 0, rank: 0 },
            "chained Monkey jump must report the original square as `from`"
        );
    }

    /// When the inner piece of a Bus moves onto a friendly carrier, the filter
    /// must preserve the `PieceInCarrier` envelope so `make_move` knows which
    /// passenger is moving. Clobbering it loses `piece_index`.
    #[test]
    fn test_pieceincarrier_preserved_when_inner_lands_on_friendly_carrier() {
        let mut board = empty_board();
        let bus_with_king = PieceType::Bus(Bus {
            color: Color::White,
            pieces: vec![PieceType::new_king(Color::White)],
        });
        let empty_bus = PieceType::Bus(Bus {
            color: Color::White,
            pieces: vec![],
        });
        board.grid[3][3] = Square::new().set_piece(bus_with_king);
        board.grid[3][4] = Square::new().set_piece(empty_bus);

        let from = Coord { file: 3, rank: 3 };
        let moves = board.get_moves(&from);

        // We expect a PieceInCarrier { piece_index: 0, move_type: MoveIntoCarrier((4,3)) }.
        let found = moves.iter().any(|m| match &m.move_type {
            MoveType::PieceInCarrier { piece_index: 0, move_type } => {
                matches!(move_type.as_ref(), MoveType::MoveIntoCarrier(c) if c.file == 4 && c.rank == 3)
            }
            _ => false,
        });
        assert!(
            found,
            "expected PieceInCarrier wrapper to survive the friendly-carrier swap; moves = {moves:?}"
        );
    }

    /// Doc says the Bus moves like a rook: orthogonal sliding. The previous
    /// code declared 8 directions with range 1.
    #[test]
    fn test_bus_moves_like_rook() {
        let mut board = empty_board();
        let bus = PieceType::Bus(Bus { color: Color::White, pieces: vec![] });
        board.grid[3][3] = Square::new().set_piece(bus);

        let from = Coord { file: 3, rank: 3 };
        let moves = board.get_moves(&from);

        let move_targets: Vec<Coord> = moves
            .iter()
            .filter_map(|m| match &m.move_type {
                MoveType::MoveTo(c) => Some(c.clone()),
                _ => None,
            })
            .collect();

        for t in &move_targets {
            let df = (t.file as isize - 3).abs();
            let dr = (t.rank as isize - 3).abs();
            assert!(
                df == 0 || dr == 0,
                "Bus produced diagonal move to ({},{})",
                t.file,
                t.rank
            );
        }
        assert_eq!(
            move_targets.len(),
            14,
            "Bus on an empty 8x8 from (3,3) should have 14 orthogonal moves, got {move_targets:?}"
        );
    }

    /// Spec: Skibidi "cannot take other pieces. It can take other Skibidis."
    /// So an adjacent enemy pawn must NOT be a legal target.
    #[test]
    fn test_skibidi_cannot_capture_non_skibidi() {
        let mut board = empty_board();
        board.grid[3][3] = Square::new().set_piece(PieceType::Skibidi(Skibidi {
            color: Color::White,
            phase: 1,
        }));
        board.grid[3][4] = Square::new().set_piece(PieceType::new_pawn(Color::Black));

        let from = Coord { file: 3, rank: 3 };
        let moves = board.get_moves(&from);

        let captures_pawn = moves.iter().any(|m| matches!(
            &m.move_type,
            MoveType::MoveTo(c) if c.file == 4 && c.rank == 3
        ));
        assert!(!captures_pawn, "Skibidi must not capture a non-Skibidi enemy");
    }

    /// Spec: Skibidi can capture enemy Skibidi regardless of phase. The
    /// previous `symbol().to_lowercase()` comparison broke this when phases
    /// differed (because phase > 1 mutates the symbol string).
    #[test]
    fn test_skibidi_captures_enemy_skibidi_across_phases() {
        let mut board = empty_board();
        board.grid[3][3] = Square::new().set_piece(PieceType::Skibidi(Skibidi {
            color: Color::White,
            phase: 1,
        }));
        board.grid[3][4] = Square::new().set_piece(PieceType::Skibidi(Skibidi {
            color: Color::Black,
            phase: 2,
        }));

        let from = Coord { file: 3, rank: 3 };
        let moves = board.get_moves(&from);

        let captures_skibidi = moves.iter().any(|m| matches!(
            &m.move_type,
            MoveType::MoveTo(c) if c.file == 4 && c.rank == 3
        ));
        assert!(
            captures_skibidi,
            "Skibidi must be able to capture enemy Skibidi regardless of phase"
        );
    }

    /// Spec: phase 2 brainrots the 4 orthogonal neighbors only (radius-1
    /// Manhattan disk). The previous code painted a 3×3 Chebyshev box.
    #[test]
    fn test_brainrot_phase_2_is_orthogonal_only() {
        let mut board = empty_board();
        board.grid[4][4] = Square::new().set_piece(PieceType::Skibidi(Skibidi {
            color: Color::White,
            phase: 2,
        }));
        board.recalc_brainrot();

        for &(df, dr) in &[(1isize, 0isize), (-1, 0), (0, 1), (0, -1)] {
            let f = (4 + df) as usize;
            let r = (4 + dr) as usize;
            assert!(
                board.grid[r][f].conditions.contains(&SquareCondition::Brainrot),
                "orthogonal neighbour at file={f} rank={r} should be Brainrot"
            );
        }
        for &(df, dr) in &[(1isize, 1isize), (1, -1), (-1, 1), (-1, -1)] {
            let f = (4 + df) as usize;
            let r = (4 + dr) as usize;
            assert!(
                !board.grid[r][f].conditions.contains(&SquareCondition::Brainrot),
                "diagonal neighbour at file={f} rank={r} must NOT be Brainrot at phase 2"
            );
        }
    }

    /// Overlapping radii from two Skibidis must not stack duplicate
    /// `Brainrot` entries on the same square.
    #[test]
    fn test_brainrot_no_duplicates_on_overlap() {
        let mut board = empty_board();
        // Two phase-2 Skibidis far enough apart not to neutralise each other
        // (Manhattan distance > 1), but with overlapping orthogonal radii.
        board.grid[3][3] = Square::new().set_piece(PieceType::Skibidi(Skibidi {
            color: Color::White,
            phase: 2,
        }));
        board.grid[3][5] = Square::new().set_piece(PieceType::Skibidi(Skibidi {
            color: Color::Black,
            phase: 2,
        }));
        board.recalc_brainrot();

        // (4,3) is orthogonally adjacent to both.
        let count = board.grid[3][4]
            .conditions
            .iter()
            .filter(|c| matches!(c, SquareCondition::Brainrot))
            .count();
        assert_eq!(
            count, 1,
            "overlap square must have exactly one Brainrot condition, not {count}"
        );
    }

    /// Spec: any Skibidi entering another Skibidi's brainrot radius
    /// neutralises the radiating Skibidi back to phase 1.
    #[test]
    fn test_skibidi_neutralization() {
        let mut board = empty_board();
        // White phase 3 (radius 2). Black phase 1 sits within that radius.
        board.grid[4][4] = Square::new().set_piece(PieceType::Skibidi(Skibidi {
            color: Color::White,
            phase: 3,
        }));
        board.grid[4][5] = Square::new().set_piece(PieceType::Skibidi(Skibidi {
            color: Color::Black,
            phase: 1,
        }));
        board.recalc_brainrot();

        match &board.grid[4][4].piece {
            Some(PieceType::Skibidi(sk)) => assert_eq!(
                sk.phase, 1,
                "white Skibidi must be neutralised to phase 1 by black Skibidi in its radius"
            ),
            other => panic!("expected white Skibidi at (4,4), got {other:?}"),
        }
    }

    /// Spec: "If there is no opposing Skibidi, the maximum phase your Skibidi
    /// can reach is 3."
    #[test]
    fn test_skibidi_phase_capped_at_three_without_opponent() {
        let mut board = empty_board();
        board.grid[4][4] = Square::new().set_piece(PieceType::Skibidi(Skibidi {
            color: Color::White,
            phase: 3,
        }));

        let from = Coord { file: 4, rank: 4 };
        let mv = GameMove { from: from.clone(), move_type: MoveType::PhaseShift };
        board.make_move(mv).expect("phase shift should be a legal move");

        match &board.grid[4][4].piece {
            Some(PieceType::Skibidi(sk)) => assert_eq!(
                sk.phase, 3,
                "phase must be capped at 3 without an opposing Skibidi"
            ),
            other => panic!("expected white Skibidi at (4,4), got {other:?}"),
        }
    }
}
