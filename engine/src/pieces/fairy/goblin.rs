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
    },
}

#[derive(Debug, PartialEq)]
pub struct Goblin {
    pub color: Color,
    pub state: GoblinState,
    pub home_square: Coord,
}

impl Clone for Goblin {
    fn clone(&self) -> Self {
        Goblin {
            color: self.color,
            state: self.state.clone(),
            home_square: self.home_square.clone(),
        }
    }
}

impl Goblin {
    pub fn new(color: Color, home_square: Coord) -> Self {
        Goblin {
            color,
            state: GoblinState::Free,
            home_square,
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

    fn parse_coord(pair: &str) -> Option<Coord> {
        let mut parts = pair.split('-');
        let file = parts.next()?.parse::<u8>().ok()?;
        let rank = parts.next()?.parse::<u8>().ok()?;

        Some(Coord { file, rank })
    }

    // this one is tricky because the symbol changes based on state
    // so usually, it's G(H=0-0) for free goblin
    // where H=0-0 indicates home square

    // but when kidnapping, it could be something else
    // and inside the brackets, the symbol of the piece being carried
    // e.g. `G(H=0-0,P=n)` for white goblin carrying black knight
    pub fn from_symbol(symbol: &str) -> Option<PieceType> {
        // debug print
        dbg!();
        println!("Parsing Goblin from symbol: {}", symbol);

        let first = symbol.chars().next()?;
        let color = match first {
            'G' => Color::White,
            'g' => Color::Black,
            _ => return None,
        };

        // No bracket, fallback: a generic free goblin with default home?
        // fallback to home square at a1
        let Some(start) = symbol.find('(') else {
            return Some(PieceType::Goblin(Goblin {
                color,
                state: GoblinState::Free,
                home_square: Coord { file: 0, rank: 0 },
            }));
        };

        let end = symbol.find(')')?;
        let inside = &symbol[start + 1..end];

        let mut home_square: Option<Coord> = None;
        let mut kidnapped_piece: Option<PieceType> = None;

        for field in inside.split(',') {
            let mut kv = field.splitn(2, '=');
            let key = kv.next()?.trim();
            let val = kv.next()?.trim();

            match key {
                "H" => {
                    home_square = Self::parse_coord(val);
                    if home_square.is_none() {
                        println!("Invalid Goblin home square: {}", val);
                    }
                }
                "P" => {
                    kidnapped_piece = PieceType::symbol_to_piece(val);
                    if kidnapped_piece.is_none() {
                        println!("Unknown kidnapped piece symbol: {}", val);
                    }
                }
                _ => {
                    println!("Unknown Goblin attribute: {}", field);
                }
            }
        }

        // default to home square at a1 if not specified
        let home_sq = home_square.unwrap_or(Coord { file: 0, rank: 0 });

        let state = if let Some(p) = kidnapped_piece {
            GoblinState::Kidnapping { piece: p.into() }
        } else {
            GoblinState::Free
        };

        Some(PieceType::Goblin(Goblin {
            color,
            state,
            home_square: home_sq,
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
    fn set_color(&mut self, color: Color) {
        self.color = color;
    }

    fn initial_moves(&self, board: &Board, from: &Coord) -> Vec<GameMove> {
        println!("Hello :3");
        self.generate_goblin_base_moves(board, from.clone())
    }

    fn symbol(&self) -> String {
        let prefix = match self.color {
            Color::White => "G",
            Color::Black => "g",
        };

        match &self.state {
            GoblinState::Free => {
                // Free state only encodes home square
                format!(
                    "{}(H={}-{})",
                    prefix, self.home_square.file, self.home_square.rank
                )
            }

            GoblinState::Kidnapping { piece } => {
                // Include home square AND kidnapped piece symbol
                format!(
                    "{}(H={}-{},P={})",
                    prefix,
                    self.home_square.file,
                    self.home_square.rank,
                    piece.symbol()
                )
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
                                        };
                                    }
                                }
                            }
                        }
                    }
                }
            }
            // handle the conversion of and dropping off of kidnapped piece
            GoblinState::Kidnapping { piece } => {
                let mut p: PieceType = piece.clone().into();
                p.set_color(self.color);
                if to == &self.home_square {
                    // drop off the kidnapped piece
                    board_after.set_piece_at(to, p);
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
