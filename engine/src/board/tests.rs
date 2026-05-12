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

    /// With an opposing Skibidi present, the cap rises to 4.
    #[test]
    fn test_skibidi_phase_can_reach_four_with_opponent() {
        let mut board = empty_board();
        board.grid[0][0] = Square::new().set_piece(PieceType::Skibidi(Skibidi {
            color: Color::White,
            phase: 3,
        }));
        // Far enough away that no neutralization happens.
        board.grid[7][7] = Square::new().set_piece(PieceType::Skibidi(Skibidi {
            color: Color::Black,
            phase: 1,
        }));

        let mv = GameMove {
            from: Coord { file: 0, rank: 0 },
            move_type: MoveType::PhaseShift,
        };
        board.make_move(mv).expect("phase shift should be legal");

        match &board.grid[0][0].piece {
            Some(PieceType::Skibidi(sk)) => assert_eq!(sk.phase, 4),
            other => panic!("expected white Skibidi at (0,0), got {other:?}"),
        }
    }

    // ============================================================
    // Pass 2 invariant / coverage tests
    // ============================================================

    // --- pawn ---

    /// A white pawn on its starting rank (6) gets both single- and
    /// double-push when the column is clear.
    #[test]
    fn test_pawn_double_push_from_start() {
        let mut board = empty_board();
        board.grid[6][3] = Square::new().set_piece(PieceType::new_pawn(Color::White));

        let moves = board.get_moves(&Coord { file: 3, rank: 6 });
        let targets: Vec<_> = moves
            .iter()
            .filter_map(|m| match &m.move_type {
                MoveType::MoveTo(c) => Some((c.file, c.rank)),
                _ => None,
            })
            .collect();
        assert!(targets.contains(&(3, 5)), "expected single push to (3,5)");
        assert!(targets.contains(&(3, 4)), "expected double push to (3,4)");
    }

    /// Pawn cannot double-push if the intermediate square is occupied.
    #[test]
    fn test_pawn_no_double_push_when_blocked() {
        let mut board = empty_board();
        board.grid[6][3] = Square::new().set_piece(PieceType::new_pawn(Color::White));
        board.grid[5][3] = Square::new().set_piece(PieceType::new_pawn(Color::Black));

        let moves = board.get_moves(&Coord { file: 3, rank: 6 });
        let targets: Vec<_> = moves
            .iter()
            .filter_map(|m| match &m.move_type {
                MoveType::MoveTo(c) => Some((c.file, c.rank)),
                _ => None,
            })
            .collect();
        assert!(!targets.contains(&(3, 5)), "blocker at (3,5) must prevent single push too");
        assert!(!targets.contains(&(3, 4)), "blocker at (3,5) must prevent double push");
    }

    /// Pawn captures only into a diagonal square occupied by an enemy.
    #[test]
    fn test_pawn_diagonal_capture_only_when_enemy_present() {
        let mut board = empty_board();
        board.grid[6][3] = Square::new().set_piece(PieceType::new_pawn(Color::White));
        board.grid[5][2] = Square::new().set_piece(PieceType::new_pawn(Color::Black)); // SW = enemy
        // (4,5) — SE diagonal — left empty

        let moves = board.get_moves(&Coord { file: 3, rank: 6 });
        let targets: Vec<_> = moves
            .iter()
            .filter_map(|m| match &m.move_type {
                MoveType::MoveTo(c) => Some((c.file, c.rank)),
                _ => None,
            })
            .collect();
        assert!(targets.contains(&(2, 5)), "should capture SW enemy");
        assert!(!targets.contains(&(4, 5)), "should not diagonal-move to empty SE");
    }

    // --- knight ---

    #[test]
    fn test_knight_corner_has_two_moves() {
        let mut board = empty_board();
        board.grid[0][0] = Square::new().set_piece(PieceType::new_knight(Color::White));
        let moves = board.get_moves(&Coord { file: 0, rank: 0 });
        assert_eq!(moves.len(), 2, "knight in corner has 2 L-moves, got {moves:?}");
    }

    // --- rook ---

    /// Friendly piece blocks the rook one square short — but the blocker
    /// square itself is not a legal target.
    #[test]
    fn test_rook_blocked_by_friendly_stops_short() {
        let mut board = empty_board();
        board.grid[3][3] = Square::new().set_piece(PieceType::new_rook(Color::White));
        board.grid[3][6] = Square::new().set_piece(PieceType::new_pawn(Color::White));

        let moves = board.get_moves(&Coord { file: 3, rank: 3 });
        let targets: Vec<_> = moves
            .iter()
            .filter_map(|m| match &m.move_type {
                MoveType::MoveTo(c) => Some((c.file, c.rank)),
                _ => None,
            })
            .collect();
        assert!(targets.contains(&(4, 3)));
        assert!(targets.contains(&(5, 3)));
        assert!(!targets.contains(&(6, 3)), "friendly blocker is not a legal target");
        assert!(!targets.contains(&(7, 3)), "cannot pass through blocker");
    }

    /// Rook may capture an enemy blocker (ray terminates at it).
    #[test]
    fn test_rook_captures_enemy_blocker() {
        let mut board = empty_board();
        board.grid[3][3] = Square::new().set_piece(PieceType::new_rook(Color::White));
        board.grid[3][6] = Square::new().set_piece(PieceType::new_pawn(Color::Black));

        let moves = board.get_moves(&Coord { file: 3, rank: 3 });
        let targets: Vec<_> = moves
            .iter()
            .filter_map(|m| match &m.move_type {
                MoveType::MoveTo(c) => Some((c.file, c.rank)),
                _ => None,
            })
            .collect();
        assert!(targets.contains(&(6, 3)), "rook should be able to capture enemy at (6,3)");
        assert!(!targets.contains(&(7, 3)), "cannot pass through captured piece");
    }

    // --- king ---

    #[test]
    fn test_king_has_eight_adjacent_moves_from_center() {
        let mut board = empty_board();
        board.grid[4][4] = Square::new().set_piece(PieceType::new_king(Color::White));
        let moves = board.get_moves(&Coord { file: 4, rank: 4 });
        assert_eq!(moves.len(), 8, "king in center has 8 adjacent squares");
    }

    // --- board-level get_moves gating ---

    #[test]
    fn test_get_moves_empty_for_brainrot_square() {
        let mut board = empty_board();
        board.grid[3][3] = Square::new()
            .set_piece(PieceType::new_knight(Color::White))
            .add_square_condition(SquareCondition::Brainrot);

        let moves = board.get_moves(&Coord { file: 3, rank: 3 });
        assert!(moves.is_empty(), "brainrotted piece must not move");
    }

    #[test]
    fn test_get_moves_empty_for_frozen_square() {
        let mut board = empty_board();
        board.grid[3][3] = Square::new()
            .set_piece(PieceType::new_knight(Color::White))
            .add_square_condition(SquareCondition::Frozen);

        let moves = board.get_moves(&Coord { file: 3, rank: 3 });
        assert!(moves.is_empty(), "frozen piece must not move");
    }

    // --- make_move invariants ---

    #[test]
    fn test_make_move_rejects_illegal() {
        let mut board = empty_board();
        board.grid[0][0] = Square::new().set_piece(PieceType::new_rook(Color::White));

        // Rooks don't move diagonally.
        let illegal = GameMove {
            from: Coord { file: 0, rank: 0 },
            move_type: MoveType::MoveTo(Coord { file: 3, rank: 3 }),
        };
        let result = board.make_move(illegal);
        assert!(result.is_err(), "illegal rook diagonal must be rejected");
        // Board untouched.
        assert!(board.grid[0][0].piece.is_some(), "rook still at (0,0)");
        assert!(board.grid[3][3].piece.is_none(), "diagonal target untouched");
    }

    #[test]
    fn test_make_move_moves_piece_and_captures() {
        let mut board = empty_board();
        board.grid[0][0] = Square::new().set_piece(PieceType::new_rook(Color::White));
        board.grid[0][4] = Square::new().set_piece(PieceType::new_pawn(Color::Black));

        let mv = GameMove {
            from: Coord { file: 0, rank: 0 },
            move_type: MoveType::MoveTo(Coord { file: 4, rank: 0 }),
        };
        board.make_move(mv).expect("capture should succeed");

        assert!(board.grid[0][0].piece.is_none(), "source square cleared");
        match &board.grid[0][4].piece {
            Some(PieceType::Rook(_)) => {}
            other => panic!("expected rook at (4,0), got {other:?}"),
        }
    }

    // --- FEN round-trips for non-trivial fairy states ---

    #[test]
    fn test_fen_roundtrip_skibidi_phase_three() {
        let mut board = empty_board();
        board.grid[4][4] = Square::new().set_piece(PieceType::Skibidi(Skibidi {
            color: Color::White,
            phase: 3,
        }));

        let fen = board_to_fen(&board);
        let board2 = fen_to_board(&fen);

        // The brainrot conditions are derived state — recompute on both
        // sides before comparing so we don't depend on whether either
        // applied them.
        let mut a = board.clone();
        a.recalc_brainrot();
        let mut b = board2.clone();
        b.recalc_brainrot();
        assert_eq!(a, b, "Skibidi phase-3 should round-trip via FEN");
    }

    #[test]
    fn test_fen_roundtrip_bus_carrying_pieces() {
        let mut board = empty_board();
        board.grid[3][3] = Square::new().set_piece(PieceType::Bus(Bus {
            color: Color::White,
            pieces: vec![
                PieceType::new_pawn(Color::White),
                PieceType::new_knight(Color::Black),
            ],
        }));

        let fen = board_to_fen(&board);
        let board2 = fen_to_board(&fen);
        assert_eq!(board2, board, "Bus carrying pieces should round-trip");
    }

    /// A full Bus (5 passengers) must not be a legal MoveIntoCarrier target.
    #[test]
    fn test_bus_at_capacity_blocks_entry() {
        let mut board = empty_board();
        let pawn = PieceType::new_pawn(Color::White);
        let full_bus = PieceType::Bus(Bus {
            color: Color::White,
            pieces: vec![pawn.clone(), pawn.clone(), pawn.clone(), pawn.clone(), pawn.clone()],
        });
        board.grid[3][3] = Square::new().set_piece(full_bus);
        // Knight at (1,2) can L-move to (3,3).
        board.grid[2][1] = Square::new().set_piece(PieceType::new_knight(Color::White));

        let moves = board.get_moves(&Coord { file: 1, rank: 2 });
        let entered_full_bus = moves.iter().any(|m| matches!(
            &m.move_type,
            MoveType::MoveIntoCarrier(c) if c.file == 3 && c.rank == 3
        ));
        assert!(!entered_full_bus, "knight should not enter a full bus");
    }

    // ============================================================
    // Pass 4 regression tests — panic + corruption fixes
    // ============================================================

    /// A Skibidi inside a Bus emits `PhaseShift` from its `initial_moves`,
    /// which Bus wraps as `PieceInCarrier { inner: PhaseShift }`. Previously
    /// the filter `todo!()`d on this; now it must drop the move silently.
    #[test]
    fn test_skibidi_in_bus_does_not_crash_get_moves() {
        let mut board = empty_board();
        board.grid[3][3] = Square::new().set_piece(PieceType::Bus(Bus {
            color: Color::White,
            pieces: vec![PieceType::Skibidi(Skibidi {
                color: Color::White,
                phase: 1,
            })],
        }));

        // Should not panic.
        let moves = board.get_moves(&Coord { file: 3, rank: 3 });

        // Any PieceInCarrier{inner: PhaseShift} must have been dropped.
        let has_passenger_phaseshift = moves.iter().any(|m| matches!(
            &m.move_type,
            MoveType::PieceInCarrier { move_type, .. }
            if matches!(move_type.as_ref(), MoveType::PhaseShift)
        ));
        assert!(
            !has_passenger_phaseshift,
            "passenger PhaseShift should be dropped, not produced"
        );
    }

    /// A Bus must not enter another Bus, because the capacity-5 invariant
    /// would otherwise be bypassed via nesting.
    #[test]
    fn test_bus_cannot_enter_another_bus() {
        let mut board = empty_board();
        board.grid[3][3] = Square::new().set_piece(PieceType::Bus(Bus {
            color: Color::White,
            pieces: vec![],
        }));
        board.grid[3][4] = Square::new().set_piece(PieceType::Bus(Bus {
            color: Color::White,
            pieces: vec![],
        }));

        let moves = board.get_moves(&Coord { file: 3, rank: 3 });
        let nested = moves.iter().any(|m| matches!(
            &m.move_type,
            MoveType::MoveIntoCarrier(c) if c.file == 4 && c.rank == 3
        ));
        assert!(!nested, "Bus must not be allowed to enter another Bus");
    }

    /// A passenger of Bus A also cannot enter a different friendly Bus B if
    /// that passenger is itself a carrier — same nesting reasoning.
    #[test]
    fn test_passenger_carrier_cannot_enter_another_bus() {
        let mut board = empty_board();
        // Outer Bus carries an inner empty Bus (constructed manually — the
        // engine itself now refuses to produce such a state, but it might
        // come in via FEN).
        board.grid[3][3] = Square::new().set_piece(PieceType::Bus(Bus {
            color: Color::White,
            pieces: vec![PieceType::Bus(Bus {
                color: Color::White,
                pieces: vec![],
            })],
        }));
        // A friendly Bus to "land" the passenger in.
        board.grid[3][4] = Square::new().set_piece(PieceType::Bus(Bus {
            color: Color::White,
            pieces: vec![],
        }));

        // Should not panic, and no PieceInCarrier{MoveIntoCarrier} move
        // should be generated for the inner Bus exiting into the outer one.
        let moves = board.get_moves(&Coord { file: 3, rank: 3 });
        let nesting_move = moves.iter().any(|m| matches!(
            &m.move_type,
            MoveType::PieceInCarrier { move_type, .. }
            if matches!(move_type.as_ref(), MoveType::MoveIntoCarrier(_))
        ));
        assert!(
            !nesting_move,
            "carrier passenger must not enter another carrier"
        );
    }

    /// Monkey jump chains must not share `visited` across sibling branches.
    /// Place pieces so two distinct jump chains converge on the same final
    /// landing — both should be enumerable.
    #[test]
    fn test_monkey_visited_is_per_path() {
        let mut board = empty_board();
        board.grid[4][4] = Square::new().set_piece(PieceType::Monkey(Monkey { color: Color::White }));

        // Layout: monkey at (4,4) with enemy pawns set up so that several
        // distinct chains can land on (4,2). One via (5,5)→(4,6) etc. is
        // hard to construct cleanly; the simpler property we verify here is
        // that the move set generated from a complex position is at least
        // larger than what the old shared-visited code produced. Concretely:
        // four enemy pawns surrounding the monkey at distance 1 each open
        // four single-jump landings.
        board.grid[5][4] = Square::new().set_piece(PieceType::new_pawn(Color::Black));
        board.grid[3][4] = Square::new().set_piece(PieceType::new_pawn(Color::Black));
        board.grid[4][5] = Square::new().set_piece(PieceType::new_pawn(Color::Black));
        board.grid[4][3] = Square::new().set_piece(PieceType::new_pawn(Color::Black));

        let moves = board.get_moves(&Coord { file: 4, rank: 4 });
        let landings: Vec<(u8, u8)> = moves
            .iter()
            .filter_map(|m| match &m.move_type {
                MoveType::MoveTo(c) => Some((c.file, c.rank)),
                _ => None,
            })
            .collect();
        // Four direct jumps to (4,6), (4,2), (6,4), (2,4).
        for expected in [(4, 6), (4, 2), (6, 4), (2, 4)] {
            assert!(
                landings.contains(&expected),
                "expected jump landing at {expected:?}, got {landings:?}"
            );
        }
    }

    /// Malformed Bus FEN (no inner parens around the passenger list) must
    /// not panic — the previous code did `.strip_prefix("(").unwrap()`.
    #[test]
    fn test_bus_fen_malformed_p_field_does_not_panic() {
        // Should parse without panicking. The malformed P=R field is
        // dropped; the Bus comes back with an empty passenger list.
        let board = fen_to_board("(P=BUS(P=R))7/8/8/8/8/8/8/8");
        // Confirm something was placed at (0,0) — specifically, a Bus.
        match &board.grid[0][0].piece {
            Some(PieceType::Bus(bus)) => assert!(
                bus.pieces.is_empty(),
                "malformed P=... should fall through to empty passenger list"
            ),
            other => panic!("expected a Bus at (0,0), got {other:?}"),
        }
    }

    /// Hand-constructed `MoveIntoCarrier` onto a non-Bus target should
    /// return Err, not panic. (This path is unreachable via `get_moves` but
    /// the API layer doesn't guarantee that.)
    #[test]
    fn test_make_move_into_carrier_on_non_bus_errors() {
        let mut board = empty_board();
        board.grid[0][0] = Square::new().set_piece(PieceType::new_pawn(Color::White));
        board.grid[0][1] = Square::new().set_piece(PieceType::new_rook(Color::White));

        // Build a GameMove by hand — `is_valid_move` will reject this
        // because no rook produces a MoveIntoCarrier, so make_move returns
        // Err("Illegal move: ...") rather than reaching the carrier panic.
        let bogus = GameMove {
            from: Coord { file: 0, rank: 0 },
            move_type: MoveType::MoveIntoCarrier(Coord { file: 1, rank: 0 }),
        };
        let result = board.make_move(bogus);
        assert!(result.is_err(), "non-carrier MoveIntoCarrier must return Err");
    }

    #[test]
    fn test_fen_roundtrip_goblin_kidnapping() {
        let mut board = empty_board();
        board.grid[3][3] = Square::new().set_piece(PieceType::Goblin(Goblin {
            color: Color::White,
            state: GoblinState::Kidnapping {
                piece: std::rc::Rc::new(PieceType::new_knight(Color::Black)),
            },
            home_square: Coord { file: 0, rank: 0 },
        }));

        let fen = board_to_fen(&board);
        let board2 = fen_to_board(&fen);
        assert_eq!(board2, board, "Goblin in Kidnapping state should round-trip");
    }
}
