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
            state: 2,
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
        assert_eq!(fen, "10/10/10/10/10/10/10/10/10/10 w - -");
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
            fen.ends_with(" e2"),
            "expected en-passant 'e2' on 12-tall board, got: {fen}"
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
}
