use tracing::debug;

use crate::{
    board::{
        Board, CastleSide, Coord, GameMove, MoveError, MoveType, PromotionTarget,
        square::SquareType,
    },
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

    /// Apply a move without re-running legality checks. Composes the
    /// three apply phases — bare-piece relocation, piece-level post
    /// effects, and environment reactions (train tick) — in order. The
    /// caller (e.g. `legal_moves`) is responsible for having generated
    /// the move from `get_moves` first; internal invariants are still
    /// verified and become `Err` rather than `panic!`.
    pub fn make_move_unchecked(&mut self, game_move: GameMove) -> Result<(), String> {
        let board_before = self.clone();
        self.relocate_pieces(&game_move)?;
        let ctx = PostMoveCtx {
            before_state: &board_before,
            game_move: &game_move,
        };
        self.apply_piece_post_effects(&ctx)?;
        self.apply_environment_reactions(&ctx);
        Ok(())
    }

    /// Apply a move's piece-relocation phase only — no environment
    /// reactions (no train tick). Used by `validate_move` so king-safety
    /// is evaluated on the player-move-only state, before any train tick
    /// could roll over the mover's king (which would otherwise make
    /// `WouldLeaveKingInCheck` un-detectable).
    pub(crate) fn apply_move_for_validation(
        &mut self,
        game_move: GameMove,
    ) -> Result<(), String> {
        let board_before = self.clone();
        self.relocate_pieces(&game_move)?;
        let ctx = PostMoveCtx {
            before_state: &board_before,
            game_move: &game_move,
        };
        self.apply_piece_post_effects(&ctx)?;
        Ok(())
    }

    /// Phase 1: physically move pieces around. No piece-level post hooks,
    /// no environment reactions — those are separate phases. This is the
    /// pure mechanical effect of the move on the grid.
    fn relocate_pieces(&mut self, game_move: &GameMove) -> Result<(), String> {
        // Plan 08 safety net: any move that lands a piece on a non-walkable
        // square (closed Gate, Turret, Vent) is rejected here, even if a
        // piece-level generator forgot to filter. Catches new pieces and
        // hand-crafted moves alike. ThrowSwitch is not piece-relocating, so
        // it bypasses this check; carrier-internal moves are validated at
        // their own arms.
        if let Some(landing) = piece_landing_square(game_move) {
            let walkable = self
                .get_square_at(landing)
                .map(|s| s.square_type.is_walkable())
                .unwrap_or(false);
            if !walkable {
                return Err(format!(
                    "destination {landing:?} is not walkable for {:?}",
                    game_move.move_type
                ));
            }
        }

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
                let boarder_color = piece.get_color();
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
                    let carrier_color = target_piece.get_color();
                    match target_piece.passengers_mut() {
                        Some(passengers) => {
                            // Plan 09: when a non-Neutral piece boards a
                            // Neutral cart, capture every passenger of
                            // the opposite color (the "passenger
                            // captured by enemy entering cart" rule).
                            // The cart itself is invincible — it stays
                            // put and just swaps occupants.
                            if carrier_color == Color::Neutral
                                && boarder_color != Color::Neutral
                            {
                                passengers.retain(|p| p.get_color() == boarder_color);
                            }
                            passengers.push(piece);
                        }
                        None => {
                            return Err(format!(
                                "MoveIntoCarrier target {:?} is not a carrier",
                                target
                            ));
                        }
                    }
                }
                debug!(?from, ?target, "move into carrier executed");
            }
            MoveType::ThrowSwitch { switch } => {
                // Validate the source is a Switch and clone the target list
                // before mutating the grid — the activation loop in
                // `fire_signal` borrows the grid mutably and can't share
                // with the immutable read.
                let targets = {
                    let sq = self
                        .get_square_at(switch)
                        .ok_or_else(|| format!("ThrowSwitch: no square at {switch:?}"))?;
                    match &sq.square_type {
                        SquareType::Switch { targets } => targets.clone(),
                        other => {
                            return Err(format!(
                                "ThrowSwitch target {switch:?} is not a Switch tile (got {other:?})"
                            ));
                        }
                    }
                };
                self.fire_signal(&targets);
                debug!(?switch, ?targets, "switch thrown");
            }
            MoveType::PieceInCarrier {
                piece_index,
                move_type,
            } => {
                let mut carrier = piece.clone();
                let idx = *piece_index as usize;
                let moving_out_piece = carrier
                    .passengers()
                    .and_then(|ps| ps.get(idx).cloned())
                    .ok_or_else(|| {
                        format!(
                            "PieceInCarrier source must be a carrier with passenger {idx}, got {:?}",
                            carrier
                        )
                    })?;

                match move_type.as_ref() {
                    MoveType::MoveTo(target) => {
                        let to_sq = self
                            .get_square_mut(target)
                            .ok_or_else(|| format!("No square at {:?}", target))?;
                        to_sq.piece = Some(moving_out_piece);
                        carrier
                            .passengers_mut()
                            .expect("just verified is carrier")
                            .remove(idx);
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
                        let target_carrier_color = target_piece.get_color();
                        let boarder_color = moving_out_piece.get_color();
                        let target_is_bus = matches!(target_piece, PieceType::Bus(_));
                        let Some(target_passengers) = target_piece.passengers_mut()
                        else {
                            return Err(format!(
                                "PieceInCarrier->MoveIntoCarrier target {:?} is not a carrier",
                                target
                            ));
                        };
                        // Same cart-invincibility rule as the top-level
                        // MoveIntoCarrier arm: a non-Neutral piece
                        // boarding a Neutral cart captures any
                        // opposite-colour passengers. Same-colour
                        // carriers (Bus) are unaffected here because
                        // the filter only emits MoveIntoCarrier when
                        // the carrier and boarder agree on colour.
                        if target_carrier_color == Color::Neutral
                            && boarder_color != Color::Neutral
                        {
                            target_passengers
                                .retain(|p| p.get_color() == boarder_color);
                        }
                        // Bus cap-5 defence in depth — the carrier-
                        // capacity guard inside `PieceType::get_moves`'
                        // `retain_mut` is the primary check, but a
                        // hand-crafted move bypassing the filter
                        // shouldn't overfill a Bus.
                        if target_is_bus && target_passengers.len() >= 5 {
                            return Err(format!(
                                "PieceInCarrier->MoveIntoCarrier: target Bus at {target:?} at capacity"
                            ));
                        }
                        target_passengers.push(moving_out_piece);
                        carrier
                            .passengers_mut()
                            .expect("just verified is carrier")
                            .remove(idx);
                        debug!(?from, ?target, "passenger moved between carriers");
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
                from_sq.piece = Some(carrier);
            }
        };

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
            // Neutral pieces never hold castle rights.
            Color::Neutral => {}
        }
    }

    /// Phase 2 (validate-relevant): piece-level reactions to the move
    /// the player just made. Updates en passant tracking, runs each
    /// piece's `post_move_effects`, fires pressure plates the move
    /// landed on, and recalcs the brainrot map. These are all
    /// deterministic consequences of the player's choice — validate
    /// runs them so king-safety sees the same state real make_move
    /// would, *minus* the train tick (which lives in phase 3).
    fn apply_piece_post_effects(&mut self, ctx: &PostMoveCtx<'_>) -> Result<(), String> {
        // Reset en-passant target before piece-level hooks. Pawn's
        // post_move_effects re-sets it if this move was a double push.
        self.flags.en_passant_target = None;

        // Identify the *moving* piece and dispatch its
        // `post_move_effects`. There are four cases:
        //
        // 1. Top-level relocations (MoveTo, Promotion, EnPassant,
        //    Castle) — the mover lands at a concrete target square on
        //    the post-relocation board. We fetch that landed piece.
        //    For Promotion this is the *promoted* piece, not the
        //    pawn; today that's benign because the only promoted-rook
        //    side effect (clearing castle rights) is gated on
        //    `from == starting corner` and a promotion always
        //    originates from a non-corner pawn rank.
        //
        // 2. MoveIntoCarrier — the mover is now stored inside the
        //    target carrier's passenger list, not at a top-level
        //    square. Fetch it from `before_state` so hooks like
        //    `King::post_move_effects` (clears castle rights) and
        //    `Skibidi::post_move_effects` (resets phase) still fire.
        //    Without this, a king-into-bus silently preserves castle
        //    rights for the duration of the ride.
        //
        // 3. PieceInCarrier { inner: MoveTo(_) } — a passenger exits
        //    onto a board tile. The passenger's hook should fire on
        //    that landing.
        //
        // 4. PieceInCarrier { inner: MoveIntoCarrier(_) } — passenger
        //    hops cart A → cart B (round-4 addition). The hook still
        //    fires so a king-passenger clears its own castle rights.
        //
        // PhaseShift and ThrowSwitch don't relocate a piece — skip.
        let mover_dispatch: Option<PieceType> = match &ctx.game_move.move_type {
            MoveType::PhaseShift | MoveType::ThrowSwitch { .. } => None,
            MoveType::MoveTo(target)
            | MoveType::Promotion { target, .. }
            | MoveType::EnPassant { target, .. } => {
                let square = self
                    .get_square_at(target)
                    .ok_or_else(|| format!("No square at {:?}", target))?;
                let piece = square
                    .piece
                    .clone()
                    .ok_or_else(|| format!("No piece at {:?}", target))?;
                Some(piece)
            }
            MoveType::Castle { side } => {
                let king_file = match side {
                    CastleSide::Kingside => 6,
                    CastleSide::Queenside => 2,
                };
                let target = Coord {
                    file: king_file,
                    rank: ctx.game_move.from.rank,
                };
                let square = self
                    .get_square_at(&target)
                    .ok_or_else(|| format!("No square at {:?}", target))?;
                let piece = square
                    .piece
                    .clone()
                    .ok_or_else(|| format!("No piece at {:?}", target))?;
                Some(piece)
            }
            MoveType::MoveIntoCarrier(_) => {
                // Mover came from `from` and is now inside the target
                // carrier; before_state still has it as a top-level
                // piece at `from`.
                let square = ctx
                    .before_state
                    .get_square_at(&ctx.game_move.from)
                    .ok_or_else(|| format!("No square at {:?}", ctx.game_move.from))?;
                square.piece.clone()
            }
            MoveType::PieceInCarrier {
                piece_index,
                move_type,
            } => {
                let supported_inner = matches!(
                    move_type.as_ref(),
                    MoveType::MoveTo(_) | MoveType::MoveIntoCarrier(_)
                );
                if !supported_inner {
                    None
                } else {
                    let carrier_sq = ctx
                        .before_state
                        .get_square_at(&ctx.game_move.from)
                        .ok_or_else(|| format!("No square at {:?}", ctx.game_move.from))?;
                    carrier_sq
                        .piece
                        .as_ref()
                        .and_then(|p| p.passengers())
                        .and_then(|ps| ps.get(*piece_index as usize).cloned())
                }
            }
        };

        if let Some(piece) = mover_dispatch {
            // `post_move_effects` takes `&self` — pieces that want to
            // change their own state do it by writing a new piece
            // through `board_after.set_piece_at(...)` rather than
            // mutating a local clone whose changes would otherwise be
            // dropped. Goblin already follows this convention by
            // re-fetching its on-board piece and downcasting; Skibidi
            // builds a fresh `Skibidi` with the new phase and writes
            // it back via `set_piece_at`.
            piece.post_move_effects(ctx.before_state, self, ctx.game_move);
        }

        // Plan 08 step 4: PressurePlate scan. For every square the move
        // settled a piece on, fire that square's plate if it has one. The
        // landing-set varies by move shape — see `collect_landings` for the
        // mapping. A Castle settles two pieces (king + rook); a passenger
        // exiting a carrier settles one passenger piece on the target tile.
        for landing in collect_landings(ctx.game_move) {
            self.maybe_fire_pressure_plate(&landing);
        }

        self.recalc_brainrot();

        Ok(())
    }

    /// Phase 3 (skipped by validate): board-level auto-mechanics that
    /// run after the move + piece reactions settle. Currently just the
    /// train tick and the side-to-move flip. Validate's clone deliberately
    /// stops before this phase so the player's chosen move can be
    /// king-safety-checked without an in-progress train tick removing
    /// the king from the board mid-evaluation.
    fn apply_environment_reactions(&mut self, _ctx: &PostMoveCtx<'_>) {
        // Plan 09: trains advance one step per tick-gate opening.
        let ticked = self.maybe_advance_trains();

        // A train tick can capture pieces — most importantly a
        // Skibidi sitting on a track tile. The brainrot map computed
        // in phase 2 is keyed on Skibidi positions, so re-run the
        // recalc *if and only if* the tick actually fired. Most
        // moves don't tick (`EveryFullTurn` is the default rate), so
        // gating avoids a wasted O(N²) recalc per move.
        if ticked {
            self.recalc_brainrot();
        }

        // Plan 01: flip turn at the very end so the train tick (and any
        // future environment reactions) still see the mover as the
        // side-to-move if they need to.
        //
        // `Color::Neutral.opposite() == Color::Neutral`, so a Neutral
        // side-to-move would lock the game forever. `validate_move`
        // rejects Neutral as a mover via the `WrongTurn` check, so
        // this is unreachable via the public API — but a debug-only
        // assert leaves release builds vulnerable to silent freezing
        // if a test or future code path bypasses validate. So we both
        // assert (dev: panic loudly) and recover (release: coerce to
        // White + warn) to keep prod always moving while loudly
        // flagging the misuse.
        debug_assert_ne!(
            self.flags.side_to_move,
            crate::pieces::Color::Neutral,
            "side_to_move must never be Neutral at flip time"
        );
        if self.flags.side_to_move == crate::pieces::Color::Neutral {
            tracing::warn!(
                "side_to_move was Neutral at flip; coercing to White to keep the game progressing"
            );
            self.flags.side_to_move = crate::pieces::Color::White;
        }
        self.flags.side_to_move = self.flags.side_to_move.opposite();
    }
}

/// Bundle of inputs flowing through the post-move-effect phases. Lives
/// outside `Board` so the methods that consume it stay borrow-friendly.
/// Keeping a struct (rather than threading individual params) means
/// adding a new piece of context — say, a future "move-was-a-capture"
/// flag — is one field at the call site instead of a signature change
/// fan-out.
pub(crate) struct PostMoveCtx<'a> {
    pub before_state: &'a Board,
    pub game_move: &'a GameMove,
}

/// Plan 08 step 4 helper: every board-square where a piece settled as a
/// result of this move. Used by the PressurePlate scan in
/// `apply_piece_post_effects` to fire plates for any of the landings.
///
/// - `MoveTo` / `Promotion` / `EnPassant`: one landing (the target).
/// - `Castle`: two landings — king's destination and rook's destination.
/// - `PieceInCarrier { MoveTo }`: passenger exits onto a tile; that's a
///   landing for the passenger. Other inner shapes don't surface a tile-
///   level landing.
/// - `MoveIntoCarrier`, `PhaseShift`, `ThrowSwitch`: no new piece on a
///   tile (the carrier was already there / no piece relocated).
fn collect_landings(game_move: &GameMove) -> Vec<Coord> {
    match &game_move.move_type {
        MoveType::MoveTo(c) => vec![c.clone()],
        MoveType::Promotion { target, .. } => vec![target.clone()],
        MoveType::EnPassant { target, .. } => vec![target.clone()],
        MoveType::Castle { side } => {
            let r = game_move.from.rank;
            let (king_file, rook_file) = match side {
                CastleSide::Kingside => (6u8, 5u8),
                CastleSide::Queenside => (2u8, 3u8),
            };
            vec![
                Coord {
                    file: king_file,
                    rank: r,
                },
                Coord {
                    file: rook_file,
                    rank: r,
                },
            ]
        }
        MoveType::PieceInCarrier { move_type, .. } => match move_type.as_ref() {
            MoveType::MoveTo(c) => vec![c.clone()],
            _ => vec![],
        },
        MoveType::MoveIntoCarrier(_)
        | MoveType::PhaseShift
        | MoveType::ThrowSwitch { .. } => vec![],
    }
}

/// Plan 08 safety net helper: where does the piece end up after this move?
/// Returns `None` for moves that don't relocate a piece onto a board square
/// (ThrowSwitch, MoveIntoCarrier into the carrier's own square, passenger-
/// internal PieceInCarrier moves). The walkability check in
/// `make_move_unchecked` uses this to short-circuit non-relocating moves.
fn piece_landing_square(game_move: &GameMove) -> Option<&Coord> {
    match &game_move.move_type {
        MoveType::MoveTo(c) => Some(c),
        MoveType::Promotion { target, .. } => Some(target),
        MoveType::EnPassant { target, .. } => Some(target),
        // Boarding a carrier: the boarder's square is the carrier's tile,
        // so the carrier's tile must be walkable. (A Bus parked on a
        // closed Gate is unreachable.)
        MoveType::MoveIntoCarrier(c) => Some(c),
        // Castle moves multiple pieces. King::castle_moves already
        // requires every square on the king's path to be empty *and*
        // walkable (per the updated `empty()` closure), so the safety
        // net is redundant here. Skip rather than re-derive king/rook
        // target files for both castle sides.
        MoveType::Castle { .. } => None,
        // PieceInCarrier{MoveTo} is a real relocation — the passenger
        // exits onto `target`, so the safety net should check that
        // tile's walkability. Other PIC inners (MoveIntoCarrier =
        // cross-cart hop, anything else = unsupported) don't land a
        // piece on an outer-board square. PhaseShift doesn't relocate;
        // ThrowSwitch doesn't move the piece.
        MoveType::PieceInCarrier { move_type, .. } => match move_type.as_ref() {
            MoveType::MoveTo(c) => Some(c),
            _ => None,
        },
        MoveType::PhaseShift | MoveType::ThrowSwitch { .. } => None,
    }
}
