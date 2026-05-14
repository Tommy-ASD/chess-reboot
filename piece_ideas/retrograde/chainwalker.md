# Chainwalker

> A piece that carries a strictly-increasing counter `C{n}`,
> incremented by one each time it moves, never decreasing, never
> resettable. The number on the piece counts the moves it has made.

## Inspiration

Smullyan's "Who moved last?" genre. The classical solver counts plies
in their head from castling rights and pawn structure. The
Chainwalker writes the count directly onto the piece. A position
with a Chainwalker at `C7` *proves* exactly seven of that side's
plies have been Chainwalker moves — and therefore exactly that-many
of that side's plies were *not* available for other plans.

## Mechanic

A Chainwalker moves as a king (or any chosen movement profile —
king is the canonical choice; the counter mechanic is orthogonal).
On each move it makes, its counter `C` increments by 1.

The counter starts at 0 when the piece appears on the board. It
**cannot decrement**. There is no legal mechanism — capture, undo,
hand-placed reset — that decreases the counter while the piece
remains the same identity. The counter persists across all square
types, conditions, and signal events.

If the Chainwalker is captured, the counter dies with the piece. A
new Chainwalker (e.g. a hand-placed one) is a new identity and starts
at 0.

The counter is printed on the piece in FEN as part of its payload.

## The deduction it enables

Consider a position presented to the solver as "Black to move,
ply 24." (Twelve White plies and eleven Black plies have happened;
this is the twelfth Black ply.)

The board has a black Chainwalker showing `C7`.

The solver reasons:

1. Eleven black plies have happened. Of those, exactly seven were
   Chainwalker moves (because `C` increments only on the
   Chainwalker's own move).
2. Therefore exactly four black plies were *not* Chainwalker moves.
   Those four plies are everything else Black has done in the game
   so far.
3. Black has castled (king is on g8). Castling is one ply. So **at
   most three** non-Chainwalker, non-castling Black plies remain.
4. Black's a-pawn has advanced two squares. That requires two plies
   (a7-a5 or a7-a6+a6-a5). So **at most one** non-Chainwalker,
   non-castling, non-a-pawn Black ply remains.
5. The black knight on f6 must have arrived from g8 or h7 or
   somewhere — but g8 is occupied by the king (post-castling) and
   h7 still holds a pawn. The knight must have come from g8 *before
   castling*. That's one knight move + one castling move, plus the
   pawn moves, plus the seven Chainwalker moves = 11 plies. The
   "at most one" budget is consumed. Therefore Black has made
   *exactly* one knight move (g8-f6), no other knight has moved,
   no bishop has moved, no other pawn has moved.

This is a tight ply-budget deduction that's possible *only* because
`C=7` is visible on the board. Without the counter, the solver has
no anchor against which to count the rest of Black's history.

## Why it's interesting

The counter is the smallest possible piece of historical state — one
integer per piece — and yet it pins down an entire arithmetic
constraint on the game so far. Composition with a Chainwalker
becomes ply-arithmetic: the puzzle is built around the counter
satisfying an equation that admits exactly one history.

A puzzle with *two* Chainwalkers (one per side) lets the composer
constrain both sides' ply budgets independently. The two counters
plus the move-number entirely determine the partition of "moves
spent on these two pieces" vs "moves spent on everything else."

## Example puzzle setup

```
. . . . . . . k
. . . . . . . p
. . . . . . . .
. . . . . . . .
. . . . . . . .
. . C . . . . .
. . . . . . . .
K . . . . . . .
```

White Chainwalker `C` on c3 showing `C=10`. It is Black to move on
move 11 (i.e., 10 White plies and 10 Black plies have happened).

Prove: White has made no move other than Chainwalker moves.

Solution: ten White plies, all of which incremented `C`. Therefore
all ten White plies were Chainwalker moves. The white king on a1 has
never moved. (This in turn proves White retains the right to castle
queenside in principle, though the rook is missing — composition
detail.)

## Where it shines

- **Ply-budget puzzles.** "How many of X's moves were Y?" The
  Chainwalker makes the answer literal.
- **No-castling-yet proofs.** A Chainwalker counter equal to the
  full ply count for its side proves the king has not moved.
- **Tempo-constraint helpmates.** Composing a helpmate-in-N where
  the solver must verify the cooperating side has *exactly* the
  right number of tempi to spare.

## Where it's awkward

Two failure modes:

- **Counter inflation.** Long games push the counter into double
  digits. The FEN remains short (`C=27` is three characters) but
  visually the piece glyph becomes busy. Composition will typically
  keep counters small; analysis of long games is not the use case.
- **Capture loses the evidence.** If the puzzle premise hinges on
  knowing how many moves a *captured* Chainwalker made, the
  evidence is gone. The composer must keep the Chainwalker alive
  on the board in the presented position, or supplement with other
  evidence (Sediment, Scar).

A non-failure but a design note: the counter conflates "moves" with
"plies of this color." It does not distinguish a quiet move from a
capture. If the composer needs that finer breakdown, pair the
Chainwalker with a Sediment-bearing capture square.

## Engine dependencies

- Per-piece payload system (existing).
- A move-application hook that fires after a Chainwalker move
  completes and increments its payload counter.

## New features required

- **Per-piece counter payload.** New piece-payload key `C` with an
  integer value. Stored on the piece, persists across squares,
  serialized to FEN.
- **Post-move increment hook.** A piece-trait method called by the
  move applier; default no-op; Chainwalker increments its `C` by 1.
- **Editor support.** Brush placement of a Chainwalker should
  permit setting an initial counter (puzzle composers may want
  `C=7` from the start to encode "this piece has lived a life
  before the puzzle began").

## FEN encoding

Piece payload key: `C`. Value: a non-negative integer.

Examples:

```
(P=CH,C=0)              — fresh Chainwalker
(P=CH,C=7)              — Chainwalker that has moved seven times
(P=CH,C=27,F=COLOR:b)   — black Chainwalker, twenty-seven moves
```

The piece letter `CH` (for Chainwalker) chosen to avoid collision
with `C` already used for castling rights in board-level FEN. If
the piece tag space already has a clash, `CW` (Chainwalker) is the
fallback.

A `C=0` payload is the default but should still be emitted explicitly
for clarity. Round-trip rule: emit `C` always, even when zero, so
that a counter is never silently elided.

## Open questions

- **Does any other action increment `C`?** Current answer: only the
  Chainwalker's own move. Question: what about a *forced* relocation
  (some hypothetical Bus-like pull mechanic)? Lean toward yes —
  any change of square increments. The semantic is "movement," not
  "voluntary movement." Pending real interaction with the
  movement-stack plan.
- **Multiple Chainwalkers, same color.** Each counts its own moves.
  No interaction. Confirmed clean.
- **A Chainwalker on a Frozen square that is then thawed.** While
  frozen it cannot move; the counter stays. On thaw, the next move
  increments normally. No special case.
- **Initial-position counters in editor.** Should the editor allow
  any non-negative integer, or cap at the current game's ply count?
  Recommendation: allow any value, with a soft warning if `C >
  total plies of that color`. The composer occasionally wants a
  "pre-game history" Chainwalker, and the warning is enough.
