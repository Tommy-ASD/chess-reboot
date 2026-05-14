# Quantum

> Exists at two coordinates simultaneously, both ghosted, neither
> capturable on its own. Collapses to one (the other empties) when
> an enemy moves adjacent to either.

## Inspiration

A piece that breaks the engine's foundational assumption: *one
piece, one square*. The engine's `Board::squares: Vec<Square>`
indexes a single owner per square. The Quantum forces a
re-think: a piece that *occupies two squares* needs a new
addressing scheme.

The chess problem it answers: standard pieces have a single
location, so threats are unambiguous. The Quantum produces
*positional uncertainty*: the opponent doesn't know which
coordinate the Quantum will collapse to until it does. Both
coordinates have to be treated as if they were a piece — but
both are also "not really there" until forced.

This is the heaviest-cost piece in the brainstorm. It's here
because the design problem is genuinely interesting: can you
have a piece whose state includes *multiple positions*? If yes,
what's the FEN form? What's the move-gen shape? The answers
extend the engine's expressiveness in a direction nothing else
in the brainstorm does.

## Mechanic

State: `coords: [Coord; 2]` — two simultaneously-occupied
squares.

Movement: on the Quantum's turn, it moves *one of its two
coordinates* like a Knight. The other coordinate stays put. The
choice of which coord moves is the player's. Move generation
emits one candidate per (coord-index, Knight-offset) pair.
Captures: a Knight-jump to an enemy square removes the enemy and
relocates that coord onto the enemy's square. The other coord
still stays.

Critical property — **Ghost.** Neither coord is capturable in
isolation. An enemy move that targets one of the two coords is
blocked (the destination is "occupied by a Quantum-ghost", which
behaves like a non-walkable square for capture purposes but
*does* allow the enemy to step adjacent normally).

Special property — **Collapse.** When an enemy piece *moves to a
square adjacent to either coord*, the Quantum collapses:

1. The player whose Quantum it is *chooses* which coord becomes
   real. (Or the engine picks deterministically — see open
   questions.) v1: player choice on their next turn? Or engine
   determinism?
2. The other coord empties (no piece there anymore).
3. The remaining coord is now a single-square Quantum,
   functionally a Knight (still flagged as Quantum, but it has
   only one square).
4. If the Quantum is captured *after* collapse (since it's no
   longer ghosted), the capture proceeds normally. Quantum is
   no more durable than a Knight post-collapse.

Critical edge cases:
- Both Quantum coords end up adjacent to the same enemy on the
  same move: still one collapse event. Pick by the
  deterministic order.
- Enemy moves adjacent to one Quantum coord *and* simultaneously
  adjacent to *another* friendly Quantum's coord: each Quantum
  collapses independently.
- Quantum's two coords are themselves adjacent: legal. Both
  squares are ghosted; an enemy stepping to one triggers
  collapse.
- Quantum's two coords are at maximum distance: legal. No
  geometric constraint on `coords[0]` vs `coords[1]` distance.

## Why it's interesting

Four reasons:

1. **Breaks one-piece-one-square.** No existing piece does this.
   Forces the engine to extend its piece-location abstraction
   from `HashMap<Coord, Piece>` to something that handles
   shared/multi-coord pieces. Costly engineering, but real
   expressive payoff.

2. **Threat is observer-dependent.** The Quantum threatens both
   coords' Knight-jump squares. The opponent has *more* threat
   to dodge per Quantum than per Knight. But the Quantum's
   defenses are dual-positional: kill it by approaching, which
   collapses it. The opponent forces collapse and *then* kills.

3. **Forces "linked piece" FEN encoding.** Two squares need to
   share a piece reference. The FEN form has to express
   linkage. Plan 06's parenthesized payloads can extend to
   `LINK=N` references.

4. **Genuinely new tactical primitive.** Approach to collapse,
   then capture, requires two moves minimum. The Quantum buys
   one tempo per enemy approach. Stacked across multiple
   Quantum pieces, this is significant defensive value.

## Example scenarios

**Quantum gambit.** White Quantum at `coords: [d4, h4]`. Black
Queen on a4 wants to capture the Quantum at d4. Q-a4 to d4:
blocked (ghosted). Q-a4 to e4 (adjacent to d4): legal, but
triggers collapse. White's Quantum collapses; player picks h4 as
the real coord. Black Queen on e4 is now adjacent to *nothing*
(the d4 ghost emptied). Black has used a tempo to discover where
the Quantum actually is.

**Quantum dodge.** White Quantum at `coords: [a1, h8]`. Black
attempts approach: Knight to b3 (adjacent to a1). Collapse: white
picks h8 as real. The a1 ghost empties. The Quantum is now at
h8, far from black's Knight. Black just wasted a Knight move
finding out the Quantum was elsewhere.

**Quantum fork.** White Quantum at `coords: [c4, f4]`. From both
coords, the Quantum threatens Knight squares. c4-Knight squares
include b2, d2, a3, e3, a5, e5, b6, d6. f4-Knight squares
include e2, g2, d3, h3, d5, h5, e6, g6. Total threatened
squares: 16. Black Queen on h8 is not in either Knight set — but
black King on d3 is in *both* (c4-Knight covers d3? Let me
recheck: c4 to d3 is `(+1, -1)` = not a Knight jump. c4-Knight
moves are b2, d2, a3, e3, a5, e5, b6, d6. So d3 not threatened.
f4-Knight moves include d3 (f4 to d3 is `(-2, -1)` = yes,
Knight jump). So d3 threatened by f4 only. The Quantum still
threatens d3 — just from one coord. Fork is from the union of
both coords' Knight reach.

**Collapse trap.** White wants the Quantum on h8. Black's pieces
are spread out. White's Quantum currently at `coords: [a1, h8]`.
White's plan: get black to "approach" a1, forcing collapse to
h8. White plays a piece that threatens a useful target near a1.
Black moves a Knight to b3 adjacent to a1. Collapse: white
picks h8. The white piece near a1 is now unsupported but the
Quantum is where white wants. Trade.

## Where it shines

- Defensive variants. The Quantum is hard to kill (collapse
  requires opponent tempo).
- Long-game compositions. The Quantum's two-coord existence is
  worth more in positions where both coords can threaten.
- Variants paired with `Mirror` or `Dancer` — the Quantum's
  threat shape is uniquely dual.

## Where it's awkward

- Engine-cost is heavy. Every move-gen, every check-detection,
  every legality test has to handle the dual-coord case.
- Quantum-vs-Quantum: two Quantums each at 2 coords = up to 4
  squares per "piece pair" to track for adjacency. Combinatorics
  scale.
- FEN form is non-trivial. The two squares share state, so the
  parser has to cross-reference them.
- Player UX: explaining "this square is ghost-occupied" requires
  visual treatment the frontend doesn't currently have.
- Move-gen output is doubled per Quantum (each coord can
  Knight-move). With multiple Quantums, candidate-move count
  inflates.
- Check detection on the *Quantum's own king* is normal — the
  Quantum protects nothing extra by being dual-positioned.

## Engine dependencies

- The board's piece-indexing structure (likely needs extending).
- The capture pipeline (must respect ghost status).
- The check-detection routine (must treat both ghost coords as
  Quantum-threats, but neither as capturable).
- Knight move-gen.
- Plan 06's parenthesized FEN payload (extended with a `LINK`
  cross-reference).

## New features required

- A new abstraction: pieces that occupy multiple squares.
  Cleanest implementation: each Quantum is *one* `Piece` value
  with `coords: [Coord; 2]`, stored once in a board-level
  piece-list (a `Vec<Piece>` or `HashMap<PieceId, Piece>`),
  with each occupied square pointing to it. Significant
  refactor.
- `Piece::Quantum { coords: [Coord; 2], collapsed: bool }`.
  Collapsed = false initially; once collapse happens, coords[0]
  is the only real position and coords[1] is discarded (or
  `collapsed = true` with coords[1] = invalid sentinel).
- `MoveType::Move { from: Coord, to: Coord, piece_id }` with
  Quantum-aware `from` (which of the two coords moved).
- Collapse trigger: after every enemy move, scan all friendly
  Quantums. For each Quantum, check if the enemy's
  *destination* is adjacent to either coord. If yes, collapse
  this Quantum (apply player's pre-declared collapse choice,
  or engine deterministic rule).
- Ghost-square semantics: capture pipeline must check if the
  capture target is a ghost coord of a Quantum; if so, capture
  is illegal.
- FEN encoder/decoder for `(P=Q*,LINK=N)` cross-references.
- Many tests.

## FEN encoding

Piece tag: `Q*` (Quantum). The asterisk distinguishes from
Queen (`Q`). Both coords write the same Piece reference, linked
by a `LINK` field:

```
(P=Q*,LINK=3-4)         # one of the Quantum's squares, linked to coord index 3
(P=Q*,LINK=3-4)         # the other Quantum's square, same link
```

`LINK=3-4` is the pair of board-indices the Quantum occupies.
Both squares have the *same* `LINK` value, so the parser can
verify cross-consistency: exactly two squares have `LINK=3-4`,
and those two squares' indices match the values in `LINK=3-4`.

Post-collapse: only one square has `(P=Q*,LINK=X)` where the
pair is `[X, X]` or `LINK=X-X`, or just `(P=Q*)` with no link
field.

Alternative encoding: a board-level "linked pieces" section
listing `[3-4: Quantum]` separately from the square list, with
each square just referencing the Quantum by id. Cleaner but
breaks the square-local-payload invariant the rest of the FEN
follows. Open question.

## Open questions

- **Who chooses collapse direction?** Two options:
  - **Quantum's owner chooses.** On the *Quantum's owner's next
    turn*, they declare the collapse and *then* play their
    move. This means collapse is delayed by one ply — Quantum
    is still dual-positioned when the enemy who triggered
    collapse moves. Sneaky.
  - **Engine-deterministic.** The owner pre-declares a default,
    or the lower-indexed coord becomes real, or the coord
    closer to the triggering enemy becomes real. Deterministic.
  Recommend engine-deterministic for v1 (FEN simplicity); add
  player-choice in a later variant.
- **Collapse-on-adjacency vs. collapse-on-capture-attempt.**
  Current spec: collapse on *adjacency* (any enemy move whose
  destination neighbours either coord). Alternative: collapse
  only when an enemy *tries to capture* either coord. The
  adjacency rule is stronger (collapse from more states) and
  more interesting.
- **Two Quantums on the same side.** Each is independent.
  Each ghost-occupies up to 2 squares. No interaction.
- **A Quantum collapsing onto the other Quantum's coord.** Not
  possible — the Quantums occupy distinct squares, and the
  collapse just empties one square. No two pieces ever try to
  share a real coord (except as ghost-shared, which we
  disallow).
- **Move-gen complexity.** Per Quantum: up to 16 candidates
  (8 Knight-moves from each coord). With Quantum captures
  changing one coord's location, candidates can include
  one-coord-stays + one-coord-moves combos. Probably fine for
  v1 since variant compositions limit Quantum count.
- **Check-detection.** A Quantum is *not* a King. So Quantum
  collapse doesn't affect check status of the side-to-move's
  King. But the Quantum's threats *do* contribute to checks on
  the enemy King. Both coords contribute. Worth a test.
- **Pinning a Quantum.** A Quantum coord that, if moved, would
  expose the friendly King: the *other* coord can still move
  (since it's a different coord). So Quantum is effectively
  *half-pinned* — one coord pinned, one free. Novel but
  consistent.
- **Frozen / Brainrot conditions.** Apply to one coord or
  both? If a `Frozen` condition is on one of the Quantum's
  squares, does the Quantum freeze entirely or only that
  coord? Recommend: per-coord. The Quantum is two coords; if
  one coord is Frozen, the other coord's move is still legal.
  Asymmetric but consistent with per-coord existence.
- **Engineering cost.** This is the most expensive piece in
  the brainstorm by a wide margin. A real plan should
  estimate the refactor cost of the multi-coord piece
  abstraction before committing. May be wise to design as a
  v2 piece, after the engine has more multi-state precedents
  (Vampire's `absorbed`, Reanimator's `graveyard`).
