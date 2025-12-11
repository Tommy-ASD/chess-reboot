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
    board::{Board, Coord, GameMove, MoveType},
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
        println!("Parsing Bus from symbol: {}", symbol);

        let first = symbol.chars().next()?;
        let color = match first {
            'B' => Color::White,
            'b' => Color::Black,
            _ => return None,
        };

        let bus = Bus::new(color);

        Some(PieceType::Bus(bus))
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
    fn initial_moves(&self, board: &Board, from: &Coord) -> Vec<GameMove> {
        vec![]
    }
    fn symbol(&self) -> String {
        let mut sym = match self.color {
            Color::White => "BUS".to_string(),
            Color::Black => "bus".to_string(),
        };
        // if self.phase > 1 {
        //     sym.push_str(&format!("(PHASE={phase})", phase = self.phase));
        // }

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
