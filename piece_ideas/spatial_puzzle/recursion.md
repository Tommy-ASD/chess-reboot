# Recursion

> Any piece ending its move adjacent to a Recursion gets an immediate second
> move of half its normal range. Two adjacent Recursions chain. The whole
> thing resolves to a fixed point.

## Inspiration

The geometric primitive is **iterated rule application until fixed
point**. Baba Is You's rule cascades; Patrick's Parabox's recursive
containment unfolding; Stephen's Sausage Roll's segmented sausages
where one push triggers another. The Recursion is the simplest local
trigger that creates **combinatorial move chains**.

It's also the only piece in this set that explicitly invokes a
fixed-point resolution loop. The engine must iterate until no new
adjacency triggers — deterministic and bounded but a structural
feature of the move resolution.

## Mechanic

A Recursion is a piece that **cannot move** and is **capturable**. It
sits on one square and emits a local trigger.

**The trigger.** Whenever a piece (any colour, any type — but not
another Recursion) finishes a move on a square **orthogonally or
diagonally adjacent** to a Recursion (Chebyshev distance 1, 8 squares
around the Recursion), that piece immediately receives a **second
move** of:

- **Half its normal range, rounded down, minimum 1.**

"Half its normal range" means:

- **King**: range 1 → half is 0 → minimum 1. Second move is 1 square.
- **Knight**: range 1 jump (the L) → half is 0 → minimum 1. Second
  move is a full knight's L (the knight has only one range tier; you
  can't "half" a jump). Second knight move is a regular knight move.
- **Bishop / Rook / Queen**: range = board-diagonal length. Half it,
  round down. On 8×8 board, range up to 7; half = 3. Second move can
  cover up to 3 squares.
- **Pawn**: range 1 (or 2 from starting rank) → half is 0 → 1. Second
  move is a single-step pawn move.
- **Custom pieces (Goblin, Skibidi, Bus, Monkey, train)**: each
  declares a `range_for_recursion` in their move generator. Pieces
  with phase-based or chain-based moves declare a half version. Some
  pieces may opt out (`None` returned → no second move).

The second move is **optional** for the controller — they may decline.
If declined, no further Recursion adjacency triggers fire from this
piece in this turn.

**Chaining.** If the second move's destination square is **also**
adjacent to a Recursion (possibly the same one, possibly different),
the trigger fires **again**, granting a third move of half-of-half
range (rounded down, minimum 1).

This continues until either:

- The controller declines a triggered second move.
- The piece's most recent destination is not adjacent to any
  Recursion.
- The piece has triggered the **same Recursion twice in this chain**
  (no infinite loops; each Recursion can fire at most once per move
  chain).

The "fire at most once" rule guarantees termination: the chain length
is bounded by the number of Recursions on the board.

**Multiple adjacent Recursions.** If the destination square is adjacent
to two Recursions simultaneously, the trigger fires once (not twice).
The chain continues normally; both Recursions are now "spent" for this
chain.

**Capturing a Recursion.** Any piece moving onto a Recursion's square
captures it (standard). The Recursion is removed before any adjacency
checks for this move — i.e., capturing the Recursion does **not**
trigger its own bonus. (Specifically: the adjacency check uses the
post-move board state. After capture, the capturing piece is on the
Recursion's former square; that square no longer has a Recursion;
adjacencies are evaluated against remaining Recursions.)

**Pieces that can't normally move**: a Recursion adjacent to a piece
that can't move (e.g., a piece that's currently frozen or otherwise
restricted) doesn't grant a bonus move. The bonus requires a legal
move under the piece's normal generator (just at half range).

## Why it's interesting

It introduces **combo puzzles**. Setting up a sequence of Recursions
along a path turns a single move into a long zigzag — the piece
bounces along the chain, each hop half the length of the previous
(approximately). Composers can craft "the only way to reach square X
is to use the Recursion chain at squares A, B, C."

It's also **deterministic and FEN-checkable**. Every chain is
predictable: given a board with Recursions and a move, the engine can
enumerate the resulting position after each chain step.

The "minimum 1" rule prevents chains from dying immediately — even a
king or pawn benefits from a Recursion bounce. This makes Recursions
**locally enabling** for short-range pieces.

## Example scenarios

**Single Recursion, bishop chain:**

```
8 . . . . . . . .
7 . . . . . . . .
6 . . . . . . . .
5 . . . . . . . .
4 . . . R . . . .       R = Recursion on d4
3 . . . . . . . .
2 . . . . . . . .
1 . B . . . . . .       B = white bishop on b1
```

Bishop b1 → e4. Lands adjacent to R (e4 is adjacent to d4: NW). Trigger
fires. Bishop gets a second move of half range = 3 (on 8x8, full
diagonal range is 7; half = 3.5 → 3). Bishop on e4 can now move up to
3 squares diagonally. e4 → h7 (3 squares NE). h7 is not adjacent to
any Recursion. Chain ends.

**Two adjacent Recursions, knight cascade:**

```
8 . . . . . . . .
7 . . . . . . . .
6 . . . . . . . .
5 . . R . . . . .       R at c5
4 . . R . . . . .       R at c4
3 . . . . . . . .
2 . . . . . . . .
1 . N . . . . . .       Knight at b1
```

Knight b1 → c3. c3 is adjacent to c4 (Recursion, S direction)? c4 is
N of c3 — yes, adjacent. Trigger fires. Knight on c3 gets a second
move (knight L). Move c3 → b5. b5 is adjacent to c5 (Recursion, E)?
c5 is E of b5 — yes, adjacent. Trigger fires (different Recursion,
not yet used in chain). Knight at b5 gets third move (knight L). Move
b5 → a3. a3 is not adjacent to any Recursion. Chain ends.

Total knight movement: b1 → c3 → b5 → a3. Three moves in one turn.

**Same Recursion used twice (forbidden):**

Knight at b1 → c3, triggers R at c4. Second move c3 → d5, adjacent to
c4 again (c4 is SW of d5: adjacent). But R at c4 has already fired in
this chain — cannot fire again. Chain ends at d5.

(The "fire at most once" rule kicks in here.)

**Decline a bounce:**

Pawn at e6 → e7 (push). e7 is adjacent to R at d8. Trigger fires.
Pawn at e7 may take a second move (1-square push to e8 = promotion).
Controller may **decline** — pawn stays on e7. Strategic choice.

(Triggering the bonus promotion may not always be desirable — e.g., if
the pawn on e8 puts the king in self-check.)

**Combo with Tessera:**

A Tessera ends its slide adjacent to a Recursion. Half-range of a
Tessera's normal 1-square slide → minimum 1 → another 1-square slide.
The Tessera can effectively move 2 squares per turn, possibly chaining
through Recursions. Each Tessera slide includes its push mechanics, so
double-pushes can chain Sokoban-style.

## Where it shines

- **Combo puzzles.** Composers can craft "mate in 1, but only via the
  Recursion chain at b4-c4-d4."
- **Short-range piece amplification.** Kings, pawns, and short-range
  custom pieces become more mobile near Recursions. Balances against
  long-range pieces.
- **Defensive funnels.** A row of Recursions in front of the king
  forces attackers to commit to a chain — defenders can interpose
  during the bonus moves (well, not really; bonus moves are within
  one player's turn — but threat-counting changes).

## Where it's awkward

- **Resolution order.** The chain is resolved within one player's
  turn. Other rule pieces (Pivot, Fold) act on each individual move,
  so the chain must re-check geometry at each step. Confirm in
  engine.
- **Half-range definition.** Every piece needs a `range_for_recursion`
  method. For sliders this is obvious. For knights and pawns, the
  "half" is degenerate (minimum 1). For custom pieces it's a design
  decision.
- **King-in-check during chain.** Each bonus move must leave the king
  not in check (legal in the usual sense). The chain *itself* must
  not pass through a check state. If the king is the moving piece,
  each step must be safe.
- **Captures during chain.** A bonus move can capture; standard. The
  captured piece is removed before the next chain step's adjacency
  check.
- **Promotion during chain.** A pawn that promotes mid-chain — does
  the promoted piece continue the chain at its new piece type's
  range? Yes; the chain is per-piece, and the piece's identity has
  just changed. Promoted queen at e8 adjacent to a Recursion gets
  another bounce.

## Engine dependencies

- **Move generator with range parameter.** Every piece's move
  generator must accept a `max_range: Option<u8>` parameter (or
  similar) to support the half-range bonus moves.
- **Post-move hook for adjacency check.** Engine evaluates each move's
  destination square; if adjacent to a Recursion (and the Recursion
  is in the "not yet fired in this chain" set), trigger the bonus.

## New features required

- **Recursion piece.** No move generator. Adjacency-trigger hook.
- **`range_for_recursion()` method on every Piece trait.** Returns
  `Option<u8>` — `Some(n)` means "this piece's full range; halve it
  for the bonus." `None` opts out of the Recursion mechanic
  entirely.
- **Chain resolver.** Pseudocode:
  ```
  fn resolve_recursion_chain(move: Move, board: &mut Board) {
      let mut spent_recursions: HashSet<RecursionId> = empty;
      let mut current_piece = move.piece;
      let mut current_square = move.to;
      let mut current_range = current_piece.full_range();

      loop {
          let adjacent_recursions = find_adjacent_recursions(
              board, current_square, exclude=spent_recursions);
          if adjacent_recursions.is_empty() { break; }

          spent_recursions.insert_all(adjacent_recursions);

          let new_range = max(1, current_range / 2);
          let bonus_move = ask_controller_for_move(
              current_piece, current_square, new_range);
          if bonus_move.is_none() { break; } // declined

          apply_move(bonus_move);
          current_square = bonus_move.to;
          current_range = new_range;

          if current_piece.is_promoted() { /* update piece, range */ }
      }
  }
  ```
- **Controller prompt** for bonus moves: in interactive play, the
  player picks the bonus move. In a solver/engine context, the
  controller is the search.

## FEN encoding

```
(P=RC,C=W)              White Recursion
(P=RC,C=N)              Neutral Recursion
```

Recursions might be best as neutral pieces — they affect both sides'
movement equally. Allow per-colour for flexibility.

No additional payload — the Recursion is a pure trigger with no
internal state.

The **chain-resolution state** (which Recursions have fired in the
current chain) is **transient** — it exists only during one turn's
resolution. It does not appear in FEN, because FEN is between turns,
and between turns there is no chain in progress.

## Open questions

- **Range halving for non-slider pieces.** "Half of 1 = 0 = minimum 1"
  means knights and pawns always get a full second move when adjacent
  to a Recursion. Is that too strong? Possible adjustment: knights
  and pawns get *one* bonus per chain only — no infinite knight tours.
  But the "Recursion fires at most once per chain" rule already
  bounds this. Probably fine.
- **Pawn double push.** Pawns from starting rank can push 2. Half of
  2 = 1. So a pawn that double-pushes adjacent to a Recursion gets
  one more square — i.e., its bonus move is a single push. Fine.
- **Pieces with non-integer "range."** Custom pieces (Skibidi's
  4-phase moves, Bus's carry pattern) need specific decisions about
  what "half range" means. Default for the engine: piece declares its
  own. Document per-piece.
- **Trigger firing from the Recursion's capture-square.** A piece
  captures a Recursion at square X; piece is now at X. Adjacent
  Recursions (different Recursions) may still trigger from X.
  Confirmed yes — only the *captured* Recursion is spent; others can
  fire.
- **Self-adjacent Recursions.** A Recursion adjacent to itself
  (impossible — single square). Two adjacent Recursions are fine;
  each fires at most once per chain. No special case.
- **Bonus move that puts the moving player in check.** Forbidden by
  normal check rules. The chain ends with the previous step; the
  bonus move is rejected.
- **Bonus move that creates a discovered check (against the
  opponent).** Fine — just a normal move with normal consequences.
  The chain continues.
- **Recursion + Anchor.** A piece named by an Anchor that bounces
  through Recursions: each bonus move is itself a move, so each
  triggers an Anchor mirror. Chain length grows. Strategic
  pathology. Document, test, accept.
