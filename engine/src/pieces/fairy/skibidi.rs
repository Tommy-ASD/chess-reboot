/// Skibidi
///     Stuns all pieces (enemy and ally) in a given circle radius. They can no longer move due to extreme brainrot.
///     No effect on others (phase 1) -> the 4 non-diagonal neighboring cells (circle of radius with 1, phase 2) ->
///         a circle with a radius of 2 (phase 3) -> a circle with a radius of 3 (phase 4)
///     This effect is removed once the Skibidi is captured, or another neutralizing Skibidi enters the Brainrot radius.
///     Any Skibidi (enemy or ally) entering the Brainrot radius acts as a neutralizing Skibidi.
///     After being neutralized, the Skibidi is set back to phase 1.
///     The Skibidi can move, but it is set back to phase 1 each time it moves.
///     Increasing the radius of brainrot uses a move.
///     If there is no opposing Skibidi, the maximum phase your Skibidi can reach is 3.
///     It moves like a king (to any directly neighboring cells), but cannot take other pieces.
///     It can take other Skibidis
///     If your Skibidi your enemy cannot make a move due to your Brainrot,
///         you win by Brainrot instead of stalemate being declared.
///     If your Skibidi is captured while your opponent's Skibidi is in phase 4, there is nothing you can do.
use tracing::{trace, warn};

use crate::{
    board::{Board, Coord, GameMove, MoveType},
    pieces::{Color, Piece, piecetype::PieceType},
};

#[derive(Clone, PartialEq, Debug, serde::Serialize, serde::Deserialize)]
pub struct Skibidi {
    pub color: Color,
    pub phase: u8, // 1 to 4
}

impl Skibidi {
    pub fn new(color: Color) -> Self {
        Skibidi { color, phase: 1 }
    }

    // the skibidi has state
    // so we need custom symbol
    // the state only contains phase
    // we can keep it simple and forget the state in phase 1
    pub fn from_symbol(symbol: &str) -> Option<PieceType> {
        trace!(symbol, "parsing Skibidi");

        // Defensive: callers should only invoke `from_symbol` via the
        // dispatcher in `piecetype.rs`, which guarantees a non-empty
        // prefix-before-`(`. But the parsing surface is the engine's
        // last panic-on-malformed-FEN if invoked directly; bail
        // cleanly instead.
        let first = symbol.chars().next()?;
        let color = if first.is_lowercase() {
            Color::Black
        } else {
            Color::White
        };

        // if no custom state, skibidi phase 1
        let Some(start) = symbol.find('(') else {
            return Some(PieceType::Skibidi(Skibidi { color, phase: 1 }));
        };

        let end = symbol.find(')')?;
        let inside = &symbol[start + 1..end];

        let mut phase = 1;

        for field in inside.split(',') {
            let mut kv = field.splitn(2, '=');
            let key = kv.next()?.trim();
            let val = kv.next()?.trim();

            match key {
                "PHASE" => {
                    trace!(val, "parsed Skibidi phase");
                    // Skibidi spec: phase ∈ 1..=4. Round-3 audit
                    // surfaced that the parser previously accepted
                    // any u8 — `phase_to_radius` falls through to 0
                    // for out-of-range, but a future per-phase
                    // ability lookup would otherwise read garbage.
                    // Clamp at parse to enforce the invariant.
                    match val.parse::<u8>() {
                        Ok(ok) if (1..=4).contains(&ok) => phase = ok,
                        Ok(out) => warn!(
                            out,
                            "Skibidi phase out of range 1..=4; defaulting to 1"
                        ),
                        Err(_) => warn!(val, "invalid phase for Skibidi"),
                    };
                }
                _ => {
                    warn!(field, "unknown Skibidi attribute");
                }
            }
        }

        Some(PieceType::Skibidi(Skibidi { color, phase }))
    }
}

impl Piece for Skibidi {
    fn name(&self) -> &str {
        "Skibidi"
    }
    fn color(&self) -> Color {
        self.color
    }
    fn set_color(&mut self, color: Color) {
        self.color = color;
    }
    fn initial_moves(&self, board: &Board, from: &Coord) -> Vec<GameMove> {
        // Plan 09: Neutral non-train pieces yield no moves.
        if self.color == Color::Neutral {
            return Vec::new();
        }
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
            if !board.in_bounds(new_file, new_rank) {
                continue;
            }
            let coord = Coord {
                file: new_file as u8,
                rank: new_rank as u8,
            };
            let Some(square) = board.get_square_at(&coord) else {
                continue;
            };
            // Plan 08: non-walkable destinations (closed Gate / Turret / Vent)
            // are off-limits regardless of what's on them.
            if !square.square_type.is_walkable() {
                continue;
            }
            // Spec: cannot take other pieces, but can take other Skibidis.
            // Empty squares are allowed; non-empty squares only if the
            // occupant is an enemy Skibidi. The `PieceType::get_moves`
            // filter handles the same-colour rejection.
            //
            // Neutral carriers (train carts) are also allowed — moving
            // onto a cart boards it (the global filter rewrites MoveTo
            // → MoveIntoCarrier). Boarding ≠ capture, so it doesn't
            // violate the "cannot take other pieces" spec. Without
            // this arm, Skibidi alone among non-train pieces would
            // fail to board a cart (mirror of the round-8 Monkey fix).
            let allowed = match &square.piece {
                None => true,
                Some(PieceType::Skibidi(_)) => true,
                Some(p) if p.get_color() == Color::Neutral && p.can_carry_piece() => true,
                _ => false,
            };
            if allowed {
                moves.push(GameMove {
                    from: from.clone(),
                    move_type: MoveType::MoveTo(coord),
                });
            }
        }

        moves.push(GameMove {
            from: from.clone(),
            move_type: MoveType::PhaseShift,
        });

        moves
    }
    fn symbol(&self) -> String {
        let mut sym = match self.color {
            Color::White => 'S'.to_string(),
            Color::Black => 's'.to_string(),
            Color::Neutral => 'S'.to_string(),
        };
        if self.phase > 1 {
            sym.push_str(&format!("(PHASE={phase})", phase = self.phase));
        }

        sym
    }
    fn clone_box(&self) -> Box<dyn Piece> {
        Box::new(self.clone())
    }
    /// Per spec a Skibidi can only capture other Skibidis directly.
    /// For king-safety the only "capture" Skibidi can do that touches
    /// a king is *boarding a Neutral cart* that carries an opposite-
    /// colour king as a passenger — `passengers.retain` in make_move
    /// then culls the king (Plan 09 Q7 pinned current behaviour).
    /// So `attacks()` reports any adjacent Neutral-carrier tile whose
    /// passenger list contains an opposite-colour piece; everything
    /// else is benign for king-safety.
    ///
    /// Without this, a White Skibidi adjacent to a Neutral cart
    /// carrying a Black king reads as safe but actually captures the
    /// king on its next move (round-10 audit finding).
    fn attacks(&self, board: &Board, from: &Coord) -> Vec<Coord> {
        if self.color == Color::Neutral {
            return Vec::new();
        }
        let mut out = Vec::new();
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
            let nf = from.file as isize + df;
            let nr = from.rank as isize + dr;
            if !board.in_bounds(nf, nr) {
                continue;
            }
            let coord = Coord {
                file: nf as u8,
                rank: nr as u8,
            };
            let Some(sq) = board.get_square_at(&coord) else {
                continue;
            };
            if !sq.square_type.is_walkable() {
                continue;
            }
            let Some(piece) = sq.piece.as_ref() else {
                continue;
            };
            // Only Neutral carriers carrying an opposite-colour
            // passenger are threats. Non-carrier neighbours, same-
            // colour passengers, and empty carts are all benign for
            // king-safety.
            if piece.get_color() == Color::Neutral
                && piece.can_carry_piece()
                && piece
                    .passengers()
                    .map(|ps| ps.iter().any(|p| p.get_color() != self.color))
                    .unwrap_or(false)
            {
                out.push(coord);
            }
        }
        out
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn post_move_effects(
        &self,
        _board_before: &Board,
        board_after: &mut Board,
        game_move: &GameMove,
    ) {
        match &game_move.move_type {
            MoveType::MoveTo(target) => {
                // Per spec, any MoveTo resets the Skibidi to phase 1.
                // Build a fresh instance with the reset and write it
                // back — see `Piece::post_move_effects` doc for the
                // "use board_after, not &mut self" convention.
                let mut reset = self.clone();
                reset.phase = 1;
                board_after.set_piece_at(target, PieceType::Skibidi(reset));
            }
            MoveType::MoveIntoCarrier(target) => {
                // Spec: "set back to phase 1 each time it moves" —
                // boarding a carrier is a move. The just-boarded
                // Skibidi sits somewhere in the carrier's passenger
                // list (today `relocate_pieces` push-appends, so it's
                // the tail; relying on that ordering is brittle).
                // Identify our entry by matching color + current phase
                // and reset *that* one to phase 1.
                if let Some(sq) = board_after.get_square_mut(target) {
                    if let Some(carrier) = sq.piece.as_mut() {
                        if let Some(passengers) = carrier.passengers_mut() {
                            // Walk back-to-front so the most recently
                            // appended Skibidi matching our color+phase
                            // wins ties — that's the one this hook is
                            // firing for, even if other Skibidis of the
                            // same color happen to share `self.phase`.
                            for p in passengers.iter_mut().rev() {
                                if let PieceType::Skibidi(s) = p {
                                    if s.color == self.color && s.phase == self.phase {
                                        s.phase = 1;
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
            }
            MoveType::PieceInCarrier { move_type, .. } => {
                // Passenger-relocation counts as a move too.
                match move_type.as_ref() {
                    MoveType::MoveTo(target) => {
                        // Exit onto board — the landed Skibidi is a
                        // top-level piece at the inner target.
                        let mut reset = self.clone();
                        reset.phase = 1;
                        board_after.set_piece_at(target, PieceType::Skibidi(reset));
                    }
                    MoveType::MoveIntoCarrier(target) => {
                        // Cross-cart hop — Skibidi is now a passenger
                        // of the destination carrier. Mirror the
                        // top-level MIC arm: walk passengers back-to-
                        // front looking for a Skibidi matching our
                        // color+current-phase and reset it.
                        if let Some(sq) = board_after.get_square_mut(target) {
                            if let Some(carrier) = sq.piece.as_mut() {
                                if let Some(passengers) = carrier.passengers_mut() {
                                    for p in passengers.iter_mut().rev() {
                                        if let PieceType::Skibidi(s) = p {
                                            if s.color == self.color
                                                && s.phase == self.phase
                                            {
                                                s.phase = 1;
                                                break;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
            // PhaseShift handled by make_move directly;
            // Promotion/Castle/EnPassant are pawn/king-only — none apply to
            // a Skibidi. Explicit no-ops keep this exhaustive without
            // masking future variants.
            MoveType::PhaseShift
            | MoveType::Promotion { .. }
            | MoveType::Castle { .. }
            | MoveType::EnPassant { .. }
            | MoveType::ThrowSwitch { .. }
            // A Skibidi never makes a PlaceTornado move (Stormcaller-
            // only) — no-op here.
            | MoveType::PlaceTornado { .. } => {}
        }
    }
}
