# Mitosis

> A piece that periodically splits in two and periodically
> recombines — net material is preserved, but expressed across a
> variable number of half-pieces. [CONSERVATION / IDENTITY]

## The law it breaks

Chess conserves piece count except by capture and promotion. A
piece's value is fixed by its type; you cannot trade two halves
of a rook for an extra rook later. The Mitosis breaks this: it
fissions on a schedule into two half-value pieces, and any two
adjacent halves can fuse back. The piece's *material* is
conserved (two halves = one whole) but its *granularity* is
not. The board state has an unstable number of Mitoses at any
moment.

The break is mechanical, not metaphysical. The half-piece is a
real distinct piece-token with its own moveset, value, and FEN
entry. The whole is the same; they share the type-id, only the
`half` flag and the counter differ.

## Mechanic

State per Mitosis instance, stored in FEN:

- `phase: u32` — turn counter, increments every owner-turn.
- `half: bool` — false (whole) or true (half).

Movement primitive:

- **Whole.** Moves as a queen, range 2. Captures normally.
- **Half.** Moves as a knight. Captures normally.

Schedule (per-instance):

- A *whole* Mitosis fissions when `phase mod 4 == 3` and the
  owner chooses to commit. On that owner-turn, instead of a
  normal move, the owner declares fission: the Mitosis is
  removed and replaced by two halves on adjacent squares
  (orthogonal or diagonal to its current square). Both
  daughter halves start with `phase = 0`, `half = true`. The
  two destination squares must be empty (or contain an
  opposing piece — see *capture-on-fission* below).
- A *half* Mitosis can fuse when `phase mod 4 == 3` and the
  owner chooses to commit, **and** there is another friendly
  half Mitosis on an orthogonally or diagonally adjacent
  square. On the owner's turn, both halves are removed and a
  single whole Mitosis appears on either of the two former
  squares (owner picks). The new whole starts with `phase = 0`,
  `half = false`.
- If the owner chooses *not* to commit fission/fusion on a
  phase-3 turn, the Mitosis moves normally and `phase`
  increments. The piece is *eligible* to commit only on every
  fourth turn, but not *required*.

Fission and fusion both consume the owner's turn. They are
moves, not free actions.

**Capture-on-fission.** If a fission destination square is
occupied by an opposing piece, that piece is captured. Both
destinations can be captures, producing two captures in one
fission turn. (Friendly occupation blocks the fission entirely.)

**Capture-on-fusion.** Fusion does not produce captures. The
two halves' squares must be friendly half-Mitoses; one of them
remains the post-fusion square.

**Capture of a half.** Capturing one half kills only that half.
The remaining half is intact and continues its own phase
counter. The Mitosis "line" survives partial capture.

**Capture of a whole.** Captures the whole piece, end of line
for that token.

## Why it's interesting

The chess novelty: material *worth defending* is unstable. A
whole Mitosis on a key square has a four-turn clock — if it
fissions, the key square becomes empty. If the opponent waits
out the clock, the position changes drastically without a move
of theirs. Conversely, two halves coordinated into a fusion
candidate offer flexibility: the player commits material to a
new square at the moment of fusion. The piece punishes both
hyperactive piece-management (constant fissioning, four scattered
halves) and stasis (a single whole that never adapts).

The conceptual elegance: a finite struct (`phase`, `half`) with
a deterministic schedule produces a piece whose count varies
over time. No randomness, no hidden state. The break is
arithmetic: a counter modulo a constant.

## Example scenarios

- **The swarm fork.** White Mitosis on d4, phase 3. Fissions
  into halves on c5 and e5. Both halves attack Black's king
  on f6 and queen on d6 — a knight fork from two new pieces.
  One turn produced two attackers from one.
- **The fission ambush.** Black Mitosis on c4, phase 3. White
  pawn on b5. Black fissions into b5 (capturing the pawn) and
  d5. One pawn lost, two new halves placed.
- **The fusion trap.** White halves on f3 and g3, both phase 3.
  Black has a knight on h5 attacking g3. White fuses on f3,
  removing the g3 target and producing a whole queen on f3 —
  the knight's threat evaporates and Black has wasted tempo.
- **The decay.** White Mitosis on a1 fissions every four turns,
  with no consolidation. After 16 turns, eight halves are
  scattered across the queenside. Black ignored them; suddenly
  half the position is full of knight-jumpers.

## Where it shines

- Open middlegames with space for halves to disperse.
- Defensive standoffs: the Mitosis's clock forces both players
  to time their commitments to phase boundaries.
- Variants with restricted material — a single Mitosis is
  effectively a *renewable* resource.

## Where it's awkward

- **Fission destination scarcity.** If a whole Mitosis is
  pinned in a tight corner, it may have no legal fission. The
  rule then is "fission is illegal this turn; the piece must
  make a normal move and remain whole." This is fine but
  produces edge-case stalemate questions.
- **Phase counter drift across captures.** A captured half
  takes its phase with it; the surviving half continues on
  its own clock. Players need to track per-instance phases.
  Surfacing in the UI is non-trivial.
- **Promotion interaction.** Mitosis is a distinct piece type;
  pawns don't promote into it (by default).
- **Material counting.** Two halves != one whole for material
  evaluation in some sense — they cover more area but each is
  weaker. Engine value-tables must treat half-Mitosis as a
  distinct entry (~knight value) and whole-Mitosis (~rook
  value).
- **Schedule sync.** Multiple Mitoses with offset phases
  produce a busy clock. The board can have one fission and
  one fusion happening on the same turn (across two
  instances). Display gets crowded.

## Engine dependencies

- Per-piece FEN payload (exists).
- Turn-counter or per-piece phase counter mechanism.
- Multi-piece-removal-and-spawn effect type (similar to what
  Apocrypha needs).

## New features required

- **Phase counter on piece payload.** A small uint per instance,
  incremented on the owner's turn. Generic — usable by other
  scheduled pieces.
- **Fission move type.** `MoveKind::Fission { dest_a, dest_b }`
  with two destination squares. Captures on either or both
  count as normal captures.
- **Fusion move type.** `MoveKind::Fusion { partner, dest }`
  identifying the second half by square and the post-fusion
  destination.
- **Eligibility predicate.** On each Mitosis's turn-start, the
  engine determines whether fission/fusion is *available* and
  surfaces this in the legal-move list. The owner is never
  forced to commit, only offered the option.
- **Half-piece type.** Either a distinct `PieceType` or a flag
  on the existing Mitosis type. The latter is simpler.

## FEN encoding

Mitosis piece-id `M`. Payload tracks `PHASE` and a half-flag
`H`:

```
(P=M,PHASE=3,H=1)
```

- `PHASE` — per-instance turn counter mod 4 (or unbounded; the
  engine only checks `mod 4`). Default 0 if absent.
- `H`     — 0 (whole) or 1 (half). Default 0 if absent.

Example: White whole Mitosis on d4 at phase 2:

```
... (P=M,COL=W,PHASE=2) on d4 ...
```

Half flag omitted defaults to whole.

A position mid-game with one whole and two halves:

```
(P=M,COL=W,PHASE=2) on d4
(P=M,COL=W,PHASE=1,H=1) on c5
(P=M,COL=W,PHASE=2,H=1) on g3
```

Each instance carries its own phase.

## Determinism notes

- The fission/fusion schedule is `phase mod 4 == 3`, fully
  derived from the FEN-visible counter. No clocks, no
  randomness.
- Owner *chooses* to commit fission/fusion when eligible, but
  the choice is announced on-turn — both players see the
  legal-move list including any fission options.
- Fission destinations are owner-chosen from the set of legal
  pairs (both adjacent, both legal); no randomness.
- Fusion partner and post-fusion square are owner-chosen.
- Multiple Mitoses with offset phases produce independent
  schedules; no global Mitosis state.
- The whole-vs-half distinction is FEN-explicit; no implicit
  inference.

## Open questions

- **Fission timing.** Phase 3 = "every 4th turn" is one knob.
  Phase 5 or 7 produces a slower clock — better in long
  endgames but tedious in short games. Default: 4. Variant-
  configurable.
- **Fission destinations.** Adjacent (king-step) or knight-step
  or queen-ray range-2? Default: king-step. Knight-step
  produces dramatic fissions but doubles the rule complexity.
- **Half-mover.** Half = knight is the obvious symmetry-break
  with a whole's range-2 queen, but other choices work (half =
  ferz, half = wazir). Knight is the most distinct.
- **Fusion-into-promotion.** When two halves fuse on the back
  rank, does the result promote? Default: no, Mitosis does
  not promote.
- **Maximum population.** A single Mitosis can spawn 2^N halves
  in 4N turns. Should there be a cap? Default: no cap; the
  game ends or the player consolidates first.
