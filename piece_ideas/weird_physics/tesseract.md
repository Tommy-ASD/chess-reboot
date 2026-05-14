# Tesseract

> A piece that lives on a second board layered above the visible
> one — touchable only on the rare turns when its coordinates on
> both layers coincide. [DIMENSIONALITY]

## The law it breaks

Chess is two-dimensional. Every piece occupies one square in a
single grid. The Tesseract refuses this by existing on a second
parallel grid laid directly atop the first — same files, same
ranks, *same square-set*, but a different "altitude." On its
own layer, the Tesseract moves freely. On the real board, it is
absent. Only when both layers align — i.e., when its upper-layer
position coincides with a shared "landing" square — does the
Tesseract briefly *drop* into the real board where it can
capture and be captured.

The break is dimensional. The Tesseract's position is a pair
`(visible_pos, upper_pos)`. Most turns, only `upper_pos` is
meaningful; on landing turns, `visible_pos = upper_pos` and the
piece is on the real board.

## Mechanic

State per Tesseract instance, stored in FEN:

- `upper_pos: Square` — position on the hidden layer.
- `dropped: bool` — true on the single turn the piece is on the
  real board.
- `landings: Vec<Square>` — a static, position-defined set of
  *shared squares* where the upper layer connects to the real
  board. Configurable per variant; default is the four corners
  of the board (a1, a8, h1, h8 on 8x8) plus the centre cell or
  centre pair.

Movement primitive (upper layer):

- Slides like a rook on the upper layer. Range unrestricted on
  empty layer (the upper layer has no other pieces by default).
- Cannot occupy a square in `landings` unless the owner chooses
  to *drop* on that turn.

Turn flow:

1. **Upper move.** Owner moves the Tesseract on the upper
   layer: pick any square reachable by a rook ray from
   `upper_pos`, excluding `landings`. `upper_pos` updates,
   `dropped = false`. This is the default per-turn action.
2. **Drop turn.** When `upper_pos` is *one rook-ray step* from
   a landing square *and* the corresponding real-board landing
   square is empty (or contains an opposing piece), the owner
   may instead declare a **drop**:
   - Move `upper_pos` onto the landing square.
   - Set `dropped = true`.
   - Spawn the Tesseract on the real board at the landing
     square. If an opposing piece occupied that square, it is
     captured. Friendly piece blocks the drop.
3. **Real-board turn.** While `dropped = true` (the next owner-
   turn), the Tesseract moves as a normal piece on the real
   board. Default real-board mover: queen, any direction, any
   range. It may capture normally. After this move, the
   Tesseract *immediately* lifts back to the upper layer at
   the same square (`upper_pos = current real square`,
   `dropped = false`). The real-board square is vacated.
   - If the Tesseract is captured during the opponent's turn
     while `dropped = true`, it dies normally.
4. **No-drop default.** Most turns, the Tesseract moves
   invisibly on the upper layer. Opponents see `upper_pos` in
   the FEN (no hidden information) but cannot interact with it
   — no real-board attack on `upper_pos` reaches the piece.

**Untouchability rules while not dropped.**

- The Tesseract cannot be captured, checked, blocked, or pinned
  by real-board pieces.
- The Tesseract cannot attack, block, or pin real-board pieces.
- The Tesseract does *not* count toward stalemate, repetition,
  or material balance while undropped (configurable).

**Landings.** A static set defined at game start. The default
set (four corners + centre) gives the Tesseract a small handful
of touchpoints — usable but not dominant. Variants may set
different landing patterns: edges, central cross, etc.

## Why it's interesting

The chess novelty: the Tesseract is a *delayed threat that
chooses its delivery square*. Its menace is always visible
(opponents see `upper_pos`) but unactionable until a drop
occurs. Defending against it means controlling the landing
squares — not the Tesseract's own square. The piece reshapes
which squares the opponent must defend.

The conceptual elegance: two coordinate spaces with a small
shared sub-grid. The break is the *separation* of layers; the
landings are the only seam. The engine encodes the second
dimension as one extra field and one boolean.

## Example scenarios

- **The corner drop.** Black Tesseract has `upper_pos = a2`,
  one step from landing a1. Black has a passed pawn on a7.
  On Black's turn, the Tesseract drops onto a1 (capturing the
  White rook stationed there), then on the *next* turn, the
  Tesseract makes a queen move from a1 to push the position,
  then lifts back to the upper layer at the new square.
- **The unstoppable transit.** White Tesseract spends 8 turns
  on the upper layer moving freely from a8 to h1 — across the
  board in a path no real-board piece can interrupt. The
  opponent must garrison every landing square to prevent the
  drop.
- **The committed drop.** White Tesseract drops onto e4
  (centre landing) on turn 12. From turn 13 it acts as a
  queen on e4. Black captures it on turn 14 — the Tesseract
  was vulnerable for exactly one turn between drop and lift.

## Where it shines

- Late game where the Tesseract's mobility makes it a flexible
  trump card — committed to a queen's worth of attack at the
  moment of the owner's choosing.
- Variants with many landing squares: the Tesseract becomes
  near-omnipresent.
- Positions where the opponent's pieces are over-extended and
  cannot garrison every landing.

## Where it's awkward

- **Two-board mental load.** Players must track `upper_pos` on
  every Tesseract. The UI has to render the upper-layer state
  prominently — a small minimap or a ghost overlay on the main
  board.
- **Information asymmetry by surface area.** While the
  Tesseract is fully visible in the FEN, its tactical
  consequences are non-obvious to humans. New players will
  forget the Tesseract exists for stretches of game.
- **Landing-square scarcity.** Few landings = the Tesseract
  has few real opportunities. Too many = the Tesseract is a
  free-roaming queen with safe travel. Calibration matters.
- **Stalemate edge case.** If the Tesseract is undropped and
  has no legal upper-layer moves (every reachable square is
  blocked by other Tesseracts on the upper layer), and its
  side has no other legal move, is it stalemate? Probably yes,
  but the upper layer normally has no other pieces. Resolution:
  Tesseracts of the same side share the upper layer and can
  block each other.
- **Two-Tesseract collision.** Opposing-side Tesseracts share
  the upper layer. Can they capture each other on the upper
  layer? Default: no — upper-layer attacks have no resolution.
  Tesseracts only interact through real-board drops.

## Engine dependencies

- Per-piece FEN payload (exists).
- Move-generation hook that bypasses real-board legality for a
  piece type (Tesseract's upper move ignores real-board state).
- Threat-generation must exclude undropped Tesseracts from
  threat and threatened sets.

## New features required

- **Hidden parallel grid.** The Tesseract's upper layer is
  not a full board — it's a coordinate set co-located with the
  real grid. Implementation: an extra `Square` field on the
  Tesseract's payload, and movement rules that operate on it
  without touching `Board::squares`.
- **Per-piece "is this piece on the real board?" predicate.**
  Most engine systems already iterate pieces; the predicate
  filters undropped Tesseracts out for threat/pin/capture
  queries.
- **Drop and lift moves.** `MoveKind::TesseractDrop { landing }`
  and an implicit lift on the post-drop real-board move (or an
  explicit `MoveKind::TesseractLift`).
- **Landing-square configuration.** A board-level constant
  list (or per-variant config). Renders to the UI as marked
  squares.
- **Upper-layer rendering.** Frontend overlays — a ghost glyph
  on `upper_pos`, a marker on each landing square, an
  indicator on `dropped`.

## FEN encoding

Tesseract piece-id `T`. Two cases:

**Undropped:** the Tesseract does not occupy a real-board
square. The FEN piece-grid does *not* show it. A separate
extension field declares its upper position:

```
TESS=W:e4,B:a7
```

(Comma-separated per side, file-rank notation.)

**Dropped:** the Tesseract occupies a real-board square *and*
the same square on the upper layer. The piece-grid shows it as
a normal piece, with `DROP=1`:

```
... (P=T,COL=W,DROP=1) on e4 ...
```

When `DROP=1` is present, `upper_pos = visible_pos` implicitly.
The `TESS=` field still records the same square for
consistency.

`LANDINGS=` is part of the variant config, not per-position.
Stored once in the position header:

```
LANDINGS=a1,a8,h1,h8,e4,e5
```

## Determinism notes

- `upper_pos` is FEN-visible; both players see it.
- The landings are static and visible.
- `dropped` is visible.
- Move legality on the upper layer is a pure function of
  `upper_pos` and other Tesseracts' upper positions.
- The drop predicate is a pure function: "is `upper_pos`
  rook-adjacent to an empty/opposing landing?"
- Lift is automatic on the post-drop turn, no choice.
- No randomness. No hidden info — only a separation of
  *attack surface*, not *information*.

## Open questions

- **Upper-layer mover.** Rook is the default. Bishop on the
  upper layer would make the landings-as-corners default
  unreachable. King-step is too slow. Rook fits.
- **Landings configurability.** Static at game start or
  modifiable mid-game (via some other piece)? Default: static.
- **Multiple Tesseracts per side.** Default: one. Two
  interact through upper-layer blocking, which is fine but
  may surprise players.
- **Stalemate inclusion.** Does an undropped Tesseract count
  for stalemate when the side has no other moves? Default:
  yes — the Tesseract always has an upper move unless
  geometrically trapped, so this rarely fires.
- **King-Tesseract.** Forbidden by default; a king that can't
  be captured for most turns breaks check rules.
- **Centre-cell landings on odd boards.** The same mirror
  problem as Twin — handle by explicit landing list, not by
  derived "centre."
