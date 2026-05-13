use tracing::debug;

use crate::{
    board::{Board, CastleSide, Coord, GameMove, MoveError, MoveType, PromotionTarget},
    pieces::{Color, piecetype::PieceType},
};

impl Board {
    /// Attempts to execute a move on the board.
    /// Returns Ok(()) if successful, Err(MoveError) if illegal. The error
    /// is structured (see `MoveError`) — match on the discriminant to
    /// surface a useful message in your UI.
    pub fn make_move(&mut self, game_move: GameMove) -> Result<(), MoveError> {
        self.validate_move(&game_move)?;
        let from = game_move.from.clone();
        let attempted = game_move.move_type.clone();
        self.make_move_unchecked(game_move).map_err(|reason| MoveError::ApplyFailed {
            from,
            attempted,
            reason,
        })
    }

    /// Apply a move without re-running legality checks. The caller (e.g.
    /// `legal_moves`) is responsible for having generated the move from
    /// `get_moves` first. Internal invariants (source square exists, piece
    /// present, target shape matches variant) are still verified — they
    /// become `Err` rather than `panic!`.
    pub fn make_move_unchecked(&mut self, game_move: GameMove) -> Result<(), String> {
        let board_before = self.clone();

        let from = &game_move.from;

        let piece = {
            let square = self
                .get_square_at(from)
                .ok_or_else(|| format!("No square at {:?}", from))?;

            square
                .piece
                .clone()
                .ok_or_else(|| format!("No piece at {:?}", from))?
        };

        match &game_move.move_type {
            MoveType::MoveTo(target) => {
                // Capture-side effects (clear castle right on rook capture).
                if let Some(captured) =
                    self.get_square_at(target).and_then(|s| s.piece.clone())
                {
                    self.maybe_clear_castle_on_rook_capture(target, &captured);
                }
                {
                    let from_sq = self
                        .get_square_mut(from)
                        .ok_or_else(|| format!("No square at {:?}", from))?;
                    from_sq.piece = None;
                }
                {
                    let to_sq = self
                        .get_square_mut(target)
                        .ok_or_else(|| format!("No square at {:?}", target))?;
                    to_sq.piece = Some(piece);
                }

                debug!(?from, ?target, "move executed");
            }
            MoveType::Promotion { target, into } => {
                let pawn_color = match &piece {
                    PieceType::Pawn(p) => p.color,
                    other => {
                        return Err(format!(
                            "Promotion: source piece is not a pawn (got {:?})",
                            other
                        ));
                    }
                };

                if let Some(captured) =
                    self.get_square_at(target).and_then(|s| s.piece.clone())
                {
                    self.maybe_clear_castle_on_rook_capture(target, &captured);
                }

                let new_piece = match into {
                    PromotionTarget::Queen => PieceType::new_queen(pawn_color),
                    PromotionTarget::Rook => PieceType::new_rook(pawn_color),
                    PromotionTarget::Bishop => PieceType::new_bishop(pawn_color),
                    PromotionTarget::Knight => PieceType::new_knight(pawn_color),
                };

                {
                    let from_sq = self
                        .get_square_mut(from)
                        .ok_or_else(|| format!("No square at {:?}", from))?;
                    from_sq.piece = None;
                }
                {
                    let to_sq = self
                        .get_square_mut(target)
                        .ok_or_else(|| format!("No square at {:?}", target))?;
                    to_sq.piece = Some(new_piece);
                }

                debug!(?from, ?target, ?into, "promotion executed");
            }
            MoveType::Castle { side } => {
                let back_rank = from.rank;
                // Kingside rook lives on the right-edge file (`width - 1`),
                // not necessarily file 7 — keep this in sync with
                // `king.rs::castle_moves`.
                let right_edge = self.width().saturating_sub(1);
                let (king_target_file, rook_source_file, rook_target_file) = match side {
                    CastleSide::Kingside => (6u8, right_edge, 5u8),
                    CastleSide::Queenside => (2u8, 0u8, 3u8),
                };
                let king_target = Coord {
                    file: king_target_file,
                    rank: back_rank,
                };
                let rook_source = Coord {
                    file: rook_source_file,
                    rank: back_rank,
                };
                let rook_target = Coord {
                    file: rook_target_file,
                    rank: back_rank,
                };

                let rook = self
                    .get_square_at(&rook_source)
                    .and_then(|s| s.piece.clone())
                    .ok_or_else(|| format!("Castle: no piece at {:?}", rook_source))?;
                if !matches!(rook, PieceType::Rook(_)) {
                    return Err(format!(
                        "Castle: piece at {:?} is not a rook (got {:?})",
                        rook_source, rook
                    ));
                }

                self.get_square_mut(from)
                    .ok_or_else(|| format!("Castle: no square at {:?}", from))?
                    .piece = None;
                self.get_square_mut(&rook_source)
                    .ok_or_else(|| format!("Castle: no square at {:?}", rook_source))?
                    .piece = None;
                self.get_square_mut(&king_target)
                    .ok_or_else(|| format!("Castle: no square at {:?}", king_target))?
                    .piece = Some(piece);
                self.get_square_mut(&rook_target)
                    .ok_or_else(|| format!("Castle: no square at {:?}", rook_target))?
                    .piece = Some(rook);

                debug!(?from, ?king_target, ?side, "castle executed");
            }
            MoveType::EnPassant { target, captured } => {
                self.get_square_mut(from)
                    .ok_or_else(|| format!("EnPassant: no square at {:?}", from))?
                    .piece = None;
                self.get_square_mut(captured)
                    .ok_or_else(|| format!("EnPassant: no square at {:?}", captured))?
                    .piece = None;
                self.get_square_mut(target)
                    .ok_or_else(|| format!("EnPassant: no square at {:?}", target))?
                    .piece = Some(piece);

                debug!(?from, ?target, ?captured, "en passant executed");
            }
            MoveType::PhaseShift => match piece {
                PieceType::Skibidi(mut skib) => {
                    // Spec: max phase 4, capped at 3 unless an opposing
                    // Skibidi is on the board.
                    let has_opponent = self.all_pieces().iter().any(|(_, p)| match p {
                        PieceType::Skibidi(other) => other.color != skib.color,
                        _ => false,
                    });
                    let max_phase = if has_opponent { 4 } else { 3 };
                    if skib.phase < max_phase {
                        skib.phase += 1;
                    }

                    let from_sq = self
                        .get_square_mut(from)
                        .ok_or_else(|| format!("No square at {:?}", from))?;

                    debug!(phase = skib.phase, "skibidi phase increased");

                    from_sq.piece = Some(PieceType::Skibidi(skib));
                }
                _ => return Err("Non-skibidi piece making phaseshift move".to_string()),
            },
            MoveType::MoveIntoCarrier(target) => {
                {
                    let from_sq = self
                        .get_square_mut(from)
                        .ok_or_else(|| format!("No square at {:?}", from))?;
                    from_sq.piece = None;
                }
                {
                    let to_sq = self
                        .get_square_mut(target)
                        .ok_or_else(|| format!("No square at {:?}", target))?;
                    let target_piece = to_sq.piece.as_mut().ok_or_else(|| {
                        format!("MoveIntoCarrier target {:?} is empty", target)
                    })?;
                    match target_piece {
                        PieceType::Bus(bus) => bus.pieces.push(piece),
                        _ => {
                            return Err(format!(
                                "MoveIntoCarrier target {:?} is not a carrier",
                                target
                            ));
                        }
                    }
                }
                debug!(?from, ?target, "move into carrier executed");
            }
            MoveType::PieceInCarrier {
                piece_index,
                move_type,
            } => {
                let mut bus = match piece.clone() {
                    PieceType::Bus(bus) => bus,
                    other => {
                        return Err(format!(
                            "PieceInCarrier source must be a carrier, got {:?}",
                            other
                        ));
                    }
                };
                let idx = *piece_index as usize;
                let moving_out_piece = bus.pieces.get(idx).cloned().ok_or_else(|| {
                    format!("PieceInCarrier index {} out of range", piece_index)
                })?;

                match move_type.as_ref() {
                    MoveType::MoveTo(target) => {
                        let to_sq = self
                            .get_square_mut(target)
                            .ok_or_else(|| format!("No square at {:?}", target))?;
                        to_sq.piece = Some(moving_out_piece);
                        bus.pieces.remove(idx);
                        debug!(?from, ?target, "moved out of carrier");
                    }
                    MoveType::MoveIntoCarrier(target) => {
                        let to_sq = self
                            .get_square_mut(target)
                            .ok_or_else(|| format!("No square at {:?}", target))?;
                        let target_piece = to_sq.piece.as_mut().ok_or_else(|| {
                            format!(
                                "PieceInCarrier->MoveIntoCarrier target {:?} is empty",
                                target
                            )
                        })?;
                        match target_piece {
                            PieceType::Bus(target_bus) => {
                                target_bus.pieces.push(moving_out_piece);
                                bus.pieces.remove(idx);
                                debug!(
                                    ?from,
                                    ?target,
                                    "passenger moved between carriers"
                                );
                            }
                            _ => {
                                return Err(format!(
                                    "PieceInCarrier->MoveIntoCarrier target {:?} is not a carrier",
                                    target
                                ));
                            }
                        }
                    }
                    other => {
                        return Err(format!(
                            "unsupported PieceInCarrier inner move type: {:?}",
                            other
                        ));
                    }
                }

                let from_sq = self
                    .get_square_mut(from)
                    .ok_or_else(|| format!("No square at {:?}", from))?;
                from_sq.piece = Some(PieceType::Bus(bus));
            }
        };

        self.handle_post_move_effects(&board_before, game_move)?;

        Ok(())
    }

    /// If `captured_piece` is a rook on its color's starting rook square,
    /// drop the corresponding castle right. Mirrors the standard-chess rule
    /// that capturing a rook on h1/a1/h8/a8 cancels future castling on
    /// that side. Plan 03's `post_move_effects` covers rook *moves*; this
    /// covers the case where the rook never moves but is captured in place.
    fn maybe_clear_castle_on_rook_capture(
        &mut self,
        captured_square: &Coord,
        captured_piece: &PieceType,
    ) {
        let PieceType::Rook(r) = captured_piece else {
            return;
        };
        // Right-edge file (`width - 1`) hosts the kingside rook on any
        // board width — file 7 was the old 8-wide assumption.
        let right_edge = self.width().saturating_sub(1);
        let white_back = self.height().saturating_sub(1);
        match r.color {
            Color::White => {
                if captured_square.rank == white_back {
                    if captured_square.file == 0 {
                        self.flags.white_can_castle_queenside = false;
                    } else if captured_square.file == right_edge {
                        self.flags.white_can_castle_kingside = false;
                    }
                }
            }
            Color::Black => {
                if captured_square.rank == 0 {
                    if captured_square.file == 0 {
                        self.flags.black_can_castle_queenside = false;
                    } else if captured_square.file == right_edge {
                        self.flags.black_can_castle_kingside = false;
                    }
                }
            }
        }
    }

    fn handle_post_move_effects(
        &mut self,
        before_state: &Board,
        game_move: GameMove,
    ) -> Result<(), String> {
        // Reset en-passant target before piece-level hooks. Pawn's
        // post_move_effects re-sets it if this move was a double push.
        self.flags.en_passant_target = None;

        // The square at which the moving piece ends up (if any). For
        // PhaseShift and Bus-internal moves we skip the post-effect dispatch
        // entirely.
        let piece_target: Option<Coord> = match &game_move.move_type {
            MoveType::PhaseShift => None,
            MoveType::MoveTo(target) => Some(target.clone()),
            MoveType::Promotion { target, .. } => Some(target.clone()),
            MoveType::EnPassant { target, .. } => Some(target.clone()),
            MoveType::Castle { side } => {
                let king_file = match side {
                    CastleSide::Kingside => 6,
                    CastleSide::Queenside => 2,
                };
                Some(Coord {
                    file: king_file,
                    rank: game_move.from.rank,
                })
            }
            MoveType::MoveIntoCarrier(_) | MoveType::PieceInCarrier { .. } => None,
        };

        if let Some(target) = piece_target {
            let mut piece = {
                let square = self
                    .get_square_at(&target)
                    .ok_or_else(|| format!("No square at {:?}", target))?;

                square
                    .piece
                    .clone()
                    .ok_or_else(|| format!("No piece at {:?}", target))?
            };
            piece.post_move_effects(before_state, self, &game_move);
        }

        self.recalc_brainrot();

        // Plan 01: flip turn at the very end so post-move hooks see the
        // pre-flip state (matters if a hook ever wants to know whose move
        // it was).
        self.flags.side_to_move = self.flags.side_to_move.opposite();

        Ok(())
    }
}
