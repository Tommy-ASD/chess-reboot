use std::rc::Rc;

use tracing::{trace, warn};

/// Goblin - Moves like a queen at first, but once it takes a piece,
/// it "kidnaps" that piece and has to take it back to home base
/// After taking a piece, the goblin moves like a king until it reaches it's home square.
/// Once it reaches the home square, the "kidnapped" piece is changes color to that of who took it.
/// If the goblin is taken by an enemy piece while it has a piece kidnapped,
/// the kidnapped piece is placed where the goblin was located, and the taking piece can move again
use crate::{
    board::{
        Board, Coord, GameMove, MoveType,
        fen::{find_matching_paren, split_top_level},
    },
    movement::glider::{OMNI_DIRS, generate_glider_moves},
    pieces::{Color, Piece, piecetype::PieceType},
};

#[derive(Clone, Debug, PartialEq)]
pub enum GoblinState {
    Free, // hasn't kidnapped any piece
    Kidnapping {
        piece: Rc<PieceType>, // the piece being carried
    },
}

#[derive(Debug, PartialEq)]
pub struct Goblin {
    pub color: Color,
    pub state: GoblinState,
    pub home_square: Coord,
}

impl Clone for Goblin {
    fn clone(&self) -> Self {
        Goblin {
            color: self.color,
            state: self.state.clone(),
            home_square: self.home_square.clone(),
        }
    }
}

impl Goblin {
    pub fn new(color: Color, home_square: Coord) -> Self {
        Goblin {
            color,
            state: GoblinState::Free,
            home_square,
        }
    }

    pub fn generate_goblin_free_moves(&self, board: &Board, from: Coord) -> Vec<GameMove> {
        trace!("goblin free moves");
        generate_glider_moves(board, &from, &OMNI_DIRS, usize::MAX)
    }

    pub fn generate_goblin_kidnapping_moves(&self, board: &Board, from: Coord) -> Vec<GameMove> {
        trace!("goblin kidnapping moves");
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
            if board.in_bounds(new_file, new_rank) {
                let coord = Coord {
                    file: new_file as u8,
                    rank: new_rank as u8,
                };
                if board.square_is_empty(&coord) {
                    let game_move = GameMove {
                        from: from.clone(),
                        move_type: MoveType::MoveTo(coord.clone()),
                    };
                    moves.push(game_move);
                }
            }
        }

        moves
    }

    pub fn generate_goblin_base_moves(&self, board: &Board, from: Coord) -> Vec<GameMove> {
        match &self.state {
            GoblinState::Free => self.generate_goblin_free_moves(board, from),
            GoblinState::Kidnapping { .. } => self.generate_goblin_kidnapping_moves(board, from),
        }
    }

    fn parse_coord(pair: &str) -> Option<Coord> {
        let mut parts = pair.split('-');
        let file = parts.next()?.parse::<u8>().ok()?;
        let rank = parts.next()?.parse::<u8>().ok()?;

        Some(Coord { file, rank })
    }

    // this one is tricky because the symbol changes based on state
    // so usually, it's G(H=0-0) for free goblin
    // where H=0-0 indicates home square

    // but when kidnapping, it could be something else
    // and inside the brackets, the symbol of the piece being carried
    // e.g. `G(H=0-0,P=n)` for white goblin carrying black knight
    pub fn from_symbol(symbol: &str) -> Option<PieceType> {
        trace!(symbol, "parsing Goblin");

        let first = symbol.chars().next()?;
        let color = match first {
            'G' => Color::White,
            'g' => Color::Black,
            _ => return None,
        };

        // No bracket, fallback: a generic free goblin with default home?
        // fallback to home square at a1
        let Some(start) = symbol.find('(') else {
            return Some(PieceType::Goblin(Goblin {
                color,
                state: GoblinState::Free,
                home_square: Coord { file: 0, rank: 0 },
            }));
        };

        let end = find_matching_paren(symbol, start)?;
        let inside = &symbol[start + 1..end];

        let mut home_square: Option<Coord> = None;
        let mut kidnapped_piece: Option<PieceType> = None;

        for field in split_top_level(inside) {
            let mut kv = field.splitn(2, '=');
            // Tolerate stray empty segments (`,,`) and bare keys — a
            // single malformed segment must NOT abort the whole Goblin
            // parse and silently delete every already-parsed field.
            // (Same defensive pattern as Bus/Loco/Carriage.)
            let key = kv.next().unwrap_or("").trim();
            let val = kv.next().unwrap_or("").trim();
            if key.is_empty() {
                warn!(field, "empty Goblin field; skipping");
                continue;
            }

            trace!(field, key, val, "handling goblin field");

            match key {
                "H" => {
                    home_square = Self::parse_coord(val);
                    if home_square.is_none() {
                        warn!(val, "invalid Goblin home square");
                    }
                }
                "P" => {
                    kidnapped_piece = PieceType::symbol_to_piece(val);
                    if kidnapped_piece.is_none() {
                        warn!(val, "unknown kidnapped piece symbol");
                    }
                    // Reject king-symbols and carriers: a kidnapped
                    // king (direct or hidden inside a Bus / Loco /
                    // Carriage passenger list) would be invisible to
                    // `find_king` (Goblin doesn't expose its payload
                    // via `passengers()`, and find_king's descent is
                    // one level deep), so `is_in_check` and `status()`
                    // would silently lose the king. The spec also says
                    // the Goblin *converts* the kidnapped piece on
                    // home arrival — converting a king or a carrier
                    // is nonsensical.
                    if let Some(ref p) = kidnapped_piece {
                        if matches!(p, PieceType::King(_)) || p.can_carry_piece() {
                            warn!(
                                val,
                                "Goblin kidnap-payload cannot be a king or carrier; dropping payload"
                            );
                            kidnapped_piece = None;
                        }
                    }
                }
                _ => {
                    warn!(field, "unknown Goblin attribute");
                }
            }
        }

        // default to home square at a1 if not specified
        let home_sq = home_square.unwrap_or(Coord { file: 0, rank: 0 });

        let state = if let Some(p) = kidnapped_piece {
            GoblinState::Kidnapping { piece: p.into() }
        } else {
            GoblinState::Free
        };

        Some(PieceType::Goblin(Goblin {
            color,
            state,
            home_square: home_sq,
        }))
    }
}

impl Piece for Goblin {
    fn name(&self) -> &str {
        "Goblin"
    }

    fn color(&self) -> Color {
        self.color
    }
    fn set_color(&mut self, color: Color) {
        self.color = color;
    }

    fn initial_moves(&self, board: &Board, from: &Coord) -> Vec<GameMove> {
        trace!("goblin initial_moves");
        // Plan 09: Neutral non-train pieces yield no moves.
        if self.color == Color::Neutral {
            return Vec::new();
        }
        self.generate_goblin_base_moves(board, from.clone())
    }

    /// In Kidnapping state, the Goblin can only step onto *empty*
    /// adjacent squares (per `generate_goblin_kidnapping_moves`) and
    /// it cannot capture anything along the way. The default
    /// `Piece::attacks` would extract the empty-square destinations
    /// and report them as threats — which, combined with `is_attacked_by`'s
    /// passenger iteration, would phantom-block the opponent from
    /// castling through any empty square adjacent to a Neutral cart
    /// carrying a Kidnapping Goblin of the queryer's color. Empty
    /// the attack set entirely while Kidnapping to suppress the
    /// phantom.
    fn attacks(&self, board: &Board, from: &Coord) -> Vec<Coord> {
        match &self.state {
            GoblinState::Kidnapping { .. } => Vec::new(),
            GoblinState::Free => self
                .initial_moves(board, from)
                .into_iter()
                .filter_map(|m| match m.move_type {
                    crate::board::MoveType::MoveTo(c) => Some(c),
                    _ => None,
                })
                .collect(),
        }
    }

    /// A Kidnapping Goblin never captures — its only legitimate
    /// move-gen target is an *empty* square (drop-off on home). If
    /// king-safety ever asks "would this Goblin capture at target?",
    /// the answer is no while Kidnapping. Belt-and-suspenders with
    /// the empty `attacks()` override.
    fn would_capture_at(&self, _board: &Board, _from: &Coord, _target: &Coord) -> bool {
        matches!(self.state, GoblinState::Free)
    }

    fn symbol(&self) -> String {
        let prefix = match self.color {
            Color::White => "G",
            Color::Black => "g",
            Color::Neutral => "G",
        };

        match &self.state {
            GoblinState::Free => {
                // Free state only encodes home square
                format!(
                    "{}(H={}-{})",
                    prefix, self.home_square.file, self.home_square.rank
                )
            }

            GoblinState::Kidnapping { piece } => {
                // Include home square AND kidnapped piece symbol
                format!(
                    "{}(H={}-{},P={})",
                    prefix,
                    self.home_square.file,
                    self.home_square.rank,
                    piece.symbol()
                )
            }
        }
    }

    fn clone_box(&self) -> Box<dyn Piece> {
        Box::new(self.clone())
    }

    fn post_move_effects(
        &self,
        board_before: &Board,
        board_after: &mut Board,
        game_move: &GameMove,
    ) {
        // Goblin's mechanics only care about top-level relocation moves
        // (MoveTo). `apply_piece_post_effects` also dispatches the hook
        // for `MoveIntoCarrier` and `PieceInCarrier { inner: MoveTo(_) }`
        // — the boarding case is a no-op by design, but a *Kidnapping*
        // Goblin disembarking onto its home_square must still get to
        // drop off the kidnapped piece. So when the move is wrapped in
        // `PieceInCarrier { inner: MoveTo(target) }`, unwrap to the
        // inner target and continue.
        let to = match &game_move.move_type {
            MoveType::MoveTo(target) => target.clone(),
            MoveType::PieceInCarrier { move_type, .. } => match move_type.as_ref() {
                MoveType::MoveTo(target) => target.clone(),
                _ => return,
            },
            // MoveIntoCarrier / Promotion / EnPassant / Castle / etc.
            // — no Goblin mechanics apply.
            _ => return,
        };
        match &self.state {
            GoblinState::Free => {
                if let Some(square) = board_before.get_square_at(&to) {
                    if let Some(captured_piece) = &square.piece {
                        if captured_piece.get_color() != self.color {
                            // A kidnapped king (direct or hidden as a
                            // carrier-passenger) would be invisible
                            // to `find_king` (Goblin doesn't expose
                            // its payload via `passengers()`, and the
                            // descent is one level deep so even an
                            // exposed payload would miss kings inside
                            // a captured Bus). `is_in_check` and
                            // `status()` would silently lose the
                            // king. Reject both shapes:
                            //   1. captured_piece IS a king
                            //   2. captured_piece is a carrier (Bus /
                            //      Loco / Carriage) whose passengers
                            //      *could* include a king
                            // For case 2 we conservatively reject any
                            // carrier rather than walking passengers —
                            // kidnapping a Bus/cart is also nonsensical
                            // per the spec (the convert-at-home rule
                            // doesn't define what a converted Bus
                            // would be).
                            if matches!(captured_piece, PieceType::King(_))
                                || captured_piece.can_carry_piece()
                            {
                                trace!(
                                    "Goblin captured a king or carrier; leaving the kidnap payload empty (capture still removes the victim)"
                                );
                                return;
                            }
                            // initiate kidnapping — preserve the goblin's
                            // existing home_square; only the state changes.
                            if let Some(goblin_square) = board_after.get_square_mut(&to) {
                                if let Some(piece) = &mut goblin_square.piece {
                                    if let Some(goblin) =
                                        piece.as_any_mut().downcast_mut::<Goblin>()
                                    {
                                        goblin.state = GoblinState::Kidnapping {
                                            piece: captured_piece.clone().into(),
                                        };
                                    }
                                }
                            }
                        }
                    }
                }
            }
            GoblinState::Kidnapping { piece } => {
                let mut p: PieceType = piece.clone().into();
                p.set_color(self.color);
                if to == self.home_square {
                    // drop off the converted piece at home (the goblin is
                    // overwritten on the same square — "goblin dies").
                    board_after.set_piece_at(&to, p);
                }
            }
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
