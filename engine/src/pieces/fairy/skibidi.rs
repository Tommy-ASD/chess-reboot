/// Skibidi
///     Stuns all pieces (enemy and ally) in a given circle radius. They can no longer move due to extreme brainrot.
///     No effect on others (phase 1) -> the 4 non-diagonal neighboring cells (circle of radius with 1, phase 2) ->
///         a circle with a radius of 2 (phase 3) -> a circle with a radius of 3 (phase 4)
///     This effect is removed once the Skibidi is captured, or another neutralizing Skibidi enters the Brainrot radius.
///     Any Skibidi (enemy or ally) entering the Brainrot radius acts as a neutralizing Skibidi.
///     After being neutralized, the Skibidi is set back to phase 1.
///     The Skibidi can move, but it is set back to phase 1 each time it moves.
///     Increasing the radius of brainrot uses a move.
///     If there is no opposing Skibidi, the maximum phase your Skibidi can reach is 3.
///     It moves like a king (to any directly neighboring cells), but cannot take other pieces.
///     It can take other Skibidis
///     If your Skibidi your enemy cannot make a move due to your Brainrot,
///         you win by Brainrot instead of stalemate being declared.
///     If your Skibidi is captured while your opponent's Skibidi is in phase 4, there is nothing you can do.
use crate::{
    board::{Board, Coord, GameMove, MoveType},
    pieces::{Color, Piece, piecetype::PieceType},
};

#[derive(Clone, PartialEq, Debug)]
pub struct Skibidi {
    pub color: Color,
    pub phase: u8, // 1 to 4
}

impl Skibidi {
    pub fn new(color: Color) -> Self {
        Skibidi { color, phase: 1 }
    }

    // the skibidi has state
    // so we need custom symbol
    // the state only contains phase
    // we can keep it simple and forget the state in phase 1
    pub fn from_symbol(symbol: &str) -> Option<PieceType> {
        println!("Got symbol {symbol}");

        let color = if symbol.chars().next().unwrap().is_lowercase() {
            Color::Black
        } else {
            Color::White
        };

        // if no custom state, skibidi phase 1
        let Some(start) = symbol.find('(') else {
            return Some(PieceType::Skibidi(Skibidi { color, phase: 1 }));
        };

        let end = symbol.find(')')?;
        let inside = &symbol[start + 1..end];

        let mut phase = 1;

        for field in inside.split(',') {
            let mut kv = field.splitn(2, '=');
            let key = kv.next()?.trim();
            let val = kv.next()?.trim();

            match key {
                "PHASE" => {
                    println!("Got phase {val}");
                    match val.parse::<u8>() {
                        Ok(ok) => phase = ok,
                        Err(e) => println!("Got invalid phase for Skibidi: {val}"),
                    };
                }
                _ => {
                    println!("Unknown Goblin attribute: {}", field);
                }
            }
        }

        Some(PieceType::Skibidi(Skibidi { color, phase }))
    }
}

impl Piece for Skibidi {
    fn name(&self) -> &str {
        "Skibidi"
    }
    fn color(&self) -> Color {
        self.color
    }
    fn set_color(&mut self, color: Color) {
        self.color = color;
    }
    fn initial_moves(&self, board: &Board, from: &Coord) -> Vec<GameMove> {
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
                if let Some(square) = board.get_square_at(&coord) {
                    if square.piece.is_none()
                        || square.piece.as_ref().map_or(false, |p| {
                            p.symbol().to_lowercase() == self.symbol().to_lowercase()
                        })
                    {
                        let game_move = GameMove {
                            from: from.clone(),
                            move_type: MoveType::MoveTo(coord.clone()),
                        };
                        moves.push(game_move);
                    }
                }
            }
        }

        moves.push(GameMove {
            from: from.clone(),
            move_type: MoveType::PhaseShift,
        });

        moves
    }
    fn symbol(&self) -> String {
        let mut sym = match self.color {
            Color::White => 'S'.to_string(),
            Color::Black => 's'.to_string(),
        };
        if self.phase > 1 {
            sym.push_str(&format!("(PHASE={phase})", phase = self.phase));
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
        match &game_move.move_type {
            MoveType::PhaseShift => {}
            MoveType::MoveTo(target) => {
                self.phase = 1;
                board_after.set_piece_at(&target, PieceType::Skibidi(self.clone()));
            }
            MoveType::MoveIntoCarrier(target) => todo!(),
        }
    }
}
