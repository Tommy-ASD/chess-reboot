# TOGGLE (global/local)

> A single tile with two states. GLOBAL mode applies every rule
> everywhere; LOCAL mode applies rules only near their SUBJECT-tile.

## Role in the grammar

TOGGLE is a **mode-modifier**. It does not appear in clauses directly;
instead, exactly one (or zero) TOGGLE tiles may exist on the board, and
its current state changes how the parser **applies** the registered
rule-effects.

The parser identifies TOGGLE by `token_kind == Toggle, payload ∈
{Global, Local}`.

TOGGLE has no `subj-group` or `pred-group` semantics. It is a board-
level switch that the rule-effect application phase consults.

## Inspiration

Baba Is You has the concept of "this rule only applies HERE." Some levels
restrict rules to specific regions. The chess version generalizes: a
single switch on the board chooses between rule-application modes.

GLOBAL is the default semantics described in every other piece's
documentation: a rule applies everywhere on the board.

LOCAL is the restricted mode: a rule applies only within a king's-move
radius (Chebyshev distance ≤ 1, so the SUBJECT-tile's square and the
eight adjacent squares — a 3×3 box) of the SUBJECT-tile that the rule's
SUBJECT-payload appears on. Outside that 3×3 zone, the rule does not
apply, and the affected piece-type behaves normally.

## Mechanic

### Placement

A TOGGLE tile starts in one mode (`Global` or `Local`) determined by the
FEN. It occupies a single board square. Color::Neutral.

### Movement and capture

The TOGGLE tile slides one square per turn (like other rule-tokens), or
is captured. Capture removes it from the board entirely; with no TOGGLE
on the board, the engine defaults to **GLOBAL mode**.

### Flipping the state

TOGGLE flips between `Global` and `Local` when:

- **Captured and respawned.** v1 does not implement respawn (captures
  are permanent), so this trigger is dormant unless a hand-and-spawn
  feature lands. Document as "future trigger."
- **A piece passes over the TOGGLE's square.** A piece making a move
  *through* the TOGGLE's square (e.g. a glider sliding past) flips it.
  A piece *landing* on the TOGGLE's square captures it (no flip; the
  TOGGLE is gone). The distinction: passing-through is mid-path,
  not the destination; landing is the destination.

  This means glider-paths that include the TOGGLE flip it. A rook sliding
  from a1 to h1 over a TOGGLE at d1 flips the TOGGLE.

- **A specific designated square.** v1 alternative trigger: the FEN
  declares a "flipper square" coordinate. When any piece moves *onto*
  the flipper square (capture or not), the board's TOGGLE flips state
  (regardless of where the TOGGLE itself is). This is the simpler
  v1 mechanism; pass-through is deferred.

### Parser interaction

After the parser collects clauses and atomizes them, the rule-effect
application phase consults TOGGLE state:

- **GLOBAL.** Each atomic rule is applied to all pieces of the named
  type/category on the entire board.
- **LOCAL.** For each atomic rule, compute the set of squares within
  Chebyshev-distance-1 of the SUBJECT-tile's position. The rule applies
  only to pieces of the named type currently within those squares.

In LOCAL mode, **a rule's effective zone moves when the SUBJECT-tile
moves.** Sliding the SUBJECT-tile is now not just changing what's bound,
but changing *where* the binding takes effect.

In LOCAL mode, multi-SUBJECT clauses (`KNIGHT AND BISHOP IS WALL`) have
two SUBJECT-tile positions, each generating its own 3×3 zone. A knight
within the SUBJECT(KNIGHT)'s zone OR the SUBJECT(BISHOP)'s zone is
walled? v1: the rule applies to each SUBJECT-tile's zone independently
for the type that SUBJECT names. So the knight is walled only within
the SUBJECT(KNIGHT) zone; a knight in the SUBJECT(BISHOP) zone is not
walled. Per-SUBJECT-tile zoning.

## Composition rules

- **Exactly one TOGGLE on the board.** If two TOGGLEs are present (e.g.
  in a corrupted FEN), v1 errors at FEN-load. The grammar doesn't
  meaningfully extend to multiple toggles.
- **No TOGGLE = GLOBAL.** Default if absent.
- **TOGGLE state is per-position FEN-serialized.** The flip is a state
  change that round-trips.
- **TOGGLE does not affect ghost-piece spawning (QUOTE).** Ghost-pieces
  are spawned at the midpoint of QUOTE pairs regardless of TOGGLE
  state. Their *predicate bindings*, however, are TOGGLE-aware: a ghost
  knight is walled only if it's within the SUBJECT(KNIGHT) zone in
  LOCAL mode.
- **TOGGLE does not affect default-fallback YOU.** The "king is YOU when
  no YOU-clause exists" rule applies regardless of TOGGLE. The king's
  implicit royalty is not zone-restricted.

## Why it's interesting

TOGGLE turns the **position itself** into a rule-applicability map.

In GLOBAL mode, the puzzle is "which rules are active." In LOCAL mode,
the puzzle is *also* "where on the board are they active." The same
clause has dramatically different effects in the two modes:

- GLOBAL `KNIGHT IS WALL`: all knights paralyzed.
- LOCAL `KNIGHT IS WALL`: only the knight within 1 square of the
  SUBJECT(KNIGHT) tile is paralyzed.

This means in LOCAL mode, the player can have a rule "almost active" —
shifting the SUBJECT-tile a few squares either brings a target knight
into the zone or pushes it out.

The state-flip mechanism turns the entire puzzle on its head with a
single piece move. A position that's safe in GLOBAL can be devastating
in LOCAL (or vice versa).

## Example sentences

The toggle doesn't appear *in* sentences — but example board states:

```
[TOGGLE GLOBAL] and [SUBJECT KNIGHT] [IS] [WALL] elsewhere
  → all knights walled.

[TOGGLE LOCAL] and [SUBJECT KNIGHT] [IS] [WALL] elsewhere
  → only knights within 1 square of the SUBJECT(KNIGHT) tile are
    walled. Other knights move freely.
```

## Example puzzle

```
 . . . . k . . .
 . . . . . . . .
 . . . [SUBJECT KNIGHT] . . . .
 . . . [IS] . . . .
 . . . [WALL] . . . .
 . . . . . . . .
 . . N . . . N .
 . . . . K . . [TOGGLE LOCAL]
```

White knights on c2 and g2. The KNIGHT-IS-WALL clause is on column d,
rows 3-5, with SUBJECT(KNIGHT) at d5. TOGGLE is LOCAL at h1.

In LOCAL mode, only knights within Chebyshev-1 of d5 are walled — that's
the squares c4..e6. Neither knight is in that zone, so both move freely.

Now: white wants to mate the black king on e8. Suppose only the c2
knight can reach (g2 knight is too far or blocked). The puzzle is...
wait, the LOCAL mode makes the clause inert here. White just plays
knight to e6 normally.

The puzzle's twist: white must avoid a square that, by entering it,
flips TOGGLE to GLOBAL. Hidden constraint: the e6 square is the
"flipper square" (FEN-declared). If white plays Nc2-e6, the TOGGLE flips
to GLOBAL, and now both knights are walled — and the c2 knight has just
moved to e6 (it's not walled mid-move, but the wall takes effect at the
*next* parse pass, which is right after the move applies).

Now: e6 knight is suddenly walled. Can it deliver mate? The wall is a
*persistent terrain* effect — the piece is on e6, walled (cannot move),
but it still attacks d8 and f8 from there (does it? walled pieces don't
move, but their threats — do they still threaten? v1: walled pieces
do not project threats either, because they cannot move to capture).
So the knight on e6 is paralyzed and not delivering mate.

Solution: choose a different knight or path that avoids the flipper
square. The puzzle is now a routing problem with a rule-change trap.

## Where it shines

- Two-state puzzles where the same position has two interpretations.
- Local-zone puzzles where the player must move SUBJECTs to selectively
  apply rules to specific pieces.
- Flipper-square traps that make a move "look safe" but flip the rule
  state mid-game.

## Where it's awkward

- **Zone-tracking is computationally heavier than global.** In LOCAL mode,
  the rule-effect registry must store per-rule zones (sets of squares)
  and consult them on every piece-query. Manageable but more code than
  the boolean "this piece-type is walled" of GLOBAL.
- **Multi-tile SUBJECT zones.** If two SUBJECT(KNIGHT) tiles exist on the
  board, each generates its own zone. The union of zones is where the
  KNIGHT IS WALL rule applies. Doubles the geometry per SUBJECT-tile
  count.
- **Flipper-square abuse.** A single board-square that flips the toggle
  on entry is a special-case mechanism. Puzzles using it must declare
  the flipper square in FEN (`flipper=e6`). Not all positions need a
  flipper; absent flipper means TOGGLE only flips by future mechanisms.
- **TOGGLE captureability.** If a player captures the TOGGLE, the board
  reverts to GLOBAL. That can be devastating if the LOCAL mode was
  helping you. Defending the TOGGLE becomes a real tactical concern.

## Engine dependencies

- `VariantId::Baba`.
- The parser's atomization step, modified to produce *zoned atoms* in
  LOCAL mode (each atom carrying a set of valid squares).
- The rule-effect registry consulting zones on every per-piece query.
- A flipper-square mechanism in the move-application code: when a move's
  destination matches the flipper coord, flip the TOGGLE state at the
  same time as the move applies.

## New features required

- **`Toggle::{Global, Local}`** in the RulePiece enum.
- **Zoned-atom representation** in the rule-effect registry.
- **Per-square zone queries** for move-gen and threat-resolution code.
- **Flipper-square FEN field** (`flipper=<coord>`, optional).
- **Toggle-state FEN field** (`toggle=global` or `toggle=local`,
  optional; reads from the TOGGLE tile's payload if present, else
  defaults to global).

## FEN encoding

```
(R=TOGGLE:GLOBAL)
(R=TOGGLE:LOCAL)
```

Payload after the colon names the current state. The state is **on the
tile** (i.e. round-trips through the per-square FEN), not in a separate
board-level field. This makes the TOGGLE's state visible per-square in
the same way a SUBJECT's payload is.

Board-level: optional `flipper=<coord>` field (e.g. `flipper=e6`) names
the special square that flips the toggle on entry. Defaults to absent.

## Open questions

1. **LOCAL mode zone shape.** v1 uses Chebyshev-1 (a 3×3 box). Other
   shapes (orthogonal-only — a + shape; or Chebyshev-2 — a 5×5 box) are
   plausible. v1 fixes the 3×3; future TOGGLE variants could parameterize.
2. **Multiple SUBJECT-tile zones overlapping.** Union; a piece in either
   zone is bound by the relevant rule. Per-SUBJECT-tile zoning ensures
   the rule still applies to the type each SUBJECT names.
3. **Zones for non-SUBJECT-based atomic rules.** Category-SUBJECT atoms
   like `LEAPER IS WALL` have a zone centered on the SUBJECT(LEAPER)
   tile. What if the engine has multiple piece-types in the LEAPER
   category but only one SUBJECT(LEAPER) tile? One zone, covers all
   leapers within it. Consistent.
4. **Flipper-square stacking.** Can a single board have multiple flipper
   squares? v1: at most one. Documented.
5. **TOGGLE during the parser pass.** The parser runs after each move,
   reading the current TOGGLE state. If a move both entered a flipper
   square AND was a piece-move that should be bound by a freshly-applying
   rule, the timing is: move applies, TOGGLE flips, parser re-runs,
   rules apply with the new mode. The piece that just moved is bound by
   the post-move ruleset. The same-turn flip means a "trap move" can
   self-paralyze, as in the example puzzle.
6. **TOGGLE and YOU.** If a YOU-clause is in LOCAL mode, only pieces of
   the YOU-named type within the zone are royal. A YOU-named piece
   outside the zone is **not** royal. Combined with default-fallback,
   the king becomes royal again outside the zone. This produces
   "royalty zones" — kings royal everywhere, knights royal only near the
   SUBJECT(KNIGHT) tile. Capturing the knight outside its royal zone
   does not end the game. Wild but consistent.
