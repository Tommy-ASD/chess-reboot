# Plan 04: Custom-piece spec gaps

What's left from the per-piece doc comments that current code doesn't
satisfy. Several of these blocked on plan 01 (turns) or plan 02
(king-safety / game-over).

## Goblin: captured-while-kidnapping

Doc lines 9-10 of `engine/src/pieces/fairy/goblin.rs`:

> If the goblin is taken by an enemy piece while it has a piece
> kidnapped, the kidnapped piece is placed where the goblin was
> located, and the taking piece can move again.

Two distinct mechanics:
1. **Drop the kidnapped piece** at the goblin's previous square (which
   is the *taking piece's* source square — the goblin was at the target).
2. **Extra move for the taker** — the player gets to move again.

### Mechanic 1 (drop the piece)

Where to hook: `engine/src/board/make_move.rs`, the `MoveType::MoveTo`
branch (line ~36-58). Currently it blindly overwrites `to_sq.piece`. We
need to inspect the captured piece *before* the overwrite:

```rust
MoveType::MoveTo(target) => {
    // capture the captured-piece info before overwriting
    let captured = self.get_square_at(target).and_then(|s| s.piece.clone());

    // ... existing source-cleanup + destination-write ...

    // post-capture effect: if the captured piece was a Kidnapping Goblin,
    // drop its passenger onto the goblin's old square (`target` after the
    // taking piece moved? no — the original `from`).
    if let Some(PieceType::Goblin(g)) = captured {
        if let GoblinState::Kidnapping { piece } = g.state {
            // The goblin was at `target`. The taking piece is now there.
            // The kidnapped piece needs a free square — `from` (now empty) works.
            self.set_piece_at(from, (*piece).clone());
        }
    }
}
```

This is a "captured-piece post-effect" — distinct from the
`post_move_effects` hook that fires on the *moving* piece. Worth keeping
in mind that the trait isn't currently structured for this.

### Mechanic 2 (extra move)

This needs the turn system (plan 01) plus a way to express "your move
again." Cleanest: track an `extra_moves: u8` counter on `BoardFlags`
(or whatever `BoardFlags` evolves into). After a goblin-kidnap-capture,
increment it. When flipping `side_to_move`, decrement first — if it was
non-zero, *don't* flip.

```rust
// In make_move at the tail of apply_environment_reactions
// (phase 3 — the renamed handle_post_move_effects):
if self.flags.extra_moves > 0 {
    self.flags.extra_moves -= 1;
} else {
    self.flags.side_to_move = self.flags.side_to_move.opposite();
}
```

### Tests

- Goblin at (3,3) Kidnapping a Pawn; enemy piece captures it; assert
  pawn is now at (capturer's original square), capturer is at (3,3).
- After the capture, `side_to_move` does *not* flip until the
  capturer makes a second move.

## Skibidi: captured-while-opponent-is-phase-4

Doc line 15 of `engine/src/pieces/fairy/skibidi.rs`:

> If your Skibidi is captured while your opponent's Skibidi is in phase
> 4, there is nothing you can do.

This implies a hard loss condition that the engine does not currently
detect. Once your own Skibidi is captured *and* an enemy Skibidi is
sitting at phase 4, the game is effectively over regardless of the rest
of the position. Needs the turn system (plan 01) — done — and a new
`GameStatus` variant, e.g. `BrainrotLossLockout { winner: Color }` or
folded into `GameStatus::Checkmate`-style.

Detection: on every `make_move`, after `recalc_brainrot`, check whether
the side to move has lost its Skibidi while the opposing side still
has at least one Skibidi at `phase == 4`. If so, the game is over and
the opponent wins.

### Tests

- White captures black Skibidi while a white Skibidi is at phase 4 ->
  game status reports white wins.
- Same setup but white Skibidi is at phase 3 -> game continues normally.

## Skibidi: win-by-brainrot

Doc lines 13-14:

> If your Skibidi[,] your enemy cannot make a move due to your
> Brainrot, you win by Brainrot instead of stalemate being declared.

Needs plan 02's `GameStatus` extension. The detection:

```rust
pub fn status(&self) -> GameStatus {
    // ... existing logic ...

    if !any_legal {
        if self.is_in_check(to_move) {
            return GameStatus::Checkmate { winner: to_move.opposite() };
        }

        // Distinguish stalemate from brainrot win:
        // count squares blocked by Brainrot vs. legitimately unmovable.
        let brainrot_caused = self.all_pieces().iter()
            .filter(|(_, p)| p.get_color() == to_move)
            .all(|(coord, _)| {
                self.get_square_at(coord)
                    .map(|sq| sq.conditions.contains(&SquareCondition::Brainrot))
                    .unwrap_or(false)
            });

        if brainrot_caused {
            // Whose Skibidi caused it? Any opposing Skibidi at phase >1.
            let opposing_skibidi = self.all_pieces().iter()
                .any(|(_, p)| matches!(p, PieceType::Skibidi(sk) if sk.color != to_move && sk.phase > 1));
            if opposing_skibidi {
                return GameStatus::BrainrotWin { winner: to_move.opposite() };
            }
        }
        return GameStatus::Stalemate;
    }

    GameStatus::Ongoing
}
```

The "brainrot caused it" check above is approximate. It says "every one
of your pieces is on a brainrot square" — but a piece might be on a
non-brainrot square and still have no legal moves (pinned, etc.).
Tightening this is fiddly. Recommend: keep approximate, document the
behavior, write tests for both the "clearly brainrot" and "clearly
stalemate" cases.

## Passenger Pawn semantics

A white Pawn carried inside a Bus at rank 3 doesn't get a double-push,
because `Pawn::initial_moves` checks `from.rank == 6`. The bus puts the
pawn at the bus's coordinate before running the pawn's move-gen, so the
"from.rank" the pawn sees is the bus's rank.

Spec is silent. Two interpretations:

- **Passengers retain starting-rank rights** — would need a `has_moved`
  flag on Pawn, or special-case Bus inner-piece move-gen to compute
  pawn double-pushes by "would-be" starting rank rather than current.
- **Passengers are stuck where the Bus is** — current behavior. The bus
  is essentially a transporter; once you board, you've "moved" and lose
  starting-rank privileges.

Recommendation: **leave current behavior, document it**. Add a
docstring to `Bus::initial_moves` that explains "passengers move from
the Bus's coordinate; rank-dependent rules like pawn double-push apply
to the Bus's rank, not the passenger's original rank." Then it's a
deliberate design choice rather than a latent bug.

If the spec author wants the other behavior, that's plan 04-bis with a
`has_moved` field on every relevant piece.

## Bus capacity counting (currently top-level only)

Already partially mitigated in plan-4 (nested Bus forbidden). If you
ever want nested carriers, replace the forbid with a recursive count:

```rust
fn transported_count(piece: &PieceType) -> usize {
    match piece {
        PieceType::Bus(bus) => 1 + bus.pieces.iter().map(transported_count).sum::<usize>(),
        _ => 1,
    }
}
```

Then `at_capacity` becomes `bus.pieces.iter().map(transported_count).sum::<usize>() >= 5`. Today this is dead code since nesting is forbidden.

## Monkey: capture-then-jump chain

Doc line 4-5 implies a continuous chain — "if there's a piece next to
the new location, it can jump over that, as well." Current code stops
the chain on a capturing jump (the enemy-landing branch doesn't recurse).
Two readings:

- **Captures end the chain** (current). Standard checkers convention.
- **Captures continue the chain**, with the captured piece "consumed"
  (removed from the board, then recurse). More aggressive.

Spec is ambiguous. Current behavior is defensible. If you want the
"continue after capture" reading, the recursion in `monkey.rs` after
the capture-emit needs to mutate a board clone (remove the captured
piece) and recurse with that clone.

Recommendation: leave as-is until the spec author weighs in. Add a
test that locks in current behavior so future changes are deliberate.

## Notes

Each of these is roughly one commit. Sequencing depends on whether
plan 01 (turns) is in. Goblin extra-move and Skibidi win-by-brainrot
both need it. Passenger pawn doc and Monkey chain question can land
any time.
