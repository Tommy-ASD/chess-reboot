//! Scenario-level integration tests for the fairy pieces (Goblin,
//! Skibidi, Bus, Monkey). These exercise multi-move stories rather
//! than single-rule unit checks, and run against the engine's public
//! API only.

use engine::board::{Board, Coord, GameMove, GameStatus, MoveType};
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
            last_move: None,
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

/// Plan 04 + plan 07: a Skibidi phase-4 Brainrot wall that paralyses the
/// opponent's only piece is a *win by Brainrot*, not a stalemate. Black
/// (to move) has a lone king sitting inside a White Skibidi's phase-4
/// aura: the king is frozen (move-gen short-circuits on a Brainrot
/// source), is not in check, and has no other piece — so the
/// not-in-check / no-legal-moves branch of `status()` must resolve to
/// `BrainrotWin(White)` rather than `Stalemate`. `recalc_brainrot`
/// paints the aura exactly as a real move would before we query status.
#[test]
fn skibidi_brainrot_wall_wins_by_brainrot() {
    use engine::pieces::fairy::skibidi::Skibidi;

    let mut board = empty_board();
    board.flags.side_to_move = Color::Black;
    // White Skibidi at (3,3), phase 4 → Manhattan-3 brainrot disk.
    board.grid[3][3] = engine::board::square::Square::new().set_piece(PieceType::Skibidi(
        Skibidi {
            color: Color::White,
            phase: 4,
        },
    ));
    // Black king at (3,5): Manhattan 2 from the Skibidi → brainrotted.
    board.grid[5][3] =
        engine::board::square::Square::new().set_piece(PieceType::new_king(Color::Black));
    // White king parked in a corner, clear of the aura (Manhattan 6).
    board.grid[0][0] =
        engine::board::square::Square::new().set_piece(PieceType::new_king(Color::White));
    // Paint the brainrot conditions the way a real move would.
    board.recalc_brainrot();

    // Preconditions for the brainrot/stalemate branch: the black king is
    // genuinely frozen (no legal moves) and not in check.
    assert!(
        board.legal_moves(&Coord { file: 3, rank: 5 }).is_empty(),
        "black king should be frozen by brainrot"
    );
    assert!(
        !board.is_in_check(Color::Black),
        "black king must not be in check"
    );

    assert_eq!(
        board.status(),
        GameStatus::BrainrotWin {
            winner: Color::White
        },
        "lone frozen king under a phase-4 aura is a brainrot win for White"
    );
}

/// Ordering guard: when the brainrot-frozen king is *also* in check, the
/// result is `Checkmate`, not `BrainrotWin` — `status()` tests
/// `is_in_check` before the brainrot heuristic. Without that ordering a
/// mated king standing on a brainrot square would be mis-reported as a
/// brainrot win.
#[test]
fn brainrot_frozen_king_in_check_is_checkmate_not_brainrot() {
    use engine::pieces::fairy::skibidi::Skibidi;

    let mut board = empty_board();
    board.flags.side_to_move = Color::Black;
    board.grid[3][3] = engine::board::square::Square::new().set_piece(PieceType::Skibidi(
        Skibidi {
            color: Color::White,
            phase: 4,
        },
    ));
    // Black king at (3,5), frozen by the aura.
    board.grid[5][3] =
        engine::board::square::Square::new().set_piece(PieceType::new_king(Color::Black));
    // White rook at (7,5) checks the king along the empty rank 5.
    board.grid[5][7] =
        engine::board::square::Square::new().set_piece(PieceType::new_rook(Color::White));
    board.grid[0][0] =
        engine::board::square::Square::new().set_piece(PieceType::new_king(Color::White));
    board.recalc_brainrot();

    assert!(
        board.is_in_check(Color::Black),
        "rook must check the black king"
    );
    assert_eq!(
        board.status(),
        GameStatus::Checkmate {
            winner: Color::White
        },
        "a checked + frozen king is checkmate, not a brainrot win"
    );
}

/// Plan 04 (Skibidi spec, line 15): losing your Skibidi to a phase-4
/// enemy is an unrecoverable lockout — "there is nothing you can do" —
/// that ends the game *even while you still have legal moves*. Black (to
/// move) has a king + rook with plenty of moves and is not in check, but
/// has no Skibidi while White holds a phase-4 one. `status()` must report
/// `BrainrotLockout(White)`, not `Ongoing`.
#[test]
fn skibidi_phase4_lockout_loses_even_with_legal_moves() {
    use engine::pieces::fairy::skibidi::Skibidi;

    let mut board = empty_board();
    board.flags.side_to_move = Color::Black;
    // White phase-4 Skibidi in the far corner; its radius-3 aura never
    // reaches Black's pieces, so Black is genuinely mobile.
    board.grid[0][0] = engine::board::square::Square::new().set_piece(PieceType::Skibidi(
        Skibidi {
            color: Color::White,
            phase: 4,
        },
    ));
    board.grid[7][7] =
        engine::board::square::Square::new().set_piece(PieceType::new_king(Color::Black));
    board.grid[6][7] =
        engine::board::square::Square::new().set_piece(PieceType::new_rook(Color::Black));
    board.grid[4][4] =
        engine::board::square::Square::new().set_piece(PieceType::new_king(Color::White));
    board.recalc_brainrot();

    // The lockout must fire *despite* Black having legal moves and not
    // being in check — that's the whole point of the rule.
    assert!(
        !board.legal_moves(&Coord { file: 7, rank: 6 }).is_empty(),
        "black rook should have legal moves — the lockout fires anyway"
    );
    assert!(
        !board.is_in_check(Color::Black),
        "black is not in check; the loss is purely the phase-4 lockout"
    );
    assert_eq!(
        board.status(),
        GameStatus::BrainrotLockout {
            winner: Color::White
        },
        "no Skibidi vs a phase-4 enemy is a lockout loss for Black"
    );
}

/// Boundary guard: a phase-*3* enemy Skibidi is not a lockout. Same
/// shape as the phase-4 test, one phase lower — Black, with no Skibidi
/// but free to move, simply plays on (`Ongoing`).
#[test]
fn skibidi_phase3_enemy_is_not_a_lockout() {
    use engine::pieces::fairy::skibidi::Skibidi;

    let mut board = empty_board();
    board.flags.side_to_move = Color::Black;
    board.grid[0][0] = engine::board::square::Square::new().set_piece(PieceType::Skibidi(
        Skibidi {
            color: Color::White,
            phase: 3,
        },
    ));
    board.grid[7][7] =
        engine::board::square::Square::new().set_piece(PieceType::new_king(Color::Black));
    board.grid[6][7] =
        engine::board::square::Square::new().set_piece(PieceType::new_rook(Color::Black));
    board.grid[4][4] =
        engine::board::square::Square::new().set_piece(PieceType::new_king(Color::White));
    board.recalc_brainrot();

    assert_eq!(
        board.status(),
        GameStatus::Ongoing,
        "a phase-3 enemy Skibidi is not the unstoppable phase-4 lockout"
    );
}

/// Discriminator: the lockout requires that *you* have lost your Skibidi.
/// Black keeps its own (phase-1) Skibidi while facing White's phase-4
/// one — Black can still neutralise, so it is not locked out and the
/// game is `Ongoing`.
#[test]
fn skibidi_lockout_requires_having_lost_your_own_skibidi() {
    use engine::pieces::fairy::skibidi::Skibidi;

    let mut board = empty_board();
    board.flags.side_to_move = Color::Black;
    board.grid[0][0] = engine::board::square::Square::new().set_piece(PieceType::Skibidi(
        Skibidi {
            color: Color::White,
            phase: 4,
        },
    ));
    board.grid[7][7] =
        engine::board::square::Square::new().set_piece(PieceType::new_king(Color::Black));
    // Black's own Skibidi, far from the aura — not captured, so no lockout.
    board.grid[6][7] = engine::board::square::Square::new().set_piece(PieceType::Skibidi(
        Skibidi {
            color: Color::Black,
            phase: 1,
        },
    ));
    board.grid[4][4] =
        engine::board::square::Square::new().set_piece(PieceType::new_king(Color::White));
    board.recalc_brainrot();

    assert_eq!(
        board.status(),
        GameStatus::Ongoing,
        "Black still has its Skibidi, so the phase-4 enemy is not a lockout"
    );
}

/// Plan 04 (Goblin spec, mechanic 2): "the taking piece can move again."
/// A White rook captures a Black Kidnapping Goblin — the kidnap victim
/// drops on the rook's origin (mechanic 1, already shipped) AND the turn
/// does *not* pass to Black: White moves again. We then play a normal
/// White move and confirm the turn finally flips.
#[test]
fn goblin_kidnap_capture_grants_extra_move() {
    use engine::pieces::fairy::goblin::{Goblin, GoblinState};
    use std::sync::Arc;

    let mut board = empty_board();
    board.flags.side_to_move = Color::White;
    // Black Kidnapping Goblin at (3,4) holding a White pawn, home (0,0).
    board.grid[4][3] = engine::board::square::Square::new().set_piece(PieceType::Goblin(Goblin {
        color: Color::Black,
        state: GoblinState::Kidnapping {
            piece: Arc::new(PieceType::new_pawn(Color::White)),
        },
        home_square: Coord { file: 0, rank: 0 },
    }));
    // White rook at (3,7) captures the goblin by sliding up file 3.
    board.grid[7][3] =
        engine::board::square::Square::new().set_piece(PieceType::new_rook(Color::White));
    // Kings well clear of the action so the position is legal.
    board.grid[0][7] =
        engine::board::square::Square::new().set_piece(PieceType::new_king(Color::White));
    board.grid[7][0] =
        engine::board::square::Square::new().set_piece(PieceType::new_king(Color::Black));

    board
        .make_move(mv((3, 7), (3, 4)))
        .expect("white rook captures the kidnapping goblin");

    // Mechanic 2: the captor moves again — the turn stays White.
    assert_eq!(
        board.flags.side_to_move,
        Color::White,
        "capturing a kidnapping goblin grants the captor another move (no flip)"
    );
    // Mechanic 1 (already shipped): the kidnapped White pawn dropped on
    // the rook's origin (3,7); the rook now sits on the goblin's tile.
    match &board.grid[4][3].piece {
        Some(PieceType::Rook(r)) => assert_eq!(r.color, Color::White),
        other => panic!("expected white rook at (3,4), got {other:?}"),
    }
    match &board.grid[7][3].piece {
        Some(PieceType::Pawn(p)) => assert_eq!(p.color, Color::White),
        other => panic!("expected dropped white pawn at (3,7), got {other:?}"),
    }

    // White really is to move: a normal second move now flips the turn.
    board
        .make_move(mv((7, 0), (6, 0)))
        .expect("white's bonus move (king step)");
    assert_eq!(
        board.flags.side_to_move,
        Color::Black,
        "after the bonus move, an ordinary move flips the turn to Black"
    );
}

/// Contrast: capturing a *Free* (non-kidnapping) Goblin is an ordinary
/// capture — no victim to drop, and the turn flips normally. Pins that
/// the extra-move is specific to capturing a Goblin in Kidnapping state.
#[test]
fn capturing_free_goblin_does_not_grant_extra_move() {
    use engine::pieces::fairy::goblin::{Goblin, GoblinState};

    let mut board = empty_board();
    board.flags.side_to_move = Color::White;
    board.grid[4][3] = engine::board::square::Square::new().set_piece(PieceType::Goblin(Goblin {
        color: Color::Black,
        state: GoblinState::Free,
        home_square: Coord { file: 0, rank: 0 },
    }));
    board.grid[7][3] =
        engine::board::square::Square::new().set_piece(PieceType::new_rook(Color::White));
    board.grid[0][7] =
        engine::board::square::Square::new().set_piece(PieceType::new_king(Color::White));
    board.grid[7][0] =
        engine::board::square::Square::new().set_piece(PieceType::new_king(Color::Black));

    board
        .make_move(mv((3, 7), (3, 4)))
        .expect("white rook captures the free goblin");

    assert_eq!(
        board.flags.side_to_move,
        Color::Black,
        "capturing a Free (non-kidnapping) goblin does not grant an extra move"
    );
}

/// Plan 04 / plan 07 coverage: the phase-4 lockout also converts a
/// would-be *Stalemate* into a loss — the no-legal-moves branch of
/// `status()`, which the other lockout tests (all in the has-moves
/// branch) don't exercise. Classic K+Q-vs-K stalemate geometry (Black
/// king boxed in the corner, not in check, and *not* on a brainrot
/// square — a genuine stalemate, not a brainrot wall) plus a far-away
/// White phase-4 Skibidi and no Black Skibidi. Without the Skibidi this
/// is `Stalemate` (cf. `test_stalemate_detected`); with it, the lockout
/// makes it a `BrainrotLockout` loss rather than a draw.
#[test]
fn brainrot_lockout_converts_stalemate_to_loss() {
    use engine::pieces::fairy::skibidi::Skibidi;

    let mut board = empty_board();
    board.flags.side_to_move = Color::Black;
    // Stalemate box: black king h8 (7,0), white king f7 (5,1), white
    // queen g6 (6,2) — black has no legal move and is not in check.
    board.grid[0][7] =
        engine::board::square::Square::new().set_piece(PieceType::new_king(Color::Black));
    board.grid[1][5] =
        engine::board::square::Square::new().set_piece(PieceType::new_king(Color::White));
    board.grid[2][6] =
        engine::board::square::Square::new().set_piece(PieceType::new_queen(Color::White));
    // Far-away White phase-4 Skibidi (Manhattan 14 from the black king,
    // so its aura never reaches — the king stays off any brainrot square).
    board.grid[7][0] = engine::board::square::Square::new().set_piece(PieceType::Skibidi(
        Skibidi {
            color: Color::White,
            phase: 4,
        },
    ));
    board.recalc_brainrot();

    // Genuine stalemate preconditions: no legal moves, not in check.
    assert!(
        board.legal_moves(&Coord { file: 7, rank: 0 }).is_empty(),
        "black king is boxed — no legal moves"
    );
    assert!(
        !board.is_in_check(Color::Black),
        "black king is not in check (stalemate geometry, not mate)"
    );

    assert_eq!(
        board.status(),
        GameStatus::BrainrotLockout {
            winner: Color::White
        },
        "a stalemate that also meets the phase-4 lockout is a loss, not a draw"
    );
}

/// Plan 04 / plan 07 coverage: a Skibidi *carried inside a carrier* is
/// not captured, so it must suppress the lockout — exercising the
/// passenger descent in `is_brainrot_lockout` that the top-level
/// discriminator test doesn't reach. Black's only Skibidi rides a Black
/// Bus while White holds a phase-4 Skibidi; because the carried Skibidi
/// still belongs to Black, this is *not* a lockout and the game is
/// `Ongoing`.
#[test]
fn carried_skibidi_suppresses_phase4_lockout() {
    use engine::pieces::fairy::skibidi::Skibidi;

    let mut board = empty_board();
    board.flags.side_to_move = Color::Black;
    // Black Bus carrying Black's only Skibidi.
    board.grid[7][7] = engine::board::square::Square::new().set_piece(PieceType::Bus(
        engine::pieces::fairy::bus::Bus {
            color: Color::Black,
            pieces: vec![PieceType::Skibidi(Skibidi {
                color: Color::Black,
                phase: 1,
            })],
        },
    ));
    board.grid[5][7] =
        engine::board::square::Square::new().set_piece(PieceType::new_king(Color::Black));
    // White phase-4 Skibidi far away, plus a White king.
    board.grid[0][0] = engine::board::square::Square::new().set_piece(PieceType::Skibidi(
        Skibidi {
            color: Color::White,
            phase: 4,
        },
    ));
    board.grid[3][3] =
        engine::board::square::Square::new().set_piece(PieceType::new_king(Color::White));
    board.recalc_brainrot();

    assert_eq!(
        board.status(),
        GameStatus::Ongoing,
        "Black's Skibidi is only riding a carrier (not captured), so the phase-4 enemy is not a lockout"
    );
}

/// Round-1 audit fix: a *Neutral* Skibidi's aura is not an "opposing"
/// win condition. A lone Black king frozen by a Neutral phase-4 aura,
/// with NO White Skibidi, must be `Stalemate` — not a `BrainrotWin`
/// credited to White, which neither player caused. Pins the
/// `is_brainrot_win` opponent-colour discrimination (`== to_move.opposite()`,
/// not `!= to_move`, which used to also match Neutral).
#[test]
fn neutral_skibidi_aura_is_stalemate_not_brainrot_win() {
    use engine::pieces::fairy::skibidi::Skibidi;

    let mut board = empty_board();
    board.flags.side_to_move = Color::Black;
    // Neutral phase-4 Skibidi at (3,3) — its aura freezes the lone king.
    board.grid[3][3] = engine::board::square::Square::new().set_piece(PieceType::Skibidi(
        Skibidi {
            color: Color::Neutral,
            phase: 4,
        },
    ));
    // Black king at (3,5): Manhattan 2 → brainrotted, frozen.
    board.grid[5][3] =
        engine::board::square::Square::new().set_piece(PieceType::new_king(Color::Black));
    // White king far away; crucially, NO White Skibidi radiates.
    board.grid[0][0] =
        engine::board::square::Square::new().set_piece(PieceType::new_king(Color::White));
    board.recalc_brainrot();

    assert!(
        board.legal_moves(&Coord { file: 3, rank: 5 }).is_empty(),
        "black king is frozen by the neutral aura"
    );
    assert!(
        !board.is_in_check(Color::Black),
        "black king is not in check"
    );
    assert_eq!(
        board.status(),
        GameStatus::Stalemate,
        "a Neutral aura (no opposing player Skibidi) is a Stalemate, not a BrainrotWin"
    );
}

/// Round-1 audit gap: in the has-legal-moves branch, `status()` checks
/// the phase-4 lockout BEFORE `is_in_check`, so a side that is in check,
/// still has a legal move, and has lost its Skibidi to a phase-4 enemy
/// is reported as `BrainrotLockout` (terminal), not `Check`. Pins that
/// documented precedence.
#[test]
fn phase4_lockout_overrides_check_when_side_has_moves() {
    use engine::pieces::fairy::skibidi::Skibidi;

    let mut board = empty_board();
    board.flags.side_to_move = Color::Black;
    // Black king at (4,0), in check from a White rook on file 4.
    board.grid[0][4] =
        engine::board::square::Square::new().set_piece(PieceType::new_king(Color::Black));
    board.grid[7][4] =
        engine::board::square::Square::new().set_piece(PieceType::new_rook(Color::White));
    // White king far. White phase-4 Skibidi at (0,4) — Manhattan 8 from
    // the king and 7 from the rook, so its aura freezes nothing relevant
    // (the king keeps its escapes; the rook keeps checking). No Black Skibidi.
    board.grid[7][0] =
        engine::board::square::Square::new().set_piece(PieceType::new_king(Color::White));
    board.grid[4][0] = engine::board::square::Square::new().set_piece(PieceType::Skibidi(
        Skibidi {
            color: Color::White,
            phase: 4,
        },
    ));
    board.recalc_brainrot();

    assert!(
        board.is_in_check(Color::Black),
        "white rook checks the black king"
    );
    assert!(
        !board.legal_moves(&Coord { file: 4, rank: 0 }).is_empty(),
        "black king has a legal escape — this is the has-moves branch"
    );
    assert_eq!(
        board.status(),
        GameStatus::BrainrotLockout {
            winner: Color::White
        },
        "the lockout is checked before Check in the has-moves branch: terminal loss, not Check"
    );
}

/// Round-1 audit gap: the `saw_own_piece` guard in `is_brainrot_win`
/// stops a side with ZERO pieces from being vacuously declared a
/// brainrot winner (the all-pieces-on-Brainrot loop is vacuously true
/// for an empty side). Black has no pieces while White has a *radiating*
/// phase-3 Skibidi (phase>1 but not 4, so the lockout doesn't fire
/// either) — the result must be `Stalemate`.
#[test]
fn empty_side_facing_radiating_skibidi_is_stalemate() {
    use engine::pieces::fairy::skibidi::Skibidi;

    let mut board = empty_board();
    board.flags.side_to_move = Color::Black; // Black has no pieces at all.
    board.grid[3][3] = engine::board::square::Square::new().set_piece(PieceType::Skibidi(
        Skibidi {
            color: Color::White,
            phase: 3,
        },
    ));
    board.grid[0][0] =
        engine::board::square::Square::new().set_piece(PieceType::new_king(Color::White));
    board.recalc_brainrot();

    assert_eq!(
        board.status(),
        GameStatus::Stalemate,
        "a piece-less side is not a vacuous BrainrotWin (saw_own_piece guard); a phase-3 enemy is not a lockout"
    );
}

/// Plan 04 (Goblin mechanic 2) coverage: the extra-move fires for a
/// PROMOTION capture too, not just a plain MoveTo (the existing
/// `goblin_kidnap_capture_grants_extra_move` covers MoveTo). A White
/// pawn promotes by capturing a Black Kidnapping Goblin on the back
/// rank — the kidnap victim drops on the pawn's origin and the turn
/// stays White. Exercises `capture_targets`' Promotion arm
/// (`captor_origin = Some`) feeding `move_grants_extra_turn`.
#[test]
fn promotion_capture_of_kidnapping_goblin_grants_extra_move() {
    use engine::board::PromotionTarget;
    use engine::pieces::fairy::goblin::{Goblin, GoblinState};
    use std::sync::Arc;

    let mut board = empty_board();
    board.flags.side_to_move = Color::White;
    // White pawn at (3,1), one step from promotion at rank 0.
    board.grid[1][3] =
        engine::board::square::Square::new().set_piece(PieceType::new_pawn(Color::White));
    // Black Kidnapping Goblin on the promotion rank (4,0), holding a White knight.
    board.grid[0][4] = engine::board::square::Square::new().set_piece(PieceType::Goblin(Goblin {
        color: Color::Black,
        state: GoblinState::Kidnapping {
            piece: Arc::new(PieceType::new_knight(Color::White)),
        },
        home_square: Coord { file: 0, rank: 0 },
    }));
    board.grid[7][0] =
        engine::board::square::Square::new().set_piece(PieceType::new_king(Color::White));
    board.grid[7][7] =
        engine::board::square::Square::new().set_piece(PieceType::new_king(Color::Black));

    let promo_cap = GameMove {
        from: Coord { file: 3, rank: 1 },
        move_type: MoveType::Promotion {
            target: Coord { file: 4, rank: 0 },
            into: PromotionTarget::Queen,
        },
    };
    board
        .make_move(promo_cap)
        .expect("pawn promotes by capturing the kidnapping goblin");

    // Mechanic 2: the captor moves again — the turn stays White.
    assert_eq!(
        board.flags.side_to_move,
        Color::White,
        "promotion-capture of a kidnapping goblin grants the captor another move (no flip)"
    );
    // Mechanic 1: the kidnap victim drops on the pawn's origin (3,1).
    match &board.grid[1][3].piece {
        Some(PieceType::Knight(k)) => assert_eq!(k.color, Color::White),
        other => panic!("expected dropped white knight at (3,1), got {other:?}"),
    }
    // The promoted queen sits on the goblin's tile (4,0).
    match &board.grid[0][4].piece {
        Some(PieceType::Queen(q)) => assert_eq!(q.color, Color::White),
        other => panic!("expected white queen at (4,0), got {other:?}"),
    }
}

/// Plan 04 / plan 07 coverage: the win-vs-stalemate discriminator
/// (`is_brainrot_win` condition 2 — "every own piece sits on a Brainrot
/// square"). A genuinely stalemated side whose king is NOT on a brainrot
/// square, facing a real OPPOSING phase-3 Skibidi radiating far away
/// (phase>1 but !=4, so the lockout doesn't fire either), must be
/// `Stalemate`, not a `BrainrotWin`. Pins the off-brainrot early-return
/// distinctly from the Neutral-aura and empty-side cases.
#[test]
fn off_brainrot_stalemate_with_radiating_enemy_is_not_brainrot_win() {
    use engine::pieces::fairy::skibidi::Skibidi;

    let mut board = empty_board();
    board.flags.side_to_move = Color::Black;
    // Classic K+Q stalemate box: black king h8 (7,0), white king f7 (5,1),
    // white queen g6 (6,2).
    board.grid[0][7] =
        engine::board::square::Square::new().set_piece(PieceType::new_king(Color::Black));
    board.grid[1][5] =
        engine::board::square::Square::new().set_piece(PieceType::new_king(Color::White));
    board.grid[2][6] =
        engine::board::square::Square::new().set_piece(PieceType::new_queen(Color::White));
    // Far-away White phase-3 Skibidi (radius 2): radiates (phase>1) but
    // !=4, and Manhattan-far so the black king stays OFF any brainrot square.
    board.grid[7][0] = engine::board::square::Square::new().set_piece(PieceType::Skibidi(
        Skibidi {
            color: Color::White,
            phase: 3,
        },
    ));
    board.recalc_brainrot();

    assert!(
        board.legal_moves(&Coord { file: 7, rank: 0 }).is_empty(),
        "black king is boxed — no legal moves"
    );
    assert!(
        !board.is_in_check(Color::Black),
        "black king is not in check (stalemate geometry, not mate)"
    );
    assert_eq!(
        board.status(),
        GameStatus::Stalemate,
        "off-brainrot stalemated side facing a radiating phase-3 enemy is Stalemate, not BrainrotWin"
    );
}

/// Round-3 audit fix: a PieceInCarrier (passenger-exit) capture of a
/// Kidnapping Goblin must NOT grant an extra move — `captor_origin` is
/// `None` for PIC captures, so (like a train run-over) the kidnap victim
/// is silently lost AND the turn flips normally. Pins the
/// `captor_origin.is_some()` gate in `move_grants_extra_turn` (a prior
/// regression dead-coded it, wrongly handing the captor a free turn).
/// The pre-existing `test_pic_capture_routes_through_capture_stack`
/// never checked `side_to_move`, so the bug slipped through.
#[test]
fn pic_capture_of_kidnapping_goblin_flips_turn_no_extra_move() {
    use engine::pieces::fairy::goblin::{Goblin, GoblinState};
    use std::sync::Arc;

    let mut board = empty_board();
    board.flags.side_to_move = Color::White;
    // White Bus at (3,3) carrying a White pawn passenger.
    board.grid[3][3] = engine::board::square::Square::new().set_piece(PieceType::Bus(
        engine::pieces::fairy::bus::Bus {
            color: Color::White,
            pieces: vec![PieceType::new_pawn(Color::White)],
        },
    ));
    // Black Kidnapping Goblin diagonally up-left at (2,2), holding a White knight.
    board.grid[2][2] = engine::board::square::Square::new().set_piece(PieceType::Goblin(Goblin {
        color: Color::Black,
        state: GoblinState::Kidnapping {
            piece: Arc::new(PieceType::new_knight(Color::White)),
        },
        home_square: Coord { file: 0, rank: 0 },
    }));
    board.grid[7][7] =
        engine::board::square::Square::new().set_piece(PieceType::new_king(Color::White));
    board.grid[7][0] =
        engine::board::square::Square::new().set_piece(PieceType::new_king(Color::Black));

    // Passenger pawn exits the Bus to capture the goblin diagonally — a
    // PieceInCarrier{MoveTo}, whose capture_targets entry has captor_origin = None.
    let pic_capture = GameMove {
        from: Coord { file: 3, rank: 3 },
        move_type: MoveType::PieceInCarrier {
            piece_index: 0,
            move_type: Arc::new(MoveType::MoveTo(Coord { file: 2, rank: 2 })),
        },
    };
    board
        .make_move(pic_capture)
        .expect("passenger pawn exits to capture the kidnapping goblin");

    // The turn FLIPS — no extra move for a PIC capture.
    assert_eq!(
        board.flags.side_to_move,
        Color::Black,
        "a PIC passenger-exit capture of a kidnapping goblin grants NO extra move (captor_origin = None)"
    );
    // The kidnap victim (White knight) is silently lost — captor_origin
    // None means GoblinDropVictimCapture drops nothing.
    let any_white_knight = board
        .iter_pieces()
        .any(|(_, p)| matches!(p, PieceType::Knight(k) if k.color == Color::White));
    assert!(
        !any_white_knight,
        "PIC capture has no clean drop site, so the kidnap victim is silently lost"
    );
}

/// Round-3 audit gap: the lockout's opponent phase-4 check is top-level
/// only — a phase-4 enemy Skibidi riding INSIDE a carrier does not
/// radiate (`recalc_brainrot` scans top-level pieces) and must not
/// trigger the lockout. Black (no Skibidi, mobile, not in check) faces a
/// White phase-4 Skibidi that is merely a Bus passenger; the result is
/// `Ongoing`, not `BrainrotLockout`.
#[test]
fn carried_opponent_phase4_skibidi_is_not_a_lockout() {
    use engine::pieces::fairy::skibidi::Skibidi;

    let mut board = empty_board();
    board.flags.side_to_move = Color::Black;
    // White's ONLY phase-4 Skibidi is a passenger inside a White Bus
    // (top-level is the Bus, not a Skibidi — so it doesn't radiate).
    board.grid[3][3] = engine::board::square::Square::new().set_piece(PieceType::Bus(
        engine::pieces::fairy::bus::Bus {
            color: Color::White,
            pieces: vec![PieceType::Skibidi(Skibidi {
                color: Color::White,
                phase: 4,
            })],
        },
    ));
    // Black king, mobile and not in check; Black has no Skibidi.
    board.grid[7][7] =
        engine::board::square::Square::new().set_piece(PieceType::new_king(Color::Black));
    board.grid[0][0] =
        engine::board::square::Square::new().set_piece(PieceType::new_king(Color::White));
    board.recalc_brainrot();

    assert_eq!(
        board.status(),
        GameStatus::Ongoing,
        "a carried (non-radiating) enemy phase-4 Skibidi does not count toward the lockout"
    );
}

/// Round-5 audit gap: the lockout's opponent-colour restriction
/// (`sk.color == opponent`, which is never Neutral) is the mirror of the
/// `is_brainrot_win` Neutral guard a prior round had to bug-fix — but it
/// was untested. A side with no Skibidi facing a *Neutral* phase-4
/// Skibidi (placed far away so the side stays mobile) must be `Ongoing`,
/// not `BrainrotLockout`: a Neutral aura is nobody's lockout. Pins the
/// guard against a regression to `!= loser` (which would match Neutral).
#[test]
fn neutral_phase4_skibidi_is_not_a_lockout() {
    use engine::pieces::fairy::skibidi::Skibidi;

    let mut board = empty_board();
    board.flags.side_to_move = Color::Black;
    // Neutral phase-4 Skibidi in the far corner — Black has no Skibidi.
    board.grid[0][0] = engine::board::square::Square::new().set_piece(PieceType::Skibidi(
        Skibidi {
            color: Color::Neutral,
            phase: 4,
        },
    ));
    // Black king far from the aura: mobile and not in check.
    board.grid[7][7] =
        engine::board::square::Square::new().set_piece(PieceType::new_king(Color::Black));
    board.grid[4][4] =
        engine::board::square::Square::new().set_piece(PieceType::new_king(Color::White));
    board.recalc_brainrot();

    assert_eq!(
        board.status(),
        GameStatus::Ongoing,
        "a Neutral phase-4 Skibidi is not the opponent's lockout — Black plays on"
    );
}

/// Round-6 audit gap: every positive `BrainrotWin` test used a phase-4
/// opposing Skibidi (which also satisfies the lockout). This pins
/// `is_brainrot_win` condition 3's `phase > 1` *lower bound* in
/// isolation: an OPPOSING phase-2 Skibidi (radius 1; not phase 4, so the
/// lockout cannot fire) walling a lone enemy king is a `BrainrotWin`.
/// Guards against a regression narrowing `phase > 1` to `== 4`.
#[test]
fn phase2_brainrot_wall_wins_by_brainrot() {
    use engine::pieces::fairy::skibidi::Skibidi;

    let mut board = empty_board();
    board.flags.side_to_move = Color::Black;
    // White Skibidi PHASE 2 (radius 1) at (3,4).
    board.grid[4][3] = engine::board::square::Square::new().set_piece(PieceType::Skibidi(
        Skibidi {
            color: Color::White,
            phase: 2,
        },
    ));
    // Black king at (3,5): Manhattan 1 from the Skibidi → brainrotted, frozen.
    board.grid[5][3] =
        engine::board::square::Square::new().set_piece(PieceType::new_king(Color::Black));
    // White king far away.
    board.grid[0][7] =
        engine::board::square::Square::new().set_piece(PieceType::new_king(Color::White));
    board.recalc_brainrot();

    // Preconditions: frozen (no legal moves) and not in check.
    assert!(
        board.legal_moves(&Coord { file: 3, rank: 5 }).is_empty(),
        "black king is frozen by the phase-2 aura"
    );
    assert!(
        !board.is_in_check(Color::Black),
        "black king is not in check"
    );
    assert_eq!(
        board.status(),
        GameStatus::BrainrotWin {
            winner: Color::White
        },
        "a phase-2 opposing Skibidi wall is a BrainrotWin (the phase>1 lower bound); \
         phase!=4 means the lockout does not fire, isolating is_brainrot_win condition 3"
    );
}
