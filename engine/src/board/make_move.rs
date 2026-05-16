use tracing::debug;

use crate::{
    board::{
        Board, CastleSide, Coord, GameMove, MoveError, MoveType, PromotionTarget,
        square::{SquareCondition, SquareType},
    },
    pieces::{Color, piecetype::PieceType},
};

/// Castle geometry: `(king_target_file, rook_target_file)` for a
/// given `CastleSide`. Hardcoded to 8-wide standard-chess
/// conventions; gate on `width >= 8` at move-gen time (see
/// `king::castle_moves`). The rook's SOURCE file depends on width
/// (right_edge for kingside, 0 for queenside) and is computed
/// at the call site of `relocate_pieces`.
///
/// Centralizing this here means a future Chess960 variant or
/// non-8-wide castle scheme only touches one site instead of four.
pub(crate) fn castle_target_files(side: CastleSide) -> (u8, u8) {
    match side {
        CastleSide::Kingside => (6, 5),
        CastleSide::Queenside => (2, 3),
    }
}

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
    ///
    /// **Plan 10 step 10:** between phase 1 (relocate) and phase 2
    /// (piece post-effects), the capture stack runs for every piece
    /// that was captured by this move. Handlers return `BoardOp`s
    /// which apply before phase 2 fires.
    pub fn make_move_unchecked(&mut self, game_move: GameMove) -> Result<(), String> {
        let board_before = self.clone();
        self.relocate_pieces(&game_move)?;
        self.fire_capture_stack(&board_before, &game_move);
        let ctx = PostMoveCtx {
            before_state: &board_before,
            game_move: &game_move,
        };
        self.apply_piece_post_effects(&ctx)?;
        self.apply_environment_reactions(&ctx);
        Ok(())
    }

    /// Apply a move's piece-relocation phase + capture stack + piece
    /// post-effects, but skip environment reactions (no train tick, no
    /// brainrot recalc beyond what post-effects compute). Used by
    /// `validate_move` so king-safety is evaluated on the player-move-
    /// only state, before any train tick could roll over the mover's
    /// king.
    ///
    /// **Critical:** the capture stack runs here. Without it, validation
    /// would see a different post-move board than real `make_move`, and
    /// any `BoardOp::PlacePiece` produced by a capture handler (e.g.
    /// `GoblinDropVictimCapture` dropping the kidnap victim onto the
    /// captor's origin) would be invisible to king-safety — leading to
    /// false `WouldLeaveKingInCheck` rejections when the dropped piece
    /// would have blocked an enemy slider's ray.
    pub(crate) fn apply_move_for_validation(
        &mut self,
        game_move: GameMove,
    ) -> Result<(), String> {
        let board_before = self.clone();
        self.relocate_pieces(&game_move)?;
        self.fire_capture_stack(&board_before, &game_move);
        let ctx = PostMoveCtx {
            before_state: &board_before,
            game_move: &game_move,
        };
        self.apply_piece_post_effects(&ctx)?;
        Ok(())
    }

    /// Plan 10 step 10: fire capture-stack handlers for any piece this
    /// move just captured. Runs on the post-relocate board so handlers
    /// see the empty captor-origin square (Goblin drop-victim depends
    /// on this). Shared between `make_move_unchecked` (real apply) and
    /// `apply_move_for_validation` (hypothetical) so the two paths
    /// produce the same intermediate board state.
    fn fire_capture_stack(&mut self, board_before: &Board, game_move: &GameMove) {
        let captures = capture_targets(board_before, game_move);
        for cap in captures {
            let event = crate::movement::stack::capture::ResolutionEvent::Capture {
                captor_coord: cap.captor_coord,
                captor_origin: cap.captor_origin,
                captor: cap.captor,
                victim_coord: cap.victim_coord,
                victim: cap.victim,
                move_type: game_move.move_type.clone(),
            };
            let ops = crate::movement::stack::capture::default_capture_stack()
                .resolve_capture(self, &event);
            for op in ops {
                op.apply(self);
            }
        }
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
                let (king_target_file, rook_target_file) = castle_target_files(*side);
                let rook_source_file = match side {
                    CastleSide::Kingside => right_edge,
                    CastleSide::Queenside => 0,
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
            MoveType::PlaceTornado { target } => {
                // The placer stays put (like ThrowSwitch — the cloned
                // `piece` is intentionally unused here). Stamp a
                // Tornado on `target`. If one is already there, refresh
                // its countdown rather than stacking a second Tornado
                // condition (keeps the C= payload unambiguous).
                let dur = crate::pieces::fairy::stormcaller::TORNADO_DURATION;
                let sq = self
                    .get_square_mut(target)
                    .ok_or_else(|| format!("PlaceTornado: no square at {target:?}"))?;
                let mut refreshed = false;
                for c in sq.conditions.iter_mut() {
                    if let SquareCondition::Tornado { remaining } = c {
                        *remaining = dur;
                        refreshed = true;
                    }
                }
                if !refreshed {
                    sq.conditions
                        .push(SquareCondition::Tornado { remaining: dur });
                }
                // Audit R1/E-4e: a tornado on a non-walkable square
                // (Block/Turret/Vent) is inert — no piece can ever stand
                // there to be trapped, and WalkabilityFilter drops any
                // move that would land on it, so it never satisfies a
                // compulsion either. It still counts down harmlessly.
                // Surfaced (not silently inert) per the project's
                // lenient-but-loud convention.
                if sq.square_type.is_walkable() {
                    debug!(?from, ?target, dur, "tornado placed");
                } else {
                    debug!(
                        ?from, ?target, dur,
                        "tornado placed on a non-walkable square — inert \
                         (no piece can be trapped/compelled there) but it \
                         still counts down"
                    );
                }
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
                        // Passenger exiting the carrier can capture a
                        // corner rook (the overwrite-anything-on-target
                        // semantics below). Mirror the top-level MoveTo
                        // arm and revoke castle rights before clobbering
                        // — otherwise a Bus dropping a knight onto a1
                        // would leave white queenside still settable.
                        if let Some(captured) =
                            self.get_square_at(target).and_then(|s| s.piece.clone())
                        {
                            self.maybe_clear_castle_on_rook_capture(target, &captured);
                        }
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
    pub(crate) fn maybe_clear_castle_on_rook_capture(
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
            MoveType::PhaseShift
            | MoveType::ThrowSwitch { .. }
            | MoveType::PlaceTornado { .. } => None,
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
                let (king_file, _) = castle_target_files(*side);
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
    /// run after the move + piece reactions settle. Validate's clone
    /// deliberately stops before this phase so the player's chosen
    /// move can be king-safety-checked without an in-progress train
    /// tick removing the king from the board mid-evaluation.
    ///
    /// **Plan 10 step 11:** the inline train-tick + brainrot-recalc
    /// logic now flows through `EnvReactionRegistry`. Each
    /// auto-mechanic is a handler registered against an `EnvPhase`.
    /// Future pieces (Magnet, Bell-Ringer, Boy, Marcher, NPC) plug
    /// in via the same trait without re-touching this method.
    fn apply_environment_reactions(&mut self, ctx: &PostMoveCtx<'_>) {
        use crate::movement::env_reactions::default_registry;
        self.apply_environment_reactions_with(ctx, default_registry());
    }

    /// Same body as `apply_environment_reactions` but with an injected
    /// registry. The default-flow caller passes `default_registry()`;
    /// tests can pass a custom registry containing probe handlers to
    /// pin ordering invariants that aren't testable through the
    /// `OnceLock`-backed default. Round-4 audit added this seam to
    /// sharpen `test_last_move_written_before_post_mover_phase` —
    /// otherwise the test couldn't distinguish "write happens before
    /// PostMover" from "write happens after PostMover" since both
    /// orderings leave `last_move` populated by the time
    /// `make_move` returns.
    pub(crate) fn apply_environment_reactions_with(
        &mut self,
        ctx: &PostMoveCtx<'_>,
        reg: &crate::movement::env_reactions::EnvReactionRegistry,
    ) {
        use crate::movement::env_reactions::{EnvPhase, EnvReactionCtx};

        let mut env_ctx = EnvReactionCtx::default();

        // Round-3 audit fix: stamp `last_move` BEFORE the PostMover
        // phase. Auto-action handlers at PostMover (Boy Who Followed
        // Geese, future "react to the just-applied move" pieces) read
        // `BoardFlags.last_move` to know what just happened. With the
        // write deferred to after the side flip, PostMover handlers
        // would see a stale `last_move` from the PREVIOUS turn.
        //
        // `compute_last_move` is pure (read-only on `ctx.before_state`
        // + the move payload), so moving the write earlier doesn't
        // depend on later mutation.
        self.flags.last_move = compute_last_move(ctx.before_state, ctx.game_move);

        // PostMover fires for handlers that want to react to the
        // just-applied move on the mover's side (Boy step toward the
        // enemy that just moved, Marcher march, NPC advance). At this
        // point `side_to_move` is still the mover; the flip happens
        // after the tick.
        reg.run_phase(self, EnvPhase::PostMover, false, &mut env_ctx);
        reg.run_phase(self, EnvPhase::TickGate, false, &mut env_ctx);
        reg.run_phase(self, EnvPhase::PostTick, false, &mut env_ctx);

        // Plan 01: flip turn after env reactions so the train tick
        // and other auto-mechanics still see the mover as the
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

        // PreMover fires AFTER the side flip — "start of opponent's
        // turn" handlers (Magnet pull, Bell-Ringer toll) want to see
        // the now-current side. They also see the just-written
        // `last_move` from this turn. None registered in v1.
        reg.run_phase(self, EnvPhase::PreMover, false, &mut env_ctx);
    }
}

/// Build a `LastMove` snapshot from the move that just applied. Pulls
/// `mover_color` from the pre-move board's perspective (honouring
/// `effective_mover_color` for PieceInCarrier passengers), the
/// captured piece's symbol from the pre-move occupant of the
/// captured square, and the primary piece's symbol from the move
/// payload (for Promotion) or the source piece (otherwise).
///
/// **Read-only:** intentionally takes `&Board` for `before` only,
/// not the post-relocation/post-tick board. The Promotion primary
/// symbol is derived from `PromotionTarget` directly so that a
/// train-tick captuing the freshly-promoted piece doesn't corrupt
/// the recorded symbol.
fn compute_last_move(
    before: &Board,
    game_move: &GameMove,
) -> Option<crate::board::LastMove> {
    use crate::board::LastMoveKind;

    let from = game_move.from.clone();
    let source_piece = before.get_square_at(&from)?.piece.clone()?;
    let (mover_color, _) = before.effective_mover_color(&source_piece, game_move);

    // `to` is the destination most consumers will read as the move's
    // landing square. For Castle, expose the king's destination so a
    // Mirror-style piece sees a meaningful from→to delta. For
    // ThrowSwitch / PhaseShift / nested PieceInCarrier{!MoveTo} there
    // is no single piece-landing square; surface `None`.
    let to = match &game_move.move_type {
        MoveType::Castle { side } => {
            let (king_file, _) = castle_target_files(*side);
            Some(Coord {
                file: king_file,
                rank: game_move.from.rank,
            })
        }
        _ => piece_landing_square(game_move).cloned(),
    };

    // Captured: pre-move occupant of the captured square. For en
    // passant the captured pawn lives at a different coord than `to`.
    // The mover-color filter rejects friendly-fire pseudo-captures.
    //
    // **Carrier-as-victim:** a top-level `MoveTo` landing on an
    // opposite-coloured Bus IS a real capture (the bus dies); a
    // `MoveIntoCarrier` landing on a friendly / Neutral carrier is
    // boarding-not-capture. The distinction is the OUTER MoveType,
    // not whether the victim is a carrier. Excluding by carrier-shape
    // alone would silently drop legitimate Bus captures from
    // `LastMove`, and Mirror / Echo / Combo-Echo would treat the
    // capture as a non-capture.
    let captured_coord = match &game_move.move_type {
        MoveType::EnPassant { captured, .. } => Some(captured.clone()),
        _ => to.clone(),
    };
    let boarding_not_capture = match &game_move.move_type {
        MoveType::MoveIntoCarrier(_) => true,
        MoveType::PieceInCarrier { move_type, .. } => {
            matches!(move_type.as_ref(), MoveType::MoveIntoCarrier(_))
        }
        _ => false,
    };
    let captured_symbol = if boarding_not_capture {
        None
    } else {
        captured_coord
            .as_ref()
            .and_then(|c| before.get_square_at(c))
            .and_then(|sq| sq.piece.as_ref())
            .filter(|p| p.get_color() != mover_color)
            .map(|p| p.symbol())
    };

    // Primary piece: post-promotion if Promotion, else the source
    // piece's symbol.
    //
    // **Why not read from `after`:** the round-3 audit caught a bug
    // where reading `after.get_square_at(target).piece` returns the
    // LOCOMOTIVE that ticked onto the promotion square between
    // phases 2 and 3. The `unwrap_or_else` fallback never fires
    // (Some(loco.symbol()) is Some, not None). `primary_symbol`
    // becomes the loco's verbose symbol — corrupting both
    // `LastMove` consumers and the FEN round-trip (the loco symbol
    // contains commas that confuse the `lm=` parser).
    //
    // Derive directly from the move payload instead: `into` +
    // `mover_color` give a deterministic answer that doesn't depend
    // on subsequent env reactions.
    let primary_symbol = match &game_move.move_type {
        MoveType::Promotion { into, .. } => {
            let promoted = match into {
                PromotionTarget::Queen => PieceType::new_queen(mover_color),
                PromotionTarget::Rook => PieceType::new_rook(mover_color),
                PromotionTarget::Bishop => PieceType::new_bishop(mover_color),
                PromotionTarget::Knight => PieceType::new_knight(mover_color),
            };
            promoted.symbol()
        }
        _ => source_piece.symbol(),
    };

    let kind = match &game_move.move_type {
        MoveType::MoveTo(_) => LastMoveKind::Move,
        MoveType::MoveIntoCarrier(_) => LastMoveKind::MoveIntoCarrier,
        MoveType::Promotion { .. } => LastMoveKind::Promote,
        MoveType::Castle { .. } => LastMoveKind::Castle,
        MoveType::EnPassant { .. } => LastMoveKind::EnPassant,
        MoveType::PhaseShift => LastMoveKind::PhaseShift,
        MoveType::ThrowSwitch { .. } => LastMoveKind::ThrowSwitch,
        MoveType::PieceInCarrier { .. } => LastMoveKind::PieceInCarrier,
        MoveType::PlaceTornado { .. } => LastMoveKind::PlaceTornado,
    };

    Some(crate::board::LastMove {
        mover_color,
        from,
        to,
        captured_symbol,
        primary_symbol,
        kind,
    })
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

/// Plan 10 step 10 helper: identify the captures this move produced.
/// Read from `before` since the post-relocation board has already
/// cleared the victim's square.
///
/// Field semantics (mirroring `ResolutionEvent::Capture`):
///   - `captor_coord`: the captor's POST-MOVE position. Target for
///     MoveTo / Promotion / EnPassant. Inner MoveTo's target for
///     PIC{MoveTo}.
///   - `captor_origin`: pre-move position on the outer board, or
///     `None` for PIC (passenger emerged from inside a carrier, no
///     outer-board origin).
///
/// Covers four reachable capture pathways:
///   - `MoveType::MoveTo` / `Promotion` — direct capture at target.
///   - `MoveType::EnPassant` — capture at `captured` coord.
///   - `MoveType::PieceInCarrier { inner: MoveTo(target) }` —
///     passenger exit-and-capture.
///
/// Carrier-boarding enemy-passenger captures (the inline `passengers.
/// retain` in the `MoveIntoCarrier` arm) are still not surfaced.
#[derive(Debug)]
pub(crate) struct CapturePair {
    pub captor_coord: Coord,
    pub captor_origin: Option<Coord>,
    pub captor: PieceType,
    pub victim_coord: Coord,
    pub victim: PieceType,
}

pub(crate) fn capture_targets(before: &Board, game_move: &GameMove) -> Vec<CapturePair> {
    let from = game_move.from.clone();
    let Some(source_piece) = before
        .get_square_at(&from)
        .and_then(|sq| sq.piece.clone())
    else {
        return Vec::new();
    };

    let (captor, captor_coord, captor_origin, victim_coord, victim_lookup_coord) =
        match &game_move.move_type {
            MoveType::MoveTo(target) | MoveType::Promotion { target, .. } => (
                source_piece,
                target.clone(),
                Some(from.clone()),
                target.clone(),
                target.clone(),
            ),
            MoveType::EnPassant { target, captured } => (
                source_piece,
                target.clone(),
                Some(from.clone()),
                captured.clone(),
                captured.clone(),
            ),
            // PIC{MoveTo}: passenger exits the carrier and lands on
            // `target`. captor_coord is `target` (the passenger's
            // post-move position); captor_origin is None because the
            // passenger had no outer-board origin tile.
            MoveType::PieceInCarrier {
                piece_index,
                move_type,
            } => {
                let MoveType::MoveTo(target) = move_type.as_ref() else {
                    // PIC{MoveIntoCarrier} / PIC{anything else} is not a
                    // capture against an outer-board square.
                    return Vec::new();
                };
                let Some(passenger) = source_piece
                    .passengers()
                    .and_then(|ps| ps.get(*piece_index as usize))
                    .cloned()
                else {
                    return Vec::new();
                };
                (passenger, target.clone(), None, target.clone(), target.clone())
            }
            // No capture for these move types.
            _ => return Vec::new(),
        };

    let Some(victim) = before
        .get_square_at(&victim_lookup_coord)
        .and_then(|sq| sq.piece.clone())
    else {
        return Vec::new();
    };

    // Friendly-fire isn't a capture in the chess sense — and would
    // have been rejected during move generation anyway. Defence-in-
    // depth.
    if victim.get_color() == captor.get_color() {
        return Vec::new();
    }

    vec![CapturePair {
        captor_coord,
        captor_origin,
        captor,
        victim_coord,
        victim,
    }]
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
            let (king_file, rook_file) = castle_target_files(*side);
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
        | MoveType::ThrowSwitch { .. }
        | MoveType::PlaceTornado { .. } => vec![],
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
        MoveType::PhaseShift
        | MoveType::ThrowSwitch { .. }
        | MoveType::PlaceTornado { .. } => None,
    }
}
