# Jackhammer

> A piece that, when it leaves the board, leaves behind a permanent impassable hole on its square.

## Source

From `engine/src/pieces/ideas.txt:18-19`:

> Jackhammer piece
>     When taken (or taking another piece? would only be able to use once), the Jackhammer creates a hole where it once was. The hole acts as a barrier.

The source itself is uncertain: the trigger is either "when taken"
(parenthetical "or taking another piece?"). Both are explored below
and one is recommended.

## Inspiration

This is the most literal "leaves a mark" piece in the file. The
fantasy is industrial: drill, drill, drill, and where the drill was
is a hole. Mechanically it's a one-shot terrain-creator — the piece
*becomes* terrain when it dies.

This pairs naturally with plan 12's `SquareType::Block`. Plan 12
introduced a payload-free, semantics-free wall tile precisely because
hand-built scenarios and *piece interactions* might want to spawn
walls. Jackhammer is the first "spawned by piece" Block.

## Mechanic

A Jackhammer is a normal-ish piece that, when it leaves the board,
converts its square into `SquareType::Block` for the rest of the
game.

### Base movement

**Recommend rook-like** (straight lines along files and ranks). The
visual is a drill that goes straight ahead — diagonals don't fit.
Rook + straight-only also makes its movement *predictable*, which
matters because the threat of "if you capture me, you lose this
square forever" should be telegraphable.

Captures normally.

### Trigger — two variants

The source is undecided. Both are workable; this doc proposes both
and recommends one.

#### Variant A — "Hole on being captured"

When the Jackhammer is captured by an enemy piece, the capturing
piece moves onto the Jackhammer's square *as normal*, then at the
end of move resolution the square is converted to
`SquareType::Block`. But the capturing piece was just put there.
Two sub-options:

- **A1.** The capturing piece is *displaced*: it returns to its
  origin square and the destination becomes Block. Capture happens,
  Jackhammer is removed, but the territory is denied.
- **A2.** The capture is *forbidden*: a piece cannot capture a
  Jackhammer. The Jackhammer is only "captured" by sliders pinning
  it (which it ignores) or some other obscure means. **Bad** —
  effectively immortal.
- **A3.** The capture happens, the capturing piece *dies too*.
  Mutual destruction. Square becomes Block. **Bad** — too punishing
  per attempt, makes the Jackhammer a free wall maker any time you
  trade evenly.

**Recommend A1.** Capture resolves (the Jackhammer is gone), but the
attacker is pushed back to their origin and the square is denied.
The opponent paid a move for the trade but didn't gain territory.

#### Variant B — "Hole on capturing"

When the Jackhammer captures an enemy piece, the Jackhammer is
*consumed*: the captured piece is removed, the Jackhammer is removed,
and the captured piece's square becomes `SquareType::Block`. One-shot,
as the source notes.

#### Recommendation: Variant B

**Recommend Variant B.** Reasoning:

- The source's parenthetical hint ("would only be able to use once")
  fits B better — it's an *active* ability with a clear cost.
- B is *initiated* by the Jackhammer's owner. The wall placement is a
  decision, not an accident of being captured. Players love decisions.
- A1 has the awkward "displacement" rule which is hard to teach and
  has weird corner cases (where does a knight get displaced *to* if
  its origin is now occupied?).
- B is symmetric with other one-shot pieces (Goblin's "kidnap then
  return home" is also a one-shot transformation, though over time).

Variant A1 stays in the doc as the alternative for variants that want
"reactive" terrain instead of "active" terrain. A Chess 2 variant
could ship both Jackhammer-on-capture and Jackhammer-on-being-captured
as distinct pieces.

### State

A Jackhammer's only relevant state is "has it been used yet" — under
Variant B, this is implicit (the piece either exists or it doesn't;
once used, it's gone). No counter needed.

Under Variant A1, no state on the piece itself either — the trigger
is the moment of capture.

## Why it's interesting

A piece whose *death* changes the board. Standard chess pieces leave
nothing behind; captured queens are simply gone. The Jackhammer
turns a single life into permanent terrain.

It also reframes capture-trades. Trading a knight for a Jackhammer
isn't 3-for-3 — it's 3 for 3 *plus a lost square forever*. The square
the trade happened on becomes denied territory. Pawn-structure
discussions now extend to *hole-structure*.

## Example scenarios

1. **Closing a file.** White Jackhammer on e3, black rook on e8.
   The rook can pressure down the e-file. White moves the
   Jackhammer to e4 and uses it to capture a pawn on e5. The
   Jackhammer is consumed; e5 becomes Block. The e-file is now
   permanently cut at e5 — black's rook is bottled.
2. **Defensive sacrifice.** Black king on g8, fianchetto bishop on
   g7, Jackhammer on h7. White's queen is one move from h7. Black
   captures with the Jackhammer on a white pawn on h6 — h6 is now
   Block, denying white the diagonal approach.
3. **The unintended wall.** Variant A1: white Jackhammer on d4,
   captured by black knight from c2. Black knight returns to c2
   (displaced), d4 becomes Block. White wanted the Jackhammer
   alive; now there's a wall in the middle of the board they
   didn't plan for.

## Where it shines

- **Closed positions.** Walls compound — one strategic Block can be
  the difference between blockaded and open.
- **Asymmetric maps.** A Jackhammer in a narrow corridor turns the
  corridor into a one-way valve.
- **Puzzle compositions.** "Capture the right piece with your
  Jackhammer" is a clean tactical motif.

## Where it's awkward

- **Self-blockade.** If you spend a Jackhammer to create a wall
  your *own* pieces wanted to walk through, that's bad — but it's
  also a real cost the player chose, so it's design-honest.
- **End-game terrain bloat.** Many Jackhammers across a game leaves
  the board increasingly walled. In long games this could trivialise
  draw-by-stalemate (kings get cornered behind walls). Probably
  fine — Chess 2 isn't shy about decisive games — but worth noting.
- **Variant A1's displacement rule.** If recommended, A1's
  "attacker returns to origin" is fiddly. The recommendation to
  ship B sidesteps this.

## Engine dependencies

- `SquareType::Block` (shipped, plan 12).
- Move dispatch for capture (existing).
- A post-capture hook that mutates the square's type. The capture
  pipeline already routes through `make_move` / `relocate_pieces`;
  this hook adds one extra step.

## New features required

For Variant B (recommended):

- `Piece::Jackhammer` enum case.
- A "consume on capture" hook in the capture path: when the
  capturing piece is a Jackhammer, instead of moving it to the
  destination, remove it *and* convert the destination square to
  `SquareType::Block`.
- Movement spec (rook-like).
- FEN payload (likely just the piece symbol, no payload).

For Variant A1 (alternative):

- A "displacement on capture" hook: when a piece captures a
  Jackhammer, after the capture, return the capturing piece to its
  origin square and convert the destination to Block.
- Edge handling for displacement collisions (origin occupied by
  someone else mid-turn? Don't ship A1 without resolving this).

## FEN encoding

Symbol: `J` (white) / `j` (black). Free letter; clean.

Payload: none for Variant B (the piece is either present or it
isn't; no state). For Variant A1 also no per-piece payload — the
trigger is reactive.

Examples:

- Jackhammer on e3: `4J3` in that rank.
- Mid-game, Jackhammer-used square (after capture): the square is
  just `T=BLOCK` — the original Jackhammer is gone, only the wall
  remains. No "this used to be a Jackhammer" memory.

## Resolving the source's open questions

The source's explicit open question:

> When taken (or taking another piece? would only be able to use once)

**Answer: Variant B (on capturing).** Detailed reasoning above. The
parenthetical hint "would only be able to use once" lines up with B
— the piece is the wall-placement *tool*, single-use, owner-controlled.

A1 stays documented as an alternative for variants that want
"sacrificial wall on death." Both could ship as distinct piece types
in a future variant; they don't conflict.

## Open questions (new)

- **Can a Jackhammer wall be removed?** Standard rule: no, Block is
  permanent. A future "demolition" piece could be the inverse —
  out of scope here.
- **Does the wall block the engine's signal substrate?** Block
  already has no signal payload. The wall is purely physical.
- **Promotion to Jackhammer?** Standard chess promotes to Q/R/B/N.
  Variants could allow Jackhammer-promotion. Probably fine;
  flagged.
- **Carrier interaction.** A Bus carrying a Jackhammer that gets
  captured: the Bus is captured normally, and the Jackhammer
  payload doesn't trigger (the Jackhammer never *moved* to capture
  anything). Carried Jackhammers are dormant until disembarked.
- **Jackhammer captures another Jackhammer.** Both consumed,
  destination becomes Block. Mutual destruction is intuitive here
  (the wall is the result of *any* drilling action).
