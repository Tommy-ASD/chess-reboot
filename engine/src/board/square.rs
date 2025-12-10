use crate::pieces::piecetype::PieceType;

/// ------------- Square logic -------------
#[derive(Clone, PartialEq, Debug)]
pub struct Square {
    pub piece: Option<PieceType>,
    pub square_type: SquareType,
    pub conditions: Vec<SquareCondition>,
}

/// ------------- Square types -------------
#[derive(PartialEq, Debug, Clone)]
pub enum SquareType {
    Standard,
    Turret,
    Vent,
    // adding more later on
}

impl SquareType {
    pub fn as_str(&self) -> &'static str {
        match self {
            SquareType::Standard => "STANDARD",
            SquareType::Turret => "TURRET",
            SquareType::Vent => "VENT",
        }
    }
}
/// ------------- End Square types -------------

/// ------------- Square conditions -------------
#[derive(PartialEq, Debug, Clone)]
pub enum SquareCondition {
    Frozen,
    // adding more later on
}

impl SquareCondition {
    pub fn as_str(&self) -> &'static str {
        match self {
            SquareCondition::Frozen => "FROZEN",
        }
    }
}
/// ------------- End Square conditions -------------

impl Square {
    pub fn new() -> Self {
        Self {
            piece: None,
            square_type: SquareType::Standard,
            conditions: vec![],
        }
    }
    pub fn set_piece(mut self, piece: PieceType) -> Self {
        self.piece = Some(piece);
        self
    }
    pub fn remove_piece(mut self) -> Self {
        self.piece = None;
        self
    }
    pub fn set_square_type(mut self, square_type: SquareType) -> Self {
        self.square_type = square_type;
        self
    }
    pub fn add_square_condition(mut self, square_condition: SquareCondition) -> Self {
        self.conditions.push(square_condition);
        self
    }

    pub fn has_piece(&self) -> bool {
        self.piece.is_some()
    }

    pub fn has_piece_of_color(&self, color: crate::pieces::Color) -> bool {
        if let Some(piece) = &self.piece {
            piece.get_color() == color
        } else {
            false
        }
    }
}
