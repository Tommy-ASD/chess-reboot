use std::{
    any::Any,
    fmt::{Debug, Formatter, Result},
};

use crate::board::{Board, Coord, GameMove};

pub mod chess2;
pub mod fairy;
pub mod piecetype;
pub mod standard;

/// ------------- Pieces -------------

pub trait Piece {
    fn name(&self) -> &str;
    fn color(&self) -> Color;
    fn set_color(&mut self, color: Color);
    fn initial_moves(&self, board: &Board, from: &Coord) -> Vec<GameMove>;
    fn symbol(&self) -> String;

    fn clone_box(&self) -> Box<dyn Piece>;

    fn post_move_effects(
        &self,
        _board_before: &Board,
        _board_after: &mut Board,
        _from: &Coord,
        _to: &Coord,
    ) {
        // Default: do nothing
    }

    /// For downcasting support
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

impl PartialEq for dyn Piece {
    fn eq(&self, other: &Self) -> bool {
        self.symbol() == other.symbol()
    }
}

impl Debug for dyn Piece {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self.symbol())
    }
}

impl Clone for Box<dyn Piece> {
    fn clone(&self) -> Box<dyn Piece> {
        self.clone_box()
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Color {
    White,
    Black,
}

impl Color {
    pub fn opposite(&self) -> Color {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }
}
