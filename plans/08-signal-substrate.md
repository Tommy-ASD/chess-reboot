# Plan 08: Signal substrate

A generic event/wiring system. Switches fire signals at receivers;
junctions, gates, and other receivers react. Trains (plan 09) are the
first consumer that needs this, but the substrate is independent and
lands first.

Depends on plan 01 (turn system) — `ThrowSwitch` consumes a player's
turn.

## Concept

Two roles:

- **Emitter** — a square type that can produce a signal. Stores a list
  of receiver IDs to fire when triggered.
- **Receiver** — a square type that responds to a signal. Stores its own
  ID plus internal state that the signal mutates.

Wiring is by opaque `SignalId` (u32), not by coord. IDs let the editor
draw wires and detect dangling references, and they're stable even if a
square's position ever shifts (no current mechanism, but free
future-proofing).

Many-to-many wiring: one switch can target multiple receivers
(`Vec<SignalId>`); multiple switches can target the same receiver.
Receivers don't track their inverse-mapping at runtime — the editor
reconstructs it at load time by scanning all emitters and uses it for UX
(warn when a receiver has no controllers, list controllers on hover,
etc.).

**Bounded propagation.** An emitter fires its receivers; receivers update
state. Receivers cannot themselves emit during the same call. This
forbids cascades (and the cycles they enable) in v1. If chain triggers
are ever wanted later, add an explicit "delayed-fire" emitter type.

## Concrete changes

### `engine/src/board/mod.rs`

Add the signal ID type alias and the new move type:

```rust
pub type SignalId = u32;

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", content = "target")]
pub enum MoveType {
    // ...existing
    ThrowSwitch { switch: Coord },
}
```

`ThrowSwitch.switch` is technically redundant with `GameMove.from` (the
piece on the switch tile is throwing it), but explicit beats implicit.
Lets a future "switch from adjacent tile" mechanic extend the move
shape without breaking the type.

Add the `Display` impl for the new variant:

```rust
MoveType::ThrowSwitch { switch } => write!(f, "throw switch at {switch}"),
```

### `engine/src/board/square.rs` — payload-carrying `SquareType`

The current `SquareType` is a no-payload enum. Switching to
payload-carrying variants is a **breaking change to every match site**.
The existing variants stay no-payload; the new ones carry data.

```rust
pub enum SquareType {
    Standard,
    Turret,
    Vent,
    Switch {
        targets: Vec<SignalId>,
    },
    Junction {
        id: SignalId,
        /// Index into `branches`. Cycled by signal activation.
        state: u8,
        branches: Vec<TrackDir>,
    },
    Gate {
        id: SignalId,
        open: bool,
    },
    PressurePlate {
        targets: Vec<SignalId>,
        fires_for: PressureTrigger,
    },
}

#[derive(PartialEq, Debug, Clone, Copy, Serialize, Deserialize)]
pub enum TrackDir { N, S, E, W }

#[derive(PartialEq, Debug, Clone)]
pub enum PressureTrigger {
    AnyPiece,
    OnlyColor(Color),
}
```

`TrackDir` is also used by `Track` in plan 09; lives in this file so
both plans pull from one place. Diagonals (NE/SE/SW/NW) are deferred —
v1 trains only run on cardinal directions, junctions only switch between
cardinals.

Update `SquareType::as_str` to handle the new variants. The existing
`as_str` returns a static string — fine for `Standard/Turret/Vent`, not
expressive enough for payload-carrying variants. Rename to
`type_tag(&self) -> &'static str` (returns "SWITCH" / "JUNCTION" / etc.,
matching the FEN tag) and let the FEN encoder build the full
parenthesized form from the variant's data.

### `engine/src/board/fen.rs`

`square_to_fen` and `fen_to_square` already handle key=value pairs in
parenthesized blocks. Extend `fen_to_square`'s field-dispatch to parse
the new variant data:

```
(T=SWITCH,TARGETS=(3,7))
(T=JUNCTION,ID=3,STATE=0,BRANCHES=(N,E))
(T=GATE,ID=7,OPEN=0)
(T=PLATE,TARGETS=(3),FIRES=ANY)
```

`split_top_level` already nests-safely; `TARGETS=(3,7)` parses as a
single field whose value is `(3,7)`. Add a helper:

```rust
fn parse_id_list(v: &str) -> Vec<SignalId> {
    let Some(inner) = v.strip_prefix('(').and_then(|s| s.strip_suffix(')')) else {
        warn!(v, "malformed id list; expected (...)");
        return vec![];
    };
    split_top_level(inner)
        .iter()
        .filter_map(|s| s.parse::<u32>().ok())
        .collect()
}
```

Mirror that in `square_to_fen` for serialization. The output is
canonical: write fields in a fixed order (`ID`, `STATE`, `BRANCHES`,
`TARGETS`, `OPEN`, `FIRES`) so round-trips are deterministic.

### `engine/src/board/signal.rs` (new file)

The dispatcher. Lives alongside `brainrot.rs`, which is the closest
precedent (board-level recompute triggered from `handle_post_move_effects`).

```rust
use tracing::{debug, trace};
use crate::board::{Board, SignalId};
use crate::board::square::SquareType;

impl Board {
    /// Fire a signal pulse at the given target IDs. Each matching
    /// receiver's reaction runs once. Bounded propagation: receivers
    /// cannot themselves fire during this call.
    pub fn fire_signal(&mut self, targets: &[SignalId]) {
        debug!(?targets, "firing signal");
        for target_id in targets {
            self.activate_receiver(*target_id);
        }
    }

    /// Linear scan over the grid. At 8x8 this is fine; cache to a
    /// HashMap<SignalId, Coord> on Board if boards grow significantly.
    fn activate_receiver(&mut self, id: SignalId) {
        for row in &mut self.grid {
            for sq in row {
                match &mut sq.square_type {
                    SquareType::Junction { id: jid, state, branches } if *jid == id => {
                        let new_state = (*state + 1) % branches.len() as u8;
                        trace!(id, old = *state, new = new_state, "junction advanced");
                        *state = new_state;
                    }
                    SquareType::Gate { id: gid, open } if *gid == id => {
                        trace!(id, old = *open, "gate toggled");
                        *open = !*open;
                    }
                    _ => {}
                }
            }
        }
    }
}
```

### `engine/src/board/make_move.rs`

Handle `MoveType::ThrowSwitch` in the apply loop:

```rust
MoveType::ThrowSwitch { switch } => {
    let targets = {
        let sq = self.get_square_at(switch)
            .ok_or_else(|| format!("ThrowSwitch: no square at {:?}", switch))?;
        match &sq.square_type {
            SquareType::Switch { targets } => targets.clone(),
            other => return Err(format!(
                "ThrowSwitch target {:?} is not a Switch tile (got {:?})",
                switch, other
            )),
        }
    };
    self.fire_signal(&targets);
    debug!(?switch, ?targets, "switch thrown");
}
```

Crucially: `ThrowSwitch` does **not** move the piece. The piece stays
on the switch tile.

Then in `handle_post_move_effects`, scan for pressure plates the moved
piece settled onto:

```rust
// After piece_target is resolved (existing variable):
if let Some(target) = &piece_target {
    let plate_targets = self.get_square_at(target).and_then(|sq| {
        if let SquareType::PressurePlate { targets, fires_for } = &sq.square_type {
            if self.piece_matches_trigger(target, fires_for) {
                Some(targets.clone())
            } else { None }
        } else { None }
    });
    if let Some(targets) = plate_targets {
        self.fire_signal(&targets);
    }
}
```

`piece_matches_trigger` is a new helper on `Board` that resolves
`PressureTrigger` against the piece at the given coord.

### `engine/src/board/mod.rs` — `Board::get_moves`

Currently `get_moves` is a thin wrapper over `Piece::initial_moves`.
`ThrowSwitch` is a *square*-driven move, not piece-driven. Extend the
board-level entrypoint:

```rust
pub fn get_moves(&self, from: &Coord) -> Vec<GameMove> {
    let Some(square) = self.get_square_at(from) else { return vec![]; };
    if square.conditions.contains(&SquareCondition::Brainrot)
        || square.conditions.contains(&SquareCondition::Frozen) {
        return vec![];
    }
    let Some(piece) = &square.piece else { return vec![]; };

    let mut moves = piece.get_moves(self, from);

    // Square-driven additions: piece on a Switch can throw it.
    if let SquareType::Switch { .. } = square.square_type {
        moves.push(GameMove {
            from: from.clone(),
            move_type: MoveType::ThrowSwitch { switch: from.clone() },
        });
    }

    moves
}
```

The `Piece::can_throw_switch() -> bool` hook (default `true`) lets
specific pieces opt out later; not used in v1 but lays the wire.

### `engine/src/pieces/piecetype.rs` — `get_moves` filter

The carrier-rewrite filter walks `MoveType` variants. Add a passthrough
for `ThrowSwitch`:

```rust
MoveType::ThrowSwitch { .. } => return true,
```

(Same shape as the existing PhaseShift / Castle / EnPassant arms.)

### `engine/src/board/mod.rs` — `validate_move`

Add a final case for `ThrowSwitch` after the existing legality checks
flow:

- Source square exists ✓ (existing check)
- Source has a piece ✓ (existing)
- Piece is the side-to-move ✓ (existing)
- For `ThrowSwitch`: source square is a `Switch` tile, and the piece
  can throw switches (`piece.can_throw_switch()`).

No king-safety check is needed — throwing a switch doesn't move any
piece, so it can't put your own king into check by itself. (Indirect
consequences from junctions diverting a train *next* turn fall under
plan 09's `validate_move` extension.)

## Tests

In `engine/src/board/tests.rs`:

- `test_switch_fires_junction` — board with a Switch at (0,0) targeting
  junction ID 1 at (0,1). Pawn on the Switch. Throw the switch. Assert
  the junction's `state` advanced by 1, modulo branches.len().
- `test_switch_fires_multiple_targets` — Switch with `targets = [1, 2]`,
  two junctions with those IDs. Throw. Both states advance.
- `test_junction_cycles_modulo` — Junction with `branches.len() == 2`.
  Throw the switch three times. State sequence: 0 → 1 → 0 → 1.
- `test_pressure_plate_fires_on_step` — Pawn moves onto a
  `PressurePlate` square. Plate's targets fire as part of the move.
- `test_pressure_plate_color_restriction` — Plate with
  `fires_for = OnlyColor(White)`. Black piece steps on it: plate does
  *not* fire.
- `test_dangling_target_silently_ignored` — Switch targets ID 99, no
  receiver exists. Throwing must not panic; junctions/gates remain
  unchanged.
- `test_throw_switch_consumes_turn` — White throws a switch; subsequent
  white move is rejected (it's black's turn now).
- `test_throw_switch_invalid_when_not_on_switch_tile` — piece is on a
  `Standard` tile but emits `ThrowSwitch`. Validation rejects.
- `test_no_propagation_cascade` — Plate fires gate. Gate opens. The
  gate opening does not itself fire anything. (Awkward to write before
  any chainable emitter exists; add as a regression test once one ships.)
- `test_signal_fen_roundtrip` — board with each new square variant,
  FEN-encode, parse back, assert structural equality including
  `targets`, `id`, `state`, `branches`, `open`.

## Things to be careful about

- **Every match arm on `SquareType` needs updating.** The compiler will
  flag them. Hot spots: `fen.rs::square_to_fen` (the formatter),
  `make_move.rs` (no places match `SquareType` directly today, but the
  brainrot recalc indirectly relies on `square_is_empty` which calls
  `square_type == SquareType::Standard`). Audit `square_is_empty`'s
  semantics: should a Switch tile count as "empty" for movement
  purposes? Probably yes — a Switch is just decoration on a Standard
  tile from a movement standpoint. Same for Gate (when open) and Plate.
  Junction tiles are walkable (trains go through them). Decision:
  treat all new variants as empty-equivalent for `square_is_empty`,
  except `Gate { open: false }`, which blocks.
- **Borrow-checker fight in `fire_signal`.** Reading `targets` from the
  emitter while mutably scanning for receivers means the implementation
  above clones `targets` out before the mutable scan. Same pattern as
  `recalc_brainrot`.
- **ID collision detection.** Two junctions sharing an ID: when a switch
  fires that ID, *both* respond. Predictable, but probably not what the
  designer wanted. Recommendation: editor refuses to save with duplicate
  IDs *within* receiver types (two `Junction { id: 5 }` is an error;
  one `Junction { id: 5 }` and one `Gate { id: 5 }` is fine — different
  receiver kinds). Runtime: fire all matches; document the behavior.
- **ID generation in the editor.** When the user paints a new junction,
  the editor picks the lowest unused `SignalId` across all receivers
  on the board. Stored alongside the square.
- **`square_is_empty` and `SquareType::Standard`.** The current
  implementation hard-codes `SquareType::Standard`. With more variants,
  this matters more. Either:
  - Add a helper `SquareType::is_walkable(&self) -> bool` and consult
    it (cleaner).
  - Enumerate all walkable variants in `square_is_empty` (more
    explicit, more sites to update when adding new types).
  Recommend the helper.

## Open questions

1. **Switch direction control.** Junctions cycle through branches. Some
   level designs may want explicit "set junction X to branch Y."
   Options:
   - Keep cycling. Designers build with the constraint.
   - Add `MoveType::ThrowSwitchToState { switch, state: u8 }`.

   v1 ships cycling. Add the explicit variant if a real need shows up.

2. **Free-action throwing.** Currently throwing a switch consumes a
   turn. If a piece *moves onto* a Switch, should the move automatically
   throw the switch as part of entering? More fluid but loses the
   tempo trade-off. Recommend: keep them separate.

3. **Color-restricted switches.** "Only White can throw this switch."
   Useful for asymmetric levels. Add `restrictor: Option<Color>` to
   `SquareType::Switch` later. Out of v1 scope.

4. **Editor visualization.** When the editor shows a switch, it should
   render wires/lines to every target. With many switches + many
   targets the view gets noisy; hover-to-highlight is probably the
   right UX. Engine-side, this just means the API needs to expose the
   wiring graph cleanly — which falls out of FEN already.

## Build order

1. `SignalId` type alias + payload-carrying `SquareType` variants +
   `SquareType::is_walkable` helper. No dispatcher yet — verify everything
   still compiles + FEN round-trips.
2. `MoveType::ThrowSwitch` + `Board::get_moves` extension + filter
   passthrough. Verify a piece on a Switch tile gets a `ThrowSwitch` in
   its legal-move list.
3. `signal.rs` + `fire_signal` + receiver activation. Wire into
   `make_move.rs`. Junction + Gate respond correctly.
4. PressurePlate + the post-move scan in `handle_post_move_effects`.
5. Tests for all of the above. Plan 09 picks up the Junction state for
   train traversal.
