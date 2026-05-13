#[cfg(test)]
mod tests {
    use crate::{
        board::{
            Board, BoardFlags, CastleSide, Coord, GameMove, GameStatus, MoveError, MoveType,
            PromotionTarget,
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
                side_to_move: Color::White,
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
                side_to_move: Color::White,
                white_can_castle_kingside: true,
                white_can_castle_queenside: true,
                black_can_castle_kingside: true,
                black_can_castle_queenside: true,
                en_passant_target: None,
            },
        };

        let fen = board_to_fen(&board);
        assert_eq!(fen, "8/8/8/8/8/8/8/8 w KQkq -");

        let board2 = fen_to_board(&fen);
        assert_eq!(board2, board);
    }

    #[test]
    fn test_standard_pieces_fen() {
        let mut board = Board {
            grid: vec![vec![Square::new(); 8]; 8],
            flags: BoardFlags {
                side_to_move: Color::White,
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
        assert_eq!(fen, "R7/8/8/8/8/8/8/7k w KQkq -");

        let board2 = fen_to_board(&fen);
        assert_eq!(board2, board);
    }

    #[test]
    fn test_extended_square_fen() {
        let mut board = Board {
            grid: vec![vec![Square::new(); 8]; 8],
            flags: BoardFlags {
                side_to_move: Color::White,
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
        assert_eq!(fen, "(P=R,T=VENT)7/8/8/8/8/8/8/8 w KQkq -");

        let board2 = fen_to_board(&fen);
        assert_eq!(board2, board);
    }

    #[test]
    fn test_square_with_conditions_fen() {
        let mut board = Board {
            grid: vec![vec![Square::new(); 8]; 8],
            flags: BoardFlags {
                side_to_move: Color::White,
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
        assert_eq!(fen, "8/1(P=n,C=FROZEN)6/8/8/8/8/8/8 w KQkq -");

        let board2 = fen_to_board(&fen);
        assert_eq!(board2, board);
    }

    #[test]
    fn test_square_with_conditions_and_types_fen() {
        let mut board = Board {
            grid: vec![vec![Square::new(); 8]; 8],
            flags: BoardFlags {
                side_to_move: Color::White,
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
        assert_eq!(fen, "8/1(P=n,T=VENT,C=FROZEN)6/8/8/8/8/8/8 w KQkq -");

        let board2 = fen_to_board(&fen);
        assert_eq!(board2, board);
    }

    #[test]
    fn test_fen_roundtrip() {
        let mut board = Board {
            grid: vec![vec![Square::new(); 8]; 8],
            flags: BoardFlags {
                side_to_move: Color::White,
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
    /// neutralises the radiating Skibidi back to phase 1. Verifies
    /// both the phase reset *and* the down-stream effect — once the
    /// radiator is at phase 1 it stops painting Brainrot squares,
    /// so the entire previously-radiating disk should be clear after
    /// `recalc_brainrot`.
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

        // Down-stream check: with white reset to phase 1 (radius 0)
        // and black already at phase 1, no square on the board should
        // carry a Brainrot condition. This catches the "phase reset
        // but aura still painted" bug class — the original tightening
        // asserted "black phase stays 1" which was tautological since
        // `recalc_brainrot` can only ever *reduce* a phase to 1.
        let any_brainrot = board.grid.iter().flatten().any(|sq| {
            sq.conditions.contains(&SquareCondition::Brainrot)
        });
        assert!(
            !any_brainrot,
            "no square should carry Brainrot after the only radiating Skibidi was neutralised"
        );
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
    /// Specifically: NW empty → no move, SW enemy → capture, SE friendly
    /// → no move (must not capture own piece even though the diagonal is
    /// "occupied" in the wrong sense).
    #[test]
    fn test_pawn_diagonal_capture_only_when_enemy_present() {
        let mut board = empty_board();
        board.grid[6][3] = Square::new().set_piece(PieceType::new_pawn(Color::White));
        board.grid[5][2] = Square::new().set_piece(PieceType::new_pawn(Color::Black)); // SW = enemy
        board.grid[5][4] = Square::new().set_piece(PieceType::new_pawn(Color::White)); // SE = friendly

        let moves = board.get_moves(&Coord { file: 3, rank: 6 });
        let targets: Vec<_> = moves
            .iter()
            .filter_map(|m| match &m.move_type {
                MoveType::MoveTo(c) => Some((c.file, c.rank)),
                _ => None,
            })
            .collect();
        assert!(targets.contains(&(2, 5)), "should capture SW enemy");
        assert!(
            !targets.contains(&(4, 5)),
            "must not capture friendly piece on SE diagonal"
        );
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

    /// Rook may capture an enemy blocker (ray terminates at it). Also
    /// confirms the rook offers every empty intermediate square as a
    /// legal target so a "ray collapsed to just the capture" regression
    /// would be caught.
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
        assert!(targets.contains(&(4, 3)), "intermediate (4,3) must be reachable");
        assert!(targets.contains(&(5, 3)), "intermediate (5,3) must be reachable");
        assert!(targets.contains(&(6, 3)), "rook should capture enemy at (6,3)");
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
        // Board untouched — assert the rook variant specifically so a
        // regression that replaces the rook with some other piece
        // wouldn't be masked by an "is_some()" check.
        match &board.grid[0][0].piece {
            Some(PieceType::Rook(r)) => assert_eq!(r.color, Color::White),
            other => panic!("expected white rook still at (0,0), got {other:?}"),
        }
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

    /// A full Bus (5 passengers) must not be a legal MoveIntoCarrier
    /// target — and rejection of that one target must NOT cause the
    /// knight to lose its other L-moves. Regression guard against a
    /// "filter accidentally drops everything when one target fails."
    #[test]
    fn test_bus_at_capacity_blocks_entry() {
        let mut board = empty_board();
        let pawn = PieceType::new_pawn(Color::White);
        let full_bus = PieceType::Bus(Bus {
            color: Color::White,
            pieces: vec![pawn.clone(), pawn.clone(), pawn.clone(), pawn.clone(), pawn.clone()],
        });
        board.grid[3][3] = Square::new().set_piece(full_bus);
        // Knight at (1,2) can L-move to (3,3) — and to other squares.
        board.grid[2][1] = Square::new().set_piece(PieceType::new_knight(Color::White));

        let moves = board.get_moves(&Coord { file: 1, rank: 2 });
        let entered_full_bus = moves.iter().any(|m| matches!(
            &m.move_type,
            MoveType::MoveIntoCarrier(c) if c.file == 3 && c.rank == 3
        ));
        assert!(!entered_full_bus, "knight should not enter a full bus");

        // The knight's other L-moves from (1,2) must still be present.
        // From (1,2): (0,0), (2,0), (3,1), (3,3) [blocked-bus],
        // (2,4), (0,4) — six in-bounds squares total. We expect at
        // least the non-(3,3) ones.
        let targets: Vec<(u8, u8)> = moves
            .iter()
            .filter_map(|m| match &m.move_type {
                MoveType::MoveTo(c) => Some((c.file, c.rank)),
                _ => None,
            })
            .collect();
        for expected in [(0, 0), (2, 0), (3, 1), (2, 4), (0, 4)] {
            assert!(
                targets.contains(&expected),
                "knight should still reach {expected:?} despite full-bus rejection; got {targets:?}"
            );
        }
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

    /// With ladder pieces in each cardinal direction, the Monkey emits
    /// one jump-landing per direction. Renamed from the historical
    /// `test_monkey_visited_is_per_path` because the original was a
    /// breadth check, not a per-path-visited check. The actual
    /// per-path-visited property has its own dedicated test below.
    #[test]
    fn test_monkey_emits_one_landing_per_adjacent_ladder() {
        let mut board = empty_board();
        board.grid[4][4] = Square::new().set_piece(PieceType::Monkey(Monkey { color: Color::White }));
        // Four enemy pawns surrounding the monkey at distance 1 each open
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

    /// The real per-path-visited test: two distinct jump chains from
    /// the same Monkey converge on the same landing square. Under the
    /// fixed per-path `visited` discipline (push before recurse, pop
    /// after), both chains should be enumerated independently — the
    /// converging landing therefore appears in the move list *twice*.
    ///
    /// Under the historic shared-visited bug, the second chain would
    /// find the convergence square already in `visited` (left over
    /// from the first chain) and skip it. So this test fails under
    /// the bug and passes under the current code.
    ///
    /// Geometry (Monkey at (3,3)):
    ///   - Path A: (3,3) → SE jump over (4,4) → (5,5)
    ///   - Path B: (3,3) → S jump over (3,4) → (3,5)
    ///                 → SE jump over (4,5) → (5,5)
    #[test]
    fn test_monkey_visited_does_not_leak_across_chains() {
        let mut board = empty_board();
        board.grid[3][3] = Square::new()
            .set_piece(PieceType::Monkey(Monkey { color: Color::White }));
        // Ladder pieces for the two converging paths.
        board.grid[4][4] = Square::new().set_piece(PieceType::new_pawn(Color::Black));
        board.grid[4][3] = Square::new().set_piece(PieceType::new_pawn(Color::Black));
        board.grid[5][4] = Square::new().set_piece(PieceType::new_pawn(Color::Black));

        let moves = board.get_moves(&Coord { file: 3, rank: 3 });
        let landings_at_55 = moves
            .iter()
            .filter(|m| matches!(
                &m.move_type,
                MoveType::MoveTo(c) if c.file == 5 && c.rank == 5
            ))
            .count();

        assert!(
            landings_at_55 >= 2,
            "expected (5,5) reachable via both paths (shared-visited bug would collapse to 1); got {landings_at_55} occurrences in {moves:?}"
        );
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

    // ============================================================
    // Plan 01: Turn system
    // ============================================================

    #[test]
    fn test_white_cannot_move_on_blacks_turn() {
        let mut board = empty_board();
        board.flags.side_to_move = Color::Black;
        board.grid[6][3] = Square::new().set_piece(PieceType::new_pawn(Color::White));

        let mv = GameMove {
            from: Coord { file: 3, rank: 6 },
            move_type: MoveType::MoveTo(Coord { file: 3, rank: 5 }),
        };
        assert!(
            board.make_move(mv).is_err(),
            "white must not be able to move on black's turn"
        );
    }

    #[test]
    fn test_make_move_flips_turn() {
        let mut board = empty_board();
        board.grid[6][3] = Square::new().set_piece(PieceType::new_pawn(Color::White));

        let mv = GameMove {
            from: Coord { file: 3, rank: 6 },
            move_type: MoveType::MoveTo(Coord { file: 3, rank: 5 }),
        };
        board.make_move(mv).expect("legal pawn push");

        assert_eq!(
            board.flags.side_to_move,
            Color::Black,
            "side_to_move should flip after a legal move"
        );
    }

    #[test]
    fn test_fen_roundtrip_with_side_to_move() {
        let mut board = empty_board();
        board.flags.side_to_move = Color::Black;
        board.grid[0][0] = Square::new().set_piece(PieceType::new_king(Color::Black));

        let fen = board_to_fen(&board);
        assert!(
            fen.contains(" b "),
            "side-to-move 'b' should be present in FEN, got {fen:?}"
        );
        let board2 = fen_to_board(&fen);
        assert_eq!(board2.flags.side_to_move, Color::Black);
    }

    #[test]
    fn test_fen_roundtrip_with_no_castle_rights() {
        let mut board = empty_board();
        board.flags.white_can_castle_kingside = false;
        board.flags.white_can_castle_queenside = false;
        board.flags.black_can_castle_kingside = false;
        board.flags.black_can_castle_queenside = false;

        let fen = board_to_fen(&board);
        assert!(
            fen.ends_with(" -"),
            "no castle rights + no ep should end with ' -', got {fen:?}"
        );
        let board2 = fen_to_board(&fen);
        assert_eq!(board2.flags, board.flags);
    }

    #[test]
    fn test_fen_grid_only_backcompat() {
        // Pre-plan-01 callers may still hand in a grid-only FEN. The
        // parser must default sanely (white-to-move, all castle rights,
        // no ep target) rather than misinterpret the missing fields.
        let board = fen_to_board("8/8/8/8/8/8/8/8");
        assert_eq!(board.flags.side_to_move, Color::White);
        assert!(board.flags.white_can_castle_kingside);
        assert!(board.flags.black_can_castle_queenside);
        assert!(board.flags.en_passant_target.is_none());
    }

    // ============================================================
    // Plan 02: King safety
    // ============================================================

    #[test]
    fn test_king_cannot_move_into_check() {
        let mut board = empty_board();
        // White king at (3,4), black rook at (3,0) staring down file 3.
        board.grid[4][3] = Square::new().set_piece(PieceType::new_king(Color::White));
        board.grid[0][3] = Square::new().set_piece(PieceType::new_rook(Color::Black));

        let from = Coord { file: 3, rank: 4 };
        let legal = board.legal_moves(&from);
        // The king must not be allowed to step further along file 3 (into
        // (3,3) or (3,5)) — both are attacked by the rook.
        let stays_on_file = legal.iter().any(|m| matches!(
            &m.move_type,
            MoveType::MoveTo(c) if c.file == 3
        ));
        assert!(
            !stays_on_file,
            "king must not be able to move along the attacked file, got moves={legal:?}"
        );
        // It can still step to file 2 or 4 on any rank (those aren't on the rook's ray).
        let leaves_file = legal.iter().any(|m| matches!(
            &m.move_type,
            MoveType::MoveTo(c) if c.file != 3
        ));
        assert!(leaves_file, "king must be able to escape sideways");
    }

    #[test]
    fn test_pinned_piece_cannot_move() {
        let mut board = empty_board();
        // White king at (3,7), white knight at (3,4), black rook at (3,0).
        // Knight is absolutely pinned on file 3.
        board.grid[7][3] = Square::new().set_piece(PieceType::new_king(Color::White));
        board.grid[4][3] = Square::new().set_piece(PieceType::new_knight(Color::White));
        board.grid[0][3] = Square::new().set_piece(PieceType::new_rook(Color::Black));

        let legal = board.legal_moves(&Coord { file: 3, rank: 4 });
        assert!(
            legal.is_empty(),
            "pinned knight must have no legal moves, got {legal:?}"
        );
    }

    #[test]
    fn test_checkmate_detected() {
        // Back-rank mate. White king on h1 boxed in by its own pawns on
        // g2/h2; black rook on e1 sweeps the first rank.
        let mut board = empty_board();
        // h1: file 7, rank 7
        board.grid[7][7] = Square::new().set_piece(PieceType::new_king(Color::White));
        // g2 and h2: friendly pawns block king's escape forward.
        board.grid[6][6] = Square::new().set_piece(PieceType::new_pawn(Color::White));
        board.grid[6][7] = Square::new().set_piece(PieceType::new_pawn(Color::White));
        // e1: black rook delivering mate along rank 7 (algebraic rank 1).
        board.grid[7][4] = Square::new().set_piece(PieceType::new_rook(Color::Black));

        let status = board.status();
        match status {
            GameStatus::Checkmate { winner } => assert_eq!(winner, Color::Black),
            other => panic!("expected Checkmate, got {other:?}"),
        }
    }

    #[test]
    fn test_stalemate_detected() {
        // Classic stalemate: black king h8, white king f7, white queen g6.
        // It's black to move; not in check; no legal moves.
        let mut board = empty_board();
        board.flags.side_to_move = Color::Black;
        // h8 = file 7, rank 0
        board.grid[0][7] = Square::new().set_piece(PieceType::new_king(Color::Black));
        // f7 = file 5, rank 1
        board.grid[1][5] = Square::new().set_piece(PieceType::new_king(Color::White));
        // g6 = file 6, rank 2
        board.grid[2][6] = Square::new().set_piece(PieceType::new_queen(Color::White));

        let status = board.status();
        assert_eq!(status, GameStatus::Stalemate, "expected stalemate");
    }

    #[test]
    fn test_no_king_means_not_in_check() {
        // Existing tests build boards with no king and call make_move on them.
        // is_in_check must not panic on a kingless board.
        let board = empty_board();
        assert!(!board.is_in_check(Color::White));
        assert!(!board.is_in_check(Color::Black));
    }

    // ============================================================
    // Plan 03: Promotion
    // ============================================================

    #[test]
    fn test_pawn_promotion_generates_four_moves() {
        let mut board = empty_board();
        // White pawn one rank short of promotion: rank 1, advances to rank 0.
        board.grid[1][3] = Square::new().set_piece(PieceType::new_pawn(Color::White));

        let moves = board.get_moves(&Coord { file: 3, rank: 1 });
        let promotion_moves: Vec<_> = moves
            .iter()
            .filter_map(|m| match &m.move_type {
                MoveType::Promotion { target, into } => {
                    Some((target.clone(), into.clone()))
                }
                _ => None,
            })
            .collect();
        assert_eq!(
            promotion_moves.len(),
            4,
            "expected 4 promotion choices for a pawn one rank from promotion, got {promotion_moves:?}"
        );
        // All four target the same square.
        for (target, _) in &promotion_moves {
            assert_eq!(target.file, 3);
            assert_eq!(target.rank, 0);
        }
    }

    #[test]
    fn test_promotion_replaces_pawn_with_queen() {
        let mut board = empty_board();
        board.grid[1][3] = Square::new().set_piece(PieceType::new_pawn(Color::White));

        let mv = GameMove {
            from: Coord { file: 3, rank: 1 },
            move_type: MoveType::Promotion {
                target: Coord { file: 3, rank: 0 },
                into: PromotionTarget::Queen,
            },
        };
        board.make_move(mv).expect("promotion to queen should succeed");

        match &board.grid[0][3].piece {
            Some(PieceType::Queen(q)) => assert_eq!(q.color, Color::White),
            other => panic!("expected white queen at (3,0), got {other:?}"),
        }
        assert!(board.grid[1][3].piece.is_none(), "source pawn cleared");
    }

    #[test]
    fn test_capture_promotion_generated() {
        let mut board = empty_board();
        // White pawn at (3,1) with a black knight at (4,0) on its capture diagonal.
        board.grid[1][3] = Square::new().set_piece(PieceType::new_pawn(Color::White));
        board.grid[0][4] = Square::new().set_piece(PieceType::new_knight(Color::Black));

        let moves = board.get_moves(&Coord { file: 3, rank: 1 });
        let capture_promotions: Vec<_> = moves
            .iter()
            .filter(|m| matches!(
                &m.move_type,
                MoveType::Promotion { target, .. }
                if target.file == 4 && target.rank == 0
            ))
            .collect();
        assert_eq!(
            capture_promotions.len(),
            4,
            "capture-promotion should also generate 4 variants, got {capture_promotions:?}"
        );
    }

    // ============================================================
    // Plan 03: Castling
    // ============================================================

    /// Standard-setup castle: white king on e1 with both rooks, clear path,
    /// no attacker — both castle moves should be generated.
    #[test]
    fn test_white_can_castle_both_sides_when_clear() {
        let mut board = empty_board();
        // e1 = file 4, rank 7
        board.grid[7][4] = Square::new().set_piece(PieceType::new_king(Color::White));
        // h1 = file 7, rank 7
        board.grid[7][7] = Square::new().set_piece(PieceType::new_rook(Color::White));
        // a1 = file 0, rank 7
        board.grid[7][0] = Square::new().set_piece(PieceType::new_rook(Color::White));

        let moves = board.get_moves(&Coord { file: 4, rank: 7 });
        let castle_sides: Vec<CastleSide> = moves
            .iter()
            .filter_map(|m| match &m.move_type {
                MoveType::Castle { side } => Some(*side),
                _ => None,
            })
            .collect();
        assert!(
            castle_sides.contains(&CastleSide::Kingside),
            "kingside castle should be generated"
        );
        assert!(
            castle_sides.contains(&CastleSide::Queenside),
            "queenside castle should be generated"
        );
    }

    #[test]
    fn test_castle_kingside_executes() {
        let mut board = empty_board();
        board.grid[7][4] = Square::new().set_piece(PieceType::new_king(Color::White));
        board.grid[7][7] = Square::new().set_piece(PieceType::new_rook(Color::White));

        let mv = GameMove {
            from: Coord { file: 4, rank: 7 },
            move_type: MoveType::Castle {
                side: CastleSide::Kingside,
            },
        };
        board.make_move(mv).expect("kingside castle should be legal");

        // King at g1 (6,7), rook at f1 (5,7).
        assert!(matches!(
            &board.grid[7][6].piece,
            Some(PieceType::King(k)) if k.color == Color::White
        ));
        assert!(matches!(
            &board.grid[7][5].piece,
            Some(PieceType::Rook(r)) if r.color == Color::White
        ));
        // Both castle flags cleared for white.
        assert!(!board.flags.white_can_castle_kingside);
        assert!(!board.flags.white_can_castle_queenside);
    }

    #[test]
    fn test_castle_blocked_by_check() {
        // White king on e1, white rook on h1 (for kingside). Black rook on e8
        // gives check down the e-file — castling forbidden.
        let mut board = empty_board();
        board.grid[7][4] = Square::new().set_piece(PieceType::new_king(Color::White));
        board.grid[7][7] = Square::new().set_piece(PieceType::new_rook(Color::White));
        board.grid[0][4] = Square::new().set_piece(PieceType::new_rook(Color::Black));

        let moves = board.get_moves(&Coord { file: 4, rank: 7 });
        let has_castle = moves
            .iter()
            .any(|m| matches!(&m.move_type, MoveType::Castle { .. }));
        assert!(!has_castle, "king in check must not be able to castle");
    }

    #[test]
    fn test_castle_blocked_by_attacked_path() {
        // White king on e1, white rook on h1. Black rook on f8 attacks f1.
        let mut board = empty_board();
        board.grid[7][4] = Square::new().set_piece(PieceType::new_king(Color::White));
        board.grid[7][7] = Square::new().set_piece(PieceType::new_rook(Color::White));
        board.grid[0][5] = Square::new().set_piece(PieceType::new_rook(Color::Black));

        let moves = board.get_moves(&Coord { file: 4, rank: 7 });
        let has_kingside_castle = moves.iter().any(|m| matches!(
            &m.move_type,
            MoveType::Castle { side: CastleSide::Kingside }
        ));
        assert!(
            !has_kingside_castle,
            "kingside castle must be blocked when f1 is attacked"
        );
    }

    #[test]
    fn test_castle_blocked_by_piece_in_path() {
        // White king e1, white rook h1, white knight on g1 blocking.
        let mut board = empty_board();
        board.grid[7][4] = Square::new().set_piece(PieceType::new_king(Color::White));
        board.grid[7][7] = Square::new().set_piece(PieceType::new_rook(Color::White));
        board.grid[7][6] = Square::new().set_piece(PieceType::new_knight(Color::White));

        let moves = board.get_moves(&Coord { file: 4, rank: 7 });
        let has_kingside_castle = moves.iter().any(|m| matches!(
            &m.move_type,
            MoveType::Castle { side: CastleSide::Kingside }
        ));
        assert!(
            !has_kingside_castle,
            "kingside castle must be blocked when g1 is occupied"
        );
    }

    #[test]
    fn test_king_move_clears_castle_flags() {
        let mut board = empty_board();
        board.grid[7][4] = Square::new().set_piece(PieceType::new_king(Color::White));

        // Move the king one square sideways (an ordinary king move).
        let mv = GameMove {
            from: Coord { file: 4, rank: 7 },
            move_type: MoveType::MoveTo(Coord { file: 4, rank: 6 }),
        };
        board.make_move(mv).expect("king move should be legal");
        assert!(!board.flags.white_can_castle_kingside);
        assert!(!board.flags.white_can_castle_queenside);
    }

    #[test]
    fn test_rook_move_clears_only_its_side() {
        let mut board = empty_board();
        // Just the white kingside rook on h1. Move it sideways.
        board.grid[7][7] = Square::new().set_piece(PieceType::new_rook(Color::White));

        let mv = GameMove {
            from: Coord { file: 7, rank: 7 },
            move_type: MoveType::MoveTo(Coord { file: 6, rank: 7 }),
        };
        board.make_move(mv).expect("rook move should be legal");
        assert!(
            !board.flags.white_can_castle_kingside,
            "kingside flag should be cleared"
        );
        assert!(
            board.flags.white_can_castle_queenside,
            "queenside flag should remain (queenside rook hasn't moved)"
        );
    }

    // ============================================================
    // Plan 03: En passant
    // ============================================================

    #[test]
    fn test_pawn_double_push_sets_en_passant_target() {
        let mut board = empty_board();
        board.grid[6][3] = Square::new().set_piece(PieceType::new_pawn(Color::White));

        let mv = GameMove {
            from: Coord { file: 3, rank: 6 },
            move_type: MoveType::MoveTo(Coord { file: 3, rank: 4 }),
        };
        board.make_move(mv).expect("double push should be legal");

        // White went from rank 6 to rank 4. EP target = rank 5, same file.
        assert_eq!(
            board.flags.en_passant_target,
            Some(Coord { file: 3, rank: 5 })
        );
    }

    #[test]
    fn test_en_passant_target_cleared_on_non_double_push() {
        let mut board = empty_board();
        board.flags.en_passant_target = Some(Coord { file: 0, rank: 5 });
        board.grid[6][3] = Square::new().set_piece(PieceType::new_pawn(Color::White));

        // Single push, not a double — ep target must reset to None.
        let mv = GameMove {
            from: Coord { file: 3, rank: 6 },
            move_type: MoveType::MoveTo(Coord { file: 3, rank: 5 }),
        };
        board.make_move(mv).expect("single push should be legal");
        assert_eq!(board.flags.en_passant_target, None);
    }

    #[test]
    fn test_en_passant_capture_executes() {
        let mut board = empty_board();
        // Black pawn at d4 (file 3, rank 4) just double-pushed; ep target is d3 (3,5).
        // Wait — for a white pawn to en-passant-capture a black pawn, the black
        // pawn must have just double-pushed *from* rank 1 *to* rank 3, putting the
        // ep target at rank 2. The white capturing pawn sits at rank 3, file 4
        // (next to the black pawn).
        board.flags.side_to_move = Color::White;
        // Black pawn at (3, 3) — having just double-pushed.
        board.grid[3][3] = Square::new().set_piece(PieceType::new_pawn(Color::Black));
        // White pawn at (4, 3).
        board.grid[3][4] = Square::new().set_piece(PieceType::new_pawn(Color::White));
        // EP target = (3, 2): the square the black pawn passed over.
        board.flags.en_passant_target = Some(Coord { file: 3, rank: 2 });

        let from = Coord { file: 4, rank: 3 };
        let moves = board.get_moves(&from);
        let ep_move = moves
            .iter()
            .find(|m| matches!(&m.move_type, MoveType::EnPassant { .. }))
            .cloned()
            .expect("an EnPassant move should be available");
        board
            .make_move(GameMove {
                from: ep_move.from.clone(),
                move_type: ep_move.move_type.clone(),
            })
            .expect("en passant capture should execute");

        // White pawn at (3, 2), black pawn (was at (3,3)) gone.
        match &board.grid[2][3].piece {
            Some(PieceType::Pawn(p)) => assert_eq!(p.color, Color::White),
            other => panic!("expected white pawn at (3,2), got {other:?}"),
        }
        assert!(board.grid[3][3].piece.is_none(), "captured pawn removed");
        assert!(board.grid[3][4].piece.is_none(), "source square cleared");
    }

    #[test]
    fn test_fen_roundtrip_with_en_passant() {
        let mut board = empty_board();
        board.flags.en_passant_target = Some(Coord { file: 3, rank: 5 });

        let fen = board_to_fen(&board);
        // d3 = file 3, rank 5 → algebraic d3.
        assert!(
            fen.ends_with(" d3"),
            "expected FEN to end with ep target ' d3', got {fen:?}"
        );
        let board2 = fen_to_board(&fen);
        assert_eq!(
            board2.flags.en_passant_target,
            Some(Coord { file: 3, rank: 5 })
        );
    }

    // ============================================================
    // Round-3 audit regression tests (post-fix coverage)
    // ============================================================

    /// Bug 1: a King passenger inside a Bus was invisible to `find_king`,
    /// so `is_in_check` returned `false` even when the Bus's square was
    /// attacked — making checkmate impossible to detect.
    #[test]
    fn test_find_king_locates_passenger_king() {
        let mut board = empty_board();
        let bus_with_king = PieceType::Bus(Bus {
            color: Color::White,
            pieces: vec![PieceType::new_king(Color::White)],
        });
        board.grid[4][4] = Square::new().set_piece(bus_with_king);

        assert_eq!(
            board.find_king(Color::White),
            Some(Coord { file: 4, rank: 4 }),
            "passenger king should resolve to the Bus's square"
        );
    }

    /// Bug 1 follow-up: a Bus carrying the king under attack registers as
    /// in-check. Without the fix this returns false silently.
    #[test]
    fn test_is_in_check_when_passenger_king_under_attack() {
        let mut board = empty_board();
        let bus_with_king = PieceType::Bus(Bus {
            color: Color::White,
            pieces: vec![PieceType::new_king(Color::White)],
        });
        board.grid[4][4] = Square::new().set_piece(bus_with_king);
        // Black rook sweeping the Bus along rank 4.
        board.grid[4][0] = Square::new().set_piece(PieceType::new_rook(Color::Black));

        assert!(
            board.is_in_check(Color::White),
            "white king inside a Bus attacked by a rook must be in check"
        );
    }

    /// Bug 2: default `Piece::attacks` extracted MoveTo from `initial_moves`
    /// which includes Monkey's *empty* single-step squares. Spec says the
    /// Monkey can't capture by single-step, so those aren't real threats —
    /// over-reporting wrongly restricted king movement.
    ///
    /// With no ladder pieces around it, a lone Monkey threatens nothing.
    #[test]
    fn test_monkey_attacks_no_threats_without_ladder() {
        let mut board = empty_board();
        board.grid[4][4] = Square::new()
            .set_piece(PieceType::Monkey(Monkey { color: Color::White }));

        // Adjacent empty squares must NOT be reported as attacked.
        for &(df, dr) in &[
            (1isize, 0isize), (-1, 0), (0, 1), (0, -1),
            (1, 1), (1, -1), (-1, 1), (-1, -1),
        ] {
            let f = (4 + df) as u8;
            let r = (4 + dr) as u8;
            assert!(
                !board.is_attacked_by(&Coord { file: f, rank: r }, Color::White),
                "empty adjacent ({f},{r}) must not be attacked by a Monkey with no ladder"
            );
        }
    }

    /// Bug 2 follow-up: Monkey *with* a ladder attacks the jump-landing
    /// square — including when that landing square is empty (because if
    /// the king walked there, the Monkey would capture).
    #[test]
    fn test_monkey_attacks_jump_landing() {
        let mut board = empty_board();
        board.grid[4][4] = Square::new()
            .set_piece(PieceType::Monkey(Monkey { color: Color::White }));
        // Ladder piece at (4,3); jump-landing (4,2) is empty.
        board.grid[3][4] = Square::new().set_piece(PieceType::new_pawn(Color::Black));

        assert!(
            board.is_attacked_by(&Coord { file: 4, rank: 2 }, Color::White),
            "Monkey jump-landing should be attacked even when empty"
        );
        // The ladder square itself is not a threat — Monkey can't capture
        // it via single-step.
        assert!(
            !board.is_attacked_by(&Coord { file: 4, rank: 3 }, Color::White),
            "Monkey doesn't threaten the ladder square (no single-step capture)"
        );
    }

    /// `make_move` returning Err must NOT flip `side_to_move`.
    #[test]
    fn test_failed_make_move_preserves_side_to_move() {
        let mut board = empty_board();
        // Black rook with side_to_move=White → wrong-turn rejection.
        board.grid[0][0] = Square::new().set_piece(PieceType::new_rook(Color::Black));

        let mv = GameMove {
            from: Coord { file: 0, rank: 0 },
            move_type: MoveType::MoveTo(Coord { file: 4, rank: 0 }),
        };
        assert!(board.make_move(mv).is_err());
        assert_eq!(
            board.flags.side_to_move,
            Color::White,
            "side_to_move must not flip on a rejected move"
        );
    }

    /// `validate_move` returns specific `MoveError` variants in the right
    /// order (NoPieceAtSource → WrongTurn → PieceCannotMakeMove →
    /// WouldLeaveKingInCheck).
    #[test]
    fn test_validate_move_returns_specific_variants() {
        let mut board = empty_board();

        // (a) NoPieceAtSource: empty source.
        let err = board
            .validate_move(&GameMove {
                from: Coord { file: 0, rank: 0 },
                move_type: MoveType::MoveTo(Coord { file: 1, rank: 0 }),
            })
            .unwrap_err();
        assert!(matches!(err, MoveError::NoPieceAtSource { .. }));

        // (b) WrongTurn: black piece while side_to_move=White.
        board.grid[0][0] = Square::new().set_piece(PieceType::new_rook(Color::Black));
        let err = board
            .validate_move(&GameMove {
                from: Coord { file: 0, rank: 0 },
                move_type: MoveType::MoveTo(Coord { file: 1, rank: 0 }),
            })
            .unwrap_err();
        assert!(matches!(err, MoveError::WrongTurn { .. }));

        // (c) PieceCannotMakeMove: white rook attempting a diagonal.
        board.grid[0][0] = Square::new().set_piece(PieceType::new_rook(Color::White));
        let err = board
            .validate_move(&GameMove {
                from: Coord { file: 0, rank: 0 },
                move_type: MoveType::MoveTo(Coord { file: 1, rank: 1 }),
            })
            .unwrap_err();
        match err {
            MoveError::PieceCannotMakeMove {
                candidate_alternatives,
                ..
            } => {
                assert!(
                    !candidate_alternatives.is_empty(),
                    "rook should have non-empty candidate set"
                );
            }
            other => panic!("expected PieceCannotMakeMove, got {other:?}"),
        }

        // (d) WouldLeaveKingInCheck: white king pinned by black rook.
        let mut pinned_board = empty_board();
        pinned_board.grid[7][3] = Square::new().set_piece(PieceType::new_king(Color::White));
        pinned_board.grid[4][3] = Square::new().set_piece(PieceType::new_knight(Color::White));
        pinned_board.grid[0][3] = Square::new().set_piece(PieceType::new_rook(Color::Black));
        // Pinned knight tries to L-move off the file.
        let err = pinned_board
            .validate_move(&GameMove {
                from: Coord { file: 3, rank: 4 },
                move_type: MoveType::MoveTo(Coord { file: 5, rank: 5 }),
            })
            .unwrap_err();
        assert!(matches!(err, MoveError::WouldLeaveKingInCheck { .. }));
    }

    /// Capture-promotion onto an enemy rook on its starting square clears
    /// that side's castle right.
    #[test]
    fn test_capture_promotion_clears_castle_flag() {
        let mut board = empty_board();
        // White pawn at b7 = (1, 1) capturing-promoting onto a8 = (0, 0).
        board.grid[1][1] = Square::new().set_piece(PieceType::new_pawn(Color::White));
        // Black rook at a8 (queenside corner).
        board.grid[0][0] = Square::new().set_piece(PieceType::new_rook(Color::Black));

        // Sanity: black queenside flag starts true.
        assert!(board.flags.black_can_castle_queenside);

        board
            .make_move(GameMove {
                from: Coord { file: 1, rank: 1 },
                move_type: MoveType::Promotion {
                    target: Coord { file: 0, rank: 0 },
                    into: PromotionTarget::Queen,
                },
            })
            .expect("capture-promotion should succeed");

        assert!(
            !board.flags.black_can_castle_queenside,
            "capturing the black queenside rook on a8 must clear its castle right"
        );
    }

    /// Any non-pawn move clears a previously-set `en_passant_target`.
    /// (Only `Pawn::post_move_effects` may set it; the reset in
    /// `handle_post_move_effects` runs before piece hooks.)
    #[test]
    fn test_non_pawn_move_clears_stale_en_passant_target() {
        let mut board = empty_board();
        board.flags.en_passant_target = Some(Coord { file: 4, rank: 5 });
        board.grid[0][0] = Square::new().set_piece(PieceType::new_rook(Color::White));

        board
            .make_move(GameMove {
                from: Coord { file: 0, rank: 0 },
                move_type: MoveType::MoveTo(Coord { file: 0, rank: 4 }),
            })
            .expect("rook slide should be legal");

        assert_eq!(
            board.flags.en_passant_target, None,
            "non-pawn move must reset ep target"
        );
    }

    /// `GameStatus::Check` is produced when the side to move is in check
    /// but has at least one legal move. Locks the variant into the spec.
    #[test]
    fn test_game_status_check_when_in_check_with_legal_moves() {
        let mut board = empty_board();
        // White king on (4,4) in check from a black rook on (4,0), with
        // plenty of legal escape squares.
        board.grid[4][4] = Square::new().set_piece(PieceType::new_king(Color::White));
        board.grid[0][4] = Square::new().set_piece(PieceType::new_rook(Color::Black));

        match board.status() {
            GameStatus::Check { side_to_move } => {
                assert_eq!(side_to_move, Color::White);
            }
            other => panic!("expected Check, got {other:?}"),
        }
    }

    /// The structured `MoveError::message()` uses `Display` formatters,
    /// not `{:?}`, so it doesn't surface Rust syntax to UI callers.
    #[test]
    fn test_move_error_message_uses_display_not_debug() {
        let board = empty_board();
        let err = board
            .validate_move(&GameMove {
                from: Coord { file: 0, rank: 0 },
                move_type: MoveType::MoveTo(Coord { file: 1, rank: 0 }),
            })
            .unwrap_err();
        let msg = err.message();
        assert!(
            !msg.contains("Coord {"),
            "message must not leak Debug formatting, got: {msg}"
        );
        // Coord's Display is board-agnostic `(file, rank)` index notation —
        // algebraic ("a8") needs the board's height and is only available
        // via `Board::format_coord`.
        assert!(
            msg.contains("(0, 0)"),
            "message should refer to source square in (file, rank) form, got: {msg}"
        );
    }
}
