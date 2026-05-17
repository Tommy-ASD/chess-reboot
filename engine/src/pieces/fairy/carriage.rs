//! Plan 09: a follower in a train. Inherits its heading from the
//! locomotive at the head of the chain; its next-tick tile is the
//! current tile of the cart immediately in front of it (chain_index − 1).

use tracing::{trace, warn};

use crate::{
    board::{
        Board, Coord, GameMove,
        fen::{find_matching_paren, split_top_level},
    },
    pieces::{
        Color, Piece,
        fairy::locomotive::{passenger_moves, target_is_any_train_cart},
        piecetype::PieceType,
    },
};

#[derive(Clone, PartialEq, Debug, serde::Serialize, serde::Deserialize)]
pub struct Carriage {
    pub train_id: u32,
    /// 1..255; 0 is the locomotive at the head of the same `train_id`.
    pub chain_index: u8,
    pub passengers: Vec<PieceType>,
}

impl Carriage {
    pub fn new(train_id: u32, chain_index: u8) -> Self {
        Carriage {
            train_id,
            chain_index,
            passengers: vec![],
        }
    }

    /// Parse `CART(ID=1,I=1,P=(P))`.
    pub fn from_symbol(symbol: &str) -> Option<PieceType> {
        trace!(symbol, "parsing Carriage");

        let Some(start) = symbol.find('(') else {
            return Some(PieceType::Carriage(Carriage::new(0, 1)));
        };
        let end = find_matching_paren(symbol, start)?;
        let inside = &symbol[start + 1..end];

        let mut train_id: u32 = 0;
        let mut chain_index: u8 = 1;
        let mut passengers: Vec<PieceType> = vec![];

        for field in split_top_level(inside) {
            let mut kv = field.splitn(2, '=');
            // Tolerate stray empty segments (`,,`) and bare keys — a
            // single malformed segment must NOT abort the whole carriage
            // parse and silently delete every already-parsed field.
            let key = kv.next().unwrap_or("").trim();
            let val = kv.next().unwrap_or("").trim();
            if key.is_empty() {
                warn!(field, "empty Carriage field; skipping");
                continue;
            }

            match key {
                "ID" => match val.parse::<u32>() {
                    Ok(v) => train_id = v,
                    Err(e) => warn!(val, ?e, "bad Carriage ID"),
                },
                "I" => match val.parse::<u8>() {
                    Ok(v) if v == 0 => {
                        // Chain index 0 is reserved for the locomotive head;
                        // a Carriage at chain_index=0 has no upstream cart
                        // to follow and would silently stall in advance_trains.
                        // Default to 1 with a warn rather than corrupting state.
                        warn!("Carriage I=0 reserved for locomotive; defaulting to 1");
                        chain_index = 1;
                    }
                    Ok(v) => chain_index = v,
                    Err(e) => warn!(val, ?e, "bad Carriage I"),
                },
                "P" => {
                    let Some(inner) =
                        val.strip_prefix('(').and_then(|s| s.strip_suffix(')'))
                    else {
                        warn!(val, "malformed Carriage P=... field; expected (...)");
                        continue;
                    };
                    for piece_sym in split_top_level(inner) {
                        if let Some(p) = PieceType::symbol_to_piece(&piece_sym) {
                            // Reject nested carriers — see bus.rs.
                            if p.can_carry_piece() {
                                warn!(
                                    piece_sym,
                                    "rejecting nested carrier passenger in Carriage"
                                );
                                continue;
                            }
                            passengers.push(p);
                        }
                    }
                }
                _ => warn!(field, "unknown Carriage attribute"),
            }
        }

        Some(PieceType::Carriage(Carriage {
            train_id,
            chain_index,
            passengers,
        }))
    }
}

impl Piece for Carriage {
    fn name(&self) -> &str {
        "Carriage"
    }
    fn color(&self) -> Color {
        Color::Neutral
    }
    fn set_color(&mut self, _color: Color) {
        // Trains are neutral; recoloring is meaningless.
    }
    fn can_carry_piece(&self) -> bool {
        true
    }
    fn passengers(&self) -> Option<&[PieceType]> {
        Some(&self.passengers)
    }
    fn passengers_mut(&mut self) -> Option<&mut Vec<PieceType>> {
        Some(&mut self.passengers)
    }
    /// Carriages, like locomotives, don't emit player-driven moves of
    /// their own — only the passengers do.
    fn initial_moves(&self, board: &Board, from: &Coord) -> Vec<GameMove> {
        passenger_moves(&self.passengers, board, from)
    }
    /// The cart's next-tick tile (= the cart-in-front's current
    /// position). We report the next-tick tile unconditionally;
    /// the "but this is just chain-following, not a capture" guard
    /// lives in `would_capture_at` so the same-train exclusion
    /// isn't baked into the attack set per piece kind. A king
    /// parked inside the cart-in-front is therefore not flagged
    /// as in-check by the cart behind it — `is_attacked_by`
    /// queries the predicate before counting a reachable tile as
    /// a capture.
    ///
    /// Passenger threats are NOT included here — see Locomotive's
    /// `attacks` doc for the rationale. `is_attacked_by` iterates
    /// Neutral carriers' passengers separately with a per-passenger
    /// color filter, so a passenger pawn of the same colour as the
    /// king doesn't fake a self-check.
    fn attacks(&self, board: &Board, _from: &Coord) -> Vec<Coord> {
        let mut out: Vec<Coord> = Vec::new();
        if self.chain_index > 0 {
            let prev_index = self.chain_index - 1;
            for (coord, piece) in board.all_pieces() {
                let matches = match &piece {
                    PieceType::Locomotive(loco) => {
                        loco.train_id == self.train_id && prev_index == 0
                    }
                    PieceType::Carriage(c) => {
                        c.train_id == self.train_id && c.chain_index == prev_index
                    }
                    _ => false,
                };
                if matches {
                    out.push(coord);
                    break;
                }
            }
        }
        out
    }

    /// Carts never capture *any* other cart — same-train tiles are
    /// chain-following, and foreign carts trigger a Stop in
    /// `advance_trains`. King-safety queries this predicate so a
    /// king parked on either kind of cart at the carriage's
    /// next-tile is not flagged in check.
    fn would_capture_at(&self, board: &Board, _from: &Coord, target: &Coord) -> bool {
        !target_is_any_train_cart(board, target)
    }
    fn symbol(&self) -> String {
        let mut s = format!("CART(ID={},I={}", self.train_id, self.chain_index);
        if !self.passengers.is_empty() {
            let p = self
                .passengers
                .iter()
                .map(|p| p.symbol())
                .collect::<Vec<_>>()
                .join(",");
            s.push_str(&format!(",P=({p})"));
        }
        s.push(')');
        s
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
}
