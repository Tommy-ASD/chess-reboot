/// The Bus
/// Carrier piece
///
/// The Bus is a versatile carrier piece that can transport up to five allied pieces across the board.
/// This allows you to reposition your forces quickly and strategically.
/// However, if the Bus is captured, all pieces inside are lost as well.
/// The Bus moves like a standard rook - horizontally or vertically until there is a piece blocking the path.
/// The Bus cannot take pieces.
/// To exit the Bus, a piece simply moves out of the square the Bus occupies, following its usual movement rules.
use tracing::{trace, warn};

use crate::{
    board::{
        Board, Coord, GameMove, MoveType,
        fen::{find_matching_paren, split_top_level},
    },
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
        trace!(symbol, "parsing Bus");

        // Engine convention: all-uppercase prefix = White, all-lowercase
        // = Black. Mixed case ("Bus(...)", "bUS(...)") is invalid — the
        // top-level dispatcher (`PieceType::symbol_to_piece`) lowercases
        // for keyword match, so reading the *raw* first char alone would
        // silently assign White to any prefix starting with `B`. Anchor
        // on the full prefix-before-`(` to avoid the silent conversion.
        let prefix = symbol.split('(').next().unwrap_or("");
        let color = match prefix {
            "BUS" => Color::White,
            "bus" => Color::Black,
            _ => {
                warn!(prefix, "Bus prefix must be 'BUS' or 'bus' exactly");
                return None;
            }
        };

        let Some(start) = symbol.find('(') else {
            trace!("Bus with no bracketed contents");
            return Some(PieceType::Bus(Bus {
                color,
                pieces: vec![],
            }));
        };

        let end = find_matching_paren(symbol, start)?;
        let inside = &symbol[start + 1..end];

        let mut pieces = vec![];

        for field in split_top_level(inside) {
            let mut kv = field.splitn(2, '=');
            // Tolerate stray empty segments (`,,`) and bare keys (`X`) by
            // dropping that field with a warn — a single malformed segment
            // must NOT abort the whole Bus parse and silently delete every
            // already-parsed field.
            let key = kv.next().unwrap_or("").trim();
            let val = kv.next().unwrap_or("").trim();
            if key.is_empty() {
                warn!(field, "empty Bus field; skipping");
                continue;
            }

            trace!(field, key, val, "handling bus field");

            match key {
                // P=(piece, piece, ...) — array of passenger symbols
                "P" => {
                    let Some(inner) = val.strip_prefix('(').and_then(|s| s.strip_suffix(')'))
                    else {
                        warn!(val, "malformed Bus P=... field; expected (...)");
                        continue;
                    };
                    for piece_sym in split_top_level(inner) {
                        let opt_inner_piece = PieceType::symbol_to_piece(&piece_sym);
                        trace!(piece_sym, ?opt_inner_piece, "parsed inner piece");
                        if let Some(inner_piece) = opt_inner_piece {
                            // Reject nested carriers: the carrier
                            // filter (piecetype.rs) refuses to
                            // *produce* a move that boards one
                            // carrier into another, so accepting it
                            // from a hand-rolled FEN would leave the
                            // engine in a state it can't otherwise
                            // reach. Drop the passenger with a warn.
                            if inner_piece.can_carry_piece() {
                                warn!(
                                    piece_sym,
                                    "rejecting nested carrier passenger in Bus"
                                );
                                continue;
                            }
                            // Bus invariant: capacity-5. The boarding
                            // filter enforces this for legal play, but
                            // a hand-rolled FEN can describe a Bus with
                            // more than 5 passengers. Drop overflow
                            // with a warn so the over-cap state never
                            // enters the engine — future pieces that
                            // split or duplicate Buses can then trust
                            // `bus.pieces.len() <= 5` as a hard
                            // invariant.
                            if pieces.len() >= 5 {
                                warn!(
                                    piece_sym,
                                    "Bus over capacity-5 in FEN; dropping overflow passenger"
                                );
                                continue;
                            }
                            // Bus invariant: passengers share the Bus's
                            // colour. The boarding filter enforces this
                            // in legal play, so a mismatched-colour
                            // passenger can only land here from a
                            // hand-rolled FEN. `Bus::attacks` filters
                            // mismatched colours out (round-11 fix); we
                            // surface the malformed input here so it's
                            // loud instead of silent.
                            let passenger_color = inner_piece.get_color();
                            if passenger_color != color {
                                warn!(
                                    piece_sym,
                                    ?color,
                                    ?passenger_color,
                                    "Bus passenger colour mismatches Bus colour; \
                                     attacks will filter it out but the FEN is malformed"
                                );
                            }
                            pieces.push(inner_piece);
                        }
                    }
                }
                _ => {
                    warn!(field, "unknown Bus attribute");
                }
            }
        }

        Some(PieceType::Bus(Bus { color, pieces }))
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
    fn can_carry_piece(&self) -> bool {
        true
    }
    fn passengers(&self) -> Option<&[PieceType]> {
        Some(&self.pieces)
    }
    fn passengers_mut(&mut self) -> Option<&mut Vec<PieceType>> {
        Some(&mut self.pieces)
    }
    fn initial_moves(&self, board: &Board, from: &Coord) -> Vec<GameMove> {
        trace!("bus initial_moves");
        // Plan 09: Neutral non-train pieces yield no moves.
        if self.color == Color::Neutral {
            return Vec::new();
        }
        let mut moves = Vec::new();
        // Bus moves like a rook: orthogonal sliding, no captures. The filter
        // in `PieceType::get_moves` rewrites landings on a friendly carrier to
        // `MoveIntoCarrier`. We stop at the first blocking piece — friendly or
        // enemy — and only emit the blocker's square if it's a friendly Bus
        // (so the filter can swap it).
        let directions: [(isize, isize); 4] = [(1, 0), (-1, 0), (0, 1), (0, -1)];

        for (df, dr) in &directions {
            let mut step: isize = 1;
            loop {
                let new_file = from.file as isize + df * step;
                let new_rank = from.rank as isize + dr * step;
                if !board.in_bounds(new_file, new_rank) {
                    break;
                }
                let coord = Coord {
                    file: new_file as u8,
                    rank: new_rank as u8,
                };
                match board.get_square_at(&coord) {
                    // Plan 08: non-walkable terrain blocks the Bus's slide
                    // the same way a piece does — and we don't emit a move
                    // onto it.
                    Some(sq) if !sq.square_type.is_walkable() => break,
                    Some(sq) if sq.piece.is_none() => {
                        moves.push(GameMove {
                            from: from.clone(),
                            move_type: MoveType::MoveTo(coord),
                        });
                    }
                    Some(sq) => {
                        // Blocker. Emit only if it's a friendly carrier; the
                        // filter will rewrite this MoveTo to MoveIntoCarrier.
                        if let Some(piece) = &sq.piece {
                            if piece.get_color() == self.color && piece.can_carry_piece() {
                                moves.push(GameMove {
                                    from: from.clone(),
                                    move_type: MoveType::MoveTo(coord),
                                });
                            }
                        }
                        break;
                    }
                    None => break,
                }
                step += 1;
            }
        }

        for (idx, piece) in self.pieces.iter().enumerate() {
            let mut board_clone = board.clone();
            board_clone.set_piece_at(from, piece.clone());
            let inner_piece_moves = board_clone.get_moves(from);
            for game_move in inner_piece_moves {
                // Whitelist inner move types that `relocate_pieces`' PIC
                // arm actually handles: only `MoveTo` and
                // `MoveIntoCarrier`. Anything else (Promotion, Castle,
                // EnPassant, ThrowSwitch, PhaseShift, nested
                // PieceInCarrier) would pass `get_moves` and then fail
                // at apply time. Mirrors the locomotive filter.
                if !matches!(
                    game_move.move_type,
                    MoveType::MoveTo(_) | MoveType::MoveIntoCarrier(_)
                ) {
                    continue;
                }
                moves.push(GameMove {
                    from: from.clone(),
                    move_type: MoveType::PieceInCarrier {
                        piece_index: idx as u8,
                        move_type: game_move.move_type.into(),
                    },
                });
            }
        }

        moves
    }
    fn symbol(&self) -> String {
        let mut sym = match self.color {
            Color::White => "BUS".to_string(),
            Color::Black => "bus".to_string(),
            Color::Neutral => "BUS".to_string(),
        };

        if !self.pieces.is_empty() {
            let pieces_map = self
                .pieces
                .iter()
                .map(|piece| piece.symbol())
                .collect::<Vec<String>>()
                .join(",");
            trace!(pieces_map, "bus symbol pieces");
            sym.push_str("(P=(");
            sym.push_str(&pieces_map);
            sym.push_str("))");
        }

        sym
    }
    fn clone_box(&self) -> Box<dyn Piece> {
        Box::new(self.clone())
    }
    /// The Bus itself cannot capture (spec), so a Bus on `from` contributes
    /// nothing. Its passengers, however, can exit the Bus straight onto an
    /// enemy square — those threats *do* matter for king safety. We compute
    /// each passenger's attacks as if it were standing on the Bus's square.
    fn attacks(&self, board: &Board, from: &Coord) -> Vec<Coord> {
        // Plan 09 S1: a Neutral Bus is a degenerate state (Bus is
        // always White or Black). Skip passenger attacks to be
        // consistent with the other S1 guards.
        if self.color == Color::Neutral {
            return Vec::new();
        }
        // Filter passengers by colour. Bus's invariant is "same-colour
        // passengers only" (the boarding filter at piecetype.rs and
        // the round-3 capture-on-board rules together enforce it for
        // legal moves), but a hand-rolled FEN can produce a Bus with
        // mismatched-colour passengers. `is_attacked_by`'s rule says
        // "non-Neutral carriers must return colour-correct attacks";
        // a wrong-colour passenger inside a coloured Bus would
        // otherwise leak phantom threats for the wrong side. Filter
        // here so the contract holds regardless of FEN provenance.
        let mut out = Vec::new();
        for passenger in &self.pieces {
            if passenger.get_color() != self.color {
                continue;
            }
            out.extend(passenger.attacks(board, from));
        }
        out
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
    // post_move_effects: default no-op from the trait
}
