# Lens

> Projects a line — a chosen row or column. Same-colour adjacent pieces on
> that line fuse into stacks. Captured stacks scatter back out.

## Inspiration

The geometric primitive is **dimensional compaction**. Patrick's
Parabox's nesting; the way Sokoban-likes occasionally collapse two
crates into one stack; the lossy-projection step in many puzzle games
where information is intentionally reduced so the player must invert
the projection.

The Lens is the only piece in this set that creates **vertical
stacking** on a 2D board. Squares may hold multiple pieces. This is
a structural break from "one piece per square" — and it's the central
puzzle.

## Mechanic

A Lens is placed with a **line tag**: a specific row `R=k` or column
`F=k`. The line is set at placement and does not change.

**Compaction.** On every turn end, the engine scans the Lens's line.
For each pair of *adjacent* squares on the line that both contain
pieces of the **same colour**, the pair fuses:

- Both pieces move into the **higher-numbered square** (higher rank
  for column-lines, higher file for row-lines).
- The lower-numbered square becomes empty.
- The resulting square contains a **stack** of two pieces.

Repeat the scan until no adjacent-same-colour-pair exists. (Order:
scan from low to high along the line, fuse the first applicable pair,
restart. Deterministic.)

Stacks can fuse further: a 2-stack and an adjacent 1-piece same-colour
fuse into a 3-stack. There is no maximum stack size.

**Stack identity.** A stack is **not a piece** — it's a list of pieces
all living in the same square. Each piece in the stack retains its
type, ID, and any payload. The stack itself has no separate state.

**Stack moves.**

- **Stack does not move as a unit.** Pieces in a stack each still have
  their normal moves available, but the moves are emitted from the
  stack's square. Only **one piece at a time** can leave the stack
  per turn.
- Pieces in a stack **count as blockers** for other pieces — the
  stack square is opaque to sliders.
- Pieces in a stack **all attack** the squares they normally would
  from the stack's square. (Five pawns stacked all attack the same
  two squares — high local threat.)
- A piece *enters* the stack square only via a normal move. If a
  friendly piece moves to the stack's square, it joins the stack
  (provided one of them was already there — but the Lens's
  compaction step handles re-fusing).

**Capture.** When **any** piece in a stack is captured, **the entire
stack scatters**:

- Capturing piece occupies the stack square (standard).
- All non-captured stack members **scatter** one square each, in a
  deterministic outward pattern. The pattern:
  1. Start at the captured-and-now-replaced square (call it `S`).
  2. Iterate the 8 surrounding squares in clockwise order from N.
  3. For each non-captured stack member, place it on the next available
     (empty, walkable, on-board) surrounding square.
  4. If 8 squares aren't enough, continue with the next ring (distance
     2 in same clockwise order).
  5. If no square is reachable, the stack member is **captured** as
     well (collapsed against the board's edge or topology).
- After scattering, the Lens's compaction step does NOT run until the
  next turn. (Scattering is in the middle of a turn; compaction
  always happens at end-of-turn.)

The compactor is **idempotent**: once the line has no adjacent
same-colour pairs, it does nothing. Two-colour boards on the line
freely interleave without fusing.

**Multiple Lenses.** Each Lens has its own line. After each Lens's
compaction runs, the engine re-runs all Lenses until a fixed point
is reached. (Fixed point is guaranteed because total piece count on
each line is monotonically non-increasing per compaction step, and
non-trivial reductions strictly decrease an ordering invariant.)

## Why it's interesting

It compresses **multiple pieces' worth of attacking power** into one
square. A four-pawn stack on the 7th rank attacks two squares with the
power of four pawns — meaning four recapturers can scatter in response
to any single attack on the stack.

It also **mass-destroys**. One capture on a 5-stack scatters 4 pieces;
some may scatter onto unwalkable squares (Block, Turret, Vent) and
auto-die. A well-placed Lens turns a single capture into a board-clear.

Strategically, **avoiding the Lens's line** becomes a real constraint.
A Lens on the 4th rank means you don't want adjacent same-colour
pieces on rank 4 — they'll fuse next turn.

## Example scenarios

**Compaction on the e-file with three white pawns:**

```
Before:                         After Lens (column-e):
8 . . . . . . . .              8 . . . . . . . .
7 . . . . . . . .              7 . . . . P . . .   <- stack of 3
6 . . . . P . . .              6 . . . . . . . .
5 . . . . P . . .              5 . . . . . . . .
4 . . . . P . . .              4 . . . . . . . .
3 . . . . L . . .              3 . . . . L . . .   <- Lens
2 . . . . . . . .              2 . . . . . . . .
1 . . . . . . . .              1 . . . . . . . .
```

Three white pawns at e4, e5, e6. Lens on e3, line = e-file. End of
turn:

- Scan e1..e8 for adjacent same-colour pairs.
- e4 and e5 — both white pawns, adjacent. Fuse: both move to e5
  (higher). Now e5 is a 2-stack.
- Scan again. e5 and e6 — e5 is a 2-stack (white), e6 is white pawn.
  Adjacent, same colour. Fuse: stack moves to e6 (higher). e6 now
  has 3 pawns.
- Scan again. No more pairs. Stop.

Final: e6 has a stack of 3 white pawns. e4 and e5 empty.

**Scatter on capture:**

```
. . . . . . . .
. . . . . . . .
. . . . . . . .
. . . n . . . .     n = black knight on d5
. . . . S . . .     S = 4-stack of white pawns on e4
. . . . . . . .
. . . . . . . .
. . . . . . . .
```

Black knight on d5 captures the stack on e4. Capturing move:
n d5→e4. Knight lands on e4. Three remaining white pawns scatter:

- Clockwise from N: e5 (empty, available — pawn 1 goes here)
- E: f4 (empty — pawn 2)
- SE: f3 (empty — pawn 3)
- Done.

Final: knight on e4, white pawns on e5, f4, f3.

(Note: pawn 1 on e5 is now adjacent to the knight on e4 — and is on
the e-file, which is the Lens's line. The compactor runs at *end of
turn*, so on white's next move-completion the Lens may immediately
re-fuse any same-colour adjacencies that result.)

**Cascade scatter via unwalkable terrain:**

```
. . | . . . . .
. . | . . . . .       | = Block square in column 3 (file c)
. . | . . . . .
. . S . . . . .       S = 5-stack of black pawns on c5
. . | . . . . .
. . | . . . . .
. . . . . . . .
. . . . . . . .
```

A white piece captures the stack on c5. 4 remaining pawns scatter
clockwise from N. N is c6 (Block) — skipped. NE is d6 (available — pawn
1). E is d5 (available — pawn 2). SE is d4 (available — pawn 3). S is
c4 (Block) — skipped. SW is b4 (available — pawn 4). Done.

Pawns scattered to d6, d5, d4, b4. The Block squares acted as a
"shield" that redirected the scatter pattern.

## Where it shines

- **Mass-attack squares.** A stack of attackers on one square = many
  recapturers if attacked. Effectively a "fortress" piece.
- **End-of-turn surprise tactics.** Player moves a pawn to e4 next to
  another e3 pawn — at end of turn, both fuse to e4 (higher), and the
  e4 pawn now has 2 attackers' worth of defence. Or, more strongly:
  fuse onto a square that puts the stack adjacent to enemy king.
- **Composition with Block / Turret.** Scatter patterns are highly
  position-dependent. Composers can use unwalkable terrain to force
  scatters in specific directions.

## Where it's awkward

- **Engine state.** Stacks break the "one piece per square" invariant
  globally. Every place in the engine that reads "the piece at this
  square" needs to handle "a list of pieces."
- **Move generation cost.** A stack of N pieces generates moves for
  all N. Some pieces may have many moves (queens stacked = many
  moves from one square). Performance concern for large stacks.
- **Captures of stacks.** Which piece "captures" the stack? Any
  piece that attacks the stack square can capture-the-stack, but
  the captured piece is the topmost — wait, there's no "topmost"
  in 2D. Suggest: the stack member captured is **the most recently
  added** (LIFO). Composers can predict.
- **Check evaluation.** Stack of pieces attacks the same squares.
  Detecting check from a stack: check whether any stack member's
  move pattern includes the king's square. Standard, just slower.
- **Promotion in a stack.** A stack of pawns crosses the back rank?
  Pawns don't move as a stack — only individuals leave the stack.
  When an individual pawn leaves via its own move and lands on
  back rank, it promotes. Normal.

## Engine dependencies

- **One-piece-per-square invariant** — the Lens breaks it. Refactor
  required.
- **End-of-turn hook** — engine needs a phase between moves where
  compaction can run. May be the same hook as Tide's pulse.
- **Walkable-square check** — used by the scatter algorithm.

## New features required

- **Stack as a board cell type.** `Square::pieces: Vec<Piece>` instead
  of `Option<Piece>`. Most queries (`is_empty`, `is_blocker`) become
  `pieces.is_empty()` / `!pieces.is_empty()`. Most piece access
  (`piece()`) needs a "main piece" or "iter pieces" interface.
- **Compaction phase.** A new end-of-turn step run per Lens. Scans
  the line, fuses adjacent same-colour pairs greedily until no more
  exist. Fix-point loop.
- **Scatter algorithm.** Deterministic clockwise-ring expansion.
  Used at capture time.
- **Move generator updates.** Any piece on a stack square must emit
  its own moves; legality checks must operate on individual pieces.

## FEN encoding

The Lens itself is a piece:

```
(P=LS,C=W,F=e)          White Lens on the e-file
(P=LS,C=B,R=4)          Black Lens on rank 4
```

`F=` (file) and `R=` (rank) are alternatives; exactly one is required.

Stacks need an FEN extension. Two options:

1. **Multi-piece square.** Encode multiple pieces in one square with a
   delimiter: `(P=P+P+P,C=W)` for a 3-stack of white pawns. Hard to
   parse if individual pieces have payloads.
2. **Stack square.** A square containing a stack is encoded with a
   special marker, and the pieces are listed as sub-payloads:
   `(STACK,C=W,M=[P|P|P])` — the `M=` (members) payload has
   pipe-delimited piece specs.

Recommend option 2 for general extensibility. The `STACK` marker
declares the square has multiple pieces; `M=[...]` lists them.

```
(STACK,C=W,M=[P|P|P])               three white pawns
(STACK,M=[(P=P,C=W)|(P=Q,C=B)])     mixed-colour stack — possible?
```

(The Lens only fuses *same-colour* adjacents, so a mixed-colour stack
can only arise from manual placement or scattering. Allow it.)

## Open questions

- **Mixed-colour stacks.** Lens won't create them, but they can arise
  from scatter. Treat as: each piece in the stack belongs to its
  colour; both sides can move their respective pieces out. The stack
  square is opaque to both sides' sliders.
- **Stack size limit.** Unbounded? Practical max is roughly the number
  of pieces in the game. Probably no limit needed; just a sanity
  cap (32 pieces?).
- **Compaction order.** "Scan from low to high, fuse the first
  applicable pair, restart" is deterministic. Alternative: "fuse all
  pairs simultaneously" leaves the order ambiguous for >2 adjacents.
  Stick with greedy low-to-high.
- **King in a stack.** Can the king be stacked? Strange but allowed.
  Capturing the stack while the king is in it = checkmate / win.
  Edge case for puzzle composers.
- **Lens line is also a Fold's crease.** Both rules act on the same
  line. Fold reflects rays through it; Lens compacts pieces on it.
  Resolution order: Fold acts during move generation (ray-trace);
  Lens acts at end-of-turn (compaction). No conflict.
- **Scatter into another stack.** Pawn scatters into a square that
  *already has* a friendly piece — does it stack again? Yes; the
  scatter target's contents grow by one. Next compaction may further
  fuse if the resulting stack is now adjacent to another same-colour
  piece on a Lens's line.
- **Captured-stack member.** Spec says "most recently added." Need a
  per-piece "stack-arrival timestamp" tracked across moves. Adds
  state. Alternative: define the captured member as **the first
  piece in FEN order**, which is deterministic and FEN-checkable.
  Recommend FEN-order rule.
