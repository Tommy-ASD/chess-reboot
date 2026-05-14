# Architect

> A King-mover that, instead of moving, paints a `Block` wall on any
> adjacent empty square — a slow, attritional king-trap piece.

## Inspiration

Plan 12 introduces `SquareType::Block`: a semantics-free wall tile,
final-form, payload-free, terminal in the walkability predicate. The
plan ships a brush for the editor, but the engine has no *piece* that
produces walls during play. The Architect is the in-game source.

The design problem it answers: in a normal chess game, "topology" is
static — the board is what it is. Variants like Crazyhouse or
Bughouse perturb piece supply but leave geometry alone. A piece that
paints terrain during play turns king safety from a static evaluation
into a deteriorating one. The opponent's escape squares vanish move
by move.

## Mechanic

Movement set: identical to King (one square any direction, captures
normally, subject to walkability filters).

Special action — **Wall-paint.** Once per turn, *instead* of moving
or capturing, the Architect designates one empty adjacent square (8
neighbours, same shape as its move set) and converts that square's
`SquareType` to `Block`. The Architect itself does not move. The
action consumes the turn.

Constraints on the target square:
- Must be empty of pieces.
- Current `SquareType` must be `Standard`. Painting over `Track`,
  `Switch`, `Gate`, etc. is disallowed (avoids fights with plan 08
  wiring and plan 09 trains).
- Must be inside the board.

No cooldown, no resource limit. Architects are presumed scarce by
position, not by accounting.

## Why it's interesting

The Architect inverts the usual capture-pressure dynamic. A King in
the corner is normally safe — three squares of escape, but enough.
Place an Architect five tempi away and that king's three escape
squares can be reduced to zero across the game. The Architect doesn't
need to *reach* the king; it needs to be near a square the king will
eventually need.

Mechanically novel because the piece's threat radius is *temporal*,
not spatial. A normal piece threatens squares now. The Architect
threatens *future* squares — the squares the opponent might want
later. Defending against it requires either capturing the Architect
or moving so that the walls don't matter, both of which are
information-theoretically expensive.

It also stress-tests the engine's walkability predicate. Every
in-progress move generation must already correctly route through
`is_walkable()` because plan 12's correctness depends on it. The
Architect produces those non-walkable squares in-game, which is the
first piece that exercises walkability *as a state transition* rather
than a static board property.

## Example scenarios

**King trap, slow.** White Architect on d3, black king on h8, white
to move. Sequence: Wall e7, Wall f7, Wall g7. After three full moves
black has nothing on the queenside to engage and the king has no
flight squares on the seventh rank. The Architect itself never
moves; it doesn't need to.

**Pawn shelter denial.** Black has castled kingside, pawns on f7,
g7, h7. White Architect arrives on g6 (via slow King-movement). Each
Architect turn paints one of {f8, g8, h8}. The king cannot recapture
the Architect (squares around it become walls) and cannot escape
sideways (`Block` blocks slider paths and king steps alike).

**Corridor construction.** Mid-board, white wants to herd black's
queen toward a knight fork on e6. Architect on c5 paints d5 then d4
across two turns, leaving the queen with only one direction to
retreat — straight into the fork.

## Where it shines

- Variant compositions with the `Engineer`: a wall + rails combo lets
  one side terraform the board into a maze.
- Endgames. Architects ignore material economy; a single Architect
  vs. a lone king in a corner is a mechanical mate as long as the
  Architect survives.
- Puzzle compositions. The walls-as-pieces motif gives composers a
  new threat geometry.

## Where it's awkward

- Open middlegames with high tempo pressure. Spending a turn to paint
  a wall ten ranks from the action is a one-tempo loss with delayed
  payoff. Architects fold under tactical pressure.
- Variants where territory matters less than activity (e.g.,
  Locomotive-rich boards where the train carves its own corridor).
- Symmetric Architect-vs-Architect compositions risk a draw by
  mutual fortress construction.
- Replay/review readability suffers — the board the game ended on
  doesn't look like the board it started on.

## Engine dependencies

- `SquareType::Block` from plan 12.
- The walkability predicate already routed through every move-gen
  site (plan 12's chokepoints list).
- King-movement primitive (existing).
- A move-type variant capable of expressing "act in place" — the
  Skibidi 4-phase brainrot precedent shows the engine can represent
  non-moving actions.

## New features required

- `MoveType::PaintSquare { coord: Coord, new_type: SquareType }`.
  Generic enough that `Engineer` can reuse it. The matching apply-side
  code mutates `board.squares[coord].kind` and clears any signal
  state on the previous type.
- A move-gen entry for the Architect that emits one `PaintSquare`
  move per legal adjacent empty `Standard` square, plus the standard
  King moves.
- Test coverage: round-trip a paint move through make/undo,
  confirming the previous `SquareType` and any condition list are
  restored.

## FEN encoding

Piece symbol: `A` (Architect) — clean, unused. Or `Ar` if the
single-letter slot is contested; the engine already supports
multi-character piece tags via `(P=BUS(...))` syntax.

The piece itself carries no state, so the simple form is enough:

```
(P=A)            # white Architect
(P=a)            # black Architect
```

No payload, no `passengers`, no cooldown — every Architect turn
either it moved or it painted, both fully reconstructible from move
history. If a future feature adds an action cooldown, extend with
`(P=A,CD=N)`.

## Open questions

- **Should `PaintSquare` count as a "move" for triple-repetition?**
  The board state changes irreversibly, so a paint sequence can't
  repeat by definition. Argues for: yes, count it as a normal move
  for halfmove clock too (it's not a pawn move, not a capture, so
  the 50-move clock increments). Probably uncontroversial.
- **Pin interaction.** Can a pinned Architect paint? Painting
  doesn't move the Architect, so it doesn't expose the king. The
  natural answer is yes, paint-while-pinned is legal. Worth a
  test.
- **Painting over a square with `SquareCondition`s.** Plan 12
  recommends leaving conditions inert on non-walkable squares.
  Architect painting a `Standard` square with a `Frozen` condition
  list: drop the conditions (since the piece that would have been
  affected can no longer stand there)? Or keep them dormant for
  if/when the wall is removed (which currently never happens, but
  future plans might add a Demolitionist)? Recommend: drop on paint,
  re-add nothing on undo if the conditions were derived rather than
  static. Worth a doc-comment.
- **Painting under a piece.** Disallowed (square must be empty). But
  what about painting under a flying piece in a future variant where
  Block doesn't block flyers? Out of scope for v1, but the door
  exists.
