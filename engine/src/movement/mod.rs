use crate::{board::Sq, pieces::piecetype::PieceType};

/// Movement types
pub enum MoveKind {
    Normal,
    Capture,
    Castling,
    Promotion(PieceType),
    EnPassant,
    Teleport,
    Custom(&'static str), // fallback
}

pub struct Move {
    pub from: Option<Sq>,
    pub to: Sq,
    pub kind: MoveKind,
}
