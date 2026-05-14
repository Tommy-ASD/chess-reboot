# Plan 02: King safety — check, pinned pieces, checkmate, stalemate

Blocks on plan 01 (turn system). The current engine lets the king walk
into attacked squares and lets pinned pieces move freely.

## Three layers

### Layer A: attack detection

Given a board and a square, is that square attacked by any piece of a
given color?

```rust
impl Board {
    pub fn is_attacked_by(&self, target: &Coord, attacker: Color) -> bool {
        for (coord, piece) in self.all_pieces() {
            if piece.get_color() != attacker { continue; }
            let moves = piece.get_moves(self, &coord);
            for m in &moves {
                if let MoveType::MoveTo(c) = &m.move_type {
                    if c == target { return true; }
                }
            }
        }
        false
    }
}
```

This is the naive O(N · M) implementation. Fine for an 8×8 board; do not
prematurely optimize. Pieces' `get_moves` already does the work of "where
could this piece move?", so reusing it is the right call.

**Subtlety**: `get_moves` calls the filter, which drops same-color
targets. For attack detection we want the *raw* threats — including
the target square's current occupant. Two ways:

1. Add a `Piece::attacks(&self, board, from) -> Vec<Coord>` method that
   skips the same-color filter.
2. Compute `get_moves` on a board where the target square has been
   temporarily emptied or filled with a sacrificial enemy piece.

Option 1 is cleaner. The filter already separates "the move is generated"
from "the move is kept." Adding an attacks() helper makes that distinction
public.

**Edge case — pawn**: a pawn's `get_moves` includes forward pushes, but a
pawn does *not* attack the square in front of it (only the diagonals).
`attacks()` for pawn must return only the diagonal squares.

### Layer B: king-in-check + move filtering

After every move generation, filter out moves that leave the mover's own
king in check.

```rust
impl Board {
    pub fn is_in_check(&self, color: Color) -> bool {
        let king_coord = self.find_king(color);
        match king_coord {
            Some(c) => self.is_attacked_by(&c, color.opposite()),
            None => false,  // no king on board — defensive
        }
    }

    fn find_king(&self, color: Color) -> Option<Coord> {
        self.all_pieces().into_iter().find_map(|(coord, piece)| {
            if let PieceType::King(k) = piece {
                if k.color == color { return Some(coord); }
            }
            None
        })
    }
}
```

Then add a `legal_moves()` method that calls `get_moves` and filters out
moves leaving the king in check:

```rust
impl Board {
    pub fn legal_moves(&self, from: &Coord) -> Vec<GameMove> {
        let raw = self.get_moves(from);
        let moving_color = match self.get_square_at(from).and_then(|s| s.piece.as_ref()) {
            Some(p) => p.get_color(),
            None => return vec![],
        };
        raw.into_iter()
            .filter(|m| {
                let mut hypothetical = self.clone();
                // make_move would check is_valid_move; we already know it's in raw,
                // so call a lower-level apply that skips validation.
                if hypothetical.make_move_unchecked(m.clone()).is_ok() {
                    !hypothetical.is_in_check(moving_color)
                } else {
                    false
                }
            })
            .collect()
    }
}
```

Note `make_move_unchecked` doesn't exist yet — split the current
`make_move` into the validation path and the mutation path so this
filter can use the latter without re-validating.

Plan 01's `is_valid_move` should switch to call `legal_moves(from)`
instead of `get_moves(from)` so that pin/check filtering applies.

### Layer C: game-over detection

```rust
pub enum GameStatus {
    Ongoing,
    Check { side_to_move: Color },  // added during implementation — see note below
    Checkmate { winner: Color },
    Stalemate,
    BrainrotWin { winner: Color },  // see plan 04
}

impl Board {
    pub fn status(&self) -> GameStatus {
        let to_move = self.flags.side_to_move;
        let any_legal = self.all_pieces().iter()
            .filter(|(_, p)| p.get_color() == to_move)
            .any(|(coord, _)| !self.legal_moves(coord).is_empty());

        if any_legal { return GameStatus::Ongoing; }

        if self.is_in_check(to_move) {
            GameStatus::Checkmate { winner: to_move.opposite() }
        } else {
            GameStatus::Stalemate
        }
    }
}
```

The win-by-brainrot variant is best handled here too — see plan 04 for the
specific rule. The branch slots in cleanly: if `to_move` has no legal
moves and the cause is brainrot covering everything, that's a brainrot
win for the side whose Skibidi caused it. But that requires distinguishing
"no moves because every piece is brainrotted" from "no moves because every
piece is pinned" — non-trivial. Defer to plan 04.

## Tests to add

- `test_king_cannot_move_into_check` — king adjacent to an enemy rook's
  ray; the king's MoveTo onto the attacked square is rejected.
- `test_pinned_piece_cannot_move` — white knight between white king and
  black rook on the same file; knight's moves filtered out.
- `test_checkmate_detected` — fool's mate position; `status()` returns
  `Checkmate { winner: Black }`.
- `test_stalemate_detected` — king with no legal moves but not in check.
- `test_castle_path_must_not_be_attacked` — depends on plan 03; flag for
  later.

## Things to be careful about

- **Recursion**: `is_attacked_by` calls `get_moves` which calls the
  filter which... does not call `is_attacked_by`. No recursion. But
  `legal_moves` clones the board and calls `make_move_unchecked` —
  if `make_move`'s post-effects (like `recalc_brainrot`) ever call
  `legal_moves`, infinite loop. They don't today; keep an eye on it.
- **Performance**: `legal_moves` clones the whole board per candidate
  move. For an 8×8 with ~30 candidate moves average, that's fine. If
  the board ever grows or move counts explode, switch to make/unmake
  or copy-make on just the squares that change.
- **Skibidi PhaseShift and king safety**: a `PhaseShift` doesn't move
  the king or any piece, but it does change brainrot conditions, which
  could uncover a discovered check. `legal_moves` correctly handles
  this by cloning the board and applying the full move (including
  `recalc_brainrot` post-effect), so the check filter sees the new
  brainrot state.
- **No king on the board**: tests today often use boards with no king.
  `is_in_check` must return `false` in that case, not panic.

## Open question

In a chess-2 variant, is "checkmate" still the canonical win condition,
or do custom pieces enable other wins (e.g. brainrot, kidnap-all-pieces)?
Spec implies yes to brainrot. Spec is silent on whether checkmate still
ends the game when other conditions could trigger. Recommend treating
checkmate as a hard win regardless of other state.

## Implementation notes (post-landing)

- **`GameStatus::Check` was added** beyond this plan's original enum. It
  fires when the side to move is in check but has at least one legal
  move. Clients (and a future API) get to distinguish "you must respond
  to a check" from "ordinary turn", which is more useful than reducing
  both to `Ongoing`. Costless given `is_in_check` was already needed.
- **`find_king` descends into every carrier (Bus, Locomotive, Carriage)**: a king
  parked inside a carrier is effectively standing on the carrier's square (capture
  the carrier, capture the king). Without the descent, `is_in_check` silently
  returned false and games couldn't end. Regression test:
  `test_is_in_check_when_passenger_king_under_attack`. Round 3 extended the
  descent semantics to train carts via the `passengers()` trait method, which
  dispatches uniformly across Bus/Locomotive/Carriage.
- **`Monkey::attacks` is overridden**: the default
  `extract MoveTo from initial_moves` over-reports threats for the
  Monkey (empty single-step squares show up as attacks even though
  Monkey can't capture by single-step). The override emits only
  jump-landings — the squares Monkey could actually capture on.
- **`Piece::would_capture_at` filters phantom threats** (plan-09
  audit). A piece's `attacks()` set is its *geometric reach* — train
  carts include their next-tick tile, for instance. But a same-train
  cart on that tile would just be chain-following, not a capture; a
  king parked there is not in check. The per-piece predicate
  `would_capture_at(board, from, target)` filters those phantom
  hits, and `Board::is_attacked_by` queries it before counting a
  reachable tile as a real threat. Default-`true`; Locomotive and
  Carriage override to skip same-train carts. Plan 10's movement
  stack will fold this into the modifier registry.
- **`apply_move_for_validation` skips the train tick** (plan-09
  audit). `validate_move` and `legal_moves` call this in-house
  helper instead of `make_move_unchecked`, so a train can't roll
  over the mover's king during a hypothetical apply and mask a
  `WouldLeaveKingInCheck` error.
- **Neutral-carrier passenger threats are colour-filtered**
  (round-3 audit). `Locomotive::attacks` / `Carriage::attacks`
  return *only* the cart's own next-tick tile. The cart's passengers
  threaten for *their own* colour, not the cart's Neutral colour;
  `Board::is_attacked_by` iterates those passengers separately with
  a per-passenger `passenger.get_color() == attacker` filter. Without
  this, a Black king adjacent to a Neutral cart carrying a Black
  pawn would be flagged in check by its own pawn.
- **`would_capture_at` excludes *all* carts on Loco/Carriage**
  (round-4 audit). Round-3's `advance_trains` foreign-cart Stop
  means a train never actually captures any cart — same-train
  (chain-follow) or foreign (stop short). Both predicates now
  return `false` for any cart on the next-tile, so a king parked
  inside *either* kind of cart at the loco's next-tile is not
  flagged in check.
