# Vampire

> A King-mover that absorbs the movement set of every piece it
> captures, accumulating into a late-game chimera.

## Inspiration

The `Bus.passengers: Vec<Piece>` field is the engine's existing
template for a piece that carries a variable-length list of other
pieces' types. The Vampire reuses that exact shape — `absorbed:
Vec<PieceType>` — but instead of *carrying* pieces it carries their
*movement primitives*.

The Goblin already establishes that the engine can do
"piece-as-actor with captured-thing state": when a Goblin captures,
the victim becomes the Goblin's passenger and is returned home. The
Vampire is the same shape but the absorbed thing is a moveset, not
a piece. Both pieces share the answer to "how do you serialize a
list of `PieceType` next to a piece in FEN" (plan 06's
parenthesized payload syntax).

The design problem: how do you make individual captures *matter*
across many moves? Standard chess rewards captures with material
swing, but the captures themselves are forgotten. The Vampire
gives the player an in-game record of what it has eaten — and that
record changes how it threatens the next position.

## Mechanic

Initial movement set: identical to King.

State: `absorbed: Vec<PieceType>`. Empty at the start of the game
unless FEN-authored otherwise.

When the Vampire captures a piece of type `T`:
1. The victim is removed (normal capture).
2. `T` is appended to `absorbed`.
3. The Vampire's effective movement set becomes the *union* of King
   plus every move-primitive in `absorbed`.

Move-gen enumeration:
- The Vampire's move-gen iterates `[King] + absorbed.iter()` and
  asks each piece-type's move generator for its candidate moves
  *from the Vampire's square*. Concatenate, dedupe.
- Captures by any of the absorbed movesets still count as captures
  by the Vampire, so they also feed `absorbed`. Snowball.

Special rules:
- Absorbing pieces of type `K` (King) or another `Vampire` is
  disallowed at capture-time (King capture is win-condition; Vampire
  capture absorbs the *empty list*, which is a no-op — but if the
  victim Vampire had absorbed pieces, the absorbing Vampire takes
  the *union*). Edge case; see open questions.
- Absorbing carrier pieces (`Bus` with passengers, `Reanimator` with
  graveyard, another `Vampire` with absorbed list) takes the
  carrier's *movement primitive* (slider for Bus, King for
  Reanimator and Vampire) but does not inherit their carried list.
  Passengers/graveyard/absorbed-of-victim die with the victim — by
  whatever the engine's existing rule for carrier death is.

## Why it's interesting

Three layers of novelty:

1. **Statefulness that scales with game length.** A Vampire at move
   5 is a King. A Vampire at move 50 with three captures behind it
   may move like King + Queen + Knight + Rook. The same piece has a
   different threat profile at different game phases.

2. **Cross-piece composability.** The Vampire is the only piece (so
   far) whose mechanic emerges from *other* pieces' mechanics. It's
   parasitic on the rest of the piece set. Adding a new piece to the
   variant automatically makes the Vampire deeper.

3. **Reuses the Bus passenger shape.** The engine already has a
   `Vec<PieceType>` slot on a piece, serialized via plan 06's
   parenthesized payload. The Vampire is a near-zero-cost extension:
   one new piece-type, one new payload key, one new move-gen entry
   that delegates to existing per-piece move generators.

## Example scenarios

**Early Vampire.** White Vampire on d4, move 12. Has captured one
black Knight (`absorbed: [N]`). The Vampire now threatens (a) all
King squares around d4, (b) all Knight squares from d4 — b3, b5, c2,
c6, e2, e6, f3, f5. Combined threat: 16 squares.

**Late-game terror.** Black Vampire on e5, move 45. `absorbed: [Q,
N, R]`. Effective moveset: King + Queen + Knight + Rook =
essentially "Amazon + Knight" = move anywhere along eight lines,
plus the eight knight moves, plus one king step (redundant). Black
just has a single super-piece roaming.

**Vampire vs. Vampire.** Both sides have a Vampire. White's has
absorbed `[N]`, black's has absorbed `[B, R]`. White captures black
Vampire (one-step King move, since they were adjacent). White's
Vampire now has `[N, B, R]` — the union. The captured Vampire's
absorbed list transferred wholesale. Vampires are
hyper-transitive.

## Where it shines

- Long games with many captures. Vampire value compounds.
- Variants with diverse piece types. The more piece-types the
  Vampire can eat, the broader its endgame moveset.
- King-hunt endgames. A Vampire that's eaten a Queen and a Knight
  out-paces an unsupported enemy king.

## Where it's awkward

- Short games. A Vampire with empty `absorbed` is just a King — a
  weak piece you have to keep alive for its potential.
- Symmetric Vampire setups can devolve into a single chimera
  decided by which side captured first.
- Move-gen cost grows linearly with `absorbed.len()`. Each absorbed
  type adds its own enumeration. Not catastrophic, but a Vampire
  with 10 absorbed types is doing 10× the move-gen work of a
  Queen. Probably fine; flag for measurement.
- Visual readability — at a glance the Vampire's threat squares
  aren't obvious. Frontend will want an "explain this piece's
  moves" affordance.

## Engine dependencies

- The `Bus.passengers: Vec<Piece>` precedent for storing a
  list-of-piece-types on a piece, including FEN serialization.
- Per-piece-type move generators (already exist for every piece in
  the variant).
- The capture pipeline (already centralized — captures are observed
  in one place).

## New features required

- `Piece::Vampire { absorbed: Vec<PieceType> }` (or as a payload
  field on the existing `Piece` struct, mirroring Bus's shape).
- A post-capture hook: when the Vampire captures, append victim's
  `PieceType` to `absorbed`. The hook already has a slot (per plan
  04 for Goblin); Vampire reuses it.
- Move-gen entry: emit `King` moves ∪ for-each `absorbed`: emit
  that piece-type's moves from the Vampire's square.
- FEN encoder + decoder for `(P=VAMP,A=(Q,N,R))`.
- Tests: capture-and-absorb round trip, FEN round trip with various
  `absorbed` lists, Vampire-eats-Vampire transfer.

## FEN encoding

Piece symbol: `V` (Vampire) — clean. Multi-character payload for
the absorbed list, mirroring plan 06's Bus syntax:

```
(P=V,A=(Q,N,R))      # white Vampire that has eaten Queen, Knight, Rook
(P=v,A=())           # black Vampire, no captures yet (equivalent to V with empty A)
(P=V)                # white Vampire, empty A elided
```

Key `A` for "absorbed." Empty `A` is elidable (same convention as
Bus's `P=()`). Inner list is comma-separated `PieceType` tags
using existing one-letter conventions (`K=King`, `Q=Queen`,
`R=Rook`, `B=Bishop`, `N=Knight`, `P=Pawn`, with fairy pieces
spelled out where appropriate).

Color discipline: `absorbed` entries are unsigned (no case);
direction-sensitive pieces (Pawns) inherit the Vampire's color
when their moveset is enumerated.

## Open questions

- **Pawn absorption.** A Vampire that eats a Pawn — does it gain
  pawn moves? Pawn moves are color-direction-dependent. The
  Vampire's color is fixed, so a White Vampire with `absorbed:
  [P]` gains white pawn moves (forward = up the board). Probably
  fine. Open: does it also gain promotion if it reaches the
  back rank using pawn moves? Recommend no — promotion is a
  pawn-identity thing, not a moveset thing.
- **King absorption.** Disallowed at capture (you've won by
  capturing a King). FEN-authored Vampire with `A=(K)` produces
  what — King + King moves = King moves. Inert. Recommend
  allow + warn.
- **Vampire absorbs another Vampire's `A` list.** Recommended:
  union the lists (set semantics, not multiset — same `PieceType`
  twice in `A` is redundant). Open: should there be a soft cap on
  `A.len()` to prevent runaway move-gen cost in pathological
  Vampire-eats-Vampire chains? Recommend no cap; rely on game
  length to bound it naturally.
- **Goblin interaction.** A Vampire captures a Goblin carrying a
  passenger. The Vampire gains King moves (Goblin's primitive),
  the passenger drops where the Goblin was (per plan 04). The
  Vampire does *not* gain the passenger's moveset, only the
  Goblin's. Correct? Mechanically yes — the Vampire ate the
  Goblin, not the passenger. Document explicitly.
- **Move-gen ordering / determinism.** When the Vampire's
  effective moveset is the union of multiple piece-type
  generators, move ordering for evaluator stability matters.
  Pick a canonical order: King first, then `absorbed.iter()` in
  insertion order. Already deterministic.
- **Pin/check legality.** A Vampire with absorbed Queen is in
  check from a Bishop. Can it block by moving like a Knight to
  the blocking square? Yes — same as any piece with multiple
  movesets. Legality is per-candidate-move, not per-piece-type.
