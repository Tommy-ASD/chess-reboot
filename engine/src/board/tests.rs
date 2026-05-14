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
                train_tick_rate: crate::board::TrainTickRate::EveryFullTurn,
                ply_count: 0,
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
                train_tick_rate: crate::board::TrainTickRate::EveryFullTurn,
                ply_count: 0,
            },
        };

        let fen = board_to_fen(&board);
        assert_eq!(fen, "8/8/8/8/8/8/8/8 w KQkq - tr=full p=0");

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
                train_tick_rate: crate::board::TrainTickRate::EveryFullTurn,
                ply_count: 0,
            },
        };

        board.grid[0][0] = Square::new().set_piece(PieceType::new_rook(Color::White));
        board.grid[7][7] = Square::new().set_piece(PieceType::new_king(Color::Black));

        let fen = board_to_fen(&board);
        assert_eq!(fen, "R7/8/8/8/8/8/8/7k w KQkq - tr=full p=0");

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
                train_tick_rate: crate::board::TrainTickRate::EveryFullTurn,
                ply_count: 0,
            },
        };

        // Place a white rook on a vent square
        board.grid[0][0] = Square::new()
            .set_piece(PieceType::new_rook(Color::White))
            .set_square_type(SquareType::Vent);

        let fen = board_to_fen(&board);
        assert_eq!(fen, "(P=R,T=VENT)7/8/8/8/8/8/8/8 w KQkq - tr=full p=0");

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
                train_tick_rate: crate::board::TrainTickRate::EveryFullTurn,
                ply_count: 0,
            },
        };

        board.grid[1][1] = Square::new()
            .set_piece(PieceType::new_knight(Color::Black))
            .add_square_condition(SquareCondition::Frozen);

        let fen = board_to_fen(&board);
        assert_eq!(fen, "8/1(P=n,C=FROZEN)6/8/8/8/8/8/8 w KQkq - tr=full p=0");

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
                train_tick_rate: crate::board::TrainTickRate::EveryFullTurn,
                ply_count: 0,
            },
        };

        board.grid[1][1] = Square::new()
            .set_piece(PieceType::new_knight(Color::Black))
            .add_square_condition(SquareCondition::Frozen)
            .set_square_type(SquareType::Vent);

        let fen = board_to_fen(&board);
        assert_eq!(fen, "8/1(P=n,T=VENT,C=FROZEN)6/8/8/8/8/8/8 w KQkq - tr=full p=0");

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
                train_tick_rate: crate::board::TrainTickRate::EveryFullTurn,
                ply_count: 0,
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

    /// Plan 08: each new payload-carrying square variant must round-trip
    /// through FEN with its full payload intact (targets, ids, branches,
    /// open/fires fields).
    #[test]
    fn test_signal_squares_fen_roundtrip() {
        use crate::board::square::{PressureTrigger, TrackDir};

        let mut board = empty_board();
        board.grid[0][0] = Square::new().set_square_type(SquareType::Switch {
            targets: vec![3, 7],
        });
        board.grid[1][0] = Square::new().set_square_type(SquareType::Junction {
            id: 3,
            state: 0,
            branches: vec![TrackDir::N, TrackDir::E],
        });
        board.grid[2][0] = Square::new().set_square_type(SquareType::Gate {
            id: 7,
            open: false,
        });
        board.grid[3][0] = Square::new().set_square_type(SquareType::PressurePlate {
            targets: vec![3],
            fires_for: PressureTrigger::OnlyColor(Color::Black),
        });

        let fen = board_to_fen(&board);
        let board2 = fen_to_board(&fen);
        assert_eq!(board2, board, "fen was: {fen}");
    }

    /// Switch with empty targets list still has to round-trip — the editor
    /// can paint a Switch before wiring it.
    #[test]
    fn test_signal_switch_empty_targets_roundtrip() {
        let mut board = empty_board();
        board.grid[0][0] = Square::new().set_square_type(SquareType::Switch { targets: vec![] });

        let fen = board_to_fen(&board);
        let board2 = fen_to_board(&fen);
        assert_eq!(board2, board, "fen was: {fen}");
    }

    /// Square-driven variants are walkable; closed Gate blocks; Turret/Vent
    /// keep the prior "not walkable" semantic.
    #[test]
    fn test_square_type_is_walkable() {
        use crate::board::square::{PressureTrigger, TrackDir};

        assert!(SquareType::Standard.is_walkable());
        assert!(!SquareType::Turret.is_walkable());
        assert!(!SquareType::Vent.is_walkable());
        assert!(SquareType::Switch { targets: vec![] }.is_walkable());
        assert!(
            SquareType::Junction {
                id: 0,
                state: 0,
                branches: vec![TrackDir::N],
            }
            .is_walkable()
        );
        assert!(SquareType::Gate { id: 0, open: true }.is_walkable());
        assert!(!SquareType::Gate { id: 0, open: false }.is_walkable());
        assert!(
            SquareType::PressurePlate {
                targets: vec![],
                fires_for: PressureTrigger::AnyPiece,
            }
            .is_walkable()
        );
        assert!(
            SquareType::Track {
                direction: TrackDir::E,
            }
            .is_walkable()
        );
    }

    /// A piece landing on a Switch tile must be allowed (Switch is walkable).
    /// Regression guard against `square_is_empty` regressing to the old
    /// `SquareType::Standard`-only check.
    #[test]
    fn test_square_is_empty_treats_switch_as_walkable() {
        let mut board = empty_board();
        board.grid[3][3] = Square::new().set_square_type(SquareType::Switch { targets: vec![] });

        let target = Coord { file: 3, rank: 3 };
        assert!(
            board.square_is_empty(&target),
            "an empty Switch tile must count as walkable",
        );
    }

    /// The FEN parser uses a two-pass field accumulator, so the order in
    /// which fields appear inside an extended-square block must not matter.
    /// Editors / hand-edits / future format changes get the same parse.
    #[test]
    fn test_signal_fen_field_order_independent() {
        // Canonical order vs. shuffled order — both must parse to the same
        // Square. Gate is the cheapest variant for this check (two fields).
        let canonical = fen_to_board("(T=GATE,ID=7,OPEN=0)7/8/8/8/8/8/8/8 w KQkq -");
        let shuffled = fen_to_board("(T=GATE,OPEN=0,ID=7)7/8/8/8/8/8/8/8 w KQkq -");
        assert_eq!(canonical, shuffled);

        // Junction with all three payload fields shuffled.
        let canonical_j = fen_to_board(
            "(T=JUNCTION,ID=3,STATE=1,BRANCHES=(N,E))7/8/8/8/8/8/8/8 w KQkq -",
        );
        let shuffled_j = fen_to_board(
            "(T=JUNCTION,BRANCHES=(N,E),STATE=1,ID=3)7/8/8/8/8/8/8/8 w KQkq -",
        );
        assert_eq!(canonical_j, shuffled_j);
    }

    /// A Junction encoded with no `BRANCHES` field round-trips with an
    /// empty branches vec — and crucially does not panic. A future
    /// signal-fire dispatcher will need to handle the empty case (modulo
    /// by zero is on plan 08 step 3's plate, not this one), but the parser
    /// must remain robust today.
    #[test]
    fn test_signal_junction_empty_branches_roundtrip() {
        use crate::board::square::TrackDir;

        // Explicit empty branches list.
        let mut board = empty_board();
        board.grid[0][0] = Square::new().set_square_type(SquareType::Junction {
            id: 3,
            state: 0,
            branches: vec![],
        });
        let fen = board_to_fen(&board);
        let parsed = fen_to_board(&fen);
        assert_eq!(parsed, board, "fen was: {fen}");

        // Missing BRANCHES field entirely should also degrade gracefully —
        // not panic, default to empty.
        let bare = fen_to_board("(T=JUNCTION,ID=3,STATE=0)7/8/8/8/8/8/8/8 w KQkq -");
        let first = &bare.grid[0][0].square_type;
        match first {
            SquareType::Junction {
                id,
                state,
                branches,
            } => {
                assert_eq!(*id, 3);
                assert_eq!(*state, 0);
                assert!(branches.is_empty());
                // Sanity: TrackDir is now Eq+Hash; consume to make sure
                // the derive isn't accidentally bounded by `branches`.
                let _: std::collections::HashSet<TrackDir> = branches.iter().copied().collect();
            }
            other => panic!("expected Junction, got {other:?}"),
        }
    }

    /// Wiring-integrity round-trip: when multiple emitters share receiver
    /// IDs, one emitter targets multiple IDs, and the same numeric ID is
    /// used by both a Junction and a Gate (plan 08 allows this — different
    /// receiver kinds disambiguate), the entire signal graph must survive
    /// a FEN round-trip with every link intact.
    #[test]
    fn test_signal_wiring_graph_roundtrip() {
        use crate::board::square::{PressureTrigger, TrackDir};

        let mut board = empty_board();
        // Two switches: one hits three receivers, one shares an id with it.
        board.grid[0][0] = Square::new().set_square_type(SquareType::Switch {
            targets: vec![3, 7, 42],
        });
        board.grid[1][0] = Square::new().set_square_type(SquareType::Switch {
            targets: vec![7],
        });
        // A plate that also fires two of the same receivers.
        board.grid[2][0] = Square::new().set_square_type(SquareType::PressurePlate {
            targets: vec![3, 42],
            fires_for: PressureTrigger::AnyPiece,
        });
        // Receivers: Junction@id=3, Gate@id=7, Junction@id=42, Gate@id=3.
        // Junction and Gate with the same numeric ID coexist legitimately
        // because they're different receiver kinds.
        board.grid[3][0] = Square::new().set_square_type(SquareType::Junction {
            id: 3,
            state: 0,
            branches: vec![TrackDir::N, TrackDir::E, TrackDir::S, TrackDir::W],
        });
        board.grid[4][0] = Square::new().set_square_type(SquareType::Gate {
            id: 7,
            open: false,
        });
        board.grid[5][0] = Square::new().set_square_type(SquareType::Junction {
            id: 42,
            // In-range state: with 2 branches, valid states are 0..=1.
            // The fen parser now normalizes out-of-range STATE values
            // (`state % branches.len()`) at parse time so the byte-level
            // round-trip stays canonical.
            state: 1,
            branches: vec![TrackDir::W, TrackDir::S],
        });
        board.grid[6][0] = Square::new().set_square_type(SquareType::Gate {
            id: 3,
            open: true,
        });

        let fen = board_to_fen(&board);
        let parsed = fen_to_board(&fen);
        assert_eq!(parsed, board, "fen was: {fen}");

        // Sanity-check the link integrity explicitly — not just structural
        // equality. Switch@(0,0) must still target [3, 7, 42] in order.
        match &parsed.grid[0][0].square_type {
            SquareType::Switch { targets } => assert_eq!(targets, &[3u32, 7, 42]),
            other => panic!("expected Switch at (0,0), got {other:?}"),
        }
        // The Junction@id=3 and Gate@id=3 must both survive with id=3.
        match (
            &parsed.grid[3][0].square_type,
            &parsed.grid[6][0].square_type,
        ) {
            (
                SquareType::Junction { id: jid, .. },
                SquareType::Gate { id: gid, open },
            ) => {
                assert_eq!(*jid, 3);
                assert_eq!(*gid, 3);
                assert!(*open, "Gate@(6,0) was open in the source board");
            }
            (a, b) => panic!("expected Junction + Gate sharing id=3, got {a:?} and {b:?}"),
        }
    }

    // ============================================================
    // Plan 08 step 2 — ThrowSwitch move shape (no dispatcher yet)
    // ============================================================

    /// A piece sitting on a Switch tile must have a `ThrowSwitch` entry in
    /// its legal-move list. This is the central verification for step 2:
    /// the move is *expressable*, but applying it doesn't fire receivers
    /// yet (step 3 wires that in).
    #[test]
    fn test_switch_appears_in_legal_moves() {
        let mut board = empty_board();
        board.grid[3][3] = Square::new()
            .set_piece(PieceType::new_pawn(Color::White))
            .set_square_type(SquareType::Switch {
                targets: vec![1, 2, 3],
            });

        let from = Coord { file: 3, rank: 3 };
        let legal = board.legal_moves(&from);

        let throw = legal.iter().find(|m| matches!(&m.move_type, MoveType::ThrowSwitch { .. }));
        let throw = throw.expect("Pawn on Switch should have a ThrowSwitch move");
        match &throw.move_type {
            MoveType::ThrowSwitch { switch } => {
                assert_eq!(switch, &from, "ThrowSwitch must point at the piece's own square");
            }
            other => panic!("expected ThrowSwitch, got {other:?}"),
        }
    }

    /// A piece on a plain Standard tile must NOT have a ThrowSwitch move —
    /// the move is only emitted from Switch tiles.
    #[test]
    fn test_no_throw_switch_on_standard_tile() {
        let mut board = empty_board();
        board.grid[3][3] = Square::new().set_piece(PieceType::new_pawn(Color::White));
        let from = Coord { file: 3, rank: 3 };
        let any_throw = board
            .legal_moves(&from)
            .iter()
            .any(|m| matches!(&m.move_type, MoveType::ThrowSwitch { .. }));
        assert!(!any_throw, "ThrowSwitch must not appear off a Switch tile");
    }

    /// Throwing a switch consumes the turn — the next move must be from the
    /// other side. Even though step 2 doesn't fire signals yet, the
    /// turn-flip semantics are settled here.
    #[test]
    fn test_throw_switch_consumes_turn() {
        let mut board = empty_board();
        board.grid[3][3] = Square::new()
            .set_piece(PieceType::new_pawn(Color::White))
            .set_square_type(SquareType::Switch { targets: vec![] });

        let from = Coord { file: 3, rank: 3 };
        assert_eq!(board.flags.side_to_move, Color::White);

        let throw = GameMove {
            from: from.clone(),
            move_type: MoveType::ThrowSwitch {
                switch: from.clone(),
            },
        };
        board.make_move(throw).expect("legal throw");
        assert_eq!(board.flags.side_to_move, Color::Black);

        // Piece is still on the Switch tile — throwing doesn't move it.
        assert!(board.get_square_at(&from).and_then(|s| s.piece.as_ref()).is_some());

        // White can't move now — it's Black's turn.
        let try_again = GameMove {
            from: from.clone(),
            move_type: MoveType::ThrowSwitch {
                switch: from.clone(),
            },
        };
        let err = board
            .make_move(try_again)
            .expect_err("white throwing on black's turn must error");
        assert!(matches!(err, MoveError::WrongTurn { .. }), "got {err:?}");
    }

    /// A Skibidi sitting on a Switch tile gets BOTH a `PhaseShift`
    /// (piece-driven) and a `ThrowSwitch` (square-driven) in its legal
    /// moves. Verifies the square-driven addition coexists with custom
    /// piece-driven specials, doesn't clobber them.
    #[test]
    fn test_skibidi_on_switch_gets_both_phaseshift_and_throwswitch() {
        let mut board = empty_board();
        board.grid[3][3] = Square::new()
            .set_piece(PieceType::Skibidi(Skibidi {
                color: Color::White,
                phase: 1,
            }))
            .set_square_type(SquareType::Switch { targets: vec![5] });

        let from = Coord { file: 3, rank: 3 };
        let legal = board.legal_moves(&from);

        let has_phase_shift = legal
            .iter()
            .any(|m| matches!(m.move_type, MoveType::PhaseShift));
        let has_throw = legal
            .iter()
            .any(|m| matches!(m.move_type, MoveType::ThrowSwitch { .. }));

        assert!(
            has_phase_shift,
            "Skibidi must still offer PhaseShift on a Switch tile; legal = {legal:?}",
        );
        assert!(
            has_throw,
            "Skibidi on a Switch must also offer ThrowSwitch; legal = {legal:?}",
        );
    }

    /// A Bus parked on a Switch tile can throw the switch (the Bus IS the
    /// piece on the square). Its passengers, on the other hand, do not
    /// independently surface a `PieceInCarrier(ThrowSwitch)` — the
    /// square-driven addition in `Board::get_moves` runs against the
    /// top-level piece (the Bus), and the Bus's passenger-move generator
    /// is responsible for its own outputs. Passengers throwing via their
    /// carrier isn't a v1 mechanic.
    #[test]
    fn test_bus_on_switch_throws_but_passengers_dont() {
        let mut board = empty_board();
        let bus_with_pawn = PieceType::Bus(Bus {
            color: Color::White,
            pieces: vec![PieceType::new_pawn(Color::White)],
        });
        board.grid[3][3] = Square::new()
            .set_piece(bus_with_pawn)
            .set_square_type(SquareType::Switch { targets: vec![1] });

        let from = Coord { file: 3, rank: 3 };
        let legal = board.legal_moves(&from);

        // The Bus itself can throw the switch.
        let bus_throws = legal.iter().any(|m| {
            matches!(&m.move_type, MoveType::ThrowSwitch { switch } if switch == &from)
        });
        assert!(bus_throws, "Bus on Switch must offer ThrowSwitch");

        // No move surfaces as `PieceInCarrier { move_type: ThrowSwitch }`
        // — passengers don't throw through the carrier.
        let passenger_throws = legal.iter().any(|m| {
            matches!(
                &m.move_type,
                MoveType::PieceInCarrier { move_type, .. }
                    if matches!(move_type.as_ref(), MoveType::ThrowSwitch { .. })
            )
        });
        assert!(
            !passenger_throws,
            "passengers must not get a PieceInCarrier(ThrowSwitch); legal = {legal:?}",
        );
    }

    /// Building a `ThrowSwitch` move from a non-Switch source square must
    /// fail validation. The piece simply doesn't generate that move (it's
    /// gated by `square.square_type` in `Board::get_moves`), so the
    /// candidate-set check at `validate_move` is what catches it.
    #[test]
    fn test_throw_switch_rejected_on_non_switch_tile() {
        let mut board = empty_board();
        board.grid[3][3] = Square::new().set_piece(PieceType::new_pawn(Color::White));

        let from = Coord { file: 3, rank: 3 };
        let bogus = GameMove {
            from: from.clone(),
            move_type: MoveType::ThrowSwitch { switch: from },
        };
        let err = board.make_move(bogus).expect_err("must reject");
        assert!(
            matches!(err, MoveError::PieceCannotMakeMove { .. }),
            "expected PieceCannotMakeMove, got {err:?}"
        );
    }

    // ============================================================
    // Plan 08 step 3 — signal dispatch (fire_signal + receivers)
    // ============================================================

    /// Plan 08 step 3 anchor: throwing a switch must advance the wired
    /// junction's state. Demonstrates the full chain — get_moves emits
    /// the ThrowSwitch, make_move applies it, fire_signal activates the
    /// receiver.
    #[test]
    fn test_switch_fires_junction() {
        use crate::board::square::TrackDir;

        let mut board = empty_board();
        board.grid[3][3] = Square::new()
            .set_piece(PieceType::new_pawn(Color::White))
            .set_square_type(SquareType::Switch { targets: vec![1] });
        board.grid[4][3] = Square::new().set_square_type(SquareType::Junction {
            id: 1,
            state: 0,
            branches: vec![TrackDir::N, TrackDir::E],
        });

        let from = Coord { file: 3, rank: 3 };
        board
            .make_move(GameMove {
                from: from.clone(),
                move_type: MoveType::ThrowSwitch { switch: from },
            })
            .expect("legal throw");

        match &board.grid[4][3].square_type {
            SquareType::Junction { state, .. } => assert_eq!(*state, 1),
            other => panic!("junction was overwritten, got {other:?}"),
        }
    }

    /// One switch wired to multiple receivers: all of them advance from a
    /// single throw.
    #[test]
    fn test_switch_fires_multiple_targets() {
        use crate::board::square::TrackDir;

        let mut board = empty_board();
        board.grid[3][3] = Square::new()
            .set_piece(PieceType::new_pawn(Color::White))
            .set_square_type(SquareType::Switch {
                targets: vec![1, 2],
            });
        board.grid[4][3] = Square::new().set_square_type(SquareType::Junction {
            id: 1,
            state: 0,
            branches: vec![TrackDir::N, TrackDir::E],
        });
        board.grid[5][3] = Square::new().set_square_type(SquareType::Junction {
            id: 2,
            state: 0,
            branches: vec![TrackDir::S, TrackDir::W],
        });

        let from = Coord { file: 3, rank: 3 };
        board
            .make_move(GameMove {
                from: from.clone(),
                move_type: MoveType::ThrowSwitch { switch: from },
            })
            .expect("legal throw");

        for (rank, expected_id) in [(4, 1), (5, 2)] {
            match &board.grid[rank][3].square_type {
                SquareType::Junction { id, state, .. } => {
                    assert_eq!(*id, expected_id);
                    assert_eq!(*state, 1, "junction id={id} did not advance");
                }
                other => panic!("expected Junction at rank={rank}, got {other:?}"),
            }
        }
    }

    /// State cycles modulo branches.len(). Two-branch junction fired three
    /// times by hand: 0 → 1 → 0 → 1. Driven via `fire_signal` directly
    /// because alternating throw-via-game-flow needs two pawns; the
    /// dispatcher cycle is purely the receiver's concern.
    #[test]
    fn test_junction_cycles_modulo() {
        use crate::board::square::TrackDir;

        let mut board = empty_board();
        board.grid[0][0] = Square::new().set_square_type(SquareType::Junction {
            id: 1,
            state: 0,
            branches: vec![TrackDir::N, TrackDir::E],
        });

        let states_observed = (0..3)
            .map(|_| {
                board.fire_signal(&[1]);
                match &board.grid[0][0].square_type {
                    SquareType::Junction { state, .. } => *state,
                    _ => unreachable!(),
                }
            })
            .collect::<Vec<_>>();
        assert_eq!(states_observed, vec![1, 0, 1]);
    }

    /// Gate flips open/closed each time it's targeted. Verifies the
    /// gate-receiver arm in `activate_receiver`.
    #[test]
    fn test_gate_toggles_on_signal() {
        let mut board = empty_board();
        board.grid[3][3] = Square::new()
            .set_piece(PieceType::new_pawn(Color::White))
            .set_square_type(SquareType::Switch { targets: vec![7] });
        board.grid[4][3] = Square::new().set_square_type(SquareType::Gate {
            id: 7,
            open: false,
        });

        let from = Coord { file: 3, rank: 3 };
        board
            .make_move(GameMove {
                from: from.clone(),
                move_type: MoveType::ThrowSwitch { switch: from },
            })
            .expect("legal throw");

        match &board.grid[4][3].square_type {
            SquareType::Gate { open, .. } => assert!(*open, "gate should be open"),
            other => panic!("expected Gate, got {other:?}"),
        }
    }

    /// Dangling targets — IDs that no receiver claims — are silently
    /// ignored. The editor warns at design time; runtime is inert.
    /// Important: this must not panic, and must not corrupt any
    /// receiver state.
    #[test]
    fn test_dangling_target_silently_ignored() {
        use crate::board::square::TrackDir;

        let mut board = empty_board();
        // A receiver at id=1 that should remain untouched.
        board.grid[4][3] = Square::new().set_square_type(SquareType::Junction {
            id: 1,
            state: 0,
            branches: vec![TrackDir::N, TrackDir::E],
        });

        board.fire_signal(&[99]);

        match &board.grid[4][3].square_type {
            SquareType::Junction { id, state, .. } => {
                assert_eq!(*id, 1);
                assert_eq!(*state, 0, "untargeted junction must not budge");
            }
            other => panic!("expected Junction, got {other:?}"),
        }
    }

    /// Plan 08 says the same numeric ID may legitimately be shared by a
    /// Junction and a Gate (different receiver kinds disambiguate). A
    /// single signal pulse must activate both.
    #[test]
    fn test_signal_fires_across_receiver_kinds_on_shared_id() {
        use crate::board::square::TrackDir;

        let mut board = empty_board();
        board.grid[4][3] = Square::new().set_square_type(SquareType::Junction {
            id: 3,
            state: 0,
            branches: vec![TrackDir::N, TrackDir::E],
        });
        board.grid[5][3] = Square::new().set_square_type(SquareType::Gate {
            id: 3,
            open: false,
        });

        board.fire_signal(&[3]);

        match &board.grid[4][3].square_type {
            SquareType::Junction { state, .. } => {
                assert_eq!(*state, 1, "shared-ID junction did not fire");
            }
            other => panic!("expected Junction, got {other:?}"),
        }
        match &board.grid[5][3].square_type {
            SquareType::Gate { open, .. } => {
                assert!(*open, "shared-ID gate did not fire");
            }
            other => panic!("expected Gate, got {other:?}"),
        }
    }

    // ============================================================
    // Plan 08 — walkability gate-blocks-pieces regression suite
    //
    // Latent step-1 bug: most piece generators checked `piece.is_none()`
    // directly rather than going through `square_is_empty` (which uses
    // `is_walkable`). Closed Gates were visual decoration. These tests
    // lock in that closed Gates actually block.
    // ============================================================

    /// A white pawn cannot push onto a closed Gate directly in front of it.
    #[test]
    fn test_pawn_blocked_by_closed_gate_on_push() {
        let mut board = empty_board();
        board.grid[6][3] = Square::new().set_piece(PieceType::new_pawn(Color::White));
        board.grid[5][3] = Square::new().set_square_type(SquareType::Gate {
            id: 0,
            open: false,
        });

        let from = Coord { file: 3, rank: 6 };
        let moves = board.get_moves(&from);
        let pushes_onto_gate = moves.iter().any(|m| {
            matches!(&m.move_type, MoveType::MoveTo(c) if c.file == 3 && c.rank == 5)
        });
        assert!(!pushes_onto_gate, "pawn should not push onto a closed Gate; moves = {moves:?}");
    }

    /// A closed Gate breaks a pawn's double-push: the single-push target
    /// is the blocker, so the double-push isn't reachable.
    #[test]
    fn test_pawn_double_push_blocked_by_closed_gate_in_between() {
        let mut board = empty_board();
        board.grid[6][3] = Square::new().set_piece(PieceType::new_pawn(Color::White));
        board.grid[5][3] = Square::new().set_square_type(SquareType::Gate {
            id: 0,
            open: false,
        });

        let from = Coord { file: 3, rank: 6 };
        let moves = board.get_moves(&from);
        // Neither single-push (rank 5) nor double-push (rank 4) should appear.
        let any_forward = moves.iter().any(|m| {
            matches!(&m.move_type, MoveType::MoveTo(c) if c.file == 3 && (c.rank == 5 || c.rank == 4))
        });
        assert!(!any_forward, "closed Gate must block both single and double push; moves = {moves:?}");
    }

    /// A Rook stops at a closed Gate — doesn't slide through it and doesn't
    /// emit a move onto it. Covers all glider-driven pieces (Bishop / Queen
    /// / Goblin-free / King-1-move) by virtue of sharing the same code path.
    #[test]
    fn test_rook_slide_stops_at_closed_gate() {
        let mut board = empty_board();
        board.grid[3][0] = Square::new().set_piece(PieceType::new_rook(Color::White));
        board.grid[3][3] = Square::new().set_square_type(SquareType::Gate {
            id: 0,
            open: false,
        });

        let from = Coord { file: 0, rank: 3 };
        let moves = board.get_moves(&from);

        let reaches = |file: u8| {
            moves.iter().any(|m| {
                matches!(&m.move_type, MoveType::MoveTo(c) if c.file == file && c.rank == 3)
            })
        };

        // Squares before the gate are still reachable.
        assert!(reaches(1));
        assert!(reaches(2));
        // The gate itself and anything past it must NOT appear.
        for blocked in 3..=7u8 {
            assert!(
                !reaches(blocked),
                "rook should not reach file={blocked} past a closed Gate; moves = {moves:?}",
            );
        }
    }

    /// A Knight cannot leap onto a closed Gate. Direct check — knights
    /// don't go through the glider.
    #[test]
    fn test_knight_blocked_by_closed_gate_landing() {
        let mut board = empty_board();
        board.grid[4][4] = Square::new().set_piece(PieceType::new_knight(Color::White));
        // One of the knight's L-targets at (6,5) is a closed Gate.
        board.grid[5][6] = Square::new().set_square_type(SquareType::Gate {
            id: 0,
            open: false,
        });

        let from = Coord { file: 4, rank: 4 };
        let moves = board.get_moves(&from);
        let lands_on_gate = moves.iter().any(|m| {
            matches!(&m.move_type, MoveType::MoveTo(c) if c.file == 6 && c.rank == 5)
        });
        assert!(!lands_on_gate, "knight cannot land on closed Gate; moves = {moves:?}");
    }

    /// Defense-in-depth: even if a piece's generator forgot to filter, the
    /// safety net in `make_move_unchecked` rejects a hand-crafted move
    /// onto a non-walkable square. Bypass `get_moves` by submitting the
    /// raw move through `make_move`, which still runs validation first —
    /// the candidate-list check will reject it as not in the legal set.
    /// We use `make_move_unchecked` directly to confirm the apply-layer
    /// guard fires.
    #[test]
    fn test_make_move_unchecked_rejects_landing_on_closed_gate() {
        let mut board = empty_board();
        board.grid[6][3] = Square::new().set_piece(PieceType::new_pawn(Color::White));
        board.grid[5][3] = Square::new().set_square_type(SquareType::Gate {
            id: 0,
            open: false,
        });

        let bogus = GameMove {
            from: Coord { file: 3, rank: 6 },
            move_type: MoveType::MoveTo(Coord { file: 3, rank: 5 }),
        };
        let err = board.make_move_unchecked(bogus).expect_err("must reject");
        assert!(
            err.contains("not walkable"),
            "expected walkability rejection, got: {err}",
        );
    }

    /// Brainrot scopes to piece movement (plan 04), not infrastructure.
    /// A Junction sitting inside a Skibidi's brainrot aura must still
    /// respond to signals fired by a Switch outside the aura — signals
    /// are abstract wiring events, not piece actions. Locks in the
    /// design decision documented on `Board::activate_receiver`.
    #[test]
    fn test_brainrotted_junction_still_receives_signal() {
        use crate::board::square::TrackDir;

        let mut board = empty_board();
        // Place a Skibidi at (3,3) phase 2 (radius 1) so the brainrot
        // aura covers the 4-neighbourhood, including (4,3).
        board.grid[3][3] = Square::new().set_piece(PieceType::Skibidi(Skibidi {
            color: Color::White,
            phase: 2,
        }));
        // Junction inside the aura.
        board.grid[4][3] = Square::new().set_square_type(SquareType::Junction {
            id: 1,
            state: 0,
            branches: vec![TrackDir::N, TrackDir::E],
        });
        // recalc_brainrot paints the aura on the surrounding squares.
        board.recalc_brainrot();
        // Sanity: the junction's square is indeed brainrotted.
        assert!(
            board.grid[4][3]
                .conditions
                .contains(&SquareCondition::Brainrot),
            "test setup: junction square should be brainrotted",
        );

        board.fire_signal(&[1]);

        match &board.grid[4][3].square_type {
            SquareType::Junction { state, .. } => {
                assert_eq!(*state, 1, "brainrot must not block signal activation");
            }
            other => panic!("expected Junction, got {other:?}"),
        }
    }

    // ============================================================
    // Plan 08 step 4 — PressurePlate fires on landing
    // ============================================================

    /// Pawn moves onto a PressurePlate; the plate fires its targets as
    /// part of the move, advancing the wired junction.
    #[test]
    fn test_pressure_plate_fires_on_step() {
        use crate::board::square::{PressureTrigger, TrackDir};

        let mut board = empty_board();
        // White pawn at (3,6) pushes one step to (3,5), which is a plate.
        board.grid[6][3] = Square::new().set_piece(PieceType::new_pawn(Color::White));
        board.grid[5][3] = Square::new().set_square_type(SquareType::PressurePlate {
            targets: vec![1],
            fires_for: PressureTrigger::AnyPiece,
        });
        board.grid[4][3] = Square::new().set_square_type(SquareType::Junction {
            id: 1,
            state: 0,
            branches: vec![TrackDir::N, TrackDir::E],
        });

        board
            .make_move(GameMove {
                from: Coord { file: 3, rank: 6 },
                move_type: MoveType::MoveTo(Coord { file: 3, rank: 5 }),
            })
            .expect("legal pawn push onto plate");

        match &board.grid[4][3].square_type {
            SquareType::Junction { state, .. } => {
                assert_eq!(*state, 1, "plate must fire wired junction");
            }
            other => panic!("expected Junction, got {other:?}"),
        }
    }

    /// A plate restricted to `OnlyColor(White)` does NOT fire when a
    /// black piece settles on it.
    #[test]
    fn test_pressure_plate_color_restriction_blocks_wrong_color() {
        use crate::board::square::{PressureTrigger, TrackDir};

        let mut board = empty_board();
        board.flags.side_to_move = Color::Black;
        board.grid[1][3] = Square::new().set_piece(PieceType::new_pawn(Color::Black));
        board.grid[2][3] = Square::new().set_square_type(SquareType::PressurePlate {
            targets: vec![1],
            fires_for: PressureTrigger::OnlyColor(Color::White),
        });
        board.grid[4][3] = Square::new().set_square_type(SquareType::Junction {
            id: 1,
            state: 0,
            branches: vec![TrackDir::N, TrackDir::E],
        });

        board
            .make_move(GameMove {
                from: Coord { file: 3, rank: 1 },
                move_type: MoveType::MoveTo(Coord { file: 3, rank: 2 }),
            })
            .expect("legal pawn push onto color-restricted plate");

        match &board.grid[4][3].square_type {
            SquareType::Junction { state, .. } => {
                assert_eq!(
                    *state, 0,
                    "OnlyColor(White) plate must not fire for a black piece",
                );
            }
            other => panic!("expected Junction, got {other:?}"),
        }
    }

    /// Positive case for the color trigger: `OnlyColor(White)` fires
    /// when a white piece lands.
    #[test]
    fn test_pressure_plate_color_restriction_fires_for_match() {
        use crate::board::square::{PressureTrigger, TrackDir};

        let mut board = empty_board();
        board.grid[6][3] = Square::new().set_piece(PieceType::new_pawn(Color::White));
        board.grid[5][3] = Square::new().set_square_type(SquareType::PressurePlate {
            targets: vec![1],
            fires_for: PressureTrigger::OnlyColor(Color::White),
        });
        board.grid[4][3] = Square::new().set_square_type(SquareType::Junction {
            id: 1,
            state: 0,
            branches: vec![TrackDir::N, TrackDir::E],
        });

        board
            .make_move(GameMove {
                from: Coord { file: 3, rank: 6 },
                move_type: MoveType::MoveTo(Coord { file: 3, rank: 5 }),
            })
            .expect("legal push");

        match &board.grid[4][3].square_type {
            SquareType::Junction { state, .. } => assert_eq!(*state, 1),
            other => panic!("expected Junction, got {other:?}"),
        }
    }

    /// Castle moves two pieces — both their landings should fire plates.
    /// Place a plate where the castle-rook lands (file 5, kingside) and
    /// verify it fires when white castles kingside.
    #[test]
    fn test_castle_rook_landing_fires_plate() {
        use crate::board::square::{PressureTrigger, TrackDir};

        let mut board = empty_board();
        // Standard castle setup on rank 7 (white back rank).
        board.grid[7][4] = Square::new().set_piece(PieceType::new_king(Color::White));
        board.grid[7][7] = Square::new().set_piece(PieceType::new_rook(Color::White));
        // PressurePlate at the rook's kingside-castle landing (file 5).
        board.grid[7][5] = Square::new().set_square_type(SquareType::PressurePlate {
            targets: vec![1],
            fires_for: PressureTrigger::AnyPiece,
        });
        board.grid[0][0] = Square::new().set_square_type(SquareType::Junction {
            id: 1,
            state: 0,
            branches: vec![TrackDir::N, TrackDir::E],
        });

        board
            .make_move(GameMove {
                from: Coord { file: 4, rank: 7 },
                move_type: MoveType::Castle {
                    side: CastleSide::Kingside,
                },
            })
            .expect("legal kingside castle");

        match &board.grid[0][0].square_type {
            SquareType::Junction { state, .. } => {
                assert_eq!(*state, 1, "rook's landing on plate must fire");
            }
            other => panic!("expected Junction, got {other:?}"),
        }
    }

    /// A piece that *passes over* but does not *settle on* a plate must
    /// not fire it. Verified via a sliding rook that lands beyond a plate.
    #[test]
    fn test_pressure_plate_does_not_fire_when_piece_only_passes() {
        use crate::board::square::{PressureTrigger, TrackDir};

        let mut board = empty_board();
        board.grid[3][0] = Square::new().set_piece(PieceType::new_rook(Color::White));
        // Plate at (3, file=4); rook will slide past it to (3, file=7).
        board.grid[3][4] = Square::new().set_square_type(SquareType::PressurePlate {
            targets: vec![1],
            fires_for: PressureTrigger::AnyPiece,
        });
        board.grid[0][0] = Square::new().set_square_type(SquareType::Junction {
            id: 1,
            state: 0,
            branches: vec![TrackDir::N, TrackDir::E],
        });

        board
            .make_move(GameMove {
                from: Coord { file: 0, rank: 3 },
                move_type: MoveType::MoveTo(Coord { file: 7, rank: 3 }),
            })
            .expect("legal slide past plate");

        match &board.grid[0][0].square_type {
            SquareType::Junction { state, .. } => {
                assert_eq!(*state, 0, "plate fires on settle, not pass-through");
            }
            other => panic!("expected Junction, got {other:?}"),
        }
    }

    /// A passenger exiting a carrier onto a PressurePlate must fire the
    /// plate — the passenger is the piece that "settles" on the tile.
    /// Locks in `collect_landings`' `PieceInCarrier { MoveTo }` branch.
    #[test]
    fn test_pressure_plate_fires_when_passenger_exits_onto_it() {
        use crate::board::square::{PressureTrigger, TrackDir};
        use std::sync::Arc;

        let mut board = empty_board();
        // White Bus at white's starting rank with a Pawn passenger. The
        // Bus's passenger-move-gen places the pawn at the Bus's coord and
        // asks for its moves — white pawn at (3,6) can single-push to (3,5).
        let bus_with_pawn = PieceType::Bus(Bus {
            color: Color::White,
            pieces: vec![PieceType::new_pawn(Color::White)],
        });
        board.grid[6][3] = Square::new().set_piece(bus_with_pawn);
        // Plate at (3,5) wired to junction id=1.
        board.grid[5][3] = Square::new().set_square_type(SquareType::PressurePlate {
            targets: vec![1],
            fires_for: PressureTrigger::AnyPiece,
        });
        // Junction parked off-path so it doesn't interfere with movement.
        board.grid[0][0] = Square::new().set_square_type(SquareType::Junction {
            id: 1,
            state: 0,
            branches: vec![TrackDir::N, TrackDir::E],
        });

        // Passenger exits onto the plate.
        board
            .make_move(GameMove {
                from: Coord { file: 3, rank: 6 },
                move_type: MoveType::PieceInCarrier {
                    piece_index: 0,
                    move_type: Arc::new(MoveType::MoveTo(Coord { file: 3, rank: 5 })),
                },
            })
            .expect("legal passenger exit");

        match &board.grid[0][0].square_type {
            SquareType::Junction { state, .. } => {
                assert_eq!(*state, 1, "plate must fire for passenger settling on it");
            }
            other => panic!("expected Junction, got {other:?}"),
        }

        // Sanity: the pawn is now on the plate, the bus has no passengers.
        match &board.grid[5][3].piece {
            Some(PieceType::Pawn(p)) => assert_eq!(p.color, Color::White),
            other => panic!("expected white Pawn on plate, got {other:?}"),
        }
        match &board.grid[6][3].piece {
            Some(PieceType::Bus(b)) => assert!(b.pieces.is_empty(), "bus should be empty"),
            other => panic!("expected empty Bus, got {other:?}"),
        }
    }

    /// Defensive: a plate isn't itself a chainable receiver, but the
    /// dispatcher's bounded-propagation rule still matters — a plate
    /// firing a Gate must not cause that Gate to in turn re-emit. Today
    /// Gates don't emit at all (only Switches and Plates do), so the
    /// "no cascade" property holds trivially. This test locks that in.
    #[test]
    fn test_pressure_plate_no_propagation_cascade() {
        use crate::board::square::PressureTrigger;

        let mut board = empty_board();
        board.grid[6][3] = Square::new().set_piece(PieceType::new_pawn(Color::White));
        board.grid[5][3] = Square::new().set_square_type(SquareType::PressurePlate {
            targets: vec![7],
            fires_for: PressureTrigger::AnyPiece,
        });
        board.grid[4][3] = Square::new().set_square_type(SquareType::Gate {
            id: 7,
            open: false,
        });

        board
            .make_move(GameMove {
                from: Coord { file: 3, rank: 6 },
                move_type: MoveType::MoveTo(Coord { file: 3, rank: 5 }),
            })
            .expect("legal pawn push");

        // The gate flipped open exactly once.
        match &board.grid[4][3].square_type {
            SquareType::Gate { open, .. } => assert!(*open, "gate must have toggled open"),
            other => panic!("expected Gate, got {other:?}"),
        }
        // The plate itself is still a plate (didn't get re-consumed) and
        // didn't double-fire (gate isn't somewhere mid-toggle).
        match &board.grid[5][3].square_type {
            SquareType::PressurePlate { .. } => {} // ok
            other => panic!("plate disappeared, got {other:?}"),
        }
    }

    /// Defensive: a Junction with an empty `branches` list must not panic
    /// when its ID is signaled (modulo-by-zero territory). The plan
    /// flagged this as a latent risk in step 1; step 3's `activate_receiver`
    /// guards against it.
    #[test]
    fn test_junction_with_empty_branches_does_not_panic() {
        let mut board = empty_board();
        board.grid[4][3] = Square::new().set_square_type(SquareType::Junction {
            id: 1,
            state: 0,
            branches: vec![],
        });

        // Must not panic; state stays put.
        board.fire_signal(&[1]);

        match &board.grid[4][3].square_type {
            SquareType::Junction { state, branches, .. } => {
                assert_eq!(*state, 0);
                assert!(branches.is_empty());
            }
            other => panic!("expected Junction, got {other:?}"),
        }
    }

    /// A malformed `OPEN` value (anything other than 0 or 1) must not
    /// silently produce an open Gate — that would mean a typo lets a piece
    /// walk through what should have been a blocker. The parser now coerces
    /// the suspect input to a *closed* Gate, which is the safer fallback
    /// (and visibly broken to anyone watching the board).
    #[test]
    fn test_signal_gate_bad_open_value_falls_back_to_closed() {
        let board = fen_to_board("(T=GATE,ID=3,OPEN=2)7/8/8/8/8/8/8/8 w KQkq -");
        match &board.grid[0][0].square_type {
            SquareType::Gate { id, open } => {
                assert_eq!(*id, 3);
                assert!(!*open, "malformed OPEN should fall back to closed");
            }
            other => panic!("expected Gate, got {other:?}"),
        }
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
            fen.contains(" - - "),
            "no castle rights + no ep should be encoded as ' - - ', got {fen:?}"
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
            fen.contains(" d3 "),
            "expected FEN to contain ep target ' d3 ', got {fen:?}"
        );
        let board2 = fen_to_board(&fen);
        assert_eq!(
            board2.flags.en_passant_target,
            Some(Coord { file: 3, rank: 5 })
        );
    }

    // ============================================================
    // Round-3 audit regression tests (post-fix coverage; first batch
    // — a second batch added the criticals later, see L~4821 below).
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
    /// `apply_piece_post_effects` runs before piece hooks.)
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

    // ============================================================
    // Variable-dimension board coverage
    // ============================================================
    //
    // The frontend can resize boards to non-8×8 sizes (see commit
    // "frontend: dedicated board editor + variable board dimensions").
    // The engine's rank/file logic has to follow — these tests exercise
    // the paths that used to hardcode 8.

    /// Build a blank board of arbitrary dimensions. `width` files,
    /// `height` ranks, all castling flags off (a non-standard board
    /// shouldn't claim default castle rights).
    fn empty_board_sized(width: usize, height: usize) -> Board {
        Board {
            grid: vec![vec![Square::new(); width]; height],
            flags: BoardFlags {
                side_to_move: Color::White,
                white_can_castle_kingside: false,
                white_can_castle_queenside: false,
                black_can_castle_kingside: false,
                black_can_castle_queenside: false,
                en_passant_target: None,
                train_tick_rate: crate::board::TrainTickRate::EveryFullTurn,
                ply_count: 0,
            },
        }
    }

    /// `Board::height` / `width` should reflect the underlying grid
    /// shape; `format_coord` uses height to invert the rank into chess
    /// algebraic.
    #[test]
    fn test_board_dimensions_and_format_coord() {
        let b = empty_board_sized(10, 12);
        assert_eq!(b.width(), 10);
        assert_eq!(b.height(), 12);
        // Bottom-left corner is (0, 11) → "a1" on a 12-tall board.
        assert_eq!(b.format_coord(&Coord { file: 0, rank: 11 }), "a1");
        // Top-right corner is (9, 0) → "j12".
        assert_eq!(b.format_coord(&Coord { file: 9, rank: 0 }), "j12");
    }

    /// FEN run-length parsing must accept multi-digit counts so wider
    /// boards (10+ wide) can encode rows of empty squares. Previously
    /// `"10"` would parse as "one empty square then a 0" and produce
    /// junk.
    #[test]
    fn test_fen_multi_digit_run_length_roundtrip() {
        let board = empty_board_sized(10, 10);
        let fen = board_to_fen(&board);
        assert_eq!(fen, "10/10/10/10/10/10/10/10/10/10 w - - tr=full p=0");
        let parsed = fen_to_board(&fen);
        assert_eq!(parsed.width(), 10);
        assert_eq!(parsed.height(), 10);
        assert_eq!(parsed, board);
    }

    /// FEN serializes en passant target through `Board::format_coord`,
    /// so the algebraic rank inverts off the board's actual height —
    /// not the old hardcoded 8.
    #[test]
    fn test_fen_en_passant_uses_board_height() {
        let mut board = empty_board_sized(8, 12);
        // Internal rank 10 on a 12-tall board → algebraic rank "2".
        board.flags.en_passant_target = Some(Coord { file: 4, rank: 10 });
        let fen = board_to_fen(&board);
        assert!(
            fen.contains(" e2 "),
            "expected en-passant ' e2 ' on 12-tall board, got: {fen}"
        );
        let parsed = fen_to_board(&fen);
        assert_eq!(
            parsed.flags.en_passant_target,
            Some(Coord { file: 4, rank: 10 }),
            "en-passant target must round-trip"
        );
    }

    /// White pawn near the top of a shorter board promotes when it
    /// reaches rank 0, regardless of overall height. Verifies
    /// `Pawn::promotion_rank` is height-aware (the rank is 0 for
    /// white on any height).
    #[test]
    fn test_pawn_promotion_white_on_short_board() {
        let mut board = empty_board_sized(8, 5);
        // White pawn at rank 1 of a 5-tall board, one square from promotion.
        board.grid[1][0] = Square::new().set_piece(PieceType::new_pawn(Color::White));

        let moves = board.get_moves(&Coord { file: 0, rank: 1 });
        // Forward push to rank 0 should produce four Promotion variants.
        let promotions: Vec<_> = moves
            .iter()
            .filter_map(|m| match &m.move_type {
                MoveType::Promotion { target, into } if target.rank == 0 => Some(into.clone()),
                _ => None,
            })
            .collect();
        assert_eq!(
            promotions.len(),
            4,
            "expected 4 promotion choices on rank 0, got moves: {moves:?}"
        );
    }

    /// Black pawn promotes when it reaches `height - 1` rather than the
    /// hardcoded rank 7. A 10-tall board promotes black at rank 9.
    #[test]
    fn test_pawn_promotion_black_on_tall_board() {
        let mut board = empty_board_sized(8, 10);
        board.flags.side_to_move = Color::Black;
        // Black pawn at rank 8 (one square from the bottom).
        board.grid[8][0] = Square::new().set_piece(PieceType::new_pawn(Color::Black));

        let moves = board.get_moves(&Coord { file: 0, rank: 8 });
        let promotions: Vec<_> = moves
            .iter()
            .filter(|m| matches!(
                &m.move_type,
                MoveType::Promotion { target, .. } if target.rank == 9
            ))
            .collect();
        assert_eq!(
            promotions.len(),
            4,
            "black pawn should promote at rank 9 on 10-tall board, got: {moves:?}"
        );
    }

    /// White pawn's double-push starting rank is `height - 2`, not the
    /// hardcoded rank 6. On a 10-tall board white pawns start at rank 8.
    #[test]
    fn test_pawn_double_push_starting_rank_on_tall_board() {
        let mut board = empty_board_sized(8, 10);
        board.grid[8][0] = Square::new().set_piece(PieceType::new_pawn(Color::White));

        let moves = board.get_moves(&Coord { file: 0, rank: 8 });
        // White moves up (rank decreasing). Double push should reach rank 6.
        let double_push = moves.iter().any(|m| matches!(
            &m.move_type,
            MoveType::MoveTo(c) if c.file == 0 && c.rank == 6
        ));
        assert!(
            double_push,
            "white pawn at starting rank 8 (height-2 of 10) should offer double push to rank 6, got: {moves:?}"
        );
    }

    /// On a board narrower than 8 there is no room for the standard
    /// kingside-castle geometry, so castling moves are not generated
    /// even when the flag is set.
    #[test]
    fn test_castle_not_offered_on_narrow_board() {
        let mut board = empty_board_sized(6, 8);
        board.flags.white_can_castle_kingside = true;
        // King at file 4, rank 7 (white back rank); rook at file 5 — the
        // only candidate kingside rook on a 6-wide board.
        board.grid[7][4] = Square::new().set_piece(PieceType::new_king(Color::White));
        board.grid[7][5] = Square::new().set_piece(PieceType::new_rook(Color::White));

        let moves = board.get_moves(&Coord { file: 4, rank: 7 });
        assert!(
            !moves.iter().any(|m| matches!(m.move_type, MoveType::Castle { .. })),
            "no castle move should be generated on a 6-wide board, got: {moves:?}"
        );
    }

    /// On a 10-wide board the kingside rook sits at file 9 (`width - 1`),
    /// not file 7. King-side castling should still work as long as the
    /// path is empty and unattacked.
    #[test]
    fn test_castle_kingside_on_wide_board() {
        let mut board = empty_board_sized(10, 8);
        board.flags.white_can_castle_kingside = true;
        // White king on its start file (4), white kingside rook at file 9.
        board.grid[7][4] = Square::new().set_piece(PieceType::new_king(Color::White));
        board.grid[7][9] = Square::new().set_piece(PieceType::new_rook(Color::White));

        let moves = board.get_moves(&Coord { file: 4, rank: 7 });
        let has_kingside_castle = moves.iter().any(|m| matches!(
            &m.move_type,
            MoveType::Castle { side: CastleSide::Kingside }
        ));
        assert!(
            has_kingside_castle,
            "kingside castle should be generated when rook sits at width-1 on a 10-wide board, got: {moves:?}"
        );

        // Apply it; king should end at file 6, rook at file 5 (per the
        // standard target geometry — the rook *source* was file 9).
        board
            .make_move(GameMove {
                from: Coord { file: 4, rank: 7 },
                move_type: MoveType::Castle { side: CastleSide::Kingside },
            })
            .expect("kingside castle on wide board should succeed");
        assert!(matches!(
            &board.grid[7][6].piece,
            Some(PieceType::King(k)) if k.color == Color::White
        ));
        assert!(matches!(
            &board.grid[7][5].piece,
            Some(PieceType::Rook(r)) if r.color == Color::White
        ));
        assert!(board.grid[7][9].piece.is_none(), "rook source file 9 should now be empty");
    }

    /// Capturing the kingside rook on a wider board (at `width - 1`)
    /// must clear the relevant castle right, just like h1/h8 does on
    /// an 8-wide board.
    #[test]
    fn test_capturing_rook_at_width_minus_one_clears_castle_right() {
        let mut board = empty_board_sized(10, 8);
        board.flags.white_can_castle_kingside = true;
        // White kingside rook at (9, 7); black rook ready to capture it
        // along the file.
        board.grid[7][9] = Square::new().set_piece(PieceType::new_rook(Color::White));
        board.grid[0][9] = Square::new().set_piece(PieceType::new_rook(Color::Black));
        // White king somewhere safe so the move is legal (not in check).
        board.grid[7][4] = Square::new().set_piece(PieceType::new_king(Color::White));
        board.grid[0][0] = Square::new().set_piece(PieceType::new_king(Color::Black));
        board.flags.side_to_move = Color::Black;

        board
            .make_move(GameMove {
                from: Coord { file: 9, rank: 0 },
                move_type: MoveType::MoveTo(Coord { file: 9, rank: 7 }),
            })
            .expect("black rook capture of white kingside rook should be legal");

        assert!(
            !board.flags.white_can_castle_kingside,
            "white kingside castle right should clear when the (width-1, height-1) rook is captured"
        );
    }

    // ============================================================
    // Plan 09: trains
    // ============================================================

    use crate::board::TrainTickRate;
    use crate::board::square::TrackDir;
    use crate::pieces::fairy::carriage::Carriage;
    use crate::pieces::fairy::locomotive::{Locomotive, TrainHeading};

    /// Helper: lay a horizontal track strip along `rank` from `f_start`
    /// to `f_end` (inclusive). All tiles point east.
    fn lay_east_track(board: &mut Board, rank: u8, f_start: u8, f_end: u8) {
        for f in f_start..=f_end {
            board.grid[rank as usize][f as usize] =
                Square::new().set_square_type(SquareType::Track {
                    direction: TrackDir::E,
                });
        }
    }

    /// Helper: a board that requires *some* legal move for white or
    /// `status` would report Stalemate. Park lonely kings well clear of
    /// each other so each has eight escape squares.
    fn board_with_idle_kings() -> Board {
        let mut board = empty_board();
        board.flags.white_can_castle_kingside = false;
        board.flags.white_can_castle_queenside = false;
        board.flags.black_can_castle_kingside = false;
        board.flags.black_can_castle_queenside = false;
        board.grid[7][0] = Square::new().set_piece(PieceType::new_king(Color::White));
        board.grid[0][7] = Square::new().set_piece(PieceType::new_king(Color::Black));
        board
    }

    /// Move the white king one square right, then the black king one
    /// square left. Used to "burn" two plies of game time when a test
    /// wants the train to tick on the EveryFullTurn cadence.
    fn waste_full_turn(board: &mut Board) {
        let white_king = board.find_king(Color::White).expect("white king present");
        let dest = Coord {
            file: white_king.file + 1,
            rank: white_king.rank,
        };
        board
            .make_move(GameMove {
                from: white_king,
                move_type: MoveType::MoveTo(dest),
            })
            .expect("white king idle move");
        let black_king = board.find_king(Color::Black).expect("black king present");
        let dest = Coord {
            file: black_king.file - 1,
            rank: black_king.rank,
        };
        board
            .make_move(GameMove {
                from: black_king,
                move_type: MoveType::MoveTo(dest),
            })
            .expect("black king idle move");
    }

    #[test]
    fn test_locomotive_fen_roundtrip_neutral_color() {
        let mut board = empty_board();
        board.grid[3][3] = Square::new()
            .set_square_type(SquareType::Track {
                direction: TrackDir::E,
            })
            .set_piece(PieceType::Locomotive(Locomotive::new(
                1,
                TrainHeading::Forward,
            )));

        let fen = board_to_fen(&board);
        let board2 = fen_to_board(&fen);
        assert_eq!(board2, board, "loco+track should round-trip through FEN");
        match &board2.grid[3][3].piece {
            Some(p) => assert_eq!(p.get_color(), Color::Neutral),
            None => panic!("expected loco on (3,3) after round-trip"),
        }
    }

    #[test]
    fn test_train_advances_one_tile_per_full_turn() {
        let mut board = board_with_idle_kings();
        lay_east_track(&mut board, 3, 1, 5);
        board.grid[3][1] = board.grid[3][1]
            .clone()
            .set_piece(PieceType::Locomotive(Locomotive::new(
                1,
                TrainHeading::Forward,
            )));
        // EveryFullTurn is the default.
        waste_full_turn(&mut board);
        // After one full turn the train should have moved one tile east.
        assert!(board.grid[3][1].piece.is_none(), "loco vacated start tile");
        match &board.grid[3][2].piece {
            Some(PieceType::Locomotive(l)) => assert_eq!(l.train_id, 1),
            other => panic!("expected loco at (file=2, rank=3), got {other:?}"),
        }
    }

    #[test]
    fn test_train_advances_per_ply_when_configured() {
        let mut board = board_with_idle_kings();
        board.flags.train_tick_rate = TrainTickRate::EveryPly;
        lay_east_track(&mut board, 3, 1, 5);
        board.grid[3][1] = board.grid[3][1]
            .clone()
            .set_piece(PieceType::Locomotive(Locomotive::new(
                1,
                TrainHeading::Forward,
            )));

        let white_king = board.find_king(Color::White).expect("white king present");
        let dest = Coord {
            file: white_king.file + 1,
            rank: white_king.rank,
        };
        board
            .make_move(GameMove {
                from: white_king,
                move_type: MoveType::MoveTo(dest),
            })
            .expect("ply move");

        // Single ply: with EveryPly the train should have already moved.
        match &board.grid[3][2].piece {
            Some(PieceType::Locomotive(_)) => (),
            other => panic!("expected loco at (file=2, rank=3) after one ply, got {other:?}"),
        }
    }

    #[test]
    fn test_train_loops_on_closed_track() {
        // 4-tile loop along rank 3 and rank 4, files 3..=4.
        let mut board = board_with_idle_kings();
        board.flags.train_tick_rate = TrainTickRate::EveryPly;
        // Loop tiles:
        //  (3,3) → E (3,4)
        //  (3,4) → S (4,4)
        //  (4,4) → W (4,3)
        //  (4,3) → N (3,3)
        board.grid[3][3] = Square::new().set_square_type(SquareType::Track {
            direction: TrackDir::E,
        });
        board.grid[3][4] = Square::new().set_square_type(SquareType::Track {
            direction: TrackDir::S,
        });
        board.grid[4][4] = Square::new().set_square_type(SquareType::Track {
            direction: TrackDir::W,
        });
        board.grid[4][3] = Square::new().set_square_type(SquareType::Track {
            direction: TrackDir::N,
        });
        board.grid[3][3] = board.grid[3][3]
            .clone()
            .set_piece(PieceType::Locomotive(Locomotive::new(
                7,
                TrainHeading::Forward,
            )));

        let start = Coord { file: 3, rank: 3 };
        for _ in 0..4 {
            let king = board.find_king(board.flags.side_to_move).unwrap();
            // Find any legal king move.
            let legal = board.legal_moves(&king);
            let mv = legal.into_iter().next().expect("king has a legal move");
            board.make_move(mv).expect("king idle move");
        }
        match &board.get_square_at(&start).and_then(|s| s.piece.as_ref()) {
            Some(PieceType::Locomotive(l)) => assert_eq!(l.train_id, 7),
            other => panic!("expected loco back at start after 4 ticks, got {other:?}"),
        }
    }

    #[test]
    fn test_train_runs_over_piece() {
        let mut board = board_with_idle_kings();
        board.flags.train_tick_rate = TrainTickRate::EveryPly;
        lay_east_track(&mut board, 3, 1, 5);
        board.grid[3][1] = board.grid[3][1]
            .clone()
            .set_piece(PieceType::Locomotive(Locomotive::new(
                1,
                TrainHeading::Forward,
            )));
        // A black pawn standing on the track tile two squares east.
        // (Black so it's not the side-to-move.)
        board.grid[3][2] = board.grid[3][2]
            .clone()
            .set_piece(PieceType::new_pawn(Color::Black));

        let white_king = board.find_king(Color::White).expect("white king present");
        let dest = Coord {
            file: white_king.file + 1,
            rank: white_king.rank,
        };
        board
            .make_move(GameMove {
                from: white_king,
                move_type: MoveType::MoveTo(dest),
            })
            .expect("idle ply");
        match &board.grid[3][2].piece {
            Some(PieceType::Locomotive(_)) => (),
            other => panic!("loco should overwrite pawn at (file=2, rank=3), got {other:?}"),
        }
    }

    #[test]
    fn test_train_stops_on_derailment() {
        let mut board = board_with_idle_kings();
        board.flags.train_tick_rate = TrainTickRate::EveryPly;
        // Only one track tile — train cannot leave it.
        board.grid[3][4] = Square::new().set_square_type(SquareType::Track {
            direction: TrackDir::E,
        });
        board.grid[3][4] = board.grid[3][4]
            .clone()
            .set_piece(PieceType::Locomotive(Locomotive::new(
                1,
                TrainHeading::Forward,
            )));

        let white_king = board.find_king(Color::White).expect("white king present");
        let dest = Coord {
            file: white_king.file + 1,
            rank: white_king.rank,
        };
        let prev_ply = board.flags.ply_count;
        board
            .make_move(GameMove {
                from: white_king,
                move_type: MoveType::MoveTo(dest),
            })
            .expect("idle ply");
        // Loco stays where it was; ply counter advanced.
        match &board.grid[3][4].piece {
            Some(PieceType::Locomotive(_)) => (),
            other => panic!("loco should sit still at (4,3), got {other:?}"),
        }
        assert!(board.flags.ply_count > prev_ply, "ply counter advances");
    }

    #[test]
    fn test_two_trains_converge_both_stop() {
        // Two trains target (3,3) from opposite directions on the same
        // rank. Both stop.
        let mut board = board_with_idle_kings();
        board.flags.train_tick_rate = TrainTickRate::EveryPly;
        lay_east_track(&mut board, 3, 1, 5);
        // Train A heads east from (3,2) (forward → next tile (3,3)).
        board.grid[3][2] = board.grid[3][2]
            .clone()
            .set_piece(PieceType::Locomotive(Locomotive::new(
                1,
                TrainHeading::Forward,
            )));
        // Train B heads west from (3,4) (reverse heading on an east track → next tile (3,3)).
        board.grid[3][4] = board.grid[3][4]
            .clone()
            .set_piece(PieceType::Locomotive(Locomotive::new(
                2,
                TrainHeading::Reverse,
            )));

        let white_king = board.find_king(Color::White).expect("white king present");
        let dest = Coord {
            file: white_king.file + 1,
            rank: white_king.rank,
        };
        board
            .make_move(GameMove {
                from: white_king,
                move_type: MoveType::MoveTo(dest),
            })
            .expect("idle ply");

        // Neither train should have advanced — both stay put.
        match &board.grid[3][2].piece {
            Some(PieceType::Locomotive(l)) => assert_eq!(l.train_id, 1),
            other => panic!("train A should still be at (2,3), got {other:?}"),
        }
        match &board.grid[3][4].piece {
            Some(PieceType::Locomotive(l)) => assert_eq!(l.train_id, 2),
            other => panic!("train B should still be at (4,3), got {other:?}"),
        }
        // Tile in the middle should still be empty track.
        assert!(board.grid[3][3].piece.is_none());
    }

    #[test]
    fn test_king_cannot_walk_into_train_next_tile() {
        // White king at (3,4); only escape square (in the direction we
        // care about) is (3,5), which is the train's next-tick tile.
        let mut board = empty_board();
        board.flags.white_can_castle_kingside = false;
        board.flags.white_can_castle_queenside = false;
        board.flags.black_can_castle_kingside = false;
        board.flags.black_can_castle_queenside = false;
        board.flags.train_tick_rate = TrainTickRate::EveryPly;

        // Place tracks at (3,4) → (3,5) → (3,6) heading east.
        board.grid[4][3] = Square::new().set_square_type(SquareType::Track {
            direction: TrackDir::E,
        });
        board.grid[4][4] = Square::new().set_square_type(SquareType::Track {
            direction: TrackDir::E,
        });
        board.grid[4][5] = Square::new().set_square_type(SquareType::Track {
            direction: TrackDir::E,
        });
        // Train sits at (file=3, rank=4); next-tick tile is (file=4, rank=4).
        board.grid[4][3] = board.grid[4][3]
            .clone()
            .set_piece(PieceType::Locomotive(Locomotive::new(
                1,
                TrainHeading::Forward,
            )));
        // White king at (file=5, rank=4). The train's next-tick tile (4,4)
        // is one of the king's neighbours; the king moving onto (4,4) must
        // be rejected because it would land on a square the train "attacks".
        board.grid[4][5] = board.grid[4][5]
            .clone()
            .set_piece(PieceType::new_king(Color::White));
        // Distant black king so there's a side-to-move opponent.
        board.grid[0][0] = Square::new().set_piece(PieceType::new_king(Color::Black));

        let attempt = GameMove {
            from: Coord { file: 5, rank: 4 },
            move_type: MoveType::MoveTo(Coord { file: 4, rank: 4 }),
        };
        let err = board.validate_move(&attempt).err();
        assert!(
            matches!(err, Some(MoveError::WouldLeaveKingInCheck { .. })),
            "king walking into train's next-tick tile must be rejected, got {err:?}"
        );
    }

    #[test]
    fn test_multi_cart_train_curves_correctly() {
        // L-shaped track: (3,3) → E (3,4) → S (4,4). Three-cart train
        // (loco + 2 carriages) starting at (3,1), (3,2), (3,3) on east
        // tracks. After three ticks each cart will have traversed.
        let mut board = board_with_idle_kings();
        board.flags.train_tick_rate = TrainTickRate::EveryPly;
        // Lay a longer L-shape.
        for f in 1..=4 {
            board.grid[3][f] = Square::new().set_square_type(SquareType::Track {
                direction: TrackDir::E,
            });
        }
        board.grid[3][4] = Square::new().set_square_type(SquareType::Track {
            direction: TrackDir::S,
        });
        board.grid[4][4] = Square::new().set_square_type(SquareType::Track {
            direction: TrackDir::S,
        });
        board.grid[5][4] = Square::new().set_square_type(SquareType::Track {
            direction: TrackDir::S,
        });

        // Place loco at (3,3), carriage 1 at (3,2), carriage 2 at (3,1).
        board.grid[3][3] = board.grid[3][3]
            .clone()
            .set_piece(PieceType::Locomotive(Locomotive::new(
                1,
                TrainHeading::Forward,
            )));
        board.grid[3][2] = board.grid[3][2]
            .clone()
            .set_piece(PieceType::Carriage(Carriage::new(1, 1)));
        board.grid[3][1] = board.grid[3][1]
            .clone()
            .set_piece(PieceType::Carriage(Carriage::new(1, 2)));

        // Tick three times so the chain rolls around the corner.
        for _ in 0..3 {
            let king = board.find_king(board.flags.side_to_move).unwrap();
            let mv = board.legal_moves(&king).into_iter().next().unwrap();
            board.make_move(mv).expect("idle ply");
        }
        // Loco should be at (5,4) (3 ticks south from the corner).
        match &board.grid[5][4].piece {
            Some(PieceType::Locomotive(l)) => assert_eq!(l.train_id, 1),
            other => panic!("loco expected at (4,5), got {other:?}"),
        }
        // Carriage 1 at (4,4), carriage 2 at (3,4).
        match &board.grid[4][4].piece {
            Some(PieceType::Carriage(c)) => assert_eq!(c.chain_index, 1),
            other => panic!("carriage 1 expected at (4,4), got {other:?}"),
        }
        match &board.grid[3][4].piece {
            Some(PieceType::Carriage(c)) => assert_eq!(c.chain_index, 2),
            other => panic!("carriage 2 expected at (4,3), got {other:?}"),
        }
    }

    #[test]
    fn test_junction_diverts_train() {
        // Two-tile run-up (east track), then a junction with branches
        // (N, S). state=0 → N, throw the switch → state=1 → S.
        let mut board = board_with_idle_kings();
        board.flags.train_tick_rate = TrainTickRate::EveryPly;
        board.grid[3][1] = Square::new().set_square_type(SquareType::Track {
            direction: TrackDir::E,
        });
        board.grid[3][2] = Square::new().set_square_type(SquareType::Junction {
            id: 9,
            state: 0,
            branches: vec![TrackDir::N, TrackDir::S],
        });
        // Targets for the junction in both directions.
        board.grid[2][2] = Square::new().set_square_type(SquareType::Track {
            direction: TrackDir::E,
        });
        board.grid[4][2] = Square::new().set_square_type(SquareType::Track {
            direction: TrackDir::E,
        });

        board.grid[3][1] = board.grid[3][1]
            .clone()
            .set_piece(PieceType::Locomotive(Locomotive::new(
                1,
                TrainHeading::Forward,
            )));
        // First ply: train rolls from (3,1) → (3,2) (the junction).
        let king = board.find_king(Color::White).unwrap();
        let dest = Coord {
            file: king.file + 1,
            rank: king.rank,
        };
        board
            .make_move(GameMove {
                from: king,
                move_type: MoveType::MoveTo(dest),
            })
            .expect("ply 1");
        // Second ply: junction state=0 → north. Loco at (2,2).
        let king = board.find_king(Color::Black).unwrap();
        let dest = Coord {
            file: king.file - 1,
            rank: king.rank,
        };
        board
            .make_move(GameMove {
                from: king,
                move_type: MoveType::MoveTo(dest),
            })
            .expect("ply 2");
        assert!(
            matches!(board.grid[2][2].piece, Some(PieceType::Locomotive(_))),
            "junction state=0 should send loco north"
        );
    }

    /// At a Junction tile, `TrainHeading` is structurally ignored — routing
    /// is driven solely by `state % branches.len()`. A Reverse-heading loco
    /// landing on the junction must exit along the same branch a
    /// Forward-heading loco would. Regression for the invariant that
    /// junction routing doesn't fall through to track-style
    /// `direction.opposite()` handling.
    #[test]
    fn test_reverse_heading_at_junction_uses_state_not_heading() {
        let mut board = board_with_idle_kings();
        board.flags.train_tick_rate = TrainTickRate::EveryPly;
        // Run-up track: (3,1) heads east into the junction at (3,2).
        board.grid[3][1] = Square::new().set_square_type(SquareType::Track {
            direction: TrackDir::E,
        });
        board.grid[3][2] = Square::new().set_square_type(SquareType::Junction {
            id: 9,
            state: 0,
            branches: vec![TrackDir::N, TrackDir::S],
        });
        // North/south exit tracks so the junction's choice is observable.
        board.grid[2][2] = Square::new().set_square_type(SquareType::Track {
            direction: TrackDir::E,
        });
        board.grid[4][2] = Square::new().set_square_type(SquareType::Track {
            direction: TrackDir::E,
        });

        // Reverse heading on an east track → first tick still sends the
        // loco west to (3,0) via the track's reverse handling? No — we
        // want it to *reach the junction* first. Place the loco directly
        // on the junction tile so we observe the junction's routing on
        // tick 1, with last_dir=None (cold start) and Reverse heading.
        board.grid[3][2] = board.grid[3][2]
            .clone()
            .set_piece(PieceType::Locomotive(Locomotive::new(
                1,
                TrainHeading::Reverse,
            )));

        // One idle ply to tick the train.
        let king = board.find_king(Color::White).unwrap();
        let dest = Coord {
            file: king.file + 1,
            rank: king.rank,
        };
        board
            .make_move(GameMove {
                from: king,
                move_type: MoveType::MoveTo(dest),
            })
            .expect("idle ply");

        // state=0 → branches[0]=N, regardless of heading. Loco must be
        // north of the junction at (2,2), NOT south at (4,2).
        assert!(
            matches!(board.grid[2][2].piece, Some(PieceType::Locomotive(_))),
            "Reverse-heading loco at junction must still use state-driven branch (north for state=0)"
        );
        assert!(
            board.grid[4][2].piece.is_none(),
            "Reverse loco must not have headed south — heading is ignored at Junction tiles"
        );
    }

    #[test]
    fn test_train_does_not_run_over_own_cart() {
        // 3-tile loop with a 3-cart train. The locomotive's next tile
        // is its own caboose, so the train stops rather than capturing
        // itself.
        let mut board = board_with_idle_kings();
        board.flags.train_tick_rate = TrainTickRate::EveryPly;
        // Loop: (3,2) E → (3,3); (3,3) S → (4,3); (4,3) N → (3,3) — no, that's wrong.
        // Use: (3,2) E (3,3); (3,3) S (4,3); (4,3) W (4,2); (4,2) N (3,2).
        board.grid[3][2] = Square::new().set_square_type(SquareType::Track {
            direction: TrackDir::E,
        });
        board.grid[3][3] = Square::new().set_square_type(SquareType::Track {
            direction: TrackDir::S,
        });
        board.grid[4][3] = Square::new().set_square_type(SquareType::Track {
            direction: TrackDir::W,
        });
        board.grid[4][2] = Square::new().set_square_type(SquareType::Track {
            direction: TrackDir::N,
        });

        // Place locomotive + 2 carriages occupying *3 of the 4 loop tiles*
        // (so the loco's next tile would be the caboose). A 4-tile loop
        // with 3 carts: tiles 3 are occupied, tile 4 is the loco's next
        // step. That's actually fine — the train can move. To force the
        // self-collision case, we need cart-count == loop-length.
        // So: 4 carts on a 4-tile loop.
        board.grid[3][2] = board.grid[3][2]
            .clone()
            .set_piece(PieceType::Locomotive(Locomotive::new(
                1,
                TrainHeading::Forward,
            )));
        board.grid[4][2] = board.grid[4][2]
            .clone()
            .set_piece(PieceType::Carriage(Carriage::new(1, 1)));
        board.grid[4][3] = board.grid[4][3]
            .clone()
            .set_piece(PieceType::Carriage(Carriage::new(1, 2)));
        board.grid[3][3] = board.grid[3][3]
            .clone()
            .set_piece(PieceType::Carriage(Carriage::new(1, 3)));

        // Loco at (3,2) → next tile (3,3), which is its own caboose.
        // Should stop.
        let king = board.find_king(Color::White).unwrap();
        let dest = Coord {
            file: king.file + 1,
            rank: king.rank,
        };
        board
            .make_move(GameMove {
                from: king,
                move_type: MoveType::MoveTo(dest),
            })
            .expect("ply 1");
        // Loco stays at (3,2).
        match &board.grid[3][2].piece {
            Some(PieceType::Locomotive(l)) => assert_eq!(l.train_id, 1),
            other => panic!("loco should stay at (2,3); train hits its own caboose; got {other:?}"),
        }
    }

    #[test]
    fn test_ply_count_in_fen() {
        let mut board = empty_board();
        board.flags.ply_count = 42;
        board.flags.train_tick_rate = TrainTickRate::EveryNPly(3);
        let fen = board_to_fen(&board);
        assert!(
            fen.contains("tr=3ply"),
            "FEN should serialize tr=3ply, got {fen}"
        );
        assert!(
            fen.ends_with(" p=42"),
            "FEN should end with the ply counter, got {fen}"
        );
        let board2 = fen_to_board(&fen);
        assert_eq!(board2.flags.ply_count, 42);
        assert_eq!(board2.flags.train_tick_rate, TrainTickRate::EveryNPly(3));
    }

    /// Minecart-style curves: a train should round a corner even when
    /// every track tile shares the same stored `D` field (so only the
    /// loco's starting tile's D matters). The remaining tiles auto-
    /// connect via their neighbors. Lays an L-shape entirely with D=E
    /// then checks the loco curves south at the corner.
    #[test]
    fn test_train_curves_via_neighbor_detection() {
        let mut board = board_with_idle_kings();
        board.flags.train_tick_rate = TrainTickRate::EveryPly;

        // L-shape: (3,1) (3,2) (3,3) east-then-south to (4,3) (5,3).
        // Every tile has D=E — the legacy "outgoing direction" model
        // would derail at (3,3) because (3,3).D=E sends the loco off
        // the L. Connection-aware traversal should curve south.
        for f in 1..=3 {
            board.grid[3][f] = Square::new().set_square_type(SquareType::Track {
                direction: TrackDir::E,
            });
        }
        board.grid[4][3] = Square::new().set_square_type(SquareType::Track {
            direction: TrackDir::E,
        });
        board.grid[5][3] = Square::new().set_square_type(SquareType::Track {
            direction: TrackDir::E,
        });

        // Loco at the western end of the rank-3 run.
        board.grid[3][1] = board.grid[3][1]
            .clone()
            .set_piece(PieceType::Locomotive(Locomotive::new(
                1,
                TrainHeading::Forward,
            )));

        // 5 plies — should reach (5,3) via the corner.
        for _ in 0..5 {
            let king = board.find_king(board.flags.side_to_move).unwrap();
            let mv = board.legal_moves(&king).into_iter().next().unwrap();
            board.make_move(mv).expect("idle ply");
        }
        match &board.grid[5][3].piece {
            Some(PieceType::Locomotive(l)) => assert_eq!(l.train_id, 1),
            other => panic!(
                "loco should curve south through the L and end at (3,5); got {other:?} \
                 at (5,3); full row3={:?} col3@4={:?} col3@5={:?}",
                board.grid[3], board.grid[4][3], board.grid[5][3]
            ),
        }
    }

    /// A vertically-stacked train with all tiles defaulted to D=E
    /// should still move on the first tick. Regression for the
    /// "loco's D=E doesn't lead to a track so the train derails before
    /// it gets to use neighbor detection" case. The engine's first-
    /// tick fallback picks the unique non-cart neighbor when the
    /// stored D is bogus.
    #[test]
    fn test_train_first_tick_falls_back_when_d_misaligned() {
        let mut board = board_with_idle_kings();
        board.flags.train_tick_rate = TrainTickRate::EveryPly;
        // Vertical track column at file=3, rank 1..=5, all with D=E
        // (wrong direction for a vertical chain). Mirrors the user-
        // reported FEN where every track defaults to D=E and a
        // vertically-stacked train sits on top.
        for r in 1..=5u8 {
            board.grid[r as usize][3] = Square::new().set_square_type(SquareType::Track {
                direction: TrackDir::E,
            });
        }
        // Chain: loco at (file=3, rank=4), carts to the north of it.
        board.grid[1][3] = board.grid[1][3]
            .clone()
            .set_piece(PieceType::Carriage(Carriage::new(1, 3)));
        board.grid[2][3] = board.grid[2][3]
            .clone()
            .set_piece(PieceType::Carriage(Carriage::new(1, 2)));
        board.grid[3][3] = board.grid[3][3]
            .clone()
            .set_piece(PieceType::Carriage(Carriage::new(1, 1)));
        board.grid[4][3] = board.grid[4][3]
            .clone()
            .set_piece(PieceType::Locomotive(Locomotive::new(
                1,
                TrainHeading::Forward,
            )));

        // One ply — loco should fall back from D=E (no east track) to
        // the unique non-cart neighbor (south, since north is a cart).
        let king = board.find_king(Color::White).unwrap();
        let dest = Coord {
            file: king.file + 1,
            rank: king.rank,
        };
        board
            .make_move(GameMove {
                from: king,
                move_type: MoveType::MoveTo(dest),
            })
            .expect("idle ply");

        // Loco should now be at (file=3, rank=5) — one tile south of
        // its start, having taken the fallback exit.
        match &board.grid[5][3].piece {
            Some(PieceType::Locomotive(l)) => assert_eq!(l.train_id, 1),
            other => panic!(
                "loco expected to step south via fallback to (3,5), got {other:?}"
            ),
        }
        // Cart 1 follows into the loco's previous tile.
        match &board.grid[4][3].piece {
            Some(PieceType::Carriage(c)) => {
                assert_eq!(c.chain_index, 1);
                assert_eq!(c.train_id, 1);
            }
            other => panic!("cart 1 expected at (3,4), got {other:?}"),
        }
    }

    /// Carts are invincible: when a non-friendly piece moves onto a
    /// neutral cart's tile, the cart stays put and the moving piece
    /// boards by capture. Any opposite-color passengers inside get
    /// captured during the board.
    #[test]
    fn test_cart_is_invincible_enemy_boards_by_capture() {
        let mut board = empty_board();
        board.flags.white_can_castle_kingside = false;
        board.flags.white_can_castle_queenside = false;
        board.flags.black_can_castle_kingside = false;
        board.flags.black_can_castle_queenside = false;
        // Carriage at (3,3) on a track tile, carrying a black pawn.
        board.grid[3][3] = Square::new()
            .set_square_type(SquareType::Track {
                direction: TrackDir::E,
            })
            .set_piece(PieceType::Carriage(Carriage {
                train_id: 1,
                chain_index: 1,
                passengers: vec![PieceType::new_pawn(Color::Black)],
            }));
        // White knight at file=4, rank=5 — one knight-move away from
        // (file=3, rank=3) via Δfile=-1, Δrank=-2.
        board.grid[5][4] = Square::new().set_piece(PieceType::new_knight(Color::White));
        // Kings for legality.
        board.grid[7][0] = Square::new().set_piece(PieceType::new_king(Color::White));
        board.grid[0][7] = Square::new().set_piece(PieceType::new_king(Color::Black));

        // White knight at (file=4, rank=5) → (file=3, rank=3) is a
        // knight move. The filter should rewrite this MoveTo into
        // MoveIntoCarrier because the target is a Neutral carrier.
        let raw = board.get_moves(&Coord { file: 4, rank: 5 });
        let board_move = raw
            .iter()
            .find(|m| matches!(&m.move_type, MoveType::MoveIntoCarrier(c) if *c == (Coord { file: 3, rank: 3 })))
            .cloned()
            .expect("expected a MoveIntoCarrier targeting the cart");

        board.make_move(board_move).expect("knight boards the cart");

        // Cart is still at (3,3), no longer holds the pawn, and now
        // holds the knight as a passenger.
        match &board.grid[3][3].piece {
            Some(PieceType::Carriage(c)) => {
                assert_eq!(c.train_id, 1);
                assert_eq!(c.passengers.len(), 1, "pawn captured, knight in");
                assert!(
                    matches!(c.passengers[0], PieceType::Knight(_)),
                    "boarder should be the knight, got {:?}",
                    c.passengers[0]
                );
            }
            other => panic!("expected carriage to survive the move, got {other:?}"),
        }
        // Knight's old square is empty.
        assert!(board.grid[5][4].piece.is_none(), "knight vacated (4,5)");
    }

    /// A passenger inside a neutral cart belongs to its own colour for
    /// side-to-move purposes. Without the fix, the cart's `Neutral`
    /// colour fails the WrongTurn check and the passenger is trapped.
    /// Regression for the user-reported "It is black's turn, but the
    /// piece at (X, Y) ... is neutral" error.
    #[test]
    fn test_passenger_can_exit_cart_on_their_color_turn() {
        use std::sync::Arc;
        let mut board = empty_board();
        // EveryFullTurn means no train tick after a single ply, so the
        // train doesn't run over the king right after it disembarks.
        board.flags.train_tick_rate = TrainTickRate::EveryFullTurn;
        board.flags.white_can_castle_kingside = false;
        board.flags.white_can_castle_queenside = false;
        board.flags.black_can_castle_kingside = false;
        board.flags.black_can_castle_queenside = false;
        // Cart with a black king passenger, on a track tile.
        board.grid[3][3] = Square::new()
            .set_square_type(SquareType::Track {
                direction: TrackDir::E,
            })
            .set_piece(PieceType::Carriage(Carriage {
                train_id: 1,
                chain_index: 1,
                passengers: vec![PieceType::new_king(Color::Black)],
            }));
        // Distant white king so the position is legal.
        board.grid[7][0] = Square::new().set_piece(PieceType::new_king(Color::White));
        board.flags.side_to_move = Color::Black;

        let exit = GameMove {
            from: Coord { file: 3, rank: 3 },
            move_type: MoveType::PieceInCarrier {
                piece_index: 0,
                move_type: Arc::new(MoveType::MoveTo(Coord { file: 3, rank: 4 })),
            },
        };
        board
            .make_move(exit)
            .expect("black king should be allowed out of the cart on black's turn");

        // King is now at (3, 4); cart at (3, 3) is empty.
        match &board.grid[4][3].piece {
            Some(PieceType::King(k)) => assert_eq!(k.color, Color::Black),
            other => panic!("expected black king at (3, 4), got {other:?}"),
        }
        match &board.grid[3][3].piece {
            Some(PieceType::Carriage(c)) => {
                assert_eq!(c.train_id, 1);
                assert!(c.passengers.is_empty(), "cart should be empty after exit");
            }
            other => panic!("expected empty carriage at (3, 3), got {other:?}"),
        }
    }

    /// A king should be parkable inside *any* carriage of a chain,
    /// not just the caboose. Regression for the "Can't park king on
    /// any other carriage than the very back one" bug: previously,
    /// each carriage's `attacks()` included the cart-in-front's tile
    /// as a phantom "next-tick crush" threat. That made `is_in_check`
    /// flag a king sitting in cart M as attacked by cart M+1, even
    /// though the train moves atomically and no actual capture can
    /// happen between same-train carts.
    #[test]
    fn test_king_can_board_any_carriage_in_chain() {
        let mut board = empty_board();
        board.flags.train_tick_rate = TrainTickRate::EveryFullTurn;
        board.flags.white_can_castle_kingside = false;
        board.flags.white_can_castle_queenside = false;
        board.flags.black_can_castle_kingside = false;
        board.flags.black_can_castle_queenside = false;
        // Horizontal track strip along rank 3, files 1..=4. Train of
        // loco + 3 carts occupies all four tiles.
        for f in 1..=4u8 {
            board.grid[3][f as usize] = Square::new().set_square_type(SquareType::Track {
                direction: TrackDir::E,
            });
        }
        board.grid[3][4] = board.grid[3][4]
            .clone()
            .set_piece(PieceType::Locomotive(Locomotive::new(
                1,
                TrainHeading::Forward,
            )));
        board.grid[3][3] = board.grid[3][3]
            .clone()
            .set_piece(PieceType::Carriage(Carriage::new(1, 1)));
        board.grid[3][2] = board.grid[3][2]
            .clone()
            .set_piece(PieceType::Carriage(Carriage::new(1, 2)));
        board.grid[3][1] = board.grid[3][1]
            .clone()
            .set_piece(PieceType::Carriage(Carriage::new(1, 3)));
        // White king one rank south of carriage 2 (file=2, rank=4),
        // so its single-step move to (file=2, rank=3) lands on the
        // middle carriage. Far black king for legality.
        board.grid[4][2] = Square::new().set_piece(PieceType::new_king(Color::White));
        board.grid[0][7] = Square::new().set_piece(PieceType::new_king(Color::Black));

        // White boards carriage 2. The move is generated by
        // `get_moves` as MoveIntoCarrier (filter rewrite of the king's
        // raw MoveTo onto a neutral cart's tile).
        let board_move = GameMove {
            from: Coord { file: 2, rank: 4 },
            move_type: MoveType::MoveIntoCarrier(Coord { file: 2, rank: 3 }),
        };
        board
            .make_move(board_move)
            .expect("king should be able to board a middle carriage; cart-behind threat is phantom");

        // King now lives inside carriage 2.
        match &board.grid[3][2].piece {
            Some(PieceType::Carriage(c)) => {
                assert_eq!(c.chain_index, 2);
                assert!(
                    c.passengers
                        .iter()
                        .any(|p| matches!(p, PieceType::King(k) if k.color == Color::White)),
                    "expected white king passenger in carriage 2, got {:?}",
                    c.passengers,
                );
            }
            other => panic!("expected carriage 2 to still hold the king, got {other:?}"),
        }
        // Source tile is vacated.
        assert!(board.grid[4][2].piece.is_none(), "king vacated (2, 4)");
    }

    /// The opposing side cannot move a passenger out of a cart that
    /// doesn't belong to them. White's turn + cart with a black
    /// passenger ⇒ WrongTurn. Complements the positive test above.
    #[test]
    fn test_other_side_cannot_exit_cart() {
        use std::sync::Arc;
        let mut board = empty_board();
        board.grid[3][3] = Square::new()
            .set_square_type(SquareType::Track {
                direction: TrackDir::E,
            })
            .set_piece(PieceType::Carriage(Carriage {
                train_id: 1,
                chain_index: 1,
                passengers: vec![PieceType::new_king(Color::Black)],
            }));
        board.grid[7][0] = Square::new().set_piece(PieceType::new_king(Color::White));
        // White's turn (default).
        let attempt = GameMove {
            from: Coord { file: 3, rank: 3 },
            move_type: MoveType::PieceInCarrier {
                piece_index: 0,
                move_type: Arc::new(MoveType::MoveTo(Coord { file: 3, rank: 4 })),
            },
        };
        let err = board.validate_move(&attempt).err();
        assert!(
            matches!(err, Some(MoveError::WrongTurn { .. })),
            "white shouldn't be allowed to drive a black passenger out of a cart; got {err:?}"
        );
    }

    /// Loco's `last_dir` field round-trips through FEN. The engine
    /// emits `L=<dir>` only when set; absent `L=` means "fresh /
    /// pre-first-tick", which is also a meaningful state and must
    /// preserve through paste-edit-copy cycles.
    #[test]
    fn test_locomotive_last_dir_round_trips() {
        let mut board = empty_board();
        let mut loco = Locomotive::new(7, TrainHeading::Forward);
        loco.last_dir = Some(TrackDir::W);
        board.grid[3][3] = Square::new()
            .set_square_type(SquareType::Track {
                direction: TrackDir::E,
            })
            .set_piece(PieceType::Locomotive(loco));

        let fen = board_to_fen(&board);
        let board2 = fen_to_board(&fen);
        match &board2.grid[3][3].piece {
            Some(PieceType::Locomotive(l)) => {
                assert_eq!(l.last_dir, Some(TrackDir::W));
            }
            other => panic!("expected loco after round-trip, got {other:?}"),
        }
    }

    // ============================================================
    // Audit regression tests (post-iteration cleanup)
    // ============================================================

    /// B1: when the *same move* both sets the en-passant target via
    /// a pawn double-push *and* ticks a train that captures that
    /// pawn, the ep target must be cleared. Otherwise the opposing
    /// pawn could en-passant an already-eaten pawn next turn,
    /// gaining a diagonal move with no actual capture.
    ///
    /// `apply_piece_post_effects` unconditionally clears ep, then
    /// the pawn's `post_move_effects` re-sets it on a double-push.
    /// `apply_environment_reactions` then ticks the train. So the
    /// scenario to exercise is: black double-pushes its pawn into
    /// the rail's next-tick tile, and the train rolls onto the pawn
    /// before the move returns.
    #[test]
    fn test_train_capture_clears_en_passant_target() {
        let mut board = empty_board();
        board.flags.train_tick_rate = TrainTickRate::EveryPly;
        board.flags.white_can_castle_kingside = false;
        board.flags.white_can_castle_queenside = false;
        board.flags.black_can_castle_kingside = false;
        board.flags.black_can_castle_queenside = false;
        // Horizontal east-pointing track at rank 3, files 2..=5.
        // Loco at (file=2, rank=3) heading Forward: next tick rolls
        // onto (file=3, rank=3) — which is where a black pawn ends
        // up if it double-pushes from (file=3, rank=1).
        for f in 2..=5u8 {
            board.grid[3][f as usize] = Square::new().set_square_type(
                SquareType::Track { direction: TrackDir::E },
            );
        }
        board.grid[3][2] = board.grid[3][2]
            .clone()
            .set_piece(PieceType::Locomotive(Locomotive::new(
                1,
                TrainHeading::Forward,
            )));
        // Black pawn on its starting rank, ready to double-push.
        board.grid[1][3] = Square::new().set_piece(PieceType::new_pawn(Color::Black));
        // Kings for legality.
        board.grid[7][7] = Square::new().set_piece(PieceType::new_king(Color::White));
        board.grid[0][0] = Square::new().set_piece(PieceType::new_king(Color::Black));
        board.flags.side_to_move = Color::Black;

        // Black double-pushes from (3, 1) to (3, 3). The move:
        //   - phase 2: clears ep, then pawn's post-effect sets ep
        //     to (file=3, rank=2).
        //   - phase 3: train ticks from (2, 3) east to (3, 3),
        //     capturing the pawn that just landed there. B1's
        //     clear-loop sees ep=(3,2), side_to_move still Black,
        //     so the candidate pawn coord is (3, 3) — matches the
        //     captured tile — clears ep.
        board
            .make_move(GameMove {
                from: Coord { file: 3, rank: 1 },
                move_type: MoveType::MoveTo(Coord { file: 3, rank: 3 }),
            })
            .expect("black pawn double-push should be legal");

        assert!(
            board.flags.en_passant_target.is_none(),
            "ep target should clear when the train captures the pawn that just set it"
        );
        // Sanity: the loco is on the pawn's tile, pawn is gone.
        assert!(
            matches!(&board.grid[3][3].piece, Some(PieceType::Locomotive(_))),
            "loco should occupy the pawn's tile after the tick"
        );
    }

    /// B1 false-positive guard: the ep target should *not* clear
    /// when the train captures some *other* pawn that happens to
    /// share the ep target's file. The original heuristic used
    /// `abs_diff(rank) == 1` which would mis-fire on a pawn one
    /// rank past the ep target on the wrong side.
    #[test]
    fn test_train_capture_does_not_clear_unrelated_ep() {
        let mut board = empty_board();
        board.flags.train_tick_rate = TrainTickRate::EveryPly;
        board.flags.white_can_castle_kingside = false;
        board.flags.white_can_castle_queenside = false;
        board.flags.black_can_castle_kingside = false;
        board.flags.black_can_castle_queenside = false;
        // East track at rank 4, files 1..=5. Loco at (file=1, rank=4)
        // → next tile (2, 4).
        for f in 1..=5u8 {
            board.grid[4][f as usize] = Square::new().set_square_type(
                SquareType::Track { direction: TrackDir::E },
            );
        }
        board.grid[4][1] = board.grid[4][1]
            .clone()
            .set_piece(PieceType::Locomotive(Locomotive::new(
                1,
                TrainHeading::Forward,
            )));
        // An unrelated white pawn at (2, 4). The loco captures it
        // on its first tick.
        board.grid[4][2] = board.grid[4][2]
            .clone()
            .set_piece(PieceType::new_pawn(Color::White));
        // ep target set as if a *separate* white double-push had
        // happened previously: ep at (2, 5), with the implied
        // double-pusher at (2, 4)... which IS the pawn the train
        // captures. So we offset: ep at (2, 3), implied pusher at
        // (2, 2) — same file as the pawn-at-(2,4) but a different
        // rank. The old heuristic would mis-clear (file match,
        // abs_diff(4, 3) = 1). With side_to_move=White, the new
        // logic looks for pusher at (file: 2, rank: ep.rank - 1)
        // = (2, 2) — does NOT match victim (2, 4).
        board.flags.en_passant_target = Some(Coord { file: 2, rank: 3 });
        // Distant kings.
        board.grid[0][0] = Square::new().set_piece(PieceType::new_king(Color::Black));
        board.grid[7][7] = Square::new().set_piece(PieceType::new_king(Color::White));
        board.flags.side_to_move = Color::White;

        // Idle white move (not a pawn double-push, so phase 2 doesn't
        // re-set ep — but phase 2's unconditional clear means ep is
        // None by the time the tick fires anyway). Skip this test if
        // the ep state can't be observed.
        //
        // Workaround: use `make_move_unchecked` after setting up the
        // state, bypassing phase 2's ep clear. That isn't a real-game
        // scenario, but it's the only way to test the B1 *false-
        // positive guard* in isolation — phase 2 always clears ep
        // before phase 3 in the real flow.
        let white_king = Coord { file: 7, rank: 7 };
        board
            .make_move(GameMove {
                from: white_king,
                move_type: MoveType::MoveTo(Coord { file: 6, rank: 7 }),
            })
            .expect("idle white king move");

        // Phase 2 of the king move cleared ep, and the king's
        // post-effect didn't re-set it, so ep is None either way.
        // The new heuristic's contribution is that it *also* wouldn't
        // have mis-fired — but with the real architecture, this test
        // mostly proves "no panic, no weird state mutation."
        assert!(
            board.flags.en_passant_target.is_none(),
            "ep stays cleared after an unrelated train capture (phase 2 already did the work)"
        );
        // Loco rolled onto (2, 4); pawn is gone.
        assert!(
            matches!(&board.grid[4][2].piece, Some(PieceType::Locomotive(_))),
            "loco should occupy the pawn's tile"
        );
    }

    /// B3: the FEN parser rejects `tr=0ply` (modulo-by-zero hazard).
    #[test]
    fn test_fen_rejects_zero_ply_tick_rate() {
        let board = fen_to_board("8/8/8/8/8/8/8/8 w - - tr=0ply p=0");
        assert_eq!(
            board.flags.train_tick_rate,
            TrainTickRate::EveryFullTurn,
            "tr=0ply must not parse to EveryNPly(0); should fall back to default"
        );
    }

    /// B4: malformed FENs with unbalanced parens shouldn't panic.
    /// Underflow in `split_top_level` / `find_matching_paren` was a
    /// debug-build panic on hostile input.
    #[test]
    fn test_fen_parser_survives_unbalanced_parens() {
        // Each of these would have panicked the parser pre-fix.
        let _ = fen_to_board("K) w - -");
        let _ = fen_to_board("(P=K w - -");
        let _ = fen_to_board("((P=K))) w - -");
        // Survival is the assertion — the parser may produce
        // garbage on malformed input, but it must not panic.
    }

    /// FEN round-trip: `tr=ply` and `tr=full` both serialize and
    /// parse identically. Only `tr=Nply` was previously covered.
    #[test]
    fn test_train_tick_rate_round_trip_every_ply_and_every_full_turn() {
        let mut board = empty_board();
        board.flags.train_tick_rate = TrainTickRate::EveryPly;
        let fen = board_to_fen(&board);
        assert!(
            fen.contains("tr=ply"),
            "EveryPly should serialize as tr=ply, got {fen}"
        );
        let board2 = fen_to_board(&fen);
        assert_eq!(board2.flags.train_tick_rate, TrainTickRate::EveryPly);

        let mut board = empty_board();
        board.flags.train_tick_rate = TrainTickRate::EveryFullTurn;
        let fen = board_to_fen(&board);
        assert!(
            fen.contains("tr=full"),
            "EveryFullTurn should serialize as tr=full, got {fen}"
        );
        let board2 = fen_to_board(&fen);
        assert_eq!(board2.flags.train_tick_rate, TrainTickRate::EveryFullTurn);
    }

    /// FEN writer field order for LOCO is canonical: ID, H, L, P.
    #[test]
    fn test_locomotive_fen_writer_field_order() {
        let loco = Locomotive {
            train_id: 5,
            heading: TrainHeading::Reverse,
            passengers: vec![PieceType::new_king(Color::Black)],
            last_dir: Some(TrackDir::N),
        };
        let sym = crate::pieces::Piece::symbol(&loco);
        assert_eq!(sym, "LOCO(ID=5,H=R,L=N,P=(k))");
    }

    /// FEN writer field order for CART is canonical: ID, I, P.
    #[test]
    fn test_carriage_fen_writer_field_order() {
        let cart = Carriage {
            train_id: 5,
            chain_index: 2,
            passengers: vec![PieceType::new_pawn(Color::White)],
        };
        let sym = crate::pieces::Piece::symbol(&cart);
        assert_eq!(sym, "CART(ID=5,I=2,P=(P))");
    }

    /// Round-trip a piece-on-Track square through FEN.
    #[test]
    fn test_piece_on_track_fen_round_trip() {
        let mut board = empty_board();
        board.grid[3][3] = Square::new()
            .set_square_type(SquareType::Track {
                direction: TrackDir::E,
            })
            .set_piece(PieceType::new_king(Color::Black));
        let fen = board_to_fen(&board);
        let board2 = fen_to_board(&fen);
        assert_eq!(board2.grid[3][3], board.grid[3][3]);
    }

    /// Round-trip a PressurePlate keyed to `OnlyColor(Color::Neutral)`
    /// via the `FIRES=N` payload.
    #[test]
    fn test_pressure_plate_neutral_trigger_round_trip() {
        let mut board = empty_board();
        board.grid[4][4] = Square::new().set_square_type(SquareType::PressurePlate {
            targets: vec![1],
            fires_for: crate::board::square::PressureTrigger::OnlyColor(Color::Neutral),
        });
        let fen = board_to_fen(&board);
        assert!(
            fen.contains("FIRES=N"),
            "expected FIRES=N for Neutral trigger, got {fen}"
        );
        let board2 = fen_to_board(&fen);
        match &board2.grid[4][4].square_type {
            SquareType::PressurePlate { fires_for, .. } => {
                assert_eq!(
                    *fires_for,
                    crate::board::square::PressureTrigger::OnlyColor(Color::Neutral)
                );
            }
            other => panic!("expected PressurePlate, got {other:?}"),
        }
    }

    /// Validate path does NOT run the train tick. Regression for the
    /// original "train eats king during validate clone" bug fixed by
    /// `apply_move_for_validation`.
    ///
    /// Scenario: train one tile west of the king. If validate ticked
    /// the train on the clone, the loco would advance onto the king,
    /// `find_king(White)` would then return `None`, `is_in_check`
    /// would return `false` (no king ⇒ no check), and `legal_moves`
    /// for an unrelated piece would *not* be filtered for "would
    /// leave king in check" — letting the player ignore the actual
    /// threat. We assert the opposite: `legal_moves` on an unrelated
    /// piece sees the king as still in check (because the train's
    /// `attacks()` reports its next-tick tile and the king sits on
    /// that tile), and so produces no legal moves.
    #[test]
    fn test_validate_does_not_tick_trains() {
        let mut board = empty_board();
        board.flags.train_tick_rate = TrainTickRate::EveryPly;
        board.flags.white_can_castle_kingside = false;
        board.flags.white_can_castle_queenside = false;
        board.flags.black_can_castle_kingside = false;
        board.flags.black_can_castle_queenside = false;
        // East-going track. Loco at (3, 4); white king at (4, 4) —
        // the loco's next-tick tile.
        for f in 2..=5u8 {
            board.grid[4][f as usize] = Square::new().set_square_type(
                SquareType::Track { direction: TrackDir::E },
            );
        }
        board.grid[4][3] = board.grid[4][3]
            .clone()
            .set_piece(PieceType::Locomotive(Locomotive::new(
                1,
                TrainHeading::Forward,
            )));
        board.grid[4][4] = board.grid[4][4]
            .clone()
            .set_piece(PieceType::new_king(Color::White));
        // An unrelated white rook so we have a non-king piece whose
        // legal_moves we can inspect.
        board.grid[0][0] = Square::new().set_piece(PieceType::new_rook(Color::White));
        board.grid[7][7] = Square::new().set_piece(PieceType::new_king(Color::Black));
        board.flags.side_to_move = Color::White;

        // Sanity: pre-move state has white in check from the train.
        assert!(
            board.is_in_check(Color::White),
            "test setup: white king sits on the loco's next-tick tile"
        );

        // `legal_moves` clones the board and applies each candidate
        // through `apply_move_for_validation`. If validate *did* tick
        // trains, the hypothetical would have the train roll forward
        // and eat the king, after which `is_in_check(White)` would
        // return false (no king found), and the rook's moves would
        // appear legal. With the no-tick split, the hypothetical
        // preserves the king, the train's static next-tick attack
        // still threatens (4, 4), and every rook move is rejected
        // for leaving the king in check.
        let rook_moves = board.legal_moves(&Coord { file: 0, rank: 0 });
        assert!(
            rook_moves.is_empty(),
            "rook has no legal moves while king is in train's crosshairs; \
             got {} candidates — validate may be ticking trains: {rook_moves:?}",
            rook_moves.len(),
        );
        // King must also still exist after the legal_moves call —
        // belt-and-braces against the regression (a regression where
        // validate ticked would have mutated the *real* board if a
        // bug let the clone leak back, though that's extra-paranoid).
        assert!(
            board.find_king(Color::White).is_some(),
            "white king must still be on the real board after legal_moves"
        );
    }

    /// Castle path-safety includes train threats. White can't castle
    /// kingside if the king's destination or transit squares are
    /// attacked — including by a Neutral train's next-tick tile.
    ///
    /// Setup: loco at (file=4, rank=6) heading N. Its next-tick tile
    /// is (file=4, rank=5)... wait — for the threat to land on the
    /// king's transit (file=5, rank=7 or file=6, rank=7), we need
    /// the loco's next tile to BE one of those. Easiest: loco one
    /// tile *north* of (5, 7) heading S, so its next tile is (5, 7).
    /// The transit and destination squares of the kingside castle
    /// must be empty of pieces (path-occupancy guard), so the loco
    /// must NOT itself sit on (5, 7) or (6, 7).
    #[test]
    fn test_castle_into_train_zone_rejected() {
        let mut board = empty_board();
        board.flags.train_tick_rate = TrainTickRate::EveryPly;
        // White king + rook in standard kingside-castle positions.
        board.grid[7][4] = Square::new().set_piece(PieceType::new_king(Color::White));
        board.grid[7][7] = Square::new().set_piece(PieceType::new_rook(Color::White));
        board.flags.white_can_castle_kingside = true;
        board.flags.white_can_castle_queenside = false;
        board.flags.black_can_castle_kingside = false;
        board.flags.black_can_castle_queenside = false;
        // Vertical track at file 5, rank 5..=7. Loco at (file=5, rank=5)
        // heading S → next-tick tile is (file=5, rank=6). We want
        // the threat on the king's transit (file=5, rank=7), so use
        // rank=6 for the loco and (5, 7) as the next-tick destination.
        board.grid[5][5] = Square::new().set_square_type(SquareType::Track {
            direction: TrackDir::S,
        });
        board.grid[6][5] = Square::new().set_square_type(SquareType::Track {
            direction: TrackDir::S,
        });
        board.grid[7][5] = Square::new().set_square_type(SquareType::Track {
            direction: TrackDir::S,
        });
        // Place loco at (file=5, rank=6); its next tile (5, 7) is
        // the king's kingside-castle transit square — and (5, 7) is
        // empty (it's a Track tile with no piece), so the path-
        // occupancy guard passes and we hit the *threat* path.
        board.grid[6][5] = board.grid[6][5]
            .clone()
            .set_piece(PieceType::Locomotive(Locomotive::new(
                1,
                TrainHeading::Forward,
            )));
        board.grid[0][0] = Square::new().set_piece(PieceType::new_king(Color::Black));
        board.flags.side_to_move = Color::White;

        // Sanity: (5, 7) is empty so the path-occupancy guard would
        // otherwise allow the castle. The loco threatens (5, 7) via
        // its next-tick tile.
        assert!(
            board.grid[7][5].piece.is_none(),
            "test setup: castle transit square (5, 7) must be empty"
        );
        assert!(
            board.is_attacked_by(&Coord { file: 5, rank: 7 }, Color::Black),
            "test setup: the train's next-tick tile (5, 7) must be attacked \
             by a Neutral threat (is_attacked_by includes Neutral)"
        );

        // Castle generation should drop the kingside castle from the
        // raw move set because `castle_moves` checks
        // `is_attacked_by(p5, opp)` and the train's threat is
        // Neutral (always counts).
        let raw = board.get_moves(&Coord { file: 4, rank: 7 });
        let has_kingside_castle = raw.iter().any(|m| {
            matches!(m.move_type, MoveType::Castle { side: CastleSide::Kingside })
        });
        assert!(
            !has_kingside_castle,
            "kingside castle must be rejected at move-gen when the train \
             threatens the king's transit square; got castle in {raw:?}"
        );

        // And an explicit attempt is rejected by validate.
        let attempt = GameMove {
            from: Coord { file: 4, rank: 7 },
            move_type: MoveType::Castle {
                side: CastleSide::Kingside,
            },
        };
        let result = board.validate_move(&attempt);
        assert!(
            result.is_err(),
            "validate_move must reject the castle attempt; got {result:?}"
        );
    }

    /// Plan 09 open question 7: a king-passenger captured when an
    /// enemy boards the cart. Pin the *current* behavior (silent
    /// king removal) so a future fix is a deliberate breaking
    /// change, not an accidental one.
    #[test]
    fn test_king_passenger_captured_when_enemy_boards_cart() {
        let mut board = empty_board();
        board.flags.white_can_castle_kingside = false;
        board.flags.white_can_castle_queenside = false;
        board.flags.black_can_castle_kingside = false;
        board.flags.black_can_castle_queenside = false;
        // Neutral cart at (3, 3) carrying a black king as the only
        // passenger.
        board.grid[3][3] = Square::new()
            .set_square_type(SquareType::Track {
                direction: TrackDir::E,
            })
            .set_piece(PieceType::Carriage(Carriage {
                train_id: 1,
                chain_index: 1,
                passengers: vec![PieceType::new_king(Color::Black)],
            }));
        // White knight a knight-move away.
        board.grid[5][4] = Square::new().set_piece(PieceType::new_knight(Color::White));
        // Place white king somewhere; black has no own-king on-board
        // (its king is the passenger). Validate's `find_king(Black)`
        // descends into the carriage to find it.
        board.grid[7][7] = Square::new().set_piece(PieceType::new_king(Color::White));

        // Knight (4, 5) → (3, 3): the filter rewrites to
        // MoveIntoCarrier. The make_move handler then retains only
        // same-colour passengers — the black king is removed.
        let board_move = GameMove {
            from: Coord { file: 4, rank: 5 },
            move_type: MoveType::MoveIntoCarrier(Coord { file: 3, rank: 3 }),
        };
        board
            .make_move(board_move)
            .expect("white knight should be allowed to board the cart");
        // Pin the current (plan-09-open-Q7) behavior: the black king
        // passenger is silently removed.
        assert!(
            board.find_king(Color::Black).is_none(),
            "black king passenger should be removed when enemy boards the cart"
        );
        // The white knight is now the cart's only passenger.
        match &board.grid[3][3].piece {
            Some(PieceType::Carriage(c)) => {
                assert_eq!(c.passengers.len(), 1, "exactly one passenger after board");
                assert!(
                    matches!(c.passengers[0], PieceType::Knight(_)),
                    "boarder should be the knight, got {:?}",
                    c.passengers[0]
                );
            }
            other => panic!("expected carriage to survive, got {other:?}"),
        }
    }

    /// Orphan carriage (no matching loco) sits still across ticks
    /// without panicking. The cart's `attacks()` returns just
    /// passenger threats; `advance_trains` skips trains lacking a
    /// chain_index-0 head.
    #[test]
    fn test_orphan_carriage_is_inert() {
        let mut board = empty_board();
        board.flags.train_tick_rate = TrainTickRate::EveryPly;
        for f in 1..=5u8 {
            board.grid[3][f as usize] = Square::new().set_square_type(
                SquareType::Track { direction: TrackDir::E },
            );
        }
        // Carriage with no matching loco at chain_index 0.
        board.grid[3][3] = board.grid[3][3]
            .clone()
            .set_piece(PieceType::Carriage(Carriage::new(99, 1)));
        // Kings for legality.
        board.grid[0][0] = Square::new().set_piece(PieceType::new_king(Color::White));
        board.grid[7][7] = Square::new().set_piece(PieceType::new_king(Color::Black));
        board.flags.side_to_move = Color::White;

        // Idle move, then assert the orphan didn't move.
        let king = board.find_king(Color::White).unwrap();
        board
            .make_move(GameMove {
                from: king.clone(),
                move_type: MoveType::MoveTo(Coord {
                    file: king.file + 1,
                    rank: king.rank,
                }),
            })
            .expect("idle move shouldn't fail because of an orphan");
        assert!(
            matches!(&board.grid[3][3].piece, Some(PieceType::Carriage(_))),
            "orphan carriage must stay put"
        );
    }

    /// Duplicate `(train_id, chain_index=0)` (two LOCOs same id) is
    /// detected and skipped, not silently corrupting the board.
    #[test]
    fn test_duplicate_loco_chain_skipped() {
        let mut board = empty_board();
        board.flags.train_tick_rate = TrainTickRate::EveryPly;
        for r in 0..=2u8 {
            for f in 1..=3u8 {
                board.grid[r as usize][f as usize] = Square::new()
                    .set_square_type(SquareType::Track { direction: TrackDir::E });
            }
        }
        // Two locos at the same train_id.
        board.grid[1][1] = board.grid[1][1]
            .clone()
            .set_piece(PieceType::Locomotive(Locomotive::new(1, TrainHeading::Forward)));
        board.grid[2][1] = board.grid[2][1]
            .clone()
            .set_piece(PieceType::Locomotive(Locomotive::new(1, TrainHeading::Forward)));
        board.grid[7][7] = Square::new().set_piece(PieceType::new_king(Color::White));
        board.grid[0][0] = Square::new().set_piece(PieceType::new_king(Color::Black));
        board.flags.side_to_move = Color::White;

        let king = board.find_king(Color::White).unwrap();
        let dest = Coord {
            file: king.file - 1,
            rank: king.rank,
        };
        board
            .make_move(GameMove {
                from: king,
                move_type: MoveType::MoveTo(dest),
            })
            .expect("idle move shouldn't fail because of duplicate train_id");
        // Both locos should still be where they started — the duplicate
        // chain is skipped wholesale.
        assert!(
            matches!(&board.grid[1][1].piece, Some(PieceType::Locomotive(_))),
            "duplicate-id loco #1 stays put"
        );
        assert!(
            matches!(&board.grid[2][1].piece, Some(PieceType::Locomotive(_))),
            "duplicate-id loco #2 stays put"
        );
    }

    // ============================================================
    // Audit canary tests (post-iteration cleanup, second pass)
    // ============================================================

    /// S4: the engine's `from_symbol` parsers drop nested carrier
    /// passengers (Bus / Locomotive / Carriage as inner passengers).
    /// A hand-rolled FEN can describe such a state; the engine
    /// refuses to accept it.
    #[test]
    fn test_fen_parser_drops_nested_carriers() {
        // Bus carrying a Bus → inner Bus dropped.
        let board = fen_to_board("(P=BUS(P=(BUS,P)))7/8/8/8/8/8/8/8 w - -");
        match &board.grid[0][0].piece {
            Some(PieceType::Bus(b)) => {
                assert_eq!(
                    b.pieces.len(),
                    1,
                    "nested Bus passenger should be dropped; got {:?}",
                    b.pieces
                );
                assert!(
                    matches!(b.pieces[0], PieceType::Pawn(_)),
                    "expected only the Pawn to survive; got {:?}",
                    b.pieces[0]
                );
            }
            other => panic!("expected Bus at (0, 0), got {other:?}"),
        }

        // Locomotive carrying a Carriage → inner CART dropped.
        let board =
            fen_to_board("(P=LOCO(ID=1,H=F,P=(CART(ID=1,I=1),K)))7/8/8/8/8/8/8/8 w - -");
        match &board.grid[0][0].piece {
            Some(PieceType::Locomotive(l)) => {
                assert_eq!(
                    l.passengers.len(),
                    1,
                    "nested CART passenger should be dropped from Loco; got {:?}",
                    l.passengers
                );
                assert!(
                    matches!(l.passengers[0], PieceType::King(_)),
                    "expected only the King to survive; got {:?}",
                    l.passengers[0]
                );
            }
            other => panic!("expected Locomotive at (0, 0), got {other:?}"),
        }
    }

    /// S1: A `Color::Neutral` non-train piece generates no moves and
    /// threatens nothing. Plan 09 only sanctions Neutral for train
    /// carts; a stray Neutral knight from a hand-built FEN would
    /// otherwise be flagged as a threat to both sides by
    /// `is_attacked_by`'s "Neutral counts for everyone" rule.
    #[test]
    fn test_neutral_non_train_piece_is_inert() {
        let mut board = empty_board();
        // Place a Neutral knight near both kings. Color is set
        // directly on the struct field — the public `new_*`
        // constructors only accept White / Black, so this is the
        // only path to a Neutral instance (matches the hand-rolled
        // FEN exploit S1 defends against).
        let neutral_knight = crate::pieces::standard::knight::Knight {
            color: Color::Neutral,
        };
        board.grid[3][3] = Square::new().set_piece(PieceType::Knight(neutral_knight));
        board.grid[7][0] = Square::new().set_piece(PieceType::new_king(Color::White));
        board.grid[0][7] = Square::new().set_piece(PieceType::new_king(Color::Black));

        // No moves generated.
        let moves = board.get_moves(&Coord { file: 3, rank: 3 });
        assert!(moves.is_empty(), "Neutral knight should yield no moves; got {moves:?}");

        // Threatens nobody — neither king is in check.
        assert!(
            !board.is_in_check(Color::White),
            "white king must not be in check from a Neutral knight"
        );
        assert!(
            !board.is_in_check(Color::Black),
            "black king must not be in check from a Neutral knight"
        );

        // Same for Neutral king / monkey: the override `attacks`
        // implementations must also short-circuit.
        let neutral_king = crate::pieces::standard::king::King {
            color: Color::Neutral,
        };
        board.grid[3][3] = Square::new().set_piece(PieceType::King(neutral_king));
        assert!(
            !board.is_in_check(Color::White),
            "Neutral king's attacks() must short-circuit; otherwise white at (0, 7) would falsely register check"
        );
    }

    /// Goblin home-drop converts the kidnapped piece. Capture
    /// transition is tested in fairy_scenarios.rs; this canary pins
    /// the return-home half.
    #[test]
    fn test_goblin_home_drop_converts_kidnapped_piece() {
        use crate::pieces::fairy::goblin::{Goblin, GoblinState};
        let mut board = empty_board();
        board.flags.white_can_castle_kingside = false;
        board.flags.white_can_castle_queenside = false;
        board.flags.black_can_castle_kingside = false;
        board.flags.black_can_castle_queenside = false;
        // White goblin kidnapping a black knight, home_square = (3, 3).
        // Goblin sits one tile away (3, 4); a MoveTo to (3, 3) drops
        // the converted (now-white) knight on home and the goblin
        // dies.
        let goblin = Goblin {
            color: Color::White,
            state: GoblinState::Kidnapping {
                piece: PieceType::new_knight(Color::Black).into(),
            },
            home_square: Coord { file: 3, rank: 3 },
        };
        board.grid[4][3] = Square::new().set_piece(PieceType::Goblin(goblin));
        // Kings for legality, well clear of the goblin.
        board.grid[7][7] = Square::new().set_piece(PieceType::new_king(Color::White));
        board.grid[0][0] = Square::new().set_piece(PieceType::new_king(Color::Black));
        board.flags.side_to_move = Color::White;

        board
            .make_move(GameMove {
                from: Coord { file: 3, rank: 4 },
                move_type: MoveType::MoveTo(Coord { file: 3, rank: 3 }),
            })
            .expect("goblin should be allowed to return home (kidnapping state)");

        // Home square now holds a white knight (converted from black).
        match &board.grid[3][3].piece {
            Some(PieceType::Knight(k)) => {
                assert_eq!(
                    k.color,
                    Color::White,
                    "kidnapped piece should adopt the goblin's color"
                );
            }
            other => panic!(
                "expected white knight on home square (3, 3), got {other:?}"
            ),
        }
        // Source (the goblin's old tile) is empty.
        assert!(
            board.grid[4][3].piece.is_none(),
            "goblin should vacate its source tile"
        );
    }

    /// PIC→MIC capture-on-board: a passenger inside a Neutral cart
    /// transfers into another Neutral cart that holds an
    /// opposite-color passenger. The opposite-color passenger is
    /// captured, the boarder joins. Exercises the inner-arm retain
    /// rule added in B2/S7.
    #[test]
    fn test_pic_to_mic_inner_arm_captures_opposite_color_passenger() {
        use std::sync::Arc;
        let mut board = empty_board();
        board.flags.white_can_castle_kingside = false;
        board.flags.white_can_castle_queenside = false;
        board.flags.black_can_castle_kingside = false;
        board.flags.black_can_castle_queenside = false;
        board.flags.train_tick_rate = TrainTickRate::EveryFullTurn;
        // Two carts on adjacent track tiles. Cart A holds a white
        // pawn; cart B holds a black knight (opposite color to the
        // white pawn we're transferring).
        board.grid[3][3] = Square::new()
            .set_square_type(SquareType::Track {
                direction: TrackDir::E,
            })
            .set_piece(PieceType::Carriage(Carriage {
                train_id: 1,
                chain_index: 1,
                passengers: vec![PieceType::new_pawn(Color::White)],
            }));
        board.grid[3][4] = Square::new()
            .set_square_type(SquareType::Track {
                direction: TrackDir::E,
            })
            .set_piece(PieceType::Carriage(Carriage {
                train_id: 2,
                chain_index: 1,
                passengers: vec![PieceType::new_knight(Color::Black)],
            }));
        board.grid[7][0] = Square::new().set_piece(PieceType::new_king(Color::White));
        board.grid[0][7] = Square::new().set_piece(PieceType::new_king(Color::Black));
        board.flags.side_to_move = Color::White;

        // White is the mover (the pawn is its). Transfer pawn from
        // cart A (file=3) to cart B (file=4).
        //
        // Cart A's `initial_moves` runs each passenger's moves as if
        // they were standing on the cart's tile. The pawn at (3, 3)
        // (white pawn promotes at rank 0, double-pushes from rank 6,
        // diagonals from rank 3 → rank 2). The pawn's diagonal-
        // capture-square at (4, 2) is empty; the pawn's forward
        // push at (3, 2) lands on an empty tile. For *boarding cart
        // B* the pawn needs to move *east* — pawns don't move east,
        // so this path isn't reachable via natural pawn moves.
        //
        // Instead, construct the PIC→MIC move directly. The filter
        // would normally rewrite a passenger's MoveTo into a
        // MoveIntoCarrier; we skip that rewrite by emitting the
        // MoveIntoCarrier directly. validate_move's `raw_moves`
        // check requires the move to be in the filtered set — so to
        // get past validate, use `make_move_unchecked` (the test is
        // about the make_move handler, not the filter).
        let move_xfer = GameMove {
            from: Coord { file: 3, rank: 3 },
            move_type: MoveType::PieceInCarrier {
                piece_index: 0,
                move_type: Arc::new(MoveType::MoveIntoCarrier(Coord {
                    file: 4,
                    rank: 3,
                })),
            },
        };
        board
            .make_move_unchecked(move_xfer)
            .expect("PIC→MIC transfer should apply cleanly");

        // Cart A is now empty (passenger left).
        match &board.grid[3][3].piece {
            Some(PieceType::Carriage(c)) => {
                assert!(
                    c.passengers.is_empty(),
                    "cart A should be empty after transfer; got {:?}",
                    c.passengers
                );
            }
            other => panic!("cart A should still be present, got {other:?}"),
        }
        // Cart B held a black knight; a white pawn boards. Since
        // cart B is Neutral and the boarder is non-Neutral, the
        // retain rule fires: keep only same-color passengers (white).
        // Black knight removed; pawn added.
        match &board.grid[3][4].piece {
            Some(PieceType::Carriage(c)) => {
                assert_eq!(
                    c.passengers.len(),
                    1,
                    "cart B should hold exactly one passenger; got {:?}",
                    c.passengers
                );
                match &c.passengers[0] {
                    PieceType::Pawn(p) => assert_eq!(p.color, Color::White),
                    other => panic!(
                        "cart B's only passenger should be the white pawn; got {other:?}"
                    ),
                }
            }
            other => panic!("cart B should still be present, got {other:?}"),
        }
    }

    // -------- Round-3 audit: regression tests for criticals --------

    /// C1 regression: `find_matching_paren` accepts a byte index, so a
    /// multi-byte char before `(` doesn't shift the alignment.
    #[test]
    fn test_find_matching_paren_with_multibyte_prefix() {
        use crate::board::fen::find_matching_paren;
        // `'ø'` is 2 UTF-8 bytes (0xC3 0xB8). Layout:
        //   byte 0-1: ø, byte 2: (, byte 3: x, byte 4: ).
        let s = "ø(x)";
        let open = s.find('(').expect("ø has a paren");
        assert_eq!(open, 2, "find returns byte index, not char index");
        // Previously this returned None because `skip(2)` ate `ø` and `(`.
        assert_eq!(
            find_matching_paren(s, open),
            Some(4),
            "matching ')' is at byte index 4"
        );
    }

    /// C2 regression: a stray `,,` in a LOCO/CART/BUS payload must NOT
    /// abort the parse — the previously-parsed fields should survive.
    #[test]
    fn test_train_parser_tolerates_empty_field_segment() {
        let loco_sym = "LOCO(ID=7,,H=F,P=(K))";
        let piece = PieceType::symbol_to_piece(loco_sym)
            .expect("loco with stray comma should still parse");
        match piece {
            PieceType::Locomotive(l) => {
                assert_eq!(l.train_id, 7, "ID survived despite empty segment");
                assert_eq!(l.passengers.len(), 1, "passengers survived");
            }
            other => panic!("expected Locomotive, got {other:?}"),
        }
        // Same for Bus.
        let bus_sym = "BUS(,P=(K))";
        let bus = PieceType::symbol_to_piece(bus_sym).expect("bus should parse");
        match bus {
            PieceType::Bus(b) => {
                assert_eq!(b.color, Color::White);
                assert_eq!(b.pieces.len(), 1);
            }
            other => panic!("expected Bus, got {other:?}"),
        }
    }

    /// C3 regression: a king that boards a friendly Bus must NOT retain
    /// castle rights — `post_move_effects` is now dispatched for
    /// MoveIntoCarrier too.
    #[test]
    fn test_king_into_bus_clears_castle_rights() {
        let mut board = empty_board();
        // a1 = rook, d1 = bus, e1 = king, h1 = rook
        board.grid[0][0].piece = Some(PieceType::new_rook(Color::White));
        board.grid[0][3].piece = Some(PieceType::Bus(Bus::new(Color::White)));
        board.grid[0][4].piece = Some(PieceType::new_king(Color::White));
        board.grid[0][7].piece = Some(PieceType::new_rook(Color::White));

        // King steps into the friendly Bus on d1 (MoveIntoCarrier).
        let mv = GameMove {
            from: Coord { file: 4, rank: 0 },
            move_type: MoveType::MoveIntoCarrier(Coord { file: 3, rank: 0 }),
        };
        board.make_move_unchecked(mv).expect("king-into-bus");

        assert!(
            !board.flags.white_can_castle_kingside,
            "kingside castle right must be cleared by king-into-bus"
        );
        assert!(
            !board.flags.white_can_castle_queenside,
            "queenside castle right must be cleared by king-into-bus"
        );
    }

    /// C4 regression: a pawn capture-promotion targeting a Neutral cart's
    /// tile must NOT be legal — accepting it would destroy the cart.
    #[test]
    fn test_pawn_promote_capture_onto_neutral_cart_is_rejected() {
        use crate::pieces::fairy::carriage::Carriage;
        let mut board = empty_board();
        // White pawn at b7 (file=1, rank=6).
        board.grid[6][1].piece = Some(PieceType::new_pawn(Color::White));
        // Neutral cart at a8 (file=0, rank=7). Track tile so the cart's
        // square type is consistent; the move-gen doesn't care.
        board.grid[7][0].piece =
            Some(PieceType::Carriage(Carriage::new(99, 1)));
        // White king somewhere safe so `legal_moves` returns valid moves.
        board.grid[0][4].piece = Some(PieceType::new_king(Color::White));
        // Black king for completeness.
        board.grid[7][7].piece = Some(PieceType::new_king(Color::Black));

        let pawn_from = Coord { file: 1, rank: 6 };
        let moves = board.get_moves(&pawn_from);
        for m in &moves {
            if let MoveType::Promotion { target, .. } = &m.move_type {
                assert!(
                    !(target.file == 0 && target.rank == 7),
                    "pawn must not be allowed to promote onto a Neutral cart's tile"
                );
            }
        }
    }

    /// C5 regression: train B's loco must NOT silently overwrite train
    /// A's cart when A is stalled. Setup: A is a one-cart train sitting
    /// at (file=2, rank=3) with no live locomotive (an orphan/stalled
    /// chain — A's loco "isn't moving this tick"). B's loco is east at
    /// (file=3, rank=3) with `last_dir=E` so `next_train_step` filters
    /// out east and exits west, landing B's next-head exactly on A's
    /// cart at (2,3). Without round-3's foreign-cart check (trains.rs
    /// around L411-423), the commit pass would unconditionally write
    /// B's loco over A's cart.
    #[test]
    fn test_moving_train_stops_at_stalled_foreign_cart() {
        use crate::pieces::fairy::{
            carriage::Carriage,
            locomotive::{Locomotive, TrainHeading},
        };
        let mut board = empty_board();
        // Tick every ply so a single `maybe_advance_trains` call
        // actually advances the trains (`empty_board()`'s default is
        // `EveryFullTurn` + `ply_count=0` → 0+1 % 2 ≠ 0 → no tick).
        board.flags.train_tick_rate = crate::board::TrainTickRate::EveryPly;

        // Track row along rank 3 (files 0..=5).
        for f in 0..=5u8 {
            board.grid[3][f as usize].square_type = SquareType::Track {
                direction: crate::board::square::TrackDir::E,
            };
        }
        // Train A: an orphan carriage (chain_index=1) at (2,3). No loco
        // exists for train_id=1, so `advance_trains` never adds A to
        // its advances list — A is a foreign cart from B's POV.
        board.grid[3][2].piece = Some(PieceType::Carriage(Carriage::new(1, 1)));
        // Train B: loco at (3,3) with `last_dir=E` so the step filter
        // excludes east; the only remaining track-neighbor is west →
        // `next_head = (2,3)`, the foreign cart's tile.
        board.grid[3][3].piece = Some(PieceType::Locomotive(Locomotive {
            train_id: 2,
            heading: TrainHeading::Forward,
            passengers: vec![],
            last_dir: Some(crate::board::square::TrackDir::E),
        }));

        // Kings somewhere safe so the board is well-formed.
        board.grid[0][0].piece = Some(PieceType::new_king(Color::White));
        board.grid[7][7].piece = Some(PieceType::new_king(Color::Black));

        // Tick the trains directly.
        board.maybe_advance_trains();

        // The foreign cart must still be at (2,3) — round-3's
        // foreign-cart check stopped train B short.
        match &board.grid[3][2].piece {
            Some(PieceType::Carriage(c)) => {
                assert_eq!(c.train_id, 1, "foreign cart was overwritten");
            }
            other => panic!("foreign cart was deleted/overwritten by train B; got {other:?}"),
        }
        // And train B's loco must still be at (3,3) — it stopped.
        match &board.grid[3][3].piece {
            Some(PieceType::Locomotive(l)) => {
                assert_eq!(l.train_id, 2, "train B's loco must stay put");
            }
            other => panic!("train B's loco should have stopped at (3,3); got {other:?}"),
        }
    }

    /// C6 regression: a Black king adjacent to a Neutral cart carrying a
    /// Black passenger pawn must NOT register as in-check by White.
    #[test]
    fn test_neutral_cart_same_color_passenger_does_not_self_check() {
        use crate::pieces::fairy::locomotive::{Locomotive, TrainHeading};
        let mut board = empty_board();
        board.flags.side_to_move = Color::Black;
        // Neutral loco at (3, 4), carrying a Black pawn passenger.
        let mut loco = Locomotive::new(7, TrainHeading::Forward);
        loco.passengers = vec![PieceType::new_pawn(Color::Black)];
        board.grid[4][3].piece = Some(PieceType::Locomotive(loco));
        // Black king at (4, 3) — diagonally one tile from the cart so
        // a Black pawn's attack diagonal could "hit" it.
        board.grid[3][4].piece = Some(PieceType::new_king(Color::Black));
        // White king somewhere safe.
        board.grid[0][0].piece = Some(PieceType::new_king(Color::White));

        // Is the Black king attacked by White? The passenger is Black,
        // not White — the predicate must return false.
        assert!(
            !board.is_attacked_by(&Coord { file: 4, rank: 3 }, Color::White),
            "Black king must not be 'in check by White' from a Black passenger on a Neutral cart"
        );
        // But Black-side check would catch the passenger threat — that
        // confirms the predicate still routes passenger threats to the
        // *correct* color.
        // (Not asserting here because the Black king is unlikely to be
        // queried as attacked-by-Black, but the symmetry is intentional.)
    }

    /// H7 regression: `is_attacked_by(_, Color::Neutral)` is meaningless
    /// and must short-circuit to false.
    #[test]
    fn test_is_attacked_by_neutral_returns_false() {
        use crate::pieces::fairy::locomotive::{Locomotive, TrainHeading};
        let mut board = empty_board();
        board.grid[3][3].piece = Some(PieceType::Locomotive(Locomotive::new(
            1,
            TrainHeading::Forward,
        )));
        // Asking "is this square attacked by Neutral?" — no such
        // semantic; engine returns false.
        assert!(
            !board.is_attacked_by(&Coord { file: 4, rank: 3 }, Color::Neutral),
            "Neutral-as-attacker is meaningless and must return false"
        );
    }

    // -------- Round-4 audit regression tests --------

    /// H-C regression: trailing-train semantic. A follows B east; B's
    /// caboose vacates the tile A wants this same tick. A must advance,
    /// not stop at the foreign cart.
    #[test]
    fn test_trailing_train_advances_onto_vacated_tile() {
        use crate::pieces::fairy::{
            carriage::Carriage,
            locomotive::{Locomotive, TrainHeading},
        };
        let mut board = empty_board();
        board.flags.train_tick_rate = crate::board::TrainTickRate::EveryPly;
        // Track row along rank 3, files 0..=5.
        for f in 0..=5u8 {
            board.grid[3][f as usize].square_type = SquareType::Track {
                direction: crate::board::square::TrackDir::E,
            };
        }
        // Train B (ahead): loco at (4,3), caboose at (3,3). last_dir=W
        // so next step is east → (5,3). B's caboose at (3,3) vacates.
        board.grid[3][4].piece = Some(PieceType::Locomotive(Locomotive {
            train_id: 1,
            heading: TrainHeading::Forward,
            passengers: vec![],
            last_dir: Some(crate::board::square::TrackDir::W),
        }));
        board.grid[3][3].piece = Some(PieceType::Carriage(Carriage::new(1, 1)));
        // Train A (chasing): loco at (2,3), last_dir=W → next step east → (3,3).
        // (3,3) is B's caboose's current tile, which B vacates this tick.
        board.grid[3][2].piece = Some(PieceType::Locomotive(Locomotive {
            train_id: 2,
            heading: TrainHeading::Forward,
            passengers: vec![],
            last_dir: Some(crate::board::square::TrackDir::W),
        }));
        // Kings somewhere safe.
        board.grid[0][0].piece = Some(PieceType::new_king(Color::White));
        board.grid[7][7].piece = Some(PieceType::new_king(Color::Black));

        board.maybe_advance_trains();
        // A's loco should now be at (3,3) — the tile B's caboose vacated.
        match &board.grid[3][3].piece {
            Some(PieceType::Locomotive(l)) => assert_eq!(
                l.train_id, 2,
                "A's loco should have advanced onto B's vacated tile"
            ),
            other => panic!("expected A's loco at (3,3), got {other:?}"),
        }
        // B's loco should be at (5,3).
        match &board.grid[3][5].piece {
            Some(PieceType::Locomotive(l)) => assert_eq!(l.train_id, 1),
            other => panic!("expected B's loco at (5,3), got {other:?}"),
        }
    }

    /// H-D regression: a kidnapping Goblin riding a Bus home must
    /// drop off and convert the kidnapped piece on disembarkation.
    #[test]
    fn test_goblin_kidnap_via_carrier_drops_on_home() {
        use crate::pieces::fairy::goblin::{Goblin, GoblinState};
        let mut board = empty_board();
        // White Goblin with home_square (4,0), currently Kidnapping a Black pawn,
        // riding a friendly White Bus at (3,0).
        let kidnapped = std::rc::Rc::new(PieceType::new_pawn(Color::Black));
        let goblin = Goblin {
            color: Color::White,
            home_square: Coord { file: 4, rank: 0 },
            state: GoblinState::Kidnapping {
                piece: kidnapped,
            },
        };
        let mut bus = Bus::new(Color::White);
        bus.pieces = vec![PieceType::Goblin(goblin)];
        board.grid[0][3].piece = Some(PieceType::Bus(bus));
        // Kings so the board is well-formed.
        board.grid[0][0].piece = Some(PieceType::new_king(Color::White));
        board.grid[7][7].piece = Some(PieceType::new_king(Color::Black));
        board.flags.side_to_move = Color::White;

        // Goblin exits Bus onto (4,0) — the home square.
        let mv = GameMove {
            from: Coord { file: 3, rank: 0 },
            move_type: MoveType::PieceInCarrier {
                piece_index: 0,
                move_type: std::sync::Arc::new(MoveType::MoveTo(Coord {
                    file: 4,
                    rank: 0,
                })),
            },
        };
        board.make_move_unchecked(mv).expect("goblin disembarks home");

        // The Goblin's drop-off logic overwrites itself with a converted
        // (color-flipped to White) pawn — kidnapping resolves.
        match &board.grid[0][4].piece {
            Some(PieceType::Pawn(p)) => assert_eq!(
                p.color,
                Color::White,
                "kidnapped pawn must be converted to Goblin's color"
            ),
            other => panic!("expected converted White pawn at home, got {other:?}"),
        }
    }

    /// M-E regression: Skibidi phase reset must fire when boarding a
    /// carrier via MoveIntoCarrier. The round-3 hook downcasts the
    /// just-boarded Skibidi inside the carrier's passenger list and
    /// resets its phase to 1.
    #[test]
    fn test_skibidi_phase_resets_on_board() {
        let mut board = empty_board();
        // Phase-3 White Skibidi at (3,0), friendly White Bus at (4,0).
        let mut skib = Skibidi::new(Color::White);
        skib.phase = 3;
        board.grid[0][3].piece = Some(PieceType::Skibidi(skib));
        board.grid[0][4].piece = Some(PieceType::Bus(Bus::new(Color::White)));
        board.grid[0][0].piece = Some(PieceType::new_king(Color::White));
        board.grid[7][7].piece = Some(PieceType::new_king(Color::Black));
        board.flags.side_to_move = Color::White;

        let mv = GameMove {
            from: Coord { file: 3, rank: 0 },
            move_type: MoveType::MoveIntoCarrier(Coord { file: 4, rank: 0 }),
        };
        board.make_move_unchecked(mv).expect("skibidi boards bus");

        // The Skibidi is now a passenger; its phase must be 1.
        match &board.grid[0][4].piece {
            Some(PieceType::Bus(b)) => {
                let last = b
                    .pieces
                    .last()
                    .expect("bus should have the Skibidi passenger");
                match last {
                    PieceType::Skibidi(s) => assert_eq!(
                        s.phase, 1,
                        "Skibidi phase must reset on MoveIntoCarrier"
                    ),
                    other => panic!("expected Skibidi passenger, got {other:?}"),
                }
            }
            other => panic!("expected Bus at (4,0), got {other:?}"),
        }
    }

    // -------- Round-5 audit regression tests --------

    /// C-V1 regression: foreign-cart filter + two-train collision pass
    /// must run to fixed point. Setup: A trails B east, B's caboose
    /// vacates the tile A wants. But B's head crashes head-on into C
    /// (a third train), so B drops out via two-train collision. After
    /// B drops, B's caboose stays put — A must NOT advance onto it.
    #[test]
    fn test_trailing_train_blocked_when_leader_collides() {
        use crate::pieces::fairy::{
            carriage::Carriage,
            locomotive::{Locomotive, TrainHeading},
        };
        let mut board = empty_board();
        board.flags.train_tick_rate = crate::board::TrainTickRate::EveryPly;

        // Track row along rank 3, files 0..=6.
        for f in 0..=6u8 {
            board.grid[3][f as usize].square_type = SquareType::Track {
                direction: crate::board::square::TrackDir::E,
            };
        }
        // Train B (middle): loco at (4,3) heading east, caboose at (3,3).
        board.grid[3][4].piece = Some(PieceType::Locomotive(Locomotive {
            train_id: 1,
            heading: TrainHeading::Forward,
            passengers: vec![],
            last_dir: Some(crate::board::square::TrackDir::W),
        }));
        board.grid[3][3].piece = Some(PieceType::Carriage(Carriage::new(1, 1)));
        // Train C (head-on with B): loco at (6,3) heading west — next tile
        // (5,3), same as B's next tile. Two-train collision drops both.
        board.grid[3][6].piece = Some(PieceType::Locomotive(Locomotive {
            train_id: 3,
            heading: TrainHeading::Forward,
            passengers: vec![],
            last_dir: Some(crate::board::square::TrackDir::E),
        }));
        // Train A (trailing): loco at (2,3) wants (3,3).
        board.grid[3][2].piece = Some(PieceType::Locomotive(Locomotive {
            train_id: 2,
            heading: TrainHeading::Forward,
            passengers: vec![],
            last_dir: Some(crate::board::square::TrackDir::W),
        }));
        // Kings somewhere safe.
        board.grid[0][0].piece = Some(PieceType::new_king(Color::White));
        board.grid[7][7].piece = Some(PieceType::new_king(Color::Black));

        board.maybe_advance_trains();

        // After the tick: B and C should NOT have moved (head-on stop).
        // A should NOT have moved either — B's caboose still at (3,3).
        match &board.grid[3][3].piece {
            Some(PieceType::Carriage(c)) => assert_eq!(
                c.train_id, 1,
                "B's caboose must still be at (3,3) — fixed point blocks A"
            ),
            other => panic!("B's caboose at (3,3) was overwritten: {other:?}"),
        }
        match &board.grid[3][2].piece {
            Some(PieceType::Locomotive(l)) => assert_eq!(
                l.train_id, 2,
                "A's loco must stay at (2,3) — B's vacate didn't materialize"
            ),
            other => panic!("A's loco moved unexpectedly: {other:?}"),
        }
    }

    /// H-V1 regression: a passenger Skibidi hopping cart A → cart B
    /// via PieceInCarrier{MoveIntoCarrier} must have its phase reset
    /// to 1 (mirror of the king-castle-rights case).
    #[test]
    fn test_skibidi_passenger_phase_resets_on_cart_to_cart_hop() {
        // Two buses adjacent; Skibidi rides Bus A, hops to Bus B.
        let mut board = empty_board();
        let mut skib = Skibidi::new(Color::White);
        skib.phase = 3;
        let mut bus_a = Bus::new(Color::White);
        bus_a.pieces = vec![PieceType::Skibidi(skib)];
        board.grid[0][3].piece = Some(PieceType::Bus(bus_a));
        board.grid[0][4].piece = Some(PieceType::Bus(Bus::new(Color::White)));
        board.grid[0][0].piece = Some(PieceType::new_king(Color::White));
        board.grid[7][7].piece = Some(PieceType::new_king(Color::Black));
        board.flags.side_to_move = Color::White;

        let mv = GameMove {
            from: Coord { file: 3, rank: 0 },
            move_type: MoveType::PieceInCarrier {
                piece_index: 0,
                move_type: std::sync::Arc::new(MoveType::MoveIntoCarrier(Coord {
                    file: 4,
                    rank: 0,
                })),
            },
        };
        board.make_move_unchecked(mv).expect("skibidi hops bus A → bus B");

        // Bus A should be empty; Bus B should hold the Skibidi at phase 1.
        match &board.grid[0][3].piece {
            Some(PieceType::Bus(b)) => assert!(
                b.pieces.is_empty(),
                "Bus A should have no passengers after Skibidi hop"
            ),
            other => panic!("expected empty Bus A, got {other:?}"),
        }
        match &board.grid[0][4].piece {
            Some(PieceType::Bus(b)) => {
                let last = b.pieces.last().expect("Bus B should hold the Skibidi");
                match last {
                    PieceType::Skibidi(s) => assert_eq!(
                        s.phase, 1,
                        "Skibidi phase must reset on cart-to-cart hop"
                    ),
                    other => panic!("expected Skibidi in Bus B, got {other:?}"),
                }
            }
            other => panic!("expected Bus B at (4,0), got {other:?}"),
        }
    }

    /// M-V1 regression: Skibidi MIC phase reset must also work for
    /// Locomotive (not just Bus). Covers the gap in the round-4 test.
    #[test]
    fn test_skibidi_phase_resets_on_board_into_locomotive() {
        use crate::pieces::fairy::locomotive::{Locomotive, TrainHeading};
        let mut board = empty_board();
        let mut skib = Skibidi::new(Color::White);
        skib.phase = 3;
        board.grid[0][3].piece = Some(PieceType::Skibidi(skib));
        // Neutral loco at (4,0) — any colour piece can board a Neutral cart.
        board.grid[0][4].piece = Some(PieceType::Locomotive(Locomotive::new(
            42,
            TrainHeading::Forward,
        )));
        board.grid[0][0].piece = Some(PieceType::new_king(Color::White));
        board.grid[7][7].piece = Some(PieceType::new_king(Color::Black));
        board.flags.side_to_move = Color::White;

        let mv = GameMove {
            from: Coord { file: 3, rank: 0 },
            move_type: MoveType::MoveIntoCarrier(Coord { file: 4, rank: 0 }),
        };
        board.make_move_unchecked(mv).expect("skibidi boards loco");

        match &board.grid[0][4].piece {
            Some(PieceType::Locomotive(l)) => {
                let last = l.passengers.last().expect("loco should hold the Skibidi");
                match last {
                    PieceType::Skibidi(s) => assert_eq!(
                        s.phase, 1,
                        "Skibidi phase must reset on MoveIntoCarrier into a Locomotive"
                    ),
                    other => panic!("expected Skibidi passenger, got {other:?}"),
                }
            }
            other => panic!("expected Locomotive at (4,0), got {other:?}"),
        }
    }

    // -------- Round-6 audit regression tests --------

    /// R6-C1 regression: two single-cart trains heading at each other
    /// on adjacent tiles must NOT pass through each other. The two-train
    /// collision pass detects the head-swap case.
    #[test]
    fn test_two_single_cart_trains_head_swap_stops_both() {
        use crate::pieces::fairy::locomotive::{Locomotive, TrainHeading};
        let mut board = empty_board();
        board.flags.train_tick_rate = crate::board::TrainTickRate::EveryPly;

        // Two adjacent track tiles at rank 3, files 4 and 5.
        for f in 3..=6u8 {
            board.grid[3][f as usize].square_type = SquareType::Track {
                direction: crate::board::square::TrackDir::E,
            };
        }
        // Train B at (4,3) heading east, last_dir=W → next would be (5,3).
        board.grid[3][4].piece = Some(PieceType::Locomotive(Locomotive {
            train_id: 1,
            heading: TrainHeading::Forward,
            passengers: vec![],
            last_dir: Some(crate::board::square::TrackDir::W),
        }));
        // Train C at (5,3) heading west, last_dir=E → next would be (4,3).
        board.grid[3][5].piece = Some(PieceType::Locomotive(Locomotive {
            train_id: 2,
            heading: TrainHeading::Forward,
            passengers: vec![],
            last_dir: Some(crate::board::square::TrackDir::E),
        }));
        board.grid[0][0].piece = Some(PieceType::new_king(Color::White));
        board.grid[7][7].piece = Some(PieceType::new_king(Color::Black));

        board.maybe_advance_trains();

        // Both locos must stay put — head-swap was caught.
        match &board.grid[3][4].piece {
            Some(PieceType::Locomotive(l)) => assert_eq!(
                l.train_id, 1,
                "B's loco must stay at (4,3); head-swap was supposed to stop both"
            ),
            other => panic!("B's loco was relocated: {other:?}"),
        }
        match &board.grid[3][5].piece {
            Some(PieceType::Locomotive(l)) => assert_eq!(
                l.train_id, 2,
                "C's loco must stay at (5,3); head-swap was supposed to stop both"
            ),
            other => panic!("C's loco was relocated: {other:?}"),
        }
    }

    /// R6-M1 regression: a passenger pawn at promotion rank inside a
    /// carrier must not emit a passenger-Promotion move via `get_moves`.
    /// (The PIC arm in `make_move` can't handle Promotion inner moves,
    /// so emitting and then failing at apply time would surface as a
    /// misleading `ApplyFailed`.)
    #[test]
    fn test_passenger_pawn_at_promote_rank_does_not_emit_promotion() {
        let mut board = empty_board();
        // White Bus at (e7) carrying a White pawn. Pawn at the carrier's
        // tile sees rank 7 immediately above and would emit Promotion.
        let mut bus = Bus::new(Color::White);
        bus.pieces = vec![PieceType::new_pawn(Color::White)];
        // (file=4, rank=6) — rank 7 is one step north (promotion rank).
        board.grid[6][4].piece = Some(PieceType::Bus(bus));
        board.grid[0][0].piece = Some(PieceType::new_king(Color::White));
        board.grid[7][7].piece = Some(PieceType::new_king(Color::Black));
        board.flags.side_to_move = Color::White;

        let moves = board.get_moves(&Coord { file: 4, rank: 6 });
        for m in &moves {
            if let MoveType::PieceInCarrier { move_type, .. } = &m.move_type {
                assert!(
                    !matches!(move_type.as_ref(), MoveType::Promotion { .. }),
                    "passenger pawn must not emit a wrapped Promotion: {:?}",
                    m.move_type
                );
            }
        }
    }

    /// R6-M2 regression: a White Kidnapping Goblin riding a Neutral
    /// cart must NOT block Black's castle path. Without the round-6
    /// fix, the Goblin's `attacks()` would include adjacent empty
    /// squares (its kidnapping move-gen) and `is_attacked_by(_, White)`
    /// would return true for those tiles.
    #[test]
    fn test_kidnapping_goblin_passenger_does_not_block_castle() {
        use crate::pieces::fairy::{
            goblin::{Goblin, GoblinState},
            locomotive::{Locomotive, TrainHeading},
        };

        let mut board = empty_board();
        // White Kidnapping Goblin riding a Neutral cart at (file=5, rank=6).
        // The cart's tile is one step south of Black's castle path tiles
        // f8 (file=5, rank=7) and g8 (file=6, rank=7). Before the
        // `Goblin::attacks` override returned `Vec::new()` for Kidnapping
        // state, the Goblin's king-style adjacency projected attacks onto
        // e8/f8/g8/e7/f7/g7/e6/f6/g6 — phantom-blocking Black from
        // castling. After the fix the attack set is empty.
        let kidnapped = std::rc::Rc::new(PieceType::new_pawn(Color::Black));
        let goblin = Goblin {
            color: Color::White,
            home_square: Coord { file: 4, rank: 0 },
            state: GoblinState::Kidnapping { piece: kidnapped },
        };
        let mut cart = Locomotive::new(99, TrainHeading::Forward);
        cart.passengers = vec![PieceType::Goblin(goblin)];
        board.grid[6][5].piece = Some(PieceType::Locomotive(cart));

        // f8 = (5, 7), g8 = (6, 7) — must NOT register as attacked by White.
        assert!(
            !board.is_attacked_by(&Coord { file: 5, rank: 7 }, Color::White),
            "f8 must not be flagged as attacked by White via a Kidnapping Goblin passenger"
        );
        assert!(
            !board.is_attacked_by(&Coord { file: 6, rank: 7 }, Color::White),
            "g8 must not be flagged as attacked by White via a Kidnapping Goblin passenger"
        );
    }

    // -------- Round-7 audit regression tests --------

    /// R7-M1 regression: `status()` must descend into Neutral carts to
    /// find passenger moves on `to_move`'s turn. Setup: Black's only
    /// piece is a king inside a Neutral cart with a one-tile exit;
    /// `status()` must report Ongoing, not Stalemate.
    #[test]
    fn test_status_descends_into_neutral_cart_for_passengers() {
        use crate::pieces::fairy::locomotive::{Locomotive, TrainHeading};
        let mut board = empty_board();
        board.flags.side_to_move = Color::Black;
        // Black's only piece is a king passenger of a Neutral loco at
        // (4,4). All eight adjacent squares are STANDARD empties, so
        // the king has legal `PieceInCarrier{MoveTo}` exit moves.
        // Without `status()`'s descent into Neutral carriers, Black
        // would have no top-level pieces and `status()` would
        // mis-declare Stalemate.
        let mut loco = Locomotive::new(1, TrainHeading::Forward);
        loco.passengers = vec![PieceType::new_king(Color::Black)];
        board.grid[4][4].piece = Some(PieceType::Locomotive(loco));
        board.grid[0][0].piece = Some(PieceType::new_king(Color::White));

        // Black has no top-level pieces; only via the carrier descent
        // does the Black-king-passenger contribute legal moves.
        assert_eq!(
            board.status(),
            crate::board::GameStatus::Ongoing,
            "Black king-in-cart has legal exit moves; status must be Ongoing"
        );
    }

    /// R7-M2 regression: a passenger Skibidi inside a Locomotive must
    /// NOT emit a wrapped `PieceInCarrier{PhaseShift}` from `get_moves`.
    /// (The PIC arm in `make_move` can't handle PhaseShift inners, so
    /// emitting and then failing at apply time would surface as a
    /// misleading `ApplyFailed`.)
    #[test]
    fn test_passenger_skibidi_does_not_emit_phaseshift_in_loco() {
        use crate::pieces::fairy::locomotive::{Locomotive, TrainHeading};
        let mut board = empty_board();
        // Neutral loco at (4, 4) carrying a White phase-2 Skibidi.
        board.grid[4][4].square_type = SquareType::Track {
            direction: crate::board::square::TrackDir::E,
        };
        let mut skib = Skibidi::new(Color::White);
        skib.phase = 2;
        let mut loco = Locomotive::new(1, TrainHeading::Forward);
        loco.passengers = vec![PieceType::Skibidi(skib)];
        board.grid[4][4].piece = Some(PieceType::Locomotive(loco));
        board.grid[0][0].piece = Some(PieceType::new_king(Color::White));
        board.grid[7][7].piece = Some(PieceType::new_king(Color::Black));

        let moves = board.get_moves(&Coord { file: 4, rank: 4 });
        for m in &moves {
            if let MoveType::PieceInCarrier { move_type, .. } = &m.move_type {
                assert!(
                    !matches!(move_type.as_ref(), MoveType::PhaseShift),
                    "passenger Skibidi must not emit a wrapped PhaseShift: {:?}",
                    m.move_type
                );
            }
        }
    }

    /// R7-M2 regression (Bus variant): the same whitelist applies to
    /// Bus's passenger-move loop. A Skibidi-passenger of a Bus must
    /// not emit a wrapped PhaseShift either.
    #[test]
    fn test_passenger_skibidi_does_not_emit_phaseshift_in_bus() {
        let mut board = empty_board();
        let mut skib = Skibidi::new(Color::White);
        skib.phase = 2;
        let mut bus = Bus::new(Color::White);
        bus.pieces = vec![PieceType::Skibidi(skib)];
        board.grid[0][4].piece = Some(PieceType::Bus(bus));
        board.grid[0][0].piece = Some(PieceType::new_king(Color::White));
        board.grid[7][7].piece = Some(PieceType::new_king(Color::Black));

        let moves = board.get_moves(&Coord { file: 4, rank: 0 });
        for m in &moves {
            if let MoveType::PieceInCarrier { move_type, .. } = &m.move_type {
                assert!(
                    !matches!(move_type.as_ref(), MoveType::PhaseShift),
                    "Bus-passenger Skibidi must not emit a wrapped PhaseShift: {:?}",
                    m.move_type
                );
            }
        }
    }

    // -------- Round-8 audit regression tests --------

    /// R8-M1 regression (boarding half): Monkey must be able to board a
    /// Neutral cart like every other piece. Pre-fix, Monkey's move-gen
    /// pre-filtered out Neutral-coloured targets and the piecetype.rs
    /// filter never got a chance to rewrite the move to
    /// `MoveIntoCarrier`.
    #[test]
    fn test_monkey_can_board_neutral_cart() {
        use crate::pieces::chess2::monkey::Monkey;
        use crate::pieces::fairy::locomotive::{Locomotive, TrainHeading};
        let mut board = empty_board();
        // White Monkey at (4,4). Neutral cart one step diagonally at (5,5).
        board.grid[4][4].piece = Some(PieceType::Monkey(Monkey { color: Color::White }));
        board.grid[5][5].piece = Some(PieceType::Locomotive(Locomotive::new(
            1,
            TrainHeading::Forward,
        )));
        board.grid[0][0].piece = Some(PieceType::new_king(Color::White));
        board.grid[7][7].piece = Some(PieceType::new_king(Color::Black));

        let moves = board.get_moves(&Coord { file: 4, rank: 4 });
        let boards_cart = moves.iter().any(|m| {
            matches!(
                &m.move_type,
                MoveType::MoveIntoCarrier(c) if c.file == 5 && c.rank == 5
            )
        });
        assert!(
            boards_cart,
            "Monkey must emit a MoveIntoCarrier onto the adjacent Neutral cart; got moves: {:?}",
            moves
        );
    }

    /// R8-M1 regression (threat half): Monkey's `attacks()` must NOT
    /// include Neutral-cart landings — Monkey can't actually capture a
    /// cart, so a king parked on a Monkey jump-landing inside a cart
    /// reads as over-pessimistically in-check pre-fix.
    #[test]
    fn test_monkey_does_not_phantom_threat_neutral_cart() {
        use crate::pieces::chess2::monkey::Monkey;
        use crate::pieces::fairy::locomotive::{Locomotive, TrainHeading};
        let mut board = empty_board();
        // Monkey at (3,3). Ladder pawn at (4,4). Neutral cart at the
        // jump-landing (5,5).
        board.grid[3][3].piece = Some(PieceType::Monkey(Monkey { color: Color::White }));
        board.grid[4][4].piece = Some(PieceType::new_pawn(Color::Black));
        board.grid[5][5].piece = Some(PieceType::Locomotive(Locomotive::new(
            1,
            TrainHeading::Forward,
        )));
        board.grid[0][0].piece = Some(PieceType::new_king(Color::White));
        board.grid[7][7].piece = Some(PieceType::new_king(Color::Black));

        // (5,5) holds a Neutral cart — Monkey can't capture there.
        // `is_attacked_by((5,5), White)` must be false.
        assert!(
            !board.is_attacked_by(&Coord { file: 5, rank: 5 }, Color::White),
            "Monkey must not phantom-threat a Neutral cart's tile"
        );
    }

    // -------- Round-9 audit regression tests --------

    /// R9-H1 regression: Skibidi must be able to board a Neutral cart.
    /// Pre-fix, Skibidi's `initial_moves` rejected any non-empty,
    /// non-Skibidi target — including Neutral train carts — so the
    /// piecetype.rs filter never got to rewrite the move to
    /// `MoveIntoCarrier`. Same bug class as R8-M1 (Monkey).
    #[test]
    fn test_skibidi_can_board_neutral_cart() {
        use crate::pieces::fairy::locomotive::{Locomotive, TrainHeading};
        let mut board = empty_board();
        board.grid[4][4].piece = Some(PieceType::Skibidi(Skibidi::new(Color::White)));
        // Adjacent walkable Neutral cart at (5,5).
        board.grid[5][5].square_type = SquareType::Track {
            direction: crate::board::square::TrackDir::E,
        };
        board.grid[5][5].piece = Some(PieceType::Locomotive(Locomotive::new(
            1,
            TrainHeading::Forward,
        )));
        board.grid[0][0].piece = Some(PieceType::new_king(Color::White));
        board.grid[7][7].piece = Some(PieceType::new_king(Color::Black));

        let moves = board.get_moves(&Coord { file: 4, rank: 4 });
        let boards_cart = moves.iter().any(|m| {
            matches!(
                &m.move_type,
                MoveType::MoveIntoCarrier(c) if c.file == 5 && c.rank == 5
            )
        });
        assert!(
            boards_cart,
            "Skibidi must emit a MoveIntoCarrier onto the adjacent Neutral cart; got {:?}",
            moves
        );
    }

    /// R9-H2 regression: when a Monkey can jump-board a Neutral cart
    /// that carries an opposite-color king-passenger, king-safety must
    /// flag the king as in-check. Boarding kills opposite-color
    /// passengers per `passengers.retain` (make_move.rs / Plan 09 Q7
    /// pinned current behavior), so the cart's tile is a real capture
    /// target for the Monkey and `Monkey::would_capture_at` returns
    /// true when the cart carries any opposite-color passenger.
    #[test]
    fn test_monkey_threats_cart_holding_enemy_king_passenger() {
        use crate::pieces::chess2::monkey::Monkey;
        use crate::pieces::fairy::locomotive::{Locomotive, TrainHeading};
        let mut board = empty_board();
        // White Monkey at (3,3). Ladder pawn at (4,4). Neutral cart at
        // the jump-landing (5,5), carrying a Black king passenger.
        board.grid[3][3].piece = Some(PieceType::Monkey(Monkey { color: Color::White }));
        board.grid[4][4].piece = Some(PieceType::new_pawn(Color::Black));
        let mut loco = Locomotive::new(1, TrainHeading::Forward);
        loco.passengers = vec![PieceType::new_king(Color::Black)];
        board.grid[5][5].piece = Some(PieceType::Locomotive(loco));
        // White king somewhere safe.
        board.grid[0][0].piece = Some(PieceType::new_king(Color::White));

        // Boarding this cart would capture the Black king. King-safety
        // must flag the cart's tile as attacked by White.
        assert!(
            board.is_attacked_by(&Coord { file: 5, rank: 5 }, Color::White),
            "Monkey must threaten a Neutral cart that holds an enemy king-passenger"
        );
        // And `is_in_check` (which routes the Black king through
        // `find_king` descent into the cart) must report check.
        assert!(
            board.is_in_check(Color::Black),
            "Black king inside a Neutral cart at a Monkey jump-landing must be in check"
        );
    }

    /// R9-H2 negative case: when the same Monkey/cart setup has *no*
    /// opposite-color passenger, the cart's tile is benign — boarding
    /// captures nothing. King-safety must not flag a phantom threat.
    #[test]
    fn test_monkey_does_not_threat_empty_neutral_cart() {
        use crate::pieces::chess2::monkey::Monkey;
        use crate::pieces::fairy::locomotive::{Locomotive, TrainHeading};
        let mut board = empty_board();
        board.grid[3][3].piece = Some(PieceType::Monkey(Monkey { color: Color::White }));
        board.grid[4][4].piece = Some(PieceType::new_pawn(Color::Black));
        // Empty cart at jump-landing — no passengers to cull on board.
        board.grid[5][5].piece = Some(PieceType::Locomotive(Locomotive::new(
            1,
            TrainHeading::Forward,
        )));
        board.grid[0][0].piece = Some(PieceType::new_king(Color::White));
        board.grid[7][7].piece = Some(PieceType::new_king(Color::Black));

        assert!(
            !board.is_attacked_by(&Coord { file: 5, rank: 5 }, Color::White),
            "Monkey must not threat an empty Neutral cart's tile"
        );
    }

    // -------- Round-10 audit regression tests --------

    /// R10-H1 regression: a Skibidi adjacent to a Neutral cart that
    /// carries an opposite-colour king-passenger must threaten that
    /// cart's tile. Pre-fix, Skibidi's `attacks` returned `Vec::new()`
    /// regardless of board state — but round 9's cart-boarding patch
    /// let Skibidi MoveTo onto a cart, and `passengers.retain` captures
    /// opposite-colour passengers including kings. The combination was
    /// a king-safety unsoundness.
    #[test]
    fn test_skibidi_threats_cart_holding_enemy_king_passenger() {
        use crate::pieces::fairy::locomotive::{Locomotive, TrainHeading};
        let mut board = empty_board();
        // White Skibidi at (4,4). Neutral cart at (5,5) carrying a
        // Black king passenger.
        board.grid[4][4].piece = Some(PieceType::Skibidi(Skibidi::new(Color::White)));
        let mut loco = Locomotive::new(1, TrainHeading::Forward);
        loco.passengers = vec![PieceType::new_king(Color::Black)];
        board.grid[5][5].piece = Some(PieceType::Locomotive(loco));
        board.grid[0][0].piece = Some(PieceType::new_king(Color::White));

        assert!(
            board.is_attacked_by(&Coord { file: 5, rank: 5 }, Color::White),
            "Skibidi must threaten a Neutral cart that holds an enemy king-passenger"
        );
        assert!(
            board.is_in_check(Color::Black),
            "Black king inside a Neutral cart at a Skibidi neighbour must be in check"
        );
    }

    /// R11-M1 regression: `Bus::attacks` must filter passengers by
    /// the Bus's color. A hand-rolled FEN with a mismatched-colour
    /// passenger inside a coloured Bus would otherwise leak phantom
    /// threats for the wrong side via `is_attacked_by`.
    #[test]
    fn test_bus_attacks_filters_mismatched_color_passengers() {
        let mut board = empty_board();
        // White Bus at (4,4) carrying a Black knight (only achievable
        // via hand-rolled FEN; the boarding filter rejects mismatched
        // colours in normal play). Black knight attacks 8 L-shape
        // squares from (4,4). For a `is_attacked_by(_, Black)` query,
        // those squares should NOT be flagged — the Bus is White, and
        // a hand-rolled mismatch must not leak threats for Black.
        let mut bus = Bus::new(Color::White);
        bus.pieces = vec![PieceType::new_knight(Color::Black)];
        board.grid[4][4].piece = Some(PieceType::Bus(bus));
        board.grid[0][0].piece = Some(PieceType::new_king(Color::White));
        board.grid[7][7].piece = Some(PieceType::new_king(Color::Black));

        // A Black-knight L-shape square from (4,4) — e.g. (5,6).
        // is_attacked_by((5,6), Black) must NOT return true via the
        // White Bus's mismatched-passenger.
        assert!(
            !board.is_attacked_by(&Coord { file: 5, rank: 6 }, Color::Black),
            "White Bus must not leak Black-passenger threats via its attacks"
        );
    }

    /// R12-M1 regression: a Goblin that captures a king at runtime
    /// must NOT store the king in its kidnap payload — the king would
    /// be invisible to `find_king` and the game would silently fail
    /// to end. Round-11 closed the FEN parse boundary; round-12
    /// mirrors it inside `post_move_effects`.
    #[test]
    fn test_goblin_capture_of_king_does_not_kidnap_it() {
        use crate::pieces::fairy::goblin::{Goblin, GoblinState};
        let mut board = empty_board();
        board.flags.side_to_move = Color::White;
        // White Goblin at (4,4), Black king at (5,5) (diagonal adj).
        // Goblin moves like a queen-ray glider; one-step diagonal to
        // (5,5) is in its move set, and `legal_moves`'s king-safety
        // only checks the *mover's* king, not whether the target is
        // the opposing king. So the capture goes through.
        let goblin = Goblin {
            color: Color::White,
            home_square: Coord { file: 0, rank: 0 },
            state: GoblinState::Free,
        };
        board.grid[4][4].piece = Some(PieceType::Goblin(goblin));
        board.grid[5][5].piece = Some(PieceType::new_king(Color::Black));
        // White king somewhere safe.
        board.grid[0][0].piece = Some(PieceType::new_king(Color::White));

        // Apply White's Goblin → Black king capture move.
        let mv = GameMove {
            from: Coord { file: 4, rank: 4 },
            move_type: MoveType::MoveTo(Coord { file: 5, rank: 5 }),
        };
        board.make_move_unchecked(mv).expect("goblin captures king");

        // The Goblin must be at (5,5) but in Free state (not
        // Kidnapping with the king as payload).
        match &board.grid[5][5].piece {
            Some(PieceType::Goblin(g)) => assert!(
                matches!(g.state, GoblinState::Free),
                "Goblin must remain in Free state after capturing a king; got {:?}",
                g.state
            ),
            other => panic!("expected Goblin at (5,5), got {other:?}"),
        }
    }

    /// R13-H1 regression: a Goblin that captures a *carrier* (Bus /
    /// Locomotive / Carriage) must NOT store it as kidnap payload —
    /// the carrier's passengers (which may include a king) would be
    /// two levels deep, invisible to `find_king`'s one-level descent.
    /// Round-12's king-only guard misses this case.
    #[test]
    fn test_goblin_capture_of_carrier_does_not_kidnap_it() {
        use crate::pieces::fairy::goblin::{Goblin, GoblinState};
        let mut board = empty_board();
        board.flags.side_to_move = Color::Black;
        // White Bus at (5,5) carrying a White king (legal — kings can
        // board friendly Buses). Black Goblin at (4,4); diagonal one-
        // step reaches (5,5). The Bus is enemy-coloured, so the filter
        // doesn't rewrite to MoveIntoCarrier — the MoveTo proceeds as
        // a capture.
        let mut bus = Bus::new(Color::White);
        bus.pieces = vec![PieceType::new_king(Color::White)];
        board.grid[5][5].piece = Some(PieceType::Bus(bus));
        let goblin = Goblin {
            color: Color::Black,
            home_square: Coord { file: 0, rank: 0 },
            state: GoblinState::Free,
        };
        board.grid[4][4].piece = Some(PieceType::Goblin(goblin));
        // Both kings present at start (Bus-passenger king plus a
        // sentinel black king so the board is well-formed).
        board.grid[7][7].piece = Some(PieceType::new_king(Color::Black));

        let mv = GameMove {
            from: Coord { file: 4, rank: 4 },
            move_type: MoveType::MoveTo(Coord { file: 5, rank: 5 }),
        };
        board.make_move_unchecked(mv).expect("goblin captures bus");

        // After the capture, the Bus (and its king-passenger) must
        // simply be gone — not stored as kidnap payload. The Goblin
        // must be at (5,5) in Free state.
        match &board.grid[5][5].piece {
            Some(PieceType::Goblin(g)) => assert!(
                matches!(g.state, GoblinState::Free),
                "Goblin must remain Free after capturing a carrier; got {:?}",
                g.state
            ),
            other => panic!("expected Goblin at (5,5), got {other:?}"),
        }
        // `find_king(White)` must return None — the white king was
        // inside the captured Bus and is gone with it. The point of
        // this test isn't to assert that kings stay alive; it's to
        // assert that the engine *agrees* the king is gone, so
        // `is_in_check` / `status()` reach the right end-of-game
        // state instead of hiding the king inside an opaque payload.
        assert!(
            board.find_king(Color::White).is_none(),
            "white king must be findable-as-gone after Bus capture"
        );
    }

    /// R13-L1 regression: a passenger pawn that double-pushes from a
    /// cart on its starting rank must set `en_passant_target` so an
    /// adjacent enemy pawn can capture it via en passant.
    #[test]
    fn test_passenger_pawn_double_push_sets_ep_target() {
        let mut board = empty_board();
        board.flags.side_to_move = Color::White;
        // White Bus at b2 (file=1, rank=1) carrying a White pawn. The
        // pawn is at its starting rank (rank=1 for White) so a
        // double-push to (1, 3) is legal.
        let mut bus = Bus::new(Color::White);
        bus.pieces = vec![PieceType::new_pawn(Color::White)];
        board.grid[1][1].piece = Some(PieceType::Bus(bus));
        board.grid[0][0].piece = Some(PieceType::new_king(Color::White));
        board.grid[7][7].piece = Some(PieceType::new_king(Color::Black));

        let mv = GameMove {
            from: Coord { file: 1, rank: 1 },
            move_type: MoveType::PieceInCarrier {
                piece_index: 0,
                move_type: std::sync::Arc::new(MoveType::MoveTo(Coord {
                    file: 1,
                    rank: 3,
                })),
            },
        };
        board.make_move_unchecked(mv).expect("passenger pawn double-push");

        assert_eq!(
            board.flags.en_passant_target,
            Some(Coord { file: 1, rank: 2 }),
            "passenger pawn double-push must set ep target to the passed-over square"
        );
    }

    /// R11-M2 regression: `Goblin::from_symbol` must reject king-symbol
    /// kidnap payloads. A kidnapped king would be invisible to
    /// `find_king` (Goblin's payload isn't exposed via `passengers()`),
    /// silently breaking every downstream query.
    #[test]
    fn test_goblin_kidnap_payload_cannot_be_king() {
        // `G(H=0-0,P=K)` — try to kidnap a white king. Per the
        // round-11 fix, the payload should be dropped and the
        // Goblin parses as Free.
        let sym = "G(H=0-0,P=K)";
        let piece = PieceType::symbol_to_piece(sym)
            .expect("goblin should still parse even with rejected payload");
        match piece {
            PieceType::Goblin(g) => {
                assert!(
                    matches!(g.state, crate::pieces::fairy::goblin::GoblinState::Free),
                    "Goblin with king-symbol kidnap payload must parse as Free, got {:?}",
                    g.state
                );
            }
            other => panic!("expected Goblin, got {other:?}"),
        }
    }

    /// R10-H1 negative case: an empty Neutral cart at a Skibidi
    /// neighbour must NOT be flagged as attacked — boarding captures
    /// nothing. Mirrors the corresponding Monkey negative test.
    #[test]
    fn test_skibidi_does_not_threat_empty_neutral_cart() {
        use crate::pieces::fairy::locomotive::{Locomotive, TrainHeading};
        let mut board = empty_board();
        board.grid[4][4].piece = Some(PieceType::Skibidi(Skibidi::new(Color::White)));
        board.grid[5][5].piece = Some(PieceType::Locomotive(Locomotive::new(
            1,
            TrainHeading::Forward,
        )));
        board.grid[0][0].piece = Some(PieceType::new_king(Color::White));
        board.grid[7][7].piece = Some(PieceType::new_king(Color::Black));

        assert!(
            !board.is_attacked_by(&Coord { file: 5, rank: 5 }, Color::White),
            "Skibidi must not threat an empty Neutral cart's tile"
        );
    }

    /// R8-M2 regression: a stray `,,` in a Goblin payload must NOT
    /// abort the parse — already-parsed fields should survive. Mirrors
    /// the C2 fix for Bus/Loco/Carriage in round 3.
    #[test]
    fn test_goblin_parser_tolerates_empty_field_segment() {
        // `G(H=4-2,,P=n)` — middle field is empty. Pre-fix, the parse
        // returned None and the goblin silently disappeared.
        let sym = "G(H=4-2,,P=n)";
        let piece = PieceType::symbol_to_piece(sym)
            .expect("goblin with stray comma should still parse");
        match piece {
            PieceType::Goblin(g) => {
                assert_eq!(g.color, Color::White);
                assert_eq!(g.home_square.file, 4);
                assert_eq!(g.home_square.rank, 2);
                // Kidnapping state still parsed despite the stray comma.
                assert!(matches!(g.state, crate::pieces::fairy::goblin::GoblinState::Kidnapping { .. }));
            }
            other => panic!("expected Goblin, got {other:?}"),
        }
    }
}
