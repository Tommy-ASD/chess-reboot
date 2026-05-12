use tracing::debug;

use crate::{
    board::{Board, GameMove, MoveType},
    pieces::piecetype::PieceType,
};

impl Board {
    /// Attempts to execute a move on the board.
    /// Returns Ok(()) if successful, Err(...) if illegal.
    pub fn make_move(&mut self, game_move: GameMove) -> Result<(), String> {
        // validate legality of the move
        if !self.is_valid_move(&game_move) {
            return Err(format!("Illegal move: {:?}", game_move));
        }

        let board_before = self.clone();

        let from = &game_move.from;

        // validate existence of source square + piece
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
                // mutate board: remove piece from original square
                // this logic will change later on with new pieces
                {
                    let from_sq = self
                        .get_square_mut(from)
                        .ok_or_else(|| format!("No square at {:?}", from))?;

                    from_sq.piece = None;
                }

                // handle capture or landing on new square
                // again, logic will change with new pieces
                {
                    let to_sq = self
                        .get_square_mut(target)
                        .ok_or_else(|| format!("No square at {:?}", target))?;

                    // Whatever piece is there → captured automatically
                    to_sq.piece = Some(piece);
                }

                debug!(?from, ?target, "move executed");
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
                // Remove piece from source.
                {
                    let from_sq = self
                        .get_square_mut(from)
                        .ok_or_else(|| format!("No square at {:?}", from))?;
                    from_sq.piece = None;
                }
                // Push into the target carrier.
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
                        // Passenger exits straight into a different friendly carrier.
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

                // Put the (possibly-emptied) source carrier back.
                let from_sq = self
                    .get_square_mut(from)
                    .ok_or_else(|| format!("No square at {:?}", from))?;
                from_sq.piece = Some(PieceType::Bus(bus));
            }
        };

        // 5. Special movement hooks (stub)
        self.handle_post_move_effects(&board_before, game_move)?;

        Ok(())
    }

    fn handle_post_move_effects(
        &mut self,
        before_state: &Board,
        game_move: GameMove,
    ) -> Result<(), String> {
        // call post-move effect on the piece that moved
        match &game_move.move_type {
            MoveType::PhaseShift => {
                // currently no post-move effects for PhaseShift
            }
            MoveType::MoveTo(target) => {
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
            // No piece-level post-move effect for these variants today.
            MoveType::MoveIntoCarrier(_) => {}
            MoveType::PieceInCarrier { .. } => {}
        }

        self.recalc_brainrot();

        Ok(())
    }
}
