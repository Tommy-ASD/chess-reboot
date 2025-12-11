/// The Bus
/// Carrier piece
///
/// The Bus is a versatile carrier piece that can transport up to five allied pieces across the board.
/// This allows you to reposition your forces quickly and strategically.
/// However, if the Bus is captured, all pieces inside are lost as well.
/// The Bus moves like a standard rook - horizontally or vertically until there is a piece blocking the path.
/// The Bus cannot take pieces.
/// To exit the Bus, a piece simply moves out of the square the Bus occupies, following its usual movement rules.
use crate::{
    board::{
        Board, Coord, GameMove, MoveType,
        fen::{find_matching_paren, split_top_level},
    },
    pieces::{Color, Piece, piecetype::PieceType},
};

#[derive(Clone, PartialEq, Debug)]
pub struct Bus {
    pub color: Color,
    pub pieces: Vec<PieceType>,
}

impl Bus {
    pub fn new(color: Color) -> Self {
        Bus {
            color,
            pieces: vec![],
        }
    }

    pub fn from_symbol(symbol: &str) -> Option<PieceType> {
        dbg!();
        println!("Parsing Bus from symbol: {}", symbol);

        let first = symbol.chars().next()?;
        let color = match first {
            'B' => Color::White,
            'b' => Color::Black,
            _ => return None,
        };

        dbg!();
        let Some(start) = symbol.find('(') else {
            dbg!();
            return Some(PieceType::Bus(Bus {
                color,
                pieces: vec![],
            }));
        };

        let end = find_matching_paren(symbol, start)?;
        let inside = &symbol[start + 1..end];

        let mut home_square: Option<Coord> = None;

        let mut pieces = vec![];

        for field in split_top_level(inside) {
            dbg!();
            let mut kv = field.splitn(2, '=');
            let key = kv.next()?.trim();
            let val = kv.next()?.trim();

            println!("Handling `{field}` (turned into `{key}={val}`)");

            match key {
                // is an array
                "P" => {
                    dbg!();
                    let val = val.strip_prefix("(").unwrap().strip_suffix(")").unwrap();
                    for piece_sym in split_top_level(val) {
                        let opt_inner_piece = PieceType::symbol_to_piece(&piece_sym);
                        println!("Piece symbol {piece_sym} turned into {opt_inner_piece:?}");
                        if let Some(inner_piece) = opt_inner_piece {
                            pieces.push(inner_piece);
                        }
                    }
                }
                _ => {
                    println!("Unknown Goblin attribute: {}", field);
                }
            }
        }

        Some(PieceType::Bus(Bus { color, pieces }))
    }
}

impl Piece for Bus {
    fn name(&self) -> &str {
        "Bus"
    }
    fn color(&self) -> Color {
        self.color
    }
    fn set_color(&mut self, color: Color) {
        self.color = color;
    }
    fn can_carry_piece(&self) -> bool {
        true
    }
    fn initial_moves(&self, board: &Board, from: &Coord) -> Vec<GameMove> {
        dbg!();
        let mut moves = Vec::new();
        let directions: [(isize, isize); 8] = [
            (1, 0),
            (1, 1),
            (0, 1),
            (-1, 1),
            (-1, 0),
            (-1, -1),
            (0, -1),
            (1, -1),
        ];

        for (df, dr) in &directions {
            let new_file = from.file as isize + df;
            let new_rank = from.rank as isize + dr;
            if new_file >= 0 && new_file < 8 && new_rank >= 0 && new_rank < 8 {
                let coord = Coord {
                    file: new_file as u8,
                    rank: new_rank as u8,
                };
                if board.square_is_empty(&coord) {
                    let game_move = GameMove {
                        from: from.clone(),
                        move_type: MoveType::MoveTo(coord.clone()),
                    };
                    moves.push(game_move);
                }
            }
        }

        for (idx, piece) in self.pieces.iter().enumerate() {
            let mut board_clone = board.clone();
            board_clone.set_piece_at(from, piece.clone());
            let inner_piece_moves = board_clone.get_moves(from);
            for game_move in inner_piece_moves {
                moves.push(GameMove {
                    from: from.clone(),
                    move_type: MoveType::PieceInCarrier {
                        piece_index: idx as u8,
                        move_type: game_move.move_type.into(),
                    },
                });
            }
        }

        moves
    }
    fn symbol(&self) -> String {
        let mut sym = match self.color {
            Color::White => "BUS".to_string(),
            Color::Black => "bus".to_string(),
        };

        if !self.pieces.is_empty() {
            let pieces_map = self
                .pieces
                .iter()
                .map(|piece| piece.symbol())
                .collect::<Vec<String>>()
                .join(",");
            println!("Pieces map: {pieces_map}");
            sym.push_str("(P=(");
            sym.push_str(&pieces_map);
            sym.push_str("))");
        }

        sym
    }
    fn clone_box(&self) -> Box<dyn Piece> {
        Box::new(self.clone())
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn post_move_effects(
        &mut self,
        board_before: &Board,
        board_after: &mut Board,
        game_move: &GameMove,
    ) {
        // match &game_move.move_type {
        //     MoveType::PhaseShift => {}
        //     MoveType::MoveTo(target) => {}
        // }
    }
}
