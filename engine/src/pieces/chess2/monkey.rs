/// Let's add The Monkey
/// Has a range of 1 (moves like a king), can't take
/// If there's a piece directly next to it, it can jump over
/// If there's a piece next to the new location, it can jump over that, as well
/// If there's a piece where the monkey ends up, it takes
///
/// Example moves (M = Monkey, P = generic piece, X = possible move, . = empty):
/// X M X . . . . .
/// X P X . . . . .
/// . X P X . . . .
/// . . . . . . . .
///
/// This should be done recursively, to allow for multiple jumps in a single move
use crate::{
    board::{Board, Coord, GameMove, MoveType},
    pieces::{Color, Piece},
};

#[derive(Clone, PartialEq, Debug)]
pub struct Monkey {
    pub color: Color,
}
impl Piece for Monkey {
    fn name(&self) -> &str {
        "Monkey"
    }
    fn color(&self) -> Color {
        self.color
    }
    fn set_color(&mut self, color: Color) {
        self.color = color;
    }
    fn initial_moves(&self, board: &Board, from: &Coord) -> Vec<GameMove> {
        // Plan 09: Neutral non-train pieces yield no moves. The
        // Monkey's opposite-color jump-capture rule would otherwise
        // collapse to "captures other Neutral pieces" since
        // `Color::Neutral.opposite() == Color::Neutral`.
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

        // first, handle normal one-square moves
        for (df, dr) in &directions {
            let new_file = from.file as isize + df;
            let new_rank = from.rank as isize + dr;
            if board.in_bounds(new_file, new_rank) {
                let coord = Coord {
                    file: new_file as u8,
                    rank: new_rank as u8,
                };
                // Empty walkable square: ordinary king-step.
                // Neutral cart on a walkable square: emit MoveTo so the
                // filter in `piecetype.rs` rewrites to `MoveIntoCarrier`
                // (plan-09 cart-boarding). Without this, Monkey alone
                // among pieces would fail to board a cart it stepped
                // onto.
                let target_occupant = board.get_square_at(&coord).and_then(|s| s.piece.as_ref());
                let is_neutral_carrier = matches!(target_occupant, Some(p) if p.get_color() == Color::Neutral && p.can_carry_piece());
                if board.square_is_empty(&coord) || is_neutral_carrier {
                    let game_move = GameMove {
                        from: from.clone(),
                        move_type: MoveType::MoveTo(coord.clone()),
                    };
                    moves.push(game_move);
                }
            }
        }
        // now, handle jump moves
        let mut visited = Vec::new();
        self.find_jump_moves(board, from, from, &mut visited, &mut moves);
        moves
    }
    fn symbol(&self) -> String {
        match self.color {
            Color::White => 'M'.to_string(),
            Color::Black => 'm'.to_string(),
            Color::Neutral => 'M'.to_string(),
        }
    }

    fn clone_box(&self) -> Box<dyn Piece> {
        Box::new(self.clone())
    }

    /// Spec: Monkey moves like a king (1-square) but **cannot capture by
    /// single-step**. It captures only by landing on a piece after a jump.
    /// The default `Piece::attacks` (extract MoveTo from `initial_moves`)
    /// over-includes empty single-step squares as threats, which would
    /// wrongly mark a king's escape squares as attacked. This override
    /// returns *only* jump-landings — the squares Monkey could actually
    /// capture on — including landings occupied by friendlies (those count
    /// for the "if the king were here instead, would Monkey capture?"
    /// semantics king-safety needs).
    fn attacks(&self, board: &Board, from: &Coord) -> Vec<Coord> {
        // Plan 09 S1: Neutral non-train piece threatens nothing. The
        // S1 guard in `initial_moves` doesn't flow here because this
        // override builds its own attack list via jump-detection.
        if self.color == Color::Neutral {
            return Vec::new();
        }
        let mut out = Vec::new();
        let mut visited = Vec::new();
        self.collect_jump_threats(board, from, &mut visited, &mut out);
        out
    }

    /// Monkey captures the piece on its jump-landing. The default
    /// `would_capture_at` returns true for any target, which is right
    /// for non-cart landings (jump-onto-enemy = capture, jump-onto-
    /// friendly = "if a king were here we'd take it" for king-safety).
    /// Neutral carts are the exception: Monkey *boards* a cart, and
    /// the cart's `passengers.retain(|p| p.get_color() == boarder_color)`
    /// rule (make_move.rs, Plan 09 Q7 pinned current behavior) culls
    /// only opposite-color passengers. So Monkey "captures at" a
    /// Neutral-cart tile iff that cart holds at least one opposite-
    /// color passenger; otherwise the tile is benign and king-safety
    /// must not flag it.
    fn would_capture_at(&self, board: &Board, _from: &Coord, target: &Coord) -> bool {
        let Some(sq) = board.get_square_at(target) else {
            return true;
        };
        let Some(piece) = sq.piece.as_ref() else {
            return true;
        };
        if piece.get_color() == Color::Neutral && piece.can_carry_piece() {
            return piece
                .passengers()
                .map(|ps| ps.iter().any(|p| p.get_color() != self.color))
                .unwrap_or(false);
        }
        true
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl Monkey {
    /// Recursive function to find jump moves.
    /// `origin` is the Monkey's actual starting square — the `from` field every
    /// emitted move should carry, no matter how deep the chain goes.
    /// `current_coord` is the current position of the Monkey during the jump sequence.
    /// `visited` keeps track of coordinates already jumped to in this sequence.
    /// `moves` accumulates the valid jump moves.
    fn find_jump_moves(
        &self,
        board: &Board,
        origin: &Coord,
        current_coord: &Coord,
        visited: &mut Vec<Coord>,
        moves: &mut Vec<GameMove>,
    ) {
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
            let adj_file = current_coord.file as isize + df;
            let adj_rank = current_coord.rank as isize + dr;
            let jump_file = adj_file + df;
            let jump_rank = adj_rank + dr;

            if !board.in_bounds(adj_file, adj_rank) || !board.in_bounds(jump_file, jump_rank) {
                continue;
            }
            let adj_coord = Coord {
                file: adj_file as u8,
                rank: adj_rank as u8,
            };
            let jump_coord = Coord {
                file: jump_file as u8,
                rank: jump_rank as u8,
            };

            // Avoid revisiting a square within the current jump chain — this
            // prevents cycles but `visited` must be restored after recursion
            // so a sibling branch can still reach the same square via a
            // different path.
            if visited.contains(&jump_coord) {
                continue;
            }

            let Some(adj_square) = board.get_square_at(&adj_coord) else {
                continue;
            };
            if adj_square.piece.is_none() {
                continue;
            }
            let Some(jump_square) = board.get_square_at(&jump_coord) else {
                continue;
            };
            // Plan 08: the Monkey can't land on non-walkable terrain (closed
            // Gate / Turret / Vent) — neither as a jump landing nor as a
            // capture target.
            if !jump_square.square_type.is_walkable() {
                continue;
            }

            if jump_square.piece.is_none() {
                moves.push(GameMove {
                    from: origin.clone(),
                    move_type: MoveType::MoveTo(jump_coord.clone()),
                });
                visited.push(jump_coord.clone());
                self.find_jump_moves(board, origin, &jump_coord, visited, moves);
                visited.pop();
            } else if jump_square.has_piece_of_color(self.color.opposite()) {
                moves.push(GameMove {
                    from: origin.clone(),
                    move_type: MoveType::MoveTo(jump_coord.clone()),
                });
                // Capture ends the chain — no recursion, no need to record visited.
            } else if matches!(
                jump_square.piece.as_ref(),
                Some(p) if p.get_color() == Color::Neutral && p.can_carry_piece()
            ) {
                // Jump-landing on a Neutral cart: emit MoveTo so the
                // piecetype.rs filter rewrites to MoveIntoCarrier. The
                // cart itself survives (cart-invincibility), but any
                // opposite-colour passengers are captured by the
                // boarding (see `passengers.retain` in make_move.rs
                // and Plan 09 Q7's pinned current behavior).
                // King-safety queries this scenario through
                // `Monkey::would_capture_at`, which inspects the
                // cart's passenger list. Boarding ends the chain (the
                // Monkey is now inside the cart, not free to continue
                // jumping).
                moves.push(GameMove {
                    from: origin.clone(),
                    move_type: MoveType::MoveTo(jump_coord.clone()),
                });
            }
        }
    }

    /// Sibling of `find_jump_moves` for attack-detection. Mirrors the same
    /// geometry but emits a target whenever a ladder (adjacent piece) and an
    /// in-bounds landing exist — regardless of who's on the landing today.
    /// This is what `is_attacked_by` needs to answer "if the king were on
    /// the landing square, would the Monkey capture it?"
    fn collect_jump_threats(
        &self,
        board: &Board,
        current_coord: &Coord,
        visited: &mut Vec<Coord>,
        out: &mut Vec<Coord>,
    ) {
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
            let adj_file = current_coord.file as isize + df;
            let adj_rank = current_coord.rank as isize + dr;
            let jump_file = adj_file + df;
            let jump_rank = adj_rank + dr;

            if !board.in_bounds(adj_file, adj_rank) || !board.in_bounds(jump_file, jump_rank) {
                continue;
            }
            let adj_coord = Coord {
                file: adj_file as u8,
                rank: adj_rank as u8,
            };
            let jump_coord = Coord {
                file: jump_file as u8,
                rank: jump_rank as u8,
            };
            if visited.contains(&jump_coord) {
                continue;
            }
            // Need a ladder — an adjacent piece to jump over.
            let Some(adj_square) = board.get_square_at(&adj_coord) else {
                continue;
            };
            if adj_square.piece.is_none() {
                continue;
            }

            // The landing square is threatened. Whether Monkey would
            // *actually* capture there is filtered by `would_capture_at`
            // in `is_attacked_by` — that's the central phantom-attack
            // filter for cases like Neutral-cart landings where Monkey
            // boards rather than captures. Recurse only if the landing
            // is empty (chain continues); a captured-ladder landing
            // ends the chain in `find_jump_moves`, mirror that here.
            out.push(jump_coord.clone());
            let Some(jump_square) = board.get_square_at(&jump_coord) else {
                continue;
            };
            if jump_square.piece.is_none() {
                visited.push(jump_coord.clone());
                self.collect_jump_threats(board, &jump_coord, visited, out);
                visited.pop();
            }
        }
    }
}
