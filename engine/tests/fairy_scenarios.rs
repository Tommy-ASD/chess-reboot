//! Scenario-level integration tests for the fairy pieces (Goblin,
//! Skibidi, Bus, Monkey). These exercise multi-move stories rather
//! than single-rule unit checks, and run against the engine's public
//! API only.

use engine::board::{Board, Coord, GameMove, MoveType};
use engine::pieces::Color;
use engine::pieces::piecetype::PieceType;

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

/// A Skibidi PhaseShift'd up to phase 2 (radius 1) stuns every piece
/// in its Manhattan-1 disk: pieces sitting in that disk get the
/// `Brainrot` square condition and `Board::get_moves` short-circuits
/// to an empty vector for them. Validates the full flow:
/// `make_move(PhaseShift)` → `recalc_brainrot` → `get_moves` gate.
#[test]
fn skibidi_phase_shift_freezes_pieces_in_radius() {
    use engine::pieces::fairy::skibidi::Skibidi;

    let mut board = empty_board();
    board.grid[3][3] = engine::board::square::Square::new()
        .set_piece(PieceType::Skibidi(Skibidi {
            color: Color::White,
            phase: 1,
        }));
    // Black knight at (3,4) — Manhattan-1 from the Skibidi.
    board.grid[4][3] = engine::board::square::Square::new()
        .set_piece(PieceType::new_knight(Color::Black));

    // White PhaseShift → phase 2 (radius 1).
    board
        .make_move(GameMove {
            from: Coord { file: 3, rank: 3 },
            move_type: MoveType::PhaseShift,
        })
        .expect("white Skibidi phase shift");

    let knight_moves = board.get_moves(&Coord { file: 3, rank: 4 });
    assert!(
        knight_moves.is_empty(),
        "knight in Skibidi phase-2 radius must be frozen, got {knight_moves:?}"
    );
}

fn empty_board() -> Board {
    use engine::board::{BoardFlags, TrainTickRate, square::Square};
    Board {
        grid: vec![vec![Square::new(); 8]; 8],
        flags: BoardFlags {
            side_to_move: Color::White,
            white_can_castle_kingside: false,
            white_can_castle_queenside: false,
            black_can_castle_kingside: false,
            black_can_castle_queenside: false,
            en_passant_target: None,
            train_tick_rate: TrainTickRate::EveryFullTurn,
            ply_count: 0,
        },
    }
}

/// Bus carrying a passenger pawn: the passenger can exit via the
/// PieceInCarrier move-type, landing on a normal square as a pawn.
/// Locks in the round-trip Bus pickup → carry → deposit story.
#[test]
fn bus_passenger_can_exit_via_piece_in_carrier() {
    use std::sync::Arc;

    let mut board = empty_board();
    let bus_with_pawn = PieceType::Bus(engine::pieces::fairy::bus::Bus {
        color: Color::White,
        pieces: vec![PieceType::new_pawn(Color::White)],
    });
    board.grid[3][3] = engine::board::square::Square::new().set_piece(bus_with_pawn);

    // Pawn at the Bus's square (3,3) is white, so it moves "up" (rank
    // -1). Forward push to (3,2) — empty, so legal.
    let exit = GameMove {
        from: Coord { file: 3, rank: 3 },
        move_type: MoveType::PieceInCarrier {
            piece_index: 0,
            move_type: Arc::new(MoveType::MoveTo(Coord { file: 3, rank: 2 })),
        },
    };
    board.make_move(exit).expect("passenger exit should land");

    // After: Bus at (3,3) is now empty, pawn at (3,2).
    match &board.grid[3][3].piece {
        Some(PieceType::Bus(bus)) => assert!(
            bus.pieces.is_empty(),
            "Bus should be empty after passenger exit"
        ),
        other => panic!("expected empty Bus at (3,3), got {other:?}"),
    }
    match &board.grid[2][3].piece {
        Some(PieceType::Pawn(p)) => assert_eq!(p.color, Color::White),
        other => panic!("expected white Pawn at (3,2), got {other:?}"),
    }
}

/// Monkey jump-chain ending in a capture. The chain hops over a
/// ladder pawn, lands on an enemy at the final square, captures it,
/// and stops (per spec: capture ends the chain). The ladder pawn
/// survives (Monkey only captures what it lands on).
#[test]
fn monkey_chain_capture_at_landing_executes() {
    let mut board = empty_board();
    // White Monkey at (0,0); black ladder pawn at (1,1); black
    // capture-target pawn at (2,2).
    board.grid[0][0] = engine::board::square::Square::new().set_piece(
        PieceType::Monkey(engine::pieces::chess2::monkey::Monkey {
            color: Color::White,
        }),
    );
    board.grid[1][1] = engine::board::square::Square::new()
        .set_piece(PieceType::new_pawn(Color::Black));
    board.grid[2][2] = engine::board::square::Square::new()
        .set_piece(PieceType::new_pawn(Color::Black));

    let moves = board.get_moves(&Coord { file: 0, rank: 0 });
    let capture_jump = moves
        .iter()
        .find(|m| matches!(
            &m.move_type,
            MoveType::MoveTo(c) if c.file == 2 && c.rank == 2
        ))
        .cloned()
        .expect("Monkey should emit a jump landing on the enemy at (2,2)");

    board.make_move(capture_jump).expect("jump-capture should apply");

    // Monkey at (2,2), capture-target pawn gone, ladder pawn at (1,1)
    // survives untouched.
    match &board.grid[2][2].piece {
        Some(PieceType::Monkey(_)) => {}
        other => panic!("expected Monkey at (2,2), got {other:?}"),
    }
    assert!(
        board.grid[0][0].piece.is_none(),
        "Monkey's origin must be cleared"
    );
    assert!(
        board.grid[1][1].piece.is_some(),
        "ladder pawn at (1,1) survives — Monkey only captures the landing"
    );
}

/// Monkey jump-chain scenario: chain over two ladder pieces, both of
/// which survive the chain (the Monkey ends up at the second landing
/// without capturing the ladders). Locks in that Monkey's "jumped-
/// over piece is *not* captured" reading of the spec.
#[test]
fn monkey_chains_over_two_pieces_without_capturing_them() {
    let mut board = empty_board();
    // Monkey at (0,0); ladder pawns at (1,1) and (3,3); chain
    // landings at (2,2) and (4,4).
    board.grid[0][0] = engine::board::square::Square::new().set_piece(PieceType::Monkey(
        engine::pieces::chess2::monkey::Monkey {
            color: Color::White,
        },
    ));
    board.grid[1][1] = engine::board::square::Square::new()
        .set_piece(PieceType::new_pawn(Color::Black));
    board.grid[3][3] = engine::board::square::Square::new()
        .set_piece(PieceType::new_pawn(Color::Black));

    let moves = board.get_moves(&Coord { file: 0, rank: 0 });
    let chain_landing = moves.iter().find(|m| matches!(
        &m.move_type,
        MoveType::MoveTo(c) if c.file == 4 && c.rank == 4
    ));
    assert!(
        chain_landing.is_some(),
        "expected a chain-jump landing at (4,4), got {moves:?}"
    );

    board
        .make_move(chain_landing.cloned().unwrap())
        .expect("chain jump should apply");

    // Monkey at (4,4), ladder pawns survive at (1,1) and (3,3).
    match &board.grid[4][4].piece {
        Some(PieceType::Monkey(_)) => {}
        other => panic!("expected Monkey at (4,4), got {other:?}"),
    }
    assert!(
        board.grid[1][1].piece.is_some(),
        "ladder pawn at (1,1) survives"
    );
    assert!(
        board.grid[3][3].piece.is_some(),
        "ladder pawn at (3,3) survives"
    );
}

/// Goblin in free state captures and transitions to kidnapping. After
/// the capture the Goblin's state is `Kidnapping { piece: ... }` and
/// its move-gen narrows to king-style 1-square moves.
#[test]
fn goblin_capture_transitions_to_kidnapping() {
    use engine::pieces::fairy::goblin::{Goblin, GoblinState};

    let mut board = empty_board();
    // Free Goblin on a1 = (0, 7).
    board.grid[7][0] = engine::board::square::Square::new().set_piece(
        PieceType::Goblin(Goblin {
            color: Color::White,
            state: GoblinState::Free,
            home_square: Coord { file: 0, rank: 7 },
        }),
    );
    // Knight on d4 = (3, 4) — exactly 3 squares along the (0,7)→(3,4)
    // diagonal (df=3, dr=-3), so the queen-like free Goblin can take.
    board.grid[4][3] = engine::board::square::Square::new()
        .set_piece(PieceType::new_knight(Color::Black));

    board
        .make_move(mv((0, 7), (3, 4)))
        .expect("Goblin queen-capture along the a1-h8 diagonal");

    // Goblin is now at (3,4) in Kidnapping state holding the knight.
    match &board.grid[4][3].piece {
        Some(PieceType::Goblin(g)) => {
            match &g.state {
                GoblinState::Kidnapping { piece } => match &**piece {
                    PieceType::Knight(k) => assert_eq!(k.color, Color::Black),
                    other => panic!("expected kidnapped knight, got {other:?}"),
                },
                other => panic!("expected Kidnapping state, got {other:?}"),
            }
            assert_eq!(g.color, Color::White);
        }
        other => panic!("expected white Goblin at (3,4), got {other:?}"),
    }

    // Move generation from this position should now be king-like —
    // exactly 8 1-step targets at (3,4) on an otherwise-empty board
    // (the goblin sits in the interior; every neighbour is in bounds
    // and empty).
    let moves = board.get_moves(&Coord { file: 3, rank: 4 });
    assert_eq!(
        moves.len(),
        8,
        "kidnapping Goblin at (3,4) on an empty board must offer exactly 8 king moves, got {moves:?}"
    );
    for m in &moves {
        if let MoveType::MoveTo(c) = &m.move_type {
            let df = (c.file as isize - 3).abs();
            let dr = (c.rank as isize - 4).abs();
            assert!(df <= 1 && dr <= 1, "non-king-step move {c:?} from a kidnapping Goblin");
        }
    }
}
