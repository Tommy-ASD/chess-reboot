use crate::pieces::piecekind::PieceKind;

/// ------------- Square logic -------------
#[derive(Clone, PartialEq, Debug)]
pub struct Square {
    pub piece: Option<PieceKind>,
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
    fn as_str(&self) -> &'static str {
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
    fn as_str(&self) -> &'static str {
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
    pub fn set_piece(mut self, piece: PieceKind) -> Self {
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
}

pub fn square_to_fen(square: &Square) -> String {
    let piece_symbol = square
        .piece
        .as_ref()
        .map(|p| p.symbol())
        .unwrap_or("".to_string());
    let is_standard_square =
        matches!(square.square_type, SquareType::Standard) && square.conditions.is_empty();

    if piece_symbol.len() == 1 && is_standard_square {
        return piece_symbol; // e.g., "P" or "r"
    }

    // Non-standard notation
    let mut parts = vec![];

    if !piece_symbol.is_empty() {
        parts.push(format!("P={}", piece_symbol));
    }

    if !matches!(square.square_type, SquareType::Standard) {
        parts.push(format!("T={}", square.square_type.as_str()));
    }

    for cond in &square.conditions {
        parts.push(format!("C={}", cond.as_str()));
    }

    format!("({})", parts.join(","))
}

pub fn fen_to_square(fen: &str) -> Square {
    // Standard empty square
    if fen.is_empty() || fen == "()" {
        return Square {
            piece: None,
            square_type: SquareType::Standard,
            conditions: vec![],
        };
    }

    // Extended format (P=...,T=...,C=...)
    if fen.starts_with('(') && fen.ends_with(')') {
        let inner = &fen[1..fen.len() - 1];
        let mut piece = None;
        let mut square_type = SquareType::Standard;
        let mut conditions = vec![];

        for part in inner.split(',') {
            let kv: Vec<&str> = part.split('=').collect();
            if kv.len() != 2 {
                continue;
            }

            match kv[0] {
                "P" => {
                    let sym = kv[1];
                    if let Some(p) = PieceKind::symbol_to_piece(sym.chars().next().unwrap()) {
                        piece = Some(p);
                    } else {
                        println!("Unknown piece!! {sym}");
                    }
                }
                "T" => {
                    square_type = {
                        let sqty = kv[1];
                        match sqty {
                            "TURRET" => SquareType::Turret,
                            "VENT" => SquareType::Vent,
                            _ => {
                                println!("Unknown square type!! {sqty}");
                                SquareType::Standard
                            }
                        }
                    }
                }
                "C" => {
                    let sqcon = kv[1];
                    match sqcon {
                        "FROZEN" => conditions.push(SquareCondition::Frozen),
                        _ => {
                            println!("Unknown square condition!! {sqcon}");
                        }
                    }
                }
                _ => {}
            }
        }

        return Square {
            piece,
            square_type,
            conditions,
        };
    }

    // Standard single-character piece
    let piece = PieceKind::symbol_to_piece(fen.chars().next().unwrap());
    Square {
        piece,
        square_type: SquareType::Standard,
        conditions: vec![],
    }
}
