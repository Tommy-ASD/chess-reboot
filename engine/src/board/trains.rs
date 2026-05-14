//! Plan 09: train auto-movement. Called from `apply_environment_reactions`
//! after every player move. Trains advance one step along their track
//! when the tick gate (`BoardFlags.train_tick_rate`) opens.
//!
//! What's *not* here: the collision-handler trait chain
//! (`on_run_over_target` / `on_being_run_over`) that plan 09 §"Collision
//! handlers" sketches. v1 hard-codes the equivalent inline — the
//! own-cart `Stop` is the `own_cart_collision` check below, and the
//! default `Capture` outcome is the unconditional victim removal in
//! the commit phase. When the collision-handler chain is implemented,
//! it folds into the per-train decision loop here.

use tracing::{debug, trace};

use crate::{
    board::{
        Board, Coord, MoveType, TrainTickRate,
        square::{SquareType, TrackDir},
    },
    movement::stack::capture::{ResolutionEvent, default_capture_stack},
    pieces::{
        fairy::locomotive::TrainHeading,
        piecetype::PieceType,
    },
};

/// One train cart's pre-tick state, gathered before the tick.
#[derive(Clone, Debug)]
struct CartSnapshot {
    coord: Coord,
    train_id: u32,
    /// 0 for locomotive, 1..N for carriages in order.
    chain_index: u8,
    /// Only Some for the locomotive; carriages inherit it from the head.
    heading: Option<TrainHeading>,
    /// Only meaningful for the locomotive (None for carriages). The
    /// cardinal direction the loco entered its current tile through —
    /// drives connection-aware traversal.
    last_dir: Option<TrackDir>,
}

/// What a train wants to do this tick after all per-cart decisions have
/// been resolved locally. We build a `Vec<TrainAdvance>` first so the
/// two-train collision pass can spot conflicts before any board mutation.
#[derive(Clone, Debug)]
struct TrainAdvance {
    train_id: u32,
    /// New positions for each cart in chain order: positions[0] is the
    /// locomotive's new tile, positions[k] is carriage `k`'s new tile.
    /// A train that can't advance simply never produces a `TrainAdvance`,
    /// so the indexing is total — no per-cart `Option` here.
    new_positions: Vec<Coord>,
    /// Pre-tick positions in the same order. Used to clear old squares
    /// without disturbing tiles the train is *staying* on.
    old_positions: Vec<Coord>,
    /// Tile(s) the train would capture this tick. Always squares the
    /// train is moving *onto* that hold a non-train occupant.
    captures: Vec<Coord>,
    /// New `last_dir` for the locomotive — the cardinal side it
    /// *entered* its destination tile through. Stored on commit so the
    /// next tick's neighbor-detection can exclude this side.
    new_loco_last_dir: TrackDir,
}

impl Board {
    /// Cardinal directions from `coord` that point at a Track or
    /// Junction tile. Drives minecart-style auto-connection: a track
    /// tile's "shape" is implicit in the set of cardinal neighbors that
    /// are themselves tracks. The loco then picks the exit by
    /// excluding the side it came from.
    pub fn neighbor_track_dirs(&self, coord: &Coord) -> Vec<TrackDir> {
        let mut out = Vec::new();
        for dir in [TrackDir::N, TrackDir::S, TrackDir::E, TrackDir::W] {
            let (df, dr) = dir.delta();
            let nf = coord.file as isize + df;
            let nr = coord.rank as isize + dr;
            if !self.in_bounds(nf, nr) {
                continue;
            }
            let next = Coord {
                file: nf as u8,
                rank: nr as u8,
            };
            if let Some(sq) = self.get_square_at(&next) {
                if matches!(
                    sq.square_type,
                    SquareType::Track { .. } | SquareType::Junction { .. }
                ) {
                    out.push(dir);
                }
            }
        }
        out
    }

    /// Plan 09 connection-aware traversal. Resolves the next tile a
    /// train arrives on given the current tile, the loco's `heading`
    /// (used only on the first tick when `last_dir` is `None`), and
    /// `last_dir` (the side the loco entered its current tile through).
    /// Returns `(next_coord, new_last_dir)` where `new_last_dir` is the
    /// side of the destination the loco is entering — feed that back in
    /// on the next call.
    ///
    /// Returns `None` when the train derails: current tile isn't a
    /// track / junction, or the chosen exit is off-board / not itself a
    /// track tile, or the only connection is the one the loco came from
    /// (dead-end).
    pub fn next_train_step(
        &self,
        from: &Coord,
        heading: TrainHeading,
        last_dir: Option<TrackDir>,
    ) -> Option<(Coord, TrackDir)> {
        let sq = self.get_square_at(from)?;

        let exit_dir = match &sq.square_type {
            SquareType::Junction { state, branches, .. } => {
                if branches.is_empty() {
                    return None;
                }
                *branches.get((*state as usize) % branches.len())?
            }
            SquareType::Track { direction } => {
                let preferred = match heading {
                    TrainHeading::Forward => *direction,
                    TrainHeading::Reverse => direction.opposite(),
                };
                match last_dir {
                    // First tick — start from the tile's stored D
                    // (rotated by heading). If that doesn't actually
                    // lead to a track tile (e.g. a freshly-painted
                    // loco landed on a tile whose default D=E points
                    // off the rail bed), fall back to picking the
                    // unique non-cart-blocked neighbor track. The
                    // fallback fires only when there's a single
                    // unambiguous choice — if 0 or 2+ valid neighbors
                    // exist, we keep `preferred` so the user's D still
                    // has the final say (and the train derails through
                    // the standard bounds check below if `preferred`
                    // is truly bogus).
                    None => {
                        if direction_leads_to_track(self, from, preferred) {
                            preferred
                        } else {
                            // The first-tick fallback is a *UX* helper
                            // for the editor case where the user
                            // paints a loco onto a chain with the
                            // tile's default D pointing off-rail.
                            // It deliberately excludes *all* cart
                            // neighbors (same-train and cross-train
                            // alike) — same-train carts would just
                            // produce an own-cart collision in
                            // `advance_trains`, and the heuristic
                            // shouldn't aggressively capture an
                            // adjacent foreign train either. The
                            // `Some(came_from)` branch below has no
                            // such filter because it's the runtime
                            // path: any actual collision is caught by
                            // `advance_trains`'s collision pass, so
                            // there's no need to pre-filter here.
                            let candidates: Vec<TrackDir> = self
                                .neighbor_track_dirs(from)
                                .into_iter()
                                .filter(|&d| {
                                    let (df, dr) = d.delta();
                                    let nf = from.file as isize + df;
                                    let nr = from.rank as isize + dr;
                                    if !self.in_bounds(nf, nr) {
                                        return false;
                                    }
                                    let next = Coord {
                                        file: nf as u8,
                                        rank: nr as u8,
                                    };
                                    match self
                                        .get_square_at(&next)
                                        .and_then(|s| s.piece.as_ref())
                                    {
                                        Some(p) if p.is_train_cart() => false,
                                        _ => true,
                                    }
                                })
                                .collect();
                            if candidates.len() == 1 {
                                candidates[0]
                            } else {
                                preferred
                            }
                        }
                    }
                    Some(came_from) => {
                        // Auto-connect: scan adjacent track/junction
                        // tiles, exclude the side we came from, pick the
                        // remaining connection. Minecart semantics —
                        // the user doesn't have to set per-tile D for
                        // curves; the rails connect themselves.
                        let candidates: Vec<TrackDir> = self
                            .neighbor_track_dirs(from)
                            .into_iter()
                            .filter(|d| *d != came_from)
                            .collect();
                        match candidates.len() {
                            0 => return None, // dead-end stub
                            1 => candidates[0],
                            // Ambiguous (3+ connections — a manual
                            // intersection). Prefer the tile's stored
                            // direction if it's among the live
                            // candidates; otherwise pick the first
                            // candidate so we still make progress.
                            _ => {
                                if candidates.contains(&preferred) {
                                    preferred
                                } else {
                                    candidates[0]
                                }
                            }
                        }
                    }
                }
            }
            _ => return None,
        };

        let (df, dr) = exit_dir.delta();
        let nf = from.file as isize + df;
        let nr = from.rank as isize + dr;
        if !self.in_bounds(nf, nr) {
            return None;
        }
        let next = Coord {
            file: nf as u8,
            rank: nr as u8,
        };
        let next_sq = self.get_square_at(&next)?;
        match &next_sq.square_type {
            SquareType::Track { .. } | SquareType::Junction { .. } => {
                // The loco enters `next` from the *opposite* side of
                // the direction it left through.
                Some((next, exit_dir.opposite()))
            }
            _ => None,
        }
    }

    /// Plan 09 entry point. Bumps the ply counter and ticks trains when
    /// the configured rate calls for it. Called from
    /// `apply_environment_reactions` (phase 3 of make_move) after
    /// every successful player move.
    /// Returns `true` iff this call actually ticked the trains (i.e.
    /// the rate gate fired). Callers can use this to skip downstream
    /// recomputations like `recalc_brainrot` when nothing moved.
    pub fn maybe_advance_trains(&mut self) -> bool {
        // `saturating_add` means once `ply_count` hits `u32::MAX` it
        // stays there forever, and the modulo gate below may then
        // never satisfy the rate again (e.g. EveryFullTurn at
        // MAX % 2 == 1, or any EveryNPly(n) where MAX % n != 0).
        // At ~4 billion plies this is wildly theoretical, but warn
        // *exactly once* on the transition so a debugger can notice
        // rather than the game silently stopping its trains. We
        // detect the transition by checking `MAX - 1` *before* the
        // add — the next call will saturate.
        if self.flags.ply_count == u32::MAX - 1 {
            tracing::warn!(
                "ply_count about to saturate at u32::MAX; train tick gate may stick"
            );
        }
        self.flags.ply_count = self.flags.ply_count.saturating_add(1);
        let should_tick = match self.flags.train_tick_rate {
            TrainTickRate::EveryPly => true,
            TrainTickRate::EveryFullTurn => self.flags.ply_count % 2 == 0,
            TrainTickRate::EveryNPly(n) => {
                let n = (n as u32).max(1);
                self.flags.ply_count % n == 0
            }
        };
        trace!(
            ply = self.flags.ply_count,
            ?self.flags.train_tick_rate,
            should_tick,
            "train tick gate"
        );
        if should_tick {
            self.advance_trains();
        }
        should_tick
    }

    /// Advance every train one step along its track. Snapshots
    /// positions first, then computes per-train advances, then resolves
    /// two-train collisions, then mutates the grid.
    ///
    /// **Brainrot / Frozen conditions on a cart's tile do NOT halt the
    /// train.** Train movement is environmental (auto-driven), not
    /// player-driven, and the spec's "brainrotted pieces can't move"
    /// rule applies to *player* moves. `Board::get_moves` short-
    /// circuits on those conditions, but `advance_trains` deliberately
    /// doesn't — a brainrotted cart still ticks and carries its
    /// passengers along. Passengers' own player-moves out of the cart
    /// remain blocked by `get_moves`'s condition check (consistent
    /// with the brainrot semantics for the passenger as a piece).
    pub fn advance_trains(&mut self) {
        let mut carts: Vec<CartSnapshot> = Vec::new();
        for (coord, piece) in self.iter_pieces() {
            match piece {
                PieceType::Locomotive(loco) => carts.push(CartSnapshot {
                    coord,
                    train_id: loco.train_id,
                    chain_index: 0,
                    heading: Some(loco.heading),
                    last_dir: loco.last_dir,
                }),
                PieceType::Carriage(c) => carts.push(CartSnapshot {
                    coord,
                    train_id: c.train_id,
                    chain_index: c.chain_index,
                    heading: None,
                    last_dir: None,
                }),
                _ => {}
            }
        }
        if carts.is_empty() {
            return;
        }

        // Group carts by train_id and order each group by chain_index.
        // Carts with no matching locomotive (orphaned carriages) get
        // skipped — they sit still until a locomotive joins the chain.
        let mut train_ids: Vec<u32> = carts.iter().map(|c| c.train_id).collect();
        train_ids.sort_unstable();
        train_ids.dedup();

        let mut advances: Vec<TrainAdvance> = Vec::new();

        for train_id in train_ids {
            let mut chain: Vec<&CartSnapshot> =
                carts.iter().filter(|c| c.train_id == train_id).collect();
            chain.sort_by_key(|c| c.chain_index);

            // Defensive guards for malformed FEN states. A well-formed
            // train has exactly one chain_index 0 (the locomotive)
            // followed by carriages at strictly increasing indices.
            // Two locos sharing a train_id, or carriages with
            // duplicate `(train_id, chain_index)`, would corrupt the
            // commit phase (`new_positions[k] = old_positions[k-1]`
            // teleports pieces onto top of each other). Warn and
            // skip the whole train rather than half-apply.
            let head_count =
                chain.iter().filter(|c| c.chain_index == 0).count();
            if head_count > 1 {
                tracing::warn!(
                    train_id,
                    head_count,
                    "train has multiple chain_index 0 carts — skipping tick"
                );
                continue;
            }
            let has_duplicate_index = chain
                .windows(2)
                .any(|w| w[0].chain_index == w[1].chain_index);
            if has_duplicate_index {
                tracing::warn!(
                    train_id,
                    "train has duplicate chain_index — skipping tick"
                );
                continue;
            }
            // Gaps (e.g. chain_index 0, 2 with no 1) would let phase 3
            // teleport the gap-skipped cart onto a tile its symbolic
            // predecessor never occupied, since the commit pass uses
            // `new_positions[k] = old_positions[k-1]` (positional, not
            // index-aware). Reject malformed trains with gaps the same
            // way duplicates are rejected.
            let has_index_gap = chain
                .windows(2)
                .any(|w| w[1].chain_index != w[0].chain_index + 1);
            if has_index_gap {
                tracing::warn!(
                    train_id,
                    "train has non-consecutive chain_index — skipping tick"
                );
                continue;
            }

            // Need a locomotive at the head.
            let Some(loco) = chain.first().filter(|c| c.chain_index == 0) else {
                continue;
            };
            let heading = match loco.heading {
                Some(h) => h,
                None => continue,
            };

            let old_positions: Vec<Coord> = chain.iter().map(|c| c.coord.clone()).collect();

            // Where does the locomotive want to go? Use the connection-
            // aware step so curves auto-resolve from neighbor rails.
            let Some((next_head, new_last_dir)) =
                self.next_train_step(&loco.coord, heading, loco.last_dir)
            else {
                // Derailed (or sitting on non-track). Train stops; no
                // advance recorded.
                trace!(train_id, "train cannot find next tile; stops");
                continue;
            };

            // Build proposed new positions. Carriage k goes to the
            // pre-tick position of cart k-1.
            let mut new_positions: Vec<Coord> = Vec::with_capacity(old_positions.len());
            new_positions.push(next_head.clone());
            for k in 1..old_positions.len() {
                new_positions.push(old_positions[k - 1].clone());
            }

            // Own-cart check: if the locomotive's target is any of the
            // train's own current cart squares, the train hits itself
            // and stops. The "caboose vacates so the loco could roll
            // onto its tile" case is technically valid for a fully-
            // wrapped loop, but per the plan that's still a Stop —
            // trains can't capture themselves.
            let own_cart_collision = old_positions.iter().any(|p| *p == next_head);
            if own_cart_collision {
                trace!(train_id, "train collides with own cart; stops");
                continue;
            }

            // Capture target: a non-cart occupant on the head's
            // landing tile gets captured. Carts on the landing tile
            // are handled by own_cart_collision (above) for same-train,
            // and by the foreign-cart filter pass that runs *after*
            // this loop. Doing the filter post-loop lets us account
            // for foreign carts that are themselves vacating their
            // tile this same tick (trailing-train scenario).
            let captures: Vec<Coord> = match self
                .get_square_at(&next_head)
                .and_then(|s| s.piece.as_ref())
            {
                Some(p) if !p.is_train_cart() => vec![next_head.clone()],
                _ => vec![],
            };

            advances.push(TrainAdvance {
                train_id,
                new_positions,
                old_positions,
                captures,
                new_loco_last_dir: new_last_dir,
            });
        }

        // Foreign-cart filter: a moving train's head can't land on
        // a tile occupied by a *different* train's cart unless that
        // tile is being vacated by its current occupant this same
        // tick. Without this, a stalled foreign cart on the landing
        // tile would be silently overwritten by the commit pass
        // (the unconditional `sq.piece = Some(cart)`). With it, a
        // trailing-train (A follows B east, B's caboose vacates the
        // tile A wants) still gets to advance.
        //
        // A moving train's "vacating tiles" are its `old_positions`
        // minus its `new_positions`. For a normally-moving train
        // that's exactly the *caboose's* old position (everything
        // else shifts back by one slot and so stays occupied).
        //
        // CRITICAL: the foreign-cart filter and two-train collision
        // pass both REMOVE trains from `advances`, which changes the
        // `vacating_tiles` set. A trailing train A that passes the
        // filter relying on B's *proposed* vacating tile gets unsound
        // if B is then dropped — B's cart stays put and A's commit
        // would overwrite it. Run both passes to fixed point: keep
        // re-checking until no more trains get blocked.
        //
        // Coord doesn't derive Hash, and the train count is small (≤a
        // few per board in practice), so flat Vec scans are fine.
        loop {
            let vacating_tiles: Vec<Coord> = advances
                .iter()
                .flat_map(|adv| {
                    adv.old_positions
                        .iter()
                        .filter(|p| !adv.new_positions.contains(p))
                        .cloned()
                        .collect::<Vec<_>>()
                })
                .collect();

            // Foreign-cart filter.
            let mut blocked_train_ids: Vec<u32> = Vec::new();
            for adv in &advances {
                let head = &adv.new_positions[0];
                let target = self.get_square_at(head).and_then(|s| s.piece.as_ref());
                let target_is_foreign_cart = match target {
                    Some(PieceType::Locomotive(l)) => l.train_id != adv.train_id,
                    Some(PieceType::Carriage(c)) => c.train_id != adv.train_id,
                    _ => false,
                };
                if target_is_foreign_cart && !vacating_tiles.contains(head) {
                    trace!(
                        train_id = adv.train_id,
                        ?head,
                        "head landing on non-vacating foreign cart; stops"
                    );
                    blocked_train_ids.push(adv.train_id);
                }
            }

            // Two-train collision: heads coincide, one head lands on
            // another's body, or two trains swap heads through each
            // other (both heading at each other on adjacent tiles).
            //
            // The head-swap case isn't caught by the head-coincidence
            // check (each train's `new_positions[0]` differs) nor by
            // the body-collision check (body iteration is `skip(1)`,
            // which is empty for single-cart trains). Without an
            // explicit guard, phase 1 takes both locos out and phase
            // 3 places each at the *other's* old tile, silently
            // teleporting the trains through each other.
            let mut conflict_train_ids: Vec<u32> = Vec::new();
            for i in 0..advances.len() {
                for j in (i + 1)..advances.len() {
                    let head_i = &advances[i].new_positions[0];
                    let head_j = &advances[j].new_positions[0];
                    let old_head_i = &advances[i].old_positions[0];
                    let old_head_j = &advances[j].old_positions[0];

                    if head_i == head_j {
                        conflict_train_ids.push(advances[i].train_id);
                        conflict_train_ids.push(advances[j].train_id);
                    }
                    if advances[j]
                        .new_positions
                        .iter()
                        .skip(1)
                        .any(|p| p == head_i)
                        || advances[i]
                            .new_positions
                            .iter()
                            .skip(1)
                            .any(|p| p == head_j)
                    {
                        conflict_train_ids.push(advances[i].train_id);
                        conflict_train_ids.push(advances[j].train_id);
                    }
                    // Head-swap: i wants j's loco-tile and j wants i's
                    // loco-tile.
                    if head_i == old_head_j && head_j == old_head_i {
                        conflict_train_ids.push(advances[i].train_id);
                        conflict_train_ids.push(advances[j].train_id);
                    }
                }
            }
            if !conflict_train_ids.is_empty() {
                debug!(?conflict_train_ids, "two-train collision; trains stop");
            }

            // Combine both blocking sets and drop. If nothing new was
            // blocked this iteration, we've reached fixed point.
            let total_blocked = blocked_train_ids.len() + conflict_train_ids.len();
            if total_blocked == 0 {
                break;
            }
            advances.retain(|a| {
                !blocked_train_ids.contains(&a.train_id)
                    && !conflict_train_ids.contains(&a.train_id)
            });
        }

        // Commit in three phases so trailing-train scenarios (A's
        // next_head equals B's vacating tile) resolve correctly
        // regardless of advance iteration order. Per-train commit
        // (the previous shape) failed when A was committed before B:
        // A's `sq.piece = Some(cart)` would overwrite B's caboose,
        // then B's `sq.piece.take()` would steal A's loco.
        //
        // Phase 1: take every moving train's carts out of their old
        // squares so all vacated tiles are uniformly empty.
        // Phase 2: apply captures + the ep-clear heuristic.
        // Phase 3: place every cart on its new tile.

        // Identify the double-pushing pawn for ep-clear (see comment
        // block below for the why).
        let double_pusher_coord = self
            .flags
            .en_passant_target
            .as_ref()
            .and_then(|ep| match self.flags.side_to_move {
                crate::pieces::Color::White if ep.rank > 0 => {
                    Some(Coord { file: ep.file, rank: ep.rank - 1 })
                }
                crate::pieces::Color::Black => {
                    Some(Coord { file: ep.file, rank: ep.rank + 1 })
                }
                _ => None,
            });

        // Phase 1: snapshot + clear.
        let mut snapshots: Vec<(TrainAdvance, Vec<PieceType>)> =
            Vec::with_capacity(advances.len());
        for adv in advances {
            trace!(
                train_id = adv.train_id,
                ?adv.old_positions,
                ?adv.new_positions,
                ?adv.captures,
                "applying train advance"
            );
            let mut moving_carts: Vec<PieceType> =
                Vec::with_capacity(adv.old_positions.len());
            for old in &adv.old_positions {
                let Some(sq) = self.get_square_mut(old) else {
                    continue;
                };
                if let Some(piece) = sq.piece.take() {
                    moving_carts.push(piece);
                }
            }
            snapshots.push((adv, moving_carts));
        }

        // Commit phase 2: captures + ep-clear + corner-rook castle
        // revoke + pending capture-stack events.
        //
        // "Phase" here refers to the *commit* sub-phases inside this
        // function — 1/2/3 = clear / capture / place. Distinct from
        // `make_move`'s pipeline (relocate / piece-effects / env).
        //
        // If the captured tile is *the* pawn whose double-push
        // established `flags.en_passant_target` this same move, null
        // the ep target — otherwise the next turn's opposing pawn
        // could en-passant-capture an already-gone pawn, gaining a
        // diagonal move with no actual capture. `side_to_move` is
        // still the mover here (the flip is the last step in
        // `apply_environment_reactions`); the outer
        // `apply_piece_post_effects` just set ep via the pawn's own
        // `post_move_effects`, which fires only for a same-side
        // double-push. So the pawn lives at
        //   white: (ep.file, ep.rank - 1)  -- moves toward rank 0
        //   black: (ep.file, ep.rank + 1)  -- moves toward rank N
        //
        // Snapshot each victim before clearing so (a) the corner-rook
        // castle revoke can see the captured rook (otherwise a train
        // rolling onto a1/h1/a8/h8 leaves castle rights stale), and
        // (b) the capture stack can fire after placement (so a
        // Kidnapping Goblin run over by a train doesn't bypass
        // GoblinDropVictimCapture).
        let mut pending_captures: Vec<(Coord, PieceType, PieceType)> = Vec::new();
        for (adv, moving_carts) in &snapshots {
            for victim in &adv.captures {
                if double_pusher_coord.as_ref() == Some(victim) {
                    self.flags.en_passant_target = None;
                }
                let victim_piece = self
                    .get_square_mut(victim)
                    .and_then(|sq| sq.piece.take());
                if let Some(v) = &victim_piece {
                    self.maybe_clear_castle_on_rook_capture(victim, v);
                }
                if let (Some(captor), Some(v)) =
                    (moving_carts.first().cloned(), victim_piece)
                {
                    pending_captures.push((victim.clone(), captor, v));
                }
            }
        }

        // Phase 3: place at new positions. Update the locomotive's
        // `last_dir` to the side it entered through — this is what
        // the next tick uses to pick the next exit.
        //
        // NB: as of plan 09 v1 the engine's `next_train_step` only
        // permits a cart to roll onto Track / Junction tiles, so
        // these landings can't actually coincide with a plate
        // today. The call is wired anyway so that `FIRES=N`
        // (Neutral / trains) becomes reachable the moment a future
        // tile type lets a cart settle on a plate, instead of
        // remaining silently dead.
        let mut all_landings: Vec<Coord> = Vec::new();
        for (adv, moving_carts) in snapshots {
            for (i, (mut cart, new_pos)) in moving_carts
                .into_iter()
                .zip(adv.new_positions.iter())
                .enumerate()
            {
                if i == 0 {
                    if let PieceType::Locomotive(loco) = &mut cart {
                        loco.last_dir = Some(adv.new_loco_last_dir);
                    }
                }
                if let Some(sq) = self.get_square_mut(new_pos) {
                    sq.piece = Some(cart);
                }
                all_landings.push(new_pos.clone());
            }
        }

        // Phase 4: fire the capture stack now that trains are placed.
        // Mirrors make_move's relocate → fire_capture_stack order.
        // `captor_origin = None` because the train head has no clean
        // origin tile — its previous tile is now occupied by carriage 1,
        // so the GoblinDropVictimCapture handler's "drop kidnap victim
        // on captor_origin" rule cannot apply (silent loss matches the
        // documented PIC-capture precedent). `captor_coord = victim_coord`
        // because the head landed on the victim's tile.
        // `move_type = MoveTo(victim_coord)` is a synthesized hint; no
        // current handler distinguishes this from a player MoveTo
        // capture.
        //
        // Forward-compat note: a handler emitting `BoardOp::RemovePiece`
        // on `captor_coord` would delete the just-placed locomotive,
        // desyncing the train chain tracked by `iter_pieces()`. No
        // currently-registered handler does this — add a debug guard
        // when the first BoardOp-emitting handler for train captures
        // lands.
        for (victim_coord, captor, victim) in pending_captures {
            let event = ResolutionEvent::Capture {
                captor_coord: victim_coord.clone(),
                captor_origin: None,
                captor,
                victim_coord: victim_coord.clone(),
                victim,
                move_type: MoveType::MoveTo(victim_coord),
            };
            for op in default_capture_stack().resolve_capture(self, &event) {
                op.apply(self);
            }
        }

        for landing in all_landings {
            self.maybe_fire_pressure_plate(&landing);
        }
    }
}

/// Helper for the first-tick fallback in `next_train_step`. Does the
/// cardinal step from `from` in direction `dir` land on a Track or
/// Junction tile that's actually in bounds?
fn direction_leads_to_track(board: &Board, from: &Coord, dir: TrackDir) -> bool {
    let (df, dr) = dir.delta();
    let nf = from.file as isize + df;
    let nr = from.rank as isize + dr;
    if !board.in_bounds(nf, nr) {
        return false;
    }
    let next = Coord {
        file: nf as u8,
        rank: nr as u8,
    };
    matches!(
        board.get_square_at(&next).map(|s| &s.square_type),
        Some(SquareType::Track { .. }) | Some(SquareType::Junction { .. })
    )
}
