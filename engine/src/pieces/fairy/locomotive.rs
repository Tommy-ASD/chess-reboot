//! Plan 09: the head of a train. Moves automatically once per tick along
//! the track tiles, dragging its `Carriage`s behind it. Locomotives are
//! neutral — never the side-to-move — and carry passengers like a Bus.

use tracing::{trace, warn};

use crate::{
    board::{
        Board, Coord, GameMove, MoveType,
        fen::{find_matching_paren, split_top_level},
        square::TrackDir,
    },
    pieces::{Color, Piece, piecetype::PieceType},
};

/// Direction a train walks along its track tiles. `Forward` follows each
/// `Track`'s stored `direction`; `Reverse` follows the opposite. There is
/// no in-game way to set this in v1 — it's editor-time only.
#[derive(Clone, PartialEq, Debug, Copy, serde::Serialize, serde::Deserialize)]
pub enum TrainHeading {
    Forward,
    Reverse,
}

impl TrainHeading {
    pub fn as_str(&self) -> &'static str {
        match self {
            TrainHeading::Forward => "F",
            TrainHeading::Reverse => "R",
        }
    }

    pub fn parse_tag(s: &str) -> Option<Self> {
        match s {
            "F" => Some(TrainHeading::Forward),
            "R" => Some(TrainHeading::Reverse),
            _ => None,
        }
    }
}

#[derive(Clone, PartialEq, Debug, serde::Serialize, serde::Deserialize)]
pub struct Locomotive {
    pub train_id: u32,
    pub heading: TrainHeading,
    pub passengers: Vec<PieceType>,
    /// Cardinal direction this loco *entered* its current tile through.
    /// `None` means "no last move yet" — the next tick is the first
    /// since the loco was placed (or since the FEN was loaded), and the
    /// engine falls back to the tile's stored `D` field (rotated by
    /// `heading`) to pick the exit. Once set, subsequent ticks use
    /// neighbor-detection: the loco exits through the cardinal side
    /// that *isn't* `last_dir`, so rails auto-connect minecart-style.
    pub last_dir: Option<TrackDir>,
}

impl Locomotive {
    pub fn new(train_id: u32, heading: TrainHeading) -> Self {
        Locomotive {
            train_id,
            heading,
            passengers: vec![],
            last_dir: None,
        }
    }

    /// Parse `LOCO(ID=1,H=F,P=(K,R))` (case-insensitive prefix).
    pub fn from_symbol(symbol: &str) -> Option<PieceType> {
        trace!(symbol, "parsing Locomotive");

        let Some(start) = symbol.find('(') else {
            // Bare `LOCO` with no payload — fall back to defaults.
            return Some(PieceType::Locomotive(Locomotive::new(0, TrainHeading::Forward)));
        };
        let end = find_matching_paren(symbol, start)?;
        let inside = &symbol[start + 1..end];

        let mut train_id: u32 = 0;
        let mut heading = TrainHeading::Forward;
        let mut passengers: Vec<PieceType> = vec![];
        let mut last_dir: Option<TrackDir> = None;

        for field in split_top_level(inside) {
            let mut kv = field.splitn(2, '=');
            // Tolerate stray empty segments (`,,`) and bare keys — a
            // single malformed segment must NOT abort the whole loco
            // parse and silently delete every already-parsed field.
            let key = kv.next().unwrap_or("").trim();
            let val = kv.next().unwrap_or("").trim();
            if key.is_empty() {
                warn!(field, "empty Locomotive field; skipping");
                continue;
            }

            match key {
                "ID" => match val.parse::<u32>() {
                    Ok(v) => train_id = v,
                    Err(e) => warn!(val, ?e, "bad Locomotive ID"),
                },
                "H" => match TrainHeading::parse_tag(val) {
                    Some(h) => heading = h,
                    None => warn!(val, "bad Locomotive H"),
                },
                "L" => match TrackDir::parse_tag(val) {
                    Some(d) => last_dir = Some(d),
                    None => warn!(val, "bad Locomotive L (last_dir)"),
                },
                "P" => {
                    let Some(inner) =
                        val.strip_prefix('(').and_then(|s| s.strip_suffix(')'))
                    else {
                        warn!(val, "malformed Locomotive P=... field; expected (...)");
                        continue;
                    };
                    for piece_sym in split_top_level(inner) {
                        if let Some(p) = PieceType::symbol_to_piece(&piece_sym) {
                            // Reject nested carriers — see bus.rs.
                            if p.can_carry_piece() {
                                warn!(
                                    piece_sym,
                                    "rejecting nested carrier passenger in Locomotive"
                                );
                                continue;
                            }
                            passengers.push(p);
                        }
                    }
                }
                _ => warn!(field, "unknown Locomotive attribute"),
            }
        }

        Some(PieceType::Locomotive(Locomotive {
            train_id,
            heading,
            passengers,
            last_dir,
        }))
    }
}

impl Piece for Locomotive {
    fn name(&self) -> &str {
        "Locomotive"
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
    /// Locomotives don't emit player-driven moves of their own. Movement
    /// happens automatically during `Board::advance_trains`. Passengers,
    /// however, can exit via `PieceInCarrier`, mirroring Bus.
    fn initial_moves(&self, board: &Board, from: &Coord) -> Vec<GameMove> {
        passenger_moves(&self.passengers, board, from)
    }
    /// The tile this train will occupy on its next tick (when
    /// `next_train_step` returns `Some`) — i.e. the crush threat
    /// for any king or carrier-of-king parked there. The "but the
    /// loco can't capture its own caboose" wrap-around case is
    /// handled by `would_capture_at`, so `is_attacked_by` filters
    /// phantom threats centrally rather than each piece type baking
    /// its own exclusion logic into the attack set.
    ///
    /// Passenger threats are NOT included here. A Neutral cart's
    /// passenger has its own color (Black king parked on a cart
    /// with a Black passenger pawn shouldn't read as in-check from
    /// its own pawn), and `attacks()` is colour-blind. The board's
    /// `is_attacked_by` iterates Neutral carriers' passengers
    /// separately, filtering by `passenger.get_color() == attacker`.
    fn attacks(&self, board: &Board, from: &Coord) -> Vec<Coord> {
        let mut out: Vec<Coord> = Vec::new();
        if let Some((next, _)) = board.next_train_step(from, self.heading, self.last_dir) {
            out.push(next);
        }
        out
    }

    /// A locomotive doesn't capture *any* cart — same-train carts
    /// trigger an own-collision Stop, foreign carts trigger the
    /// foreign-cart Stop in `advance_trains`. King-safety queries
    /// this so a king parked on either kind of cart at the loco's
    /// next-tile doesn't read as in-check.
    fn would_capture_at(&self, board: &Board, _from: &Coord, target: &Coord) -> bool {
        !target_is_any_train_cart(board, target)
    }
    fn symbol(&self) -> String {
        let mut s = format!("LOCO(ID={},H={}", self.train_id, self.heading.as_str());
        if let Some(d) = self.last_dir {
            s.push_str(&format!(",L={}", d.as_str()));
        }
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

/// True if the tile at `target` currently holds any cart (Locomotive
/// or Carriage). Used by `would_capture_at` so train threats stop at
/// *any* cart on the next-tile — `advance_trains`'s foreign-cart
/// Stop and same-train own-cart Stop together mean a train never
/// actually captures any cart, so flagging the cart tile as a
/// capture threat would be a king-safety overcount.
pub(super) fn target_is_any_train_cart(board: &Board, target: &Coord) -> bool {
    matches!(
        board.get_square_at(target).and_then(|s| s.piece.as_ref()),
        Some(PieceType::Locomotive(_)) | Some(PieceType::Carriage(_))
    )
}

/// Shared passenger-move generator for Locomotive and Carriage. Mirrors
/// the second half of `Bus::initial_moves`: for each passenger, generate
/// its moves as if it were standing on the cart's tile, then wrap each
/// result in `PieceInCarrier` so the make_move dispatcher can pop the
/// passenger out.
pub(super) fn passenger_moves(
    passengers: &[PieceType],
    board: &Board,
    from: &Coord,
) -> Vec<GameMove> {
    let mut moves = Vec::new();
    // Identify the cart's train_id so we can filter out cross-cart hops
    // within the same train. The clone-and-overwrite trick below removes
    // the cart at `from`, so a passenger's move-gen sees other same-train
    // carts as Neutral targets and would emit `MoveIntoCarrier`s onto
    // them — letting a king-passenger silently hop from the locomotive
    // into a friendly carriage of the same train.
    let self_train_id: Option<u32> = match board.get_square_at(from).and_then(|s| s.piece.as_ref()) {
        Some(PieceType::Locomotive(l)) => Some(l.train_id),
        Some(PieceType::Carriage(c)) => Some(c.train_id),
        _ => None,
    };
    for (idx, piece) in passengers.iter().enumerate() {
        let mut board_clone = board.clone();
        board_clone.set_piece_at(from, piece.clone());
        for inner in board_clone.get_moves(from) {
            // Whitelist inner move types that `relocate_pieces`' PIC
            // arm actually handles: only `MoveTo` and `MoveIntoCarrier`.
            // Anything else (Promotion, Castle, EnPassant, ThrowSwitch,
            // PhaseShift, nested PieceInCarrier) would pass `get_moves`
            // here and then fail at apply time with a misleading
            // `ApplyFailed`. Drop at emission instead so `legal_moves`
            // stays consistent with `make_move_unchecked`.
            //
            // PLAYER-FACING CONSEQUENCE: a passenger pawn parked at
            // its own promotion rank cannot promote, and a passenger
            // pawn cannot perform en-passant captures, while inside
            // the carrier. Plan 04's "passenger Pawn semantics" open
            // question owns whether to lift these limits by extending
            // the PIC arm in `relocate_pieces` to handle inner
            // Promotion / EnPassant.
            match &inner.move_type {
                MoveType::MoveTo(_) => {}
                MoveType::MoveIntoCarrier(target) => {
                    // Same-train cross-cart hops: the clone-and-overwrite
                    // at the top of the loop removes the cart at `from`,
                    // so the inner filter saw remaining same-train carts
                    // as Neutral and rewrote MoveTo→MoveIntoCarrier.
                    // Drop those rewrites.
                    if let Some(tid) = self_train_id {
                        let target_is_same_train_cart = board
                            .get_square_at(target)
                            .and_then(|s| s.piece.as_ref())
                            .map(|p| match p {
                                PieceType::Locomotive(l) => l.train_id == tid,
                                PieceType::Carriage(c) => c.train_id == tid,
                                _ => false,
                            })
                            .unwrap_or(false);
                        if target_is_same_train_cart {
                            continue;
                        }
                    }
                }
                _ => continue,
            }
            moves.push(GameMove {
                from: from.clone(),
                move_type: MoveType::PieceInCarrier {
                    piece_index: idx as u8,
                    move_type: inner.move_type.into(),
                },
            });
        }
    }
    moves
}
