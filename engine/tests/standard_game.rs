//! Full-game integration tests for standard chess scenarios. These run
//! through the engine's public API only (no module-internal access),
//! so they double as a regression suite for the API surface and prove
//! the move-gen / make_move / status pipeline composes correctly
//! across a real sequence of plies.

use engine::board::{
    Board, Coord, GameMove, GameStatus, MoveType,
    fen::fen_to_board,
};

fn standard_start() -> Board {
    fen_to_board("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq -").unwrap()
}

/// Build a plain `MoveTo` GameMove from two algebraic-style coords.
/// Compact helper so the move script reads close to chess notation.
fn mv(from: (u8, u8), to: (u8, u8)) -> GameMove {
    GameMove {
        from: Coord {
            file: from.0,
            rank: from.1,
        },
        move_type: MoveType::MoveTo(Coord {
            file: to.0,
            rank: to.1,
        }),
    }
}

/// Fool's mate — the fastest possible checkmate. Two-move script:
///   1. f2-f3, e7-e5
///   2. g2-g4, Qd8-h4#
/// White's king on e1 is mated by the black queen on h4 along the
/// diagonal e1-h4, with the queen's path unblockable because the
/// f2 + g2 pawns just moved off.
#[test]
fn fools_mate_results_in_checkmate() {
    let mut board = standard_start();

    // Internal coord system: rank 0 is black's back row, rank 7 is white's.
    // f2 = (5, 6), f3 = (5, 5)
    board.make_move(mv((5, 6), (5, 5))).expect("white f2-f3");
    // e7 = (4, 1), e5 = (4, 3)
    board.make_move(mv((4, 1), (4, 3))).expect("black e7-e5");
    // g2 = (6, 6), g4 = (6, 4)
    board.make_move(mv((6, 6), (6, 4))).expect("white g2-g4");

    // Status before the mating move: white is to move, not yet mated.
    assert_eq!(board.status(), GameStatus::Ongoing);

    // d8 = (3, 0), h4 = (7, 4) — black queen delivers mate.
    board.make_move(mv((3, 0), (7, 4))).expect("black Qd8-h4#");

    match board.status() {
        GameStatus::Checkmate {
            winner: engine::pieces::Color::Black,
        } => {}
        other => panic!("expected Black checkmate, got {other:?}"),
    }
}

/// Mid-game scenario: play five plies into a benign Italian-game-style
/// opening and confirm `status()` stays `Ongoing` the whole time, that
/// turn alternation works, and no move panics or fails legality. A
/// loose smoke test that the pipeline runs forwards for many plies.
#[test]
fn opening_play_stays_ongoing_and_alternates_turns() {
    let mut board = standard_start();

    // 1. e4   (4,6)→(4,4)
    board.make_move(mv((4, 6), (4, 4))).expect("white e2-e4");
    assert_eq!(board.flags.side_to_move, engine::pieces::Color::Black);

    // 1... e5   (4,1)→(4,3)
    board.make_move(mv((4, 1), (4, 3))).expect("black e7-e5");
    assert_eq!(board.flags.side_to_move, engine::pieces::Color::White);

    // 2. Nf3   (6,7)→(5,5)
    board.make_move(mv((6, 7), (5, 5))).expect("white Ng1-f3");

    // 2... Nc6   (1,0)→(2,2)
    board.make_move(mv((1, 0), (2, 2))).expect("black Nb8-c6");

    // 3. Bc4   (5,7)→(2,4)
    board.make_move(mv((5, 7), (2, 4))).expect("white Bf1-c4");

    assert_eq!(board.status(), GameStatus::Ongoing);
    assert_eq!(board.flags.side_to_move, engine::pieces::Color::Black);
}

/// Castling actually flows through `make_move` end-to-end. White
/// kingside castle from a setup where the path is clear: king on e1,
/// rook on h1, knight + bishop already developed.
#[test]
fn white_kingside_castle_lands() {
    // King on e1 (4,7), rook on h1 (7,7), squares between empty.
    let mut board =
        fen_to_board("4k3/8/8/8/8/8/8/4K2R w KQkq -").unwrap();

    use engine::board::CastleSide;
    let castle = GameMove {
        from: Coord { file: 4, rank: 7 },
        move_type: MoveType::Castle {
            side: CastleSide::Kingside,
        },
    };
    board.make_move(castle).expect("kingside castle should land");

    // King now on g1 (6,7), rook on f1 (5,7).
    let king = board
        .get_square_at(&Coord { file: 6, rank: 7 })
        .and_then(|s| s.piece.clone());
    let rook = board
        .get_square_at(&Coord { file: 5, rank: 7 })
        .and_then(|s| s.piece.clone());
    assert!(matches!(
        king,
        Some(engine::pieces::piecetype::PieceType::King(_))
    ));
    assert!(matches!(
        rook,
        Some(engine::pieces::piecetype::PieceType::Rook(_))
    ));
    // Castle rights cleared.
    assert!(!board.flags.white_can_castle_kingside);
    assert!(!board.flags.white_can_castle_queenside);
}

/// En-passant chain: white double-push enables a black en-passant
/// capture that removes the white pawn from its actual square (one
/// rank away from the capturing pawn's destination).
#[test]
fn en_passant_capture_chain() {
    // Set up so black just needs to wait for the double push.
    // Black pawn on d4 (3,4), white pawn on e2 (4,6), side white to move.
    let mut board = fen_to_board("8/8/8/8/3p4/8/4P3/8 w - -").unwrap();

    // White e2-e4 — sets ep target to e3 = (4,5).
    board.make_move(mv((4, 6), (4, 4))).expect("white e2-e4");
    assert_eq!(
        board.flags.en_passant_target,
        Some(Coord { file: 4, rank: 5 })
    );
    assert_eq!(board.flags.side_to_move, engine::pieces::Color::Black);

    // Black d4 captures e.p. onto e3 = (4,5). Captured white pawn was
    // at (4,4).
    use engine::board::MoveType::EnPassant;
    let ep = GameMove {
        from: Coord { file: 3, rank: 4 },
        move_type: EnPassant {
            target: Coord { file: 4, rank: 5 },
            captured: Coord { file: 4, rank: 4 },
        },
    };
    board.make_move(ep).expect("black should en-passant capture");

    // White pawn at (4,4) gone; specifically a black pawn now at (4,5).
    // (Asserting type + colour catches a regression where the wrong
    // piece were placed at the landing — bare `is_some()` would miss it.)
    let captured_square = board.get_square_at(&Coord { file: 4, rank: 4 });
    assert!(
        captured_square.and_then(|s| s.piece.as_ref()).is_none(),
        "captured white pawn should be cleared from its square"
    );
    let landing_piece = board
        .get_square_at(&Coord { file: 4, rank: 5 })
        .and_then(|s| s.piece.clone());
    match landing_piece {
        Some(engine::pieces::piecetype::PieceType::Pawn(p)) => {
            assert_eq!(
                p.color,
                engine::pieces::Color::Black,
                "landing pawn must be the black capturer, not some other piece"
            );
        }
        other => panic!("expected a black pawn at the EP landing, got {other:?}"),
    }
    // ep target cleared on next move (black just moved, not a double push).
    assert_eq!(board.flags.en_passant_target, None);
}

/// Capture-promotion executed via the full make_move path. The pawn
/// is on b7 (so a8 is reachable as a *diagonal* capture target rather
/// than a blocked forward push) and the black rook on a8 is taken by
/// the promoting pawn — which clears black's queenside castle right
/// via `maybe_clear_castle_on_rook_capture`.
#[test]
fn capture_promotion_clears_castle_right() {
    // a8 = (0,0) black rook; b7 = (1,1) white pawn;
    // e8/e1 kings for legality.
    let mut board = fen_to_board("r3k3/1P6/8/8/8/8/8/4K3 w q -").unwrap();
    assert!(board.flags.black_can_castle_queenside);

    use engine::board::{MoveType, PromotionTarget};
    let promote = GameMove {
        from: Coord { file: 1, rank: 1 },
        move_type: MoveType::Promotion {
            target: Coord { file: 0, rank: 0 },
            into: PromotionTarget::Queen,
        },
    };
    board.make_move(promote).expect("capture-promotion lands");

    // Black queenside flag should be cleared by the rook capture.
    assert!(!board.flags.black_can_castle_queenside);
    // The promoted piece is a white queen at a8.
    let at_a8 = board
        .get_square_at(&Coord { file: 0, rank: 0 })
        .and_then(|s| s.piece.clone());
    match at_a8 {
        Some(engine::pieces::piecetype::PieceType::Queen(q)) => {
            assert_eq!(q.color, engine::pieces::Color::White);
        }
        other => panic!("expected white queen at a8, got {other:?}"),
    }
    // Pawn must have vacated b7 — guards against a silent no-op
    // promotion path that would leave the castle flag cleared
    // (correct) but the pawn still standing (wrong).
    let at_b7 = board
        .get_square_at(&Coord { file: 1, rank: 1 })
        .and_then(|s| s.piece.clone());
    assert!(
        at_b7.is_none(),
        "promoting pawn must vacate b7; found {at_b7:?}"
    );
}
