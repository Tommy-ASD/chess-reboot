#[cfg(test)]
mod tests {
    use crate::{
        board::{
            Board, BoardFlags, board_to_fen, fen_to_board,
            square::{Square, SquareCondition, SquareType},
        },
        pieces::{Color, piecekind::PieceKind},
    };

    use super::*;

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

        board.grid[0][0] = Square::new().set_piece(PieceKind::new_rook(Color::White));
        board.grid[7][7] = Square::new().set_piece(PieceKind::new_king(Color::Black));

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
            .set_piece(PieceKind::new_rook(Color::White))
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
            .set_piece(PieceKind::new_knight(Color::Black))
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
            .set_piece(PieceKind::new_knight(Color::Black))
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
        board.grid[0][0] = Square::new().set_piece(PieceKind::new_rook(Color::White));
        board.grid[0][1] = Square::new()
            .set_piece(PieceKind::new_knight(Color::Black))
            .set_square_type(SquareType::Turret)
            .add_square_condition(SquareCondition::Frozen);

        let fen = board_to_fen(&board);
        let board2 = fen_to_board(&fen);
        assert_eq!(board2, board);
    }
}
