# Blender

> Eats pieces, banks their value, and on a turn of your choosing transforms into any piece at or below its accumulated point value.

## Source

From `engine/src/pieces/ideas.txt:13-16`:

> Blender piece
>     The blender has an internal point score. It starts with a point value of 0, and for each piece it takes, that piece's value is added to the blender's point value.
>     The player controlling the blender can make use a turn to turn the blender into *any* piece of equal or lower value to itself.
>     Maybe add limit to how many point blencerrt can be worth??

That last line — the "blencerrt" typo — is the open question this
doc has to answer. (Yes, we will answer it.)

## Inspiration

The classic chess piece-value table — pawn 1, knight 3, bishop 3,
rook 5, queen 9 — is a *folk* heuristic that engines fudge with for
positional context but humans treat as gospel. The Blender weaponises
that table directly: it makes piece value *literal*, *accumulable*,
and *spendable*. The chess novelty is that it turns the implicit
material economy into an explicit currency.

This is the most "video-gamey" of the original ideas. Levelling up by
eating things, then spending the level on a class change, is RPG
language transplanted into 8×8. That's a feature, not a bug — Chess 2
is openly absurdist, so a piece that XPs is on brand.

## Mechanic

The Blender starts on the board with `points = 0`. It has a base
movement (see below) and gains value by capturing.

### Movement (base)

Moves as a knight. Captures normally.

Justification: a Blender that moves like a queen costs nothing to
load up. A knight Blender is awkward enough that it has to *work* to
land captures, which keeps the buildup grindy and visible. Knight is
also the cheapest "real" piece — symbolically the Blender starts
*below* the things it eats.

### Captures and points

When the Blender captures a piece, that piece's standard value is
added to `points`. Standard values:

| Piece | Value |
|-------|-------|
| Pawn | 1 |
| Knight | 3 |
| Bishop | 3 |
| Rook | 5 |
| Queen | 9 |
| King | (capture ends game; doesn't apply) |
| Goblin | 4 (kidnapper-on-king-legs; price tag matches Bishop+1) |
| Skibidi | 5 (utility scales with phase; midrange feels right) |
| Bus | 3 + sum(passengers) (transparency: the Bus + cargo is one capture) |
| Monkey | 4 |
| Locomotive | 4 |
| Carriage | 1 |
| Block square / other terrain | not capturable |

The values above are the engine's canonical table. If a separate
`Piece::value()` method already exists for AI purposes, the Blender
uses *that* table — single source of truth.

### Transform

On any turn, instead of moving, the Blender's controller may spend
the turn to **transform** the Blender into any piece P such that
`value(P) <= points`. The transform replaces the Blender on its
current square with the chosen piece P; `points` is consumed. The
Blender is gone — the transformed piece is now a normal P with no
memory of having been a Blender.

Side-rules:

- **Transform target must be legal on the square.** Pawns cannot be
  placed on the 1st/8th rank, kings cannot be summoned (per side
  king-count invariant), terrain rules apply.
- **Transform respects king-safety.** If transforming would leave
  the player in check, the transform is illegal.
- **Transform colour.** The new piece is the Blender's colour.
- **One direction.** A piece can't transform back into a Blender.
  The Blender is a one-shot growth engine.

### The cap question — answered

The source asks: *"Maybe add limit to how many point blender can be
worth?"*

**Answer: yes, soft-cap at 9 (Queen value).**

Justification:

- Above 9, the only legal transform targets are still just Queen
  (no piece is worth more than 9), so additional points are wasted.
- A Blender that has eaten 14 points of material is, from a
  game-state perspective, identical to one that has eaten 9.
  Storing the extra points is pure data without consequence.
- Capping at 9 means `points` fits in 4 bits, which is mild but the
  real win is that the *players* can reason about it: "above 9, the
  Blender is fully loaded." A single saturating bound.

The "soft" part: capture *still resolves* normally above 9. You can
keep eating; the points just clamp. The Blender doesn't refuse
captures. Refusing captures would make the Blender's late-game
behaviour weird and rules-lawyerish.

If a future variant wants a higher cap (custom pieces worth 10+),
the cap is a config knob, not a fundamental constraint.

### Edge cases

- **Empty board transform.** If you transform on a square where the
  chosen piece would have zero legal moves next turn, that's the
  controller's problem. Legal.
- **In-check Blender.** Transforming out of check is allowed as long
  as the new piece blocks/resolves the check. (Equivalent to any
  other move-out-of-check rule.)
- **Stuck Blender.** If the Blender has no legal moves *and* no
  legal transform targets (e.g. `points = 0` and the knight has no
  squares), that's stalemate-eligible for that piece, same as any
  other.

## Why it's interesting

The Blender turns the implicit economy of chess into a *visible*
state machine. Every capture has two consequences instead of one:
"I took your queen" *and* "my Blender is now a 9-point pump
primed to become anything." The opponent has to evaluate not just
the immediate trade but the implicit threat-tier of the Blender.

It also forces an unusual decision tree: *when* to transform. Cash
in at 5 (rook) and you have a piece *now*. Wait for 9 (queen) and
you have a bigger piece *later*, but the Blender remains a knight
in the meantime. That tempo question has no analogue in standard
chess.

## Example scenarios

1. **Pawn-grinder.** Blender on c4 eats a pawn (`points=1`). Next
   turn eats another (`points=2`). Two more captures get it to 4 —
   transform into a bishop, mid-game positional upgrade. The
   Blender path traded movement for promotion-without-pawn.
2. **The 9-point gambit.** Black has a Blender at `points=8` after
   eating rook + knight. White realises one more capture caps the
   Blender at queen-tier. White's options: keep the Blender away
   from any capturable piece, or trade material to remove the
   Blender entirely. Both cost tempo.
3. **Bait-and-switch.** White Blender at `points=5`. Black is
   defending a queen. White transforms the Blender into a rook
   instead of pushing for the queen — black's queen-defence was
   wasted preparation.

## Where it shines

- **Variants with extra pieces on the board.** More captures means
  more fuel. A 12×12 board with extra pawns is Blender heaven.
- **Long-game positions.** The buildup needs turns; rapid
  middlegame tactics can outpace it.
- **Compositions that need an "evolve" mechanic.** Cleaner than ad
  hoc promotion rules.

## Where it's awkward

- **Snowball risk.** A Blender that eats early eats often. Mitigated
  by knight-base-movement (limited threat range) and by king-safety
  considerations forcing it to retreat. Playtest will tell.
- **Trade math.** Chess players already do "material count" in
  their heads; a Blender adds a second running tally. Cognitively
  loud.
- **Transform animation.** UI needs to show *what the Blender will
  become* before commit. Otherwise misclicks turn your 9 points
  into a pawn.

## Engine dependencies

- A canonical `value()` for all piece types. If one doesn't exist,
  this piece forces the question.
- Move dispatch for a new transform GameMove.
- FEN payloads (already supports per-piece state via parens).
- King-safety filter (applies to transforms).

## New features required

- `GameMove::BlenderTransform { from: Square, into: PieceType }`.
- `Piece::Blender` enum case with `points: u8` field, saturating at 9.
- `Piece::value()` lookup table — central, used by Blender,
  potentially by AI eval later.
- UI: transform menu showing eligible target pieces at the current
  point level.

## FEN encoding

Symbol: `B` (white) / `b` (black). Conflict: bishop already uses
`B`/`b`. Resolution options:

- Use `BLEND` / `blend` as the multi-letter symbol, consistent with
  `LOCO` / `CART`. **Recommend.**
- Use `Z`/`z` (free letter). Cleaner but less self-documenting.

Payload: `POINTS=<n>`. Default 0, omitted when 0.

Examples:

- `BLEND` on e4, fresh: `4BLEND3` (or however the row encodes;
  payload absent means `points=0`).
- BLEND with 5 points: `BLEND(POINTS=5)`.
- BLEND maxed: `BLEND(POINTS=9)`.

Round-trip:

- Encoder: emit `(POINTS=n)` iff `n > 0`.
- Decoder: parse `POINTS` as `u8`; clamp to 9 with a warn if higher;
  default to 0 if absent.

## Resolving the source's open questions

The source asks one question:

> Maybe add limit to how many point blencerrt can be worth??

**Yes — saturate at 9.** Reasoning given in the [Mechanic
section](#the-cap-question--answered):

1. No transform target above 9 exists in the standard piece set, so
   extra points are mechanically inert.
2. The cap simplifies state (4-bit field) and reasoning ("loaded" vs
   "not loaded" becomes a meaningful threshold).
3. Variants can raise the cap if they introduce 10+ value pieces;
   it's a config knob, not a hard constraint.

The "soft cap" framing — captures still resolve, points just clamp —
preserves the natural feel of the piece while killing the
storage-without-purpose footgun.

## Open questions (new)

- **Should the Blender be allowed to transform into another Blender?**
  Symmetric to "can a king summon a king": no. The Blender is
  one-shot.
- **Captures by promotion / en passant.** A pawn promoting via
  capture doesn't go through the Blender. A pawn captured *by* a
  Blender en-passant: yes, value 1, normal handling.
- **Carrier captures.** When a Bus carrying a queen + knight is
  captured, the Blender gets `Bus + queen + knight = 3 + 9 + 3 = 15`
  in one bite, saturated to 9. Fine — but the *spectacle* of "I ate
  a whole bus" is the point.
- **Transform on enemy turn?** No. Transform consumes the Blender's
  turn. The original spec is explicit.
- **Symbol choice — `BLEND` or `Z`?** Held open. `BLEND` reads
  better in FEN; `Z` is shorter.
