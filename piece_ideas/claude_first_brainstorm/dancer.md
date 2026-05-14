# Dancer

> Moves like a Queen, but only to squares orthogonally adjacent to
> *some* friendly piece — pure move-gen filter, no new state.

## Inspiration

A piece whose value depends on its *teammates' positions*. Standard
chess has weak emergent "teammate" effects (pawn chains, piece
support), but no piece's *movement set* literally requires a friend
to be nearby. The Dancer enforces it.

Mechanically, this is the simplest piece in the brainstorm: no new
state, no new move-type, no new hook. Just a filter on the existing
Queen move generator: "for each candidate Queen destination, accept
iff some friendly piece is orthogonally adjacent to that
destination."

The design problem: encourage piece coordination as a hard
constraint rather than a soft incentive. A Dancer alone is
near-immobile. A Dancer in a flock is dominant. The piece teaches
the player that piece coordination is a *resource*.

## Mechanic

Movement set: Queen (slides along rank, file, both diagonals; any
distance).

**Filter:** A Queen-candidate destination `D` is legal for the
Dancer iff there exists a friendly piece (same color as the
Dancer) at some orthogonal neighbour of `D` (`D` ± (1,0) or `D` ±
(0,1)). The friendly piece may include other Dancers, the King,
Pawns, anything color-matched.

Notes:
- The Dancer itself doesn't count as a friendly neighbour of its
  destination (it can't self-support).
- Diagonal-only neighbours don't count — the support is
  *orthogonal*. Tighter constraint; design choice (see open
  questions).
- The Dancer can capture as a Queen on a legal destination — the
  filter applies to the destination, not to whether it's empty.
- Sliding through friendly pieces is still illegal (standard slider
  rules — the Queen doesn't gain phasing). Sliding *past* a
  square that has a friendly orthogonal neighbour is fine; the
  filter only checks the *destination*.

## Why it's interesting

Three reasons:

1. **No new state, full move-gen novelty.** This is the cheapest
   piece in the brainstorm. The engine already enumerates Queen
   moves; the filter is a single predicate. No FEN payload, no
   new MoveType, no environment-reaction hook.

2. **Coordination as resource.** The Dancer's effective mobility
   depends on the number of friendly pieces in the right squares.
   This couples *every* piece on the board to the Dancer's value.
   Trading a Pawn to a free capture often costs the Dancer one or
   two destinations.

3. **Asymmetric position-types.** In a closed position (many
   blocks, narrow corridors), the Dancer thrives — friendly pieces
   are clustered. In an open position with isolated pieces, the
   Dancer is near-useless. Same piece, opposite strategic value
   by position type.

## Example scenarios

**Flock dance.** White Dancer on d4, friendly Pawns on d5, e5, c5.
Queen candidate destinations include all rank, file, diagonal
squares from d4. Of those, legal-for-Dancer destinations are
those orthogonally adjacent to one of {d5, e5, c5} — namely
{d6, d4 (self, excluded), e4, e6, c6, c4, c6, e5/d5/c5 if
they're enemy capture squares but they're friendly so skip}.
Concretely: d6, c6, c4, e6, e4 are all legal. The Dancer can
move through a meaningful fraction of the Queen's reach.

**Alone, immobile.** White Dancer on h8, friendly pieces all on
the queenside. No friendly piece is orthogonally adjacent to any
square the Dancer could slide to. **Zero legal moves.** The
Dancer is functionally stuck.

**King support.** White Dancer on e4, white King on e1 (4 squares
south). King's orthogonal neighbours: d1, f1, e2. The Dancer's
candidate destinations along the e-file include e2 (orthogonally
adjacent to King via d1/f1/e2 — wait, e2 is itself a King-neighbour).
Let me re-check: e2 is orthogonal-adjacent to e1 (the King). So
the Dancer can slide e4→e2. Legal.

**Capture coordination.** White Dancer on a1, white Bishop on b2.
Queen-slide candidates from a1 include a8, h8, h1. Of those, the
squares orthogonally adjacent to b2 are: a2 (yes — left of b2),
c2 (right of b2), b1 (below b2), b3 (above b2). None of {a8,
h8, h1} are orthogonally adjacent to b2. So legal Dancer
destinations from a1 with only Bishop support are: a2 (along a-file),
b1 (along rank 1, requires sliding through b1 — sliding *to* b1
is fine; b1 is orthogonal-adjacent to b2 = Bishop). So Dancer
can move to a2, b1, or any square orthogonal-adjacent to b2 that
the Dancer can reach as a Queen.

## Where it shines

- Closed, clumpy positions.
- Defending the King — the King provides four orthogonal support
  neighbours.
- Variants with many Pawns or many small pieces.
- Compositions where piece-clustering is the point.

## Where it's awkward

- Sparse boards. A Dancer on an island is dead weight.
- Endgames after exchanges — once piece count drops, support
  thins.
- New players will routinely accidentally move a Dancer to a
  square that's *almost* legal (diagonal-only support, not
  orthogonal) and have the move rejected. UI affordance
  needed: highlight legal destinations explicitly.
- Trades that the player would otherwise consider neutral can
  silently destroy the Dancer's mobility. Hard to evaluate.

## Engine dependencies

- Queen move generator (already exists).
- Same-side check (every piece knows its color).
- Orthogonal-neighbour enumeration (trivial).

## New features required

- A new piece type `Dancer`. Move-gen: enumerate Queen
  candidates, filter by orthogonal-friendly-adjacency to
  destination.
- Test: alone Dancer has zero moves.
- Test: Dancer with friendly Pawn at the right square has
  expected moves.
- Test: friendly capture by Dancer's slide path doesn't break
  filter.
- Test: enemy capture at filter-legal destination is legal.
- Test: FEN round trip.

## FEN encoding

Piece symbol: `D` (Dancer). Single-letter likely free.

```
(P=D)            # white Dancer
(P=d)            # black Dancer
```

No state. Color is the only context the move-gen filter needs,
and that's already implicit in the piece's owner.

## Open questions

- **Orthogonal-only support, or include diagonal?** Current spec:
  orthogonal-only. Wider rule (orthogonal + diagonal = 8
  neighbours) gives the Dancer more mobility per friendly piece
  but makes the filter less distinctive. Recommend orthogonal-only
  for v1; it produces tighter clusters and is easier to read.
- **Pawns supporting Dancer destinations.** Pawns have weird
  threat-shape rules (they threaten diagonal, but their *square*
  is just a square). For Dancer support purposes, a Pawn's
  *square* is what matters — the Pawn's threat doesn't enter
  the filter. So a Pawn on e5 supports Dancer destinations at
  d5/f5/e4/e6 (orthogonal neighbours of e5). This is the
  natural reading.
- **Dancer supporting itself.** Excluded explicitly: the Dancer's
  own square isn't a friendly square for filter purposes. A
  Dancer doesn't satisfy its own support requirement at any
  destination. Worth a test.
- **Two Dancers supporting each other.** Two Dancers on adjacent
  squares: each is a friendly orthogonal-neighbour of the
  other's potential destination, *if* the destination is one of
  the orthogonal neighbours of the *other* Dancer. So Dancer A
  on d4 and Dancer B on d6: Dancer A can move to e6 or c6
  (orthogonal-adjacent to d6). Dancer B can move to e4 or c4
  (orthogonal-adjacent to d4). Each Dancer enables the other
  in a small local region. Fine.
- **Check evasion.** Dancer in check, has a Queen slide that
  blocks check but the destination has no orthogonal-friendly
  neighbour: move illegal. The Dancer might have *no* legal
  moves while in check — checkmate even with material on the
  board. Document.
- **Stalemate by isolation.** Dancer alone with King = the
  King's orthogonal neighbours are the only Dancer-legal
  squares within Queen range. If the King is cornered, the
  Dancer has very few moves. Stalemate becomes more achievable
  in Dancer endgames.
- **Performance.** Each Queen candidate destination triggers a
  4-neighbour-check (constant time). Total move-gen cost is the
  same order as standard Queen. No measurement concern.
