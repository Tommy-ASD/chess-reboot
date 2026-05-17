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
    /// Impassable terrain. No piece may stand on or slide through a
    /// `Block` square. No payload, no signals, no hooks — the simplest
    /// possible non-walkable type. Invariant: `piece` is always `None`
    /// on a `Block` square; the FEN parser is lenient but `relocate_
    /// pieces` rejects any move whose destination is a `Block`.
    Block,
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
    /// Plan 09: a piece of train track. Trains sitting on this tile leave
    /// in the `direction` (or its opposite, for Reverse-heading trains)
    /// on their next tick. Curves are expressed as direction changes
    /// between adjacent Track tiles. Non-train pieces can also walk over
    /// Track squares — they're walkable like Standard.
    Track { direction: TrackDir },
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

    /// `(df, dr)` step vector to add to a Coord (file, rank). Rank 0 is
    /// the *top* of the grid (FEN convention), so North decreases rank.
    pub fn delta(&self) -> (isize, isize) {
        match self {
            TrackDir::N => (0, -1),
            TrackDir::S => (0, 1),
            TrackDir::E => (1, 0),
            TrackDir::W => (-1, 0),
        }
    }

    /// 180° flip. Used by Reverse-heading trains so they walk a track
    /// backwards along the *same* tiles a Forward-heading train would.
    pub fn opposite(&self) -> Self {
        match self {
            TrackDir::N => TrackDir::S,
            TrackDir::S => TrackDir::N,
            TrackDir::E => TrackDir::W,
            TrackDir::W => TrackDir::E,
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
            SquareType::Block => "BLOCK",
            SquareType::Switch { .. } => "SWITCH",
            SquareType::Junction { .. } => "JUNCTION",
            SquareType::Gate { .. } => "GATE",
            SquareType::PressurePlate { .. } => "PLATE",
            SquareType::Track { .. } => "TRACK",
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
            | SquareType::PressurePlate { .. }
            | SquareType::Track { .. } => true,
            SquareType::Gate { open, .. } => *open,
            SquareType::Turret | SquareType::Vent | SquareType::Block => false,
        }
    }
}
/// ------------- End Square types -------------

/// ------------- Square conditions -------------
#[derive(PartialEq, Debug, Clone)]
pub enum SquareCondition {
    Frozen,
    Brainrot,
    /// Plan 13: a timed tornado. While present, the side to move is
    /// compelled toward this square (see `TornadoCompulsionFilter`) and
    /// a piece standing here is trapped until it dissipates. `remaining`
    /// is the tick countdown; the env-reaction tick decrements it and
    /// removes the condition at 0. The engine's first payload-carrying
    /// condition — hence `to_fen()` rather than `as_str()` at the
    /// serialize site.
    Tornado { remaining: u8 },
    // adding more later on
}

impl SquareCondition {
    /// Bare uppercase tag, no payload. The stable name accessor;
    /// currently the sole caller is `to_fen()` itself, which delegates
    /// here for value-less conditions. Kept `pub` as the canonical
    /// no-payload identifier (FEN serialization goes through `to_fen()`
    /// so payload-carrying conditions round-trip).
    pub fn as_str(&self) -> &'static str {
        match self {
            SquareCondition::Frozen => "FROZEN",
            SquareCondition::Brainrot => "BRAINROT",
            SquareCondition::Tornado { .. } => "TORNADO",
        }
    }

    /// Full FEN value form, including any payload. Value-less
    /// conditions are byte-identical to `as_str()`; `Tornado` appends
    /// `:<remaining>`. Paired with the parser's `:`-split in `fen.rs`.
    pub fn to_fen(&self) -> String {
        match self {
            SquareCondition::Tornado { remaining } => format!("TORNADO:{remaining}"),
            other => other.as_str().to_string(),
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
