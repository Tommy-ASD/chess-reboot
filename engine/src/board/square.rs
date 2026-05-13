use serde::{Deserialize, Serialize};

use crate::board::SignalId;
use crate::pieces::{Color, piecetype::PieceType};

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
    /// Player-thrown emitter. Stores the receiver IDs it fires when thrown.
    Switch { targets: Vec<SignalId> },
    /// Cycle-state receiver. Trains arriving here leave along
    /// `branches[state]`; signal pulses increment `state` mod len.
    Junction {
        id: SignalId,
        state: u8,
        branches: Vec<TrackDir>,
    },
    /// Toggleable blocker. Closed = not walkable.
    Gate { id: SignalId, open: bool },
    /// Auto-emitter; fires when a matching piece settles on the square.
    PressurePlate {
        targets: Vec<SignalId>,
        fires_for: PressureTrigger,
    },
}

/// Cardinal direction for tracks and junction branches. Diagonals are
/// deferred — v1 trains only run on N/S/E/W.
#[derive(PartialEq, Eq, Hash, Debug, Clone, Copy, Serialize, Deserialize)]
pub enum TrackDir {
    N,
    S,
    E,
    W,
}

impl TrackDir {
    pub fn as_str(&self) -> &'static str {
        match self {
            TrackDir::N => "N",
            TrackDir::S => "S",
            TrackDir::E => "E",
            TrackDir::W => "W",
        }
    }

    /// Inverse of `as_str`. Named `parse_tag` to avoid clippy's confusion
    /// with `std::str::FromStr::from_str` — implementing the full trait
    /// just for a 4-variant enum buys nothing today.
    pub fn parse_tag(s: &str) -> Option<Self> {
        match s {
            "N" => Some(TrackDir::N),
            "S" => Some(TrackDir::S),
            "E" => Some(TrackDir::E),
            "W" => Some(TrackDir::W),
            _ => None,
        }
    }
}

/// What triggers a `PressurePlate` to fire when a piece settles on it.
#[derive(PartialEq, Debug, Clone)]
pub enum PressureTrigger {
    AnyPiece,
    OnlyColor(Color),
}

impl SquareType {
    /// Short uppercase identifier used as the `T=` tag inside the FEN
    /// extended-square block. The serializer adds variant-specific
    /// payload fields separately.
    pub fn type_tag(&self) -> &'static str {
        match self {
            SquareType::Standard => "STANDARD",
            SquareType::Turret => "TURRET",
            SquareType::Vent => "VENT",
            SquareType::Switch { .. } => "SWITCH",
            SquareType::Junction { .. } => "JUNCTION",
            SquareType::Gate { .. } => "GATE",
            SquareType::PressurePlate { .. } => "PLATE",
        }
    }

    /// Can a piece stand on this square as a normal destination? Used by
    /// `Board::square_is_empty` (combined with "no current occupant"). The
    /// rule per plan 08: new payload-carrying variants are walkable;
    /// `Gate { open: false }` blocks. Turret/Vent keep their pre-existing
    /// "not walkable" behavior — they're terrain, not floor.
    pub fn is_walkable(&self) -> bool {
        match self {
            SquareType::Standard
            | SquareType::Switch { .. }
            | SquareType::Junction { .. }
            | SquareType::PressurePlate { .. } => true,
            SquareType::Gate { open, .. } => *open,
            SquareType::Turret | SquareType::Vent => false,
        }
    }
}
/// ------------- End Square types -------------

/// ------------- Square conditions -------------
#[derive(PartialEq, Debug, Clone)]
pub enum SquareCondition {
    Frozen,
    Brainrot,
    // adding more later on
}

impl SquareCondition {
    pub fn as_str(&self) -> &'static str {
        match self {
            SquareCondition::Frozen => "FROZEN",
            SquareCondition::Brainrot => "BRAINROT",
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
