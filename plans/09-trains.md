# Plan 09: Trains

User-built train tracks that auto-move trains carrying pieces. Trains are
neutral, run on closed-loop tracks, tick once per full turn (configurable
per board), and capture anything in their path.

Depends on plans 01 (turn system), 02 (king safety), and 08 (signal
substrate — for `TrackDir` and the Junction receiver).

## Concept

- **Track** is a `SquareType` carrying a `TrackDir`. Trains follow the
  chain of `Track` tiles. The direction field is the *outgoing*
  direction — a train arriving on this tile leaves heading that way.
  Curves are expressed as direction changes between adjacent tiles.
- **Locomotive** is a `PieceType` — the head of a train. Knows its
  `train_id` and its `heading` (Forward / Reverse).
- **Carriage** is a `PieceType` — a follower. Knows its `train_id` and
  position in the chain (`chain_index`; 0 = locomotive, 1..N = carriages
  in order).
- **Carts are invincible.** Pieces *in* carts can be captured by enemy
  pieces entering the cart's tile. The cart itself never dies.
- **Trains tick at end of each full turn** by default. `BoardFlags.train_tick_rate`
  configures per-board.
- **Collision:** train enters an occupied tile, occupant is captured.
  Extensibility hooks (see "Collision handlers") let specific pieces or
  square types override later.
- **King safety:** each cart's `attacks()` set includes its **next-tick
  tile**. King-safety filter rejects player moves where the king
  (or a piece carrying the king as a passenger) would land on that
  tile.

## Concrete changes

### `engine/src/pieces/mod.rs` — `Color::Neutral`

Add the third variant:

```rust
pub enum Color {
    White,
    Black,
    Neutral,
}

impl Color {
    pub fn opposite(&self) -> Color {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
            Color::Neutral => Color::Neutral,
        }
    }
}

impl std::fmt::Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Color::White => "white",
            Color::Black => "black",
            Color::Neutral => "neutral",
        })
    }
}
```

Sites to update (compiler will flag the rest):

- `board/fen.rs` — `board_to_fen` writes `w`/`b` for side-to-move.
  Neutral side-to-move is meaningless; assert-or-default if encountered.
  `parse_castle_rights` and `coord_to_algebraic` are color-agnostic
  internally; only the side-to-move byte cares.
- `board/make_move.rs` — `maybe_clear_castle_on_rook_capture`:
  Neutral pieces never hold castle rights; add `Color::Neutral => {}`
  arm.
- `pieces/standard/pawn.rs`, `king.rs` — pawn/king will never be
  Neutral, but the match-on-color sites need to compile. Default arm:
  `Color::Neutral => vec![]` (no moves) or `unreachable!()` if the
  invariant is strong.
- `pieces/fairy/skibidi.rs` — `from_symbol` derives color from case;
  Neutral has no case representation here. Skibidi can't be Neutral;
  leave alone.
- `board/mod.rs` — `is_in_check(Color::Neutral)` should return `false`
  (no neutral king exists). `find_king(Color::Neutral)` returns `None`.

### `engine/src/board/square.rs` — `SquareType::Track`

Plan 08 added `TrackDir`; this plan adds the variant that uses it:

```rust
pub enum SquareType {
    // ...existing including plan-08 additions
    Track {
        direction: TrackDir,
    },
}
```

Walkable per `is_walkable()` (see plan 08): yes. Trains stand on these;
non-train pieces can also walk over them (no movement restriction).

### `engine/src/pieces/fairy/locomotive.rs` (new file)

```rust
use crate::board::{Board, Coord, GameMove};
use crate::pieces::{Color, Piece, piecetype::PieceType};

#[derive(Clone, PartialEq, Debug)]
pub struct Locomotive {
    pub train_id: u32,
    pub heading: TrainHeading,
    pub passengers: Vec<PieceType>,
    /// Direction the cart entered its current tile through. None on
    /// the very first tick; the engine then falls back to a "pick the
    /// unique non-cart neighbor" heuristic. Round-trips through FEN
    /// as the `L=N|S|E|W` field on LOCO.
    pub last_dir: Option<TrackDir>,
}

#[derive(Clone, PartialEq, Debug, Copy)]
pub enum TrainHeading {
    Forward,  // follows each track tile's `direction` field
    Reverse,  // follows the inverse
}

impl Piece for Locomotive {
    fn name(&self) -> &str { "Locomotive" }
    fn color(&self) -> Color { Color::Neutral }
    fn set_color(&mut self, _color: Color) { /* no-op; trains can't recolor */ }
    fn can_carry_piece(&self) -> bool { true }

    /// Locomotives don't emit player-driven moves. Movement happens via
    /// `Board::advance_trains` during `apply_environment_reactions`.
    /// Passengers do emit `PieceInCarrier` moves — mirrors Bus.
    fn initial_moves(&self, board: &Board, from: &Coord) -> Vec<GameMove> {
        // Same passenger-move-generation pattern as Bus::initial_moves —
        // for each passenger, generate moves as if standing on the
        // cart's tile, wrap each in PieceInCarrier.
        // (Copy structure from bus.rs.)
        vec![/* ... */]
    }

    fn attacks(&self, board: &Board, from: &Coord) -> Vec<Coord> {
        // **As implemented** (plan 02 implementation-notes): this
        // sketch's union of passenger + next-tick threats was split.
        // `Locomotive::attacks` / `Carriage::attacks` return *only*
        // the cart's own next-tick tile. Passenger threats are
        // iterated by `Board::is_attacked_by` with a per-passenger
        // color filter so a Black passenger pawn doesn't fake a
        // self-check on the Black king.
        let mut out: Vec<Coord> = Vec::new();
        if let Some(next) = board.next_train_tile(from, self.heading) {
            out.push(next);
        }
        out
    }

    fn symbol(&self) -> String {
        // LOCO(ID=1,H=F,P=(K,R))
        let h = match self.heading {
            TrainHeading::Forward => "F",
            TrainHeading::Reverse => "R",
        };
        let mut s = format!("LOCO(ID={},H={}", self.train_id, h);
        if !self.passengers.is_empty() {
            let p = self.passengers.iter()
                .map(|p| p.symbol())
                .collect::<Vec<_>>()
                .join(",");
            s.push_str(&format!(",P=({})", p));
        }
        s.push(')');
        s
    }

    fn clone_box(&self) -> Box<dyn Piece> { Box::new(self.clone()) }
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }
}
```

### `engine/src/pieces/fairy/carriage.rs` (new file)

Almost identical to Locomotive but with `chain_index` and no `heading`
(it inherits its direction from the cart in front during the tick).

```rust
#[derive(Clone, PartialEq, Debug)]
pub struct Carriage {
    pub train_id: u32,
    pub chain_index: u8,  // 1..255; 0 is the locomotive
    pub passengers: Vec<PieceType>,
}
```

`attacks()` returns *only* the cart's next-tick tile (the tile the cart in
front is currently on — see "Tick logic" below). Passenger threats live in
`Board::is_attacked_by`, which iterates Neutral-carrier passengers separately
with a per-passenger color filter so a Black passenger threatens only for
Black. Same shape as `Locomotive::attacks` otherwise.

### `engine/src/pieces/piecetype.rs`

Add the new variants to `PieceType` and the `dispatch!` macro. Add to
`symbol_to_piece` with FEN dispatch on `LOCO` and `CART` (case-insensitive
since they're neutral):

```rust
pub enum PieceType {
    // ...existing
    Locomotive(crate::pieces::fairy::locomotive::Locomotive),
    Carriage(crate::pieces::fairy::carriage::Carriage),
}

// In symbol_to_piece:
"loco" => Locomotive::from_symbol(symbol),
"cart" => Carriage::from_symbol(symbol),
```

Add `PieceType::is_train_cart(&self) -> bool` returning `true` for
both variants — used by `advance_trains` to find carts quickly.

### `engine/src/board/mod.rs` — `BoardFlags`

```rust
#[derive(PartialEq, Debug, Clone, Copy, Serialize, Deserialize)]
pub enum TrainTickRate {
    EveryPly,
    EveryFullTurn,
    EveryNPly(u8),
}

pub struct BoardFlags {
    // ... existing
    pub train_tick_rate: TrainTickRate,
    /// Monotonically increasing ply counter; resets only on board reset.
    /// Used by `maybe_advance_trains` to decide when to tick.
    pub ply_count: u32,
}
```

Default `train_tick_rate = EveryFullTurn`. Default `ply_count = 0`.

FEN encoding: a new field after `ep`:

```
<grid> <stm> <castling> <ep> <train_tick> <ply>
```

Examples: `... w KQkq - tr=full p=4`, `... b - - tr=2ply p=11`.

Back-compat: missing trailing fields fall back to defaults
(`EveryFullTurn`, `0`).

### `engine/src/board/trains.rs` (new file)

The tick logic, mirroring `brainrot.rs`. This is the load-bearing piece
of the plan.

```rust
use tracing::{debug, trace};
use crate::board::{Board, Coord, square::SquareType};
use crate::pieces::piecetype::PieceType;
use crate::pieces::fairy::locomotive::TrainHeading;
use crate::board::square::TrackDir;

impl Board {
    /// **As implemented:** the entry point is `next_train_step(from, heading,
    /// last_dir) -> Option<(Coord, TrackDir)>`, returning both the next tile
    /// and the direction the cart entered it through (which becomes the
    /// loco's `last_dir` on the next tick). `next_train_tile` is a thin
    /// coord-only wrapper kept for the existing call sites in tests. The
    /// `last_dir` argument is what enables minecart-style neighbor-aware
    /// curve resolution. The sketch below is from before that refactor.
    ///
    /// Resolve the next track tile given the current tile + a heading.
    /// Returns None if the current tile isn't a Track (or Junction), or
    /// if the resulting tile is off-board / not a track tile (derailment).
    pub fn next_train_tile(&self, from: &Coord, heading: TrainHeading) -> Option<Coord> {
        let sq = self.get_square_at(from)?;
        let dir = match &sq.square_type {
            SquareType::Track { direction } => *direction,
            SquareType::Junction { state, branches, .. } => {
                *branches.get(*state as usize)?
            }
            _ => return None,
        };
        let dir = match heading {
            TrainHeading::Forward => dir,
            TrainHeading::Reverse => dir.opposite(),  // new helper on TrackDir
        };
        let (df, dr) = dir.delta();  // new helper: (isize, isize)
        let nf = from.file as isize + df;
        let nr = from.rank as isize + dr;
        if !self.in_bounds(nf, nr) { return None; }
        let next = Coord { file: nf as u8, rank: nr as u8 };
        // Derailment check: next tile must itself be a track or junction
        let next_sq = self.get_square_at(&next)?;
        match &next_sq.square_type {
            SquareType::Track { .. } | SquareType::Junction { .. } => Some(next),
            _ => None,
        }
    }

    /// Called from `apply_environment_reactions` (phase 3 of
    /// make_move; see `engine/src/board/make_move.rs`). Checks the
    /// tick rate and the ply counter to decide whether to advance.
    ///
    /// What landed (in this iteration) differs from the original
    /// sketch:
    ///   - `saturating_add(1)` instead of `+= 1`, with a one-line
    ///     warn at u32::MAX so a runaway test catches it.
    ///   - `(n as u32).max(1)` clamp for `EveryNPly`, defending
    ///     against a 0-ply rate the FEN parser would otherwise
    ///     accept; the parser also rejects `tr=0ply` directly.
    ///   - Returns `bool` instead of `()` — `true` iff the rate
    ///     gate fired and the trains actually advanced. The caller
    ///     (`apply_environment_reactions`) uses this to gate a
    ///     downstream `recalc_brainrot` so a no-tick move doesn't
    ///     pay for an unnecessary O(N²) recalc.
    pub fn maybe_advance_trains(&mut self) -> bool {
        self.flags.ply_count = self.flags.ply_count.saturating_add(1);
        let should_tick = match self.flags.train_tick_rate {
            TrainTickRate::EveryPly => true,
            TrainTickRate::EveryFullTurn => self.flags.ply_count % 2 == 0,
            TrainTickRate::EveryNPly(n) => {
                let n = (n as u32).max(1);
                self.flags.ply_count % n == 0
            }
        };
        if should_tick {
            self.advance_trains();
        }
    }

    /// Advance every train one step along its track.
    pub fn advance_trains(&mut self) {
        // 1. Collect carts: Vec<(Coord, train_id, chain_index, is_locomotive, heading_if_loco)>.
        // 2. Group by train_id, sort by chain_index ascending.
        // 3. Snapshot each train's pre-tick positions.
        // 4. For each train:
        //    a. Compute locomotive's next tile via next_train_tile.
        //    b. If None: train stops this tick.
        //    c. Run collision chain on next-tile occupant (if any).
        //       Outcome decides whether the train moves and what
        //       happens to the occupant.
        //    d. Move locomotive to next tile (clear old, place new).
        //    e. Each carriage moves to the pre-tick position of the
        //       cart that was in front of it (chain_index - 1).
        //    f. Two-train collision: if two trains' computed next-tiles
        //       coincide this tick, both trains stop. Detect by
        //       building the full next-tile map first, then committing.
    }
}
```

The snapshotting in step 3 + the deferred-commit in step 4f are the
key correctness moves:

- **Snapshotting** keeps carriages from cutting corners. Carriage at
  index 2 moves to where carriage at index 1 *was*, not where it *is*
  now.
- **Deferred commit** is needed for two-train collision: you can't
  know whether train A wants to enter tile X until you've also asked
  train B. Build a `Vec<TrainAdvance>` of proposed moves, scan for
  conflicts, then apply.
- **Foreign-cart check**: a moving train's head can't land on a tile
  occupied by another train's cart unless that tile is being *vacated*
  this same tick by its current occupant. Without this, a stalled
  foreign cart on the landing tile would be silently overwritten by
  the commit pass. The trailing-train case (A follows B east, B's
  caboose vacates the tile A wants) is allowed by computing
  `vacating_tiles` from the surviving advances. The foreign-cart
  filter + two-train collision pass both *drop* trains from
  `advances`, which changes `vacating_tiles`, so the two passes run
  to fixed point — re-check until no more trains get blocked.
- **Three-phase commit** is needed because per-train commit
  (interleaved take + place) is unsound under trailing-train ordering:
  if A's `sq.piece = Some(cart)` fires before B's `sq.piece.take()`,
  A overwrites B's caboose, then B steals A's loco. Split into:
  (1) take all moving carts out of their old squares, (2) apply
  captures + ep-clear heuristic, (3) place every cart on its new
  tile. After phase 1 every vacated tile is empty regardless of
  iteration order; phase 3 lands cleanly. See `advance_trains`.

### Collision handlers

**v1 status: deferred.** The trait chain below is *not* implemented.
The equivalent v1 behavior is hard-coded in
`engine/src/board/trains.rs::advance_trains`:
- `own_cart_collision` check stands in for the locomotive's
  `on_run_over_target` returning `Stop` when the target is a cart of
  the same train.
- The unconditional capture in the commit phase is the default
  `Capture` outcome for any non-cart victim.
- Per-piece overrides (`Piece::on_being_run_over`) don't exist; if
  added later, fold them into the per-train decision loop.

A chain of hooks called in order. Each returns a `CollisionOutcome`:

```rust
#[derive(PartialEq, Debug, Clone)]
pub enum CollisionOutcome {
    Default,   // no opinion; continue chain
    Capture,   // occupant is removed; train advances
    Stop,      // train doesn't advance
}
```

Three layers, called in order, first non-`Default` wins:

1. **The train.** A `Piece::on_run_over_target(...)` method on the
   Locomotive variant. v1 default for Locomotive: `Capture`. Future
   "passenger train" variants override.
2. **The piece being run over.** A `Piece::on_being_run_over(...)`
   method with default `Default`. Specific pieces override (e.g. Goblin
   drops its kidnap victim before dying).
3. **The square.** Not implemented in v1; reserved.

```rust
trait Piece {
    // ... existing
    fn on_run_over_target(&self, ctx: &CollisionCtx) -> CollisionOutcome {
        CollisionOutcome::Default
    }
    fn on_being_run_over(&self, ctx: &CollisionCtx) -> CollisionOutcome {
        CollisionOutcome::Default
    }
}
```

If both layers return `Default`, the final fallback is `Capture`
(consistent with "train is unstoppable force, default crushes
everything").

v1 implements only the Locomotive's `on_run_over_target` returning
`Capture`. The other hooks exist with `Default` defaults so layer 2 can
be wired in piece-by-piece as needs arise.

### King safety

Already handled by the `attacks()` extension: each cart attacks its
next-tick tile. Existing `is_attacked_by` / `legal_moves` filter does
the rest.

One subtlety: if the player's move causes the train to derail (e.g. by
boarding the train so they're now a passenger but didn't affect the
track), the train's next-tick tile may shift between "this move's
attack set" and "post-move attack set." Plan 02's filter recomputes
king-safety on the post-move board state, so this is correct
automatically.

For multi-ply tick rates (`EveryNPly(n)` with `n > 1`), the next train
tick may not happen on the current player's turn. Be conservative: the
train's `attacks()` still reports the next-tick tile, even if the tick
is several plies away. Better to forbid a king move into a future
crushing zone than allow it.

### `engine/src/board/make_move.rs`

As of this iteration, the post-move pipeline is split into:
- **`apply_piece_post_effects`** — clears the en-passant target, runs
  each piece's `post_move_effects`, fires pressure plates, then
  `recalc_brainrot`.
- **`apply_environment_reactions`** — calls `maybe_advance_trains`,
  then flips `side_to_move`. Skipped on `validate_move`'s clone so
  king-safety isn't masked by a hypothetical train tick.

So `maybe_advance_trains` lives in `apply_environment_reactions`,
not the old `handle_post_move_effects`.

Order is still: brainrot first (so any post-move brainrot zone applies
to passenger move-gen on next turn), then trains.

### FEN

New piece symbols (extending the existing `(P=...)` extended block):

```
(P=LOCO(ID=1,H=F,P=(K,R)))
(P=CART(ID=1,I=1,P=(P)))
```

New square type:

```
(T=TRACK,D=E)
```

A locomotive sitting on a track tile combines both:

```
(T=TRACK,D=E,P=LOCO(ID=1,H=F))
```

The existing `split_top_level` + key-value parsing handles this without
modification.

## Tests

In `engine/src/board/tests.rs`:

- `test_neutral_color_serialization` — Locomotive FEN round-trip; color
  reads as `Color::Neutral` after parse.
- `test_train_advances_one_tile_per_full_turn` — single-cart train on a
  3-tile linear track. One full turn (one white move, one black move).
  Train moved one tile.
- `test_train_advances_per_ply_when_configured` — same setup but
  `train_tick_rate = EveryPly`. After one ply, train moved.
- `test_train_loops_on_closed_track` — 4-tile loop, 4 ticks, train back
  at start.
- `test_train_runs_over_piece` — track passes through a Pawn's square.
  Tick. Pawn is gone; train occupies the Pawn's old tile.
- `test_train_stops_on_derailment` — track ends at edge. Train reaches
  end. Next tick: train doesn't move; ply counter still advances.
- `test_two_trains_converge_both_stop` — trains A and B approach the
  same tile from different directions. Both stop. Both ply counters
  still advance.
- `test_king_cannot_walk_into_train_next_tile` — king's only legal-look
  move is into the tile the train will occupy next tick. Move is
  rejected with `WouldLeaveKingInCheck`.
- `test_passenger_captured_by_enemy_entering_cart` — friendly Pawn in
  cart at (3,3). Enemy Knight moves to (3,3). After move: Knight is at
  (3,3) (as a normal capture), Pawn is gone. (Note: the *cart* doesn't
  go anywhere — this is the cart-stays-invincible rule.)

  Actually: rethink. The cart sits on a track tile, the enemy Knight
  moves *to the track tile* — does that "land on the cart" or "land on
  the track tile underneath"? Decision: the cart is the topmost
  occupant. Knight lands on the cart, which is equivalent to "boarding
  by capture." Cart now has Knight instead of Pawn. Same FEN slot.
- `test_multi_cart_train_curves_correctly` — 3-cart train through a 90°
  curve. After three ticks, all three carts traversed in sequence; no
  corner cutting.
- `test_junction_diverts_train` — train approaches junction with
  `state=0`, goes branch[0]. Throw switch (plan 08). Same train next
  tick goes branch[1].
- `test_train_does_not_run_over_own_cart` — short loop with long train
  (loop length == cart count). Locomotive's next tile is its own
  caboose. Collision outcome: `Stop` (train can't capture itself).
- `test_ply_count_in_fen` — ply counter round-trips through FEN.

## Things to be careful about

- **Cart length vs loop length.** A train of N carts on a loop of
  length M ≤ N has the locomotive catching its own tail. Editor
  validation: refuse to save. Runtime: train hits its own caboose →
  Stop (locomotive's `on_run_over_target` checks if the target is a
  cart of its own train, returns `Stop`).
- **Train on a non-track tile.** Editor lets you paint a Locomotive
  anywhere; runtime should handle gracefully. If `next_train_tile`
  returns None because the *current* tile isn't a track, the train
  just stops. Document this as "trains not on track sit still."
- **The cart tile is the topmost piece.** When the engine asks "what's
  at (3,3)" and gets a Locomotive back, the passenger list inside is
  accessible via the Locomotive. A king inside a cart shows up via
  `find_king`'s descent (see existing Bus handling in `find_king`).
  Locomotive and Carriage need the same recursion. **Add them to the
  `find_king` Bus-descent branch.**
- **Train under a passenger move.** Passenger inside a cart moves out
  via `PieceInCarrier`. Order on a turn: player moves passenger out,
  then `maybe_advance_trains` ticks. So passenger is on the destination
  tile, train moves on, no longer carrying that passenger. Correct
  behavior — document.
- **Train collision with phase-4 Skibidi.** Open spec question — see
  open questions below.
- **`Color::Neutral` ripple.** This change is touched on in §
  `Color::Neutral` above but it's the most cross-cutting change. Plan
  on a full afternoon hunting compile errors after the variant lands.
  The compiler is your friend; let exhaustive matching surface the
  sites.
- **Heading reversal.** `TrackDir::opposite()` is straightforward for
  cardinals (N↔S, E↔W). When/if diagonals land, the reverse mapping is
  the same.
- **FEN field ordering.** The new `tr=` and `p=` fields are append-only
  to the FEN flags. Always write in a fixed order, always tolerate
  missing trailing fields on read.

## Open questions

1. **Locomotive heading reversal mechanic.** A `TrainHeading::Reverse`
   field exists but v1 has no in-game way to set it. Options:
   - Stays Forward forever in v1 (set at editor time, immutable).
   - Add a "reverser" Switchable that toggles a train's heading when
     fired.
   - Player action `MoveType::ReverseTrain { train_id }`.

   v1: heading is set at editor time and never changes. Reserve for later.

2. **Carriage detaching.** Can a player decouple a cart mid-game?
   Mechanic: bump chain_index of all following carts down by N, leaving
   N free carts behind. Skip for v1.

3. **Phase-4 Skibidi run over by train.** The brainrot-lockout rule
   says capturing a phase-4 Skibidi ends the game in favor of the
   capturing side. A neutral train captures it — which side wins? No
   capturing side exists. Decision needed. Recommend: the train's
   capture *does not* trigger the brainrot-lockout rule (the rule is
   about *player* captures). The Skibidi just dies normally. Document
   in plan 04's brainrot section as a known interaction.

4. **Per-train tick rates.** v1 uses board-global rate. If/when needed,
   move the rate field onto the Locomotive struct; `maybe_advance_trains`
   then dispatches per-train.

5. **Boarding from adjacent.** A train rolls past a piece without
   stopping. Can the piece "grab the train" as it passes? Adds depth
   but breaks the cleanly-symmetric "move onto cart = board it" rule.
   Skip for v1.

6. **Train-vs-train at a junction.** Two trains targeting the same
   junction from different incoming directions. Today: both stop. Could
   become "train with lower train_id wins" if "both stop" turns out
   unsatisfying in practice. Revisit after playtest.

7. **King in cart, cart enters dangerous tile.** Train ticks the king
   into a tile attacked by an enemy. King-safety filter only validates
   *player* moves, not auto-moves. Should the train's auto-tick refuse
   to move into a tile that would put the king in check? Recommend:
   yes — `advance_trains` runs a king-safety check after the proposed
   move set and stops any train whose advance would create check on
   its king-passenger's side. This is a one-tick safety net; if no
   movement satisfies the constraint, the train stops. Detail this
   when wiring step 6 (king-safety integration).

   **Current behavior (this iteration):** the recommendation is NOT
   implemented. Two related behaviors are pinned by tests as the
   *current* (about-to-be-changed) state:
   - The train tick does NOT consult king-safety before advancing.
   - `MoveIntoCarrier` (when an enemy boards a Neutral cart) silently
     removes opposite-color passengers, including a king. Pinned by
     `test_king_passenger_captured_when_enemy_boards_cart`.
   When the safety-net recommendation lands, both behaviors change
   and the pinning test will need to flip from "king removed" to
   "move illegal" / "train stops."

## Build order

1. `Color::Neutral` — small, isolated commit. Let the compiler audit
   every match site.
2. `SquareType::Track { direction }` + FEN round-trip. Editor paints
   tracks; nothing else changes yet.
3. `Locomotive` piece (no carriages, no junctions) + `BoardFlags`
   train fields + `maybe_advance_trains` + straight-track `advance_trains`.
   Default collision: capture. Verify: train moves one tile per turn,
   captures pieces in its path, stops at derail.
4. `Carriage` piece + multi-cart sequencing in `advance_trains`. Verify
   chain-following on straight track.
5. Curves — already works, since `advance_trains` reads each tile's
   direction independently. Add tests.
6. Junction traversal (consume plan 08's Junction state). Adds a
   read-only branch in `next_train_tile`.
7. King-safety integration: `Locomotive::attacks` / `Carriage::attacks`
   return the next-tick tile. Verify `is_attacked_by`, `legal_moves`,
   and `validate_move` flow correctly.
8. Collision handler chain (default impl from step 3; this step adds
   the trait hooks so specific pieces can override). Add `Stop` outcome
   for "train hits own caboose."
9. Editor support (frontend) — paint Track tiles, place Loco/Cart,
   wire train_id. Out of engine scope; flag for the frontend track.

## Notes

This plan is large. Steps 1–3 are the minimum-viable trains;
4–8 each add real capability. Step 9 (frontend) is the user-facing
payoff but lives in `frontend/vite-dev/`. Treat the engine and frontend
work as separate trackable units — engine ships first, frontend
catches up.
