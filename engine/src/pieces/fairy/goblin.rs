use std::rc::Rc;

/// Goblin - Moves like a queen at first, but once it takes a piece,
/// it "kidnaps" that piece and has to take it back to home base
/// After taking a piece, the goblin moves like a king until it reaches it's home square.
/// Once it reaches the home square, the "kidnapped" piece is changes color to that of who took it.
/// If the goblin is taken by an enemy piece while it has a piece kidnapped,
/// the kidnapped piece is placed where the goblin was located, and the taking piece can move again
use crate::{
    board::{Board, Coord, GameMove},
    movement::glider::{OMNI_DIRS, generate_glider_moves},
    pieces::{Color, Piece, piecetype::PieceType},
};

#[derive(Clone, Debug, PartialEq)]
pub enum GoblinState {
    Free, // hasn't kidnapped any piece
    Kidnapping {
        piece: Rc<PieceType>, // the piece being carried
        home: Coord,          // goblin’s home
    },
}

#[derive(Debug, PartialEq)]
pub struct Goblin {
    pub color: Color,
    pub state: GoblinState,
}

impl Clone for Goblin {
    fn clone(&self) -> Self {
        Goblin {
            color: self.color,
            state: match &self.state {
                GoblinState::Free => GoblinState::Free,
                GoblinState::Kidnapping { piece, home } => GoblinState::Kidnapping {
                    piece: piece.clone(),
                    home: home.clone(),
                },
            },
        }
    }
}

impl Goblin {
    pub fn new(color: Color) -> Self {
        Goblin {
            color,
            state: GoblinState::Free,
        }
    }

    pub fn generate_goblin_free_moves(&self, board: &Board, from: Coord) -> Vec<GameMove> {
        dbg!();
        generate_glider_moves(board, &from, &OMNI_DIRS, usize::MAX)
    }

    pub fn generate_goblin_kidnapping_moves(&self, board: &Board, from: Coord) -> Vec<GameMove> {
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
                        to: coord.clone(),
                    };
                    moves.push(game_move);
                }
            }
        }

        moves
    }

    pub fn generate_goblin_base_moves(&self, board: &Board, from: Coord) -> Vec<GameMove> {
        match &self.state {
            GoblinState::Free => self.generate_goblin_free_moves(board, from),
            GoblinState::Kidnapping { .. } => self.generate_goblin_kidnapping_moves(board, from),
        }
    }

    // this one is tricky because the symbol changes based on state
    // so usually, it's just G/g for free goblin
    // but when kidnapping, it could be something else
    // thinking brackets [] for carrying a piece
    // and inside the brackets, the symbol of the piece being carried
    // e.g. `G[P=n]` for white goblin carrying black knight
    pub fn from_symbol(symbol: &str) -> Option<PieceType> {
        let color = match symbol.chars().next()? {
            'G' => Color::White,
            'g' => Color::Black,
            _ => return None,
        };
        // check for kidnapping state
        // e.g. G[P=n]
        // check for brackets, inside brackets, check for P=, if found, get the piece symbol after = (which lasts until , or ])
        if let Some(start) = symbol.find('[') {
            if let Some(end) = symbol.find(']') {
                let inside = &symbol[start + 1..end];
                let parts: Vec<&str> = inside.split(',').collect();
                for part in parts {
                    let kv: Vec<&str> = part.split('=').collect();
                    if kv.len() == 2 && kv[0] == "P" {
                        let sym = kv[1];
                        if let Some(p) = PieceType::symbol_to_piece(sym) {
                            return Some(PieceType::Goblin(Goblin {
                                color,
                                state: GoblinState::Kidnapping {
                                    piece: Rc::new(p),
                                    home: match color {
                                        Color::White => Coord { file: 0, rank: 0 },
                                        Color::Black => Coord { file: 7, rank: 7 },
                                    },
                                },
                            }));
                        } else {
                            println!("Unknown piece!! {sym}");
                        }
                    }
                }
            }
        }

        Some(PieceType::Goblin(Goblin {
            color,
            state: GoblinState::Free,
        }))
    }
}

impl Piece for Goblin {
    fn name(&self) -> &str {
        "Goblin"
    }

    fn color(&self) -> Color {
        self.color
    }

    fn initial_moves(&self, board: &Board, from: &Coord) -> Vec<GameMove> {
        println!("Hello :3");
        self.generate_goblin_base_moves(board, from.clone())
    }

    // this one is tricky because the symbol changes based on state
    // so usually, it's just G/g for free goblin
    // but when kidnapping, it could be something else
    // thinking brackets [] for carrying a piece
    // and inside the brackets, the symbol of the piece being carried
    // e.g. `G[P=n]` for white goblin carrying black knight
    // need to refactor fen generation to handle this properly
    fn symbol(&self) -> String {
        match &self.state {
            GoblinState::Free => match self.color {
                Color::White => 'G'.to_string(),
                Color::Black => 'g'.to_string(),
            },
            GoblinState::Kidnapping { piece, .. } => {
                let piece_symbol = piece.symbol();
                match self.color {
                    Color::White => format!("G[P={}]", piece_symbol),
                    Color::Black => format!("g[P={}]", piece_symbol),
                }
            }
        }
    }

    fn clone_box(&self) -> Box<dyn Piece> {
        Box::new((self.clone()))
    }

    fn post_move_effects(
        &self,
        board_before: &Board,
        board_after: &mut Board,
        from: &Coord,
        to: &Coord,
    ) {
        match &self.state {
            // handle kidnapping state change
            // check if there was an enemy piece at the destination in the board before the move
            GoblinState::Free => {
                if let Some(square) = board_before.get_square_at(to) {
                    if let Some(captured_piece) = &square.piece {
                        if captured_piece.get_color() != self.color {
                            // initiate kidnapping
                            let home = match self.color {
                                Color::White => Coord { file: 0, rank: 0 },
                                Color::Black => Coord { file: 7, rank: 7 },
                            };
                            // update goblin state in the after board
                            if let Some(goblin_square) = board_after.get_square_mut(to) {
                                if let Some(piece) = &mut goblin_square.piece {
                                    if let Some(goblin) =
                                        piece.as_any_mut().downcast_mut::<Goblin>()
                                    {
                                        goblin.state = GoblinState::Kidnapping {
                                            piece: captured_piece.clone().into(),
                                            home,
                                        };
                                    }
                                }
                            }
                        }
                    }
                }
            }
            // handle the conversion of and dropping off of kidnapped piece
            GoblinState::Kidnapping { piece, home } => {
                if to == home {
                    // drop off the kidnapped piece
                    board_after.set_piece_at(to, piece.clone().into());
                    // goblin dies (maybe change later?)
                }
            }
            _ => {}
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
