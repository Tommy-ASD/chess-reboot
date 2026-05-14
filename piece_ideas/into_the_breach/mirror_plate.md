# Mirror Plate

> [TOOL] A placeable 1-square tile. Any beam, leash, or telegraphed
> attack crossing it reflects 90°. Doesn't block movement — only
> redirects effects. 1–2 charges per puzzle.

## Inspiration

The portal mirrors in Hoplite. The reflector tiles in countless grid
puzzle games. The fundamental "redirect, don't absorb" verb. Mirror
Plate is the player's **topology-edit tool** — it doesn't kill
anything, it rewrites the route of effects.

## Mechanic

The Mirror Plate is a player tool with N charges per puzzle (default
1–2 depending on puzzle difficulty). Each use places one Mirror
Plate on a chosen empty square. The Plate is then a **square
condition** (in the same family as Frozen, Brainrot) — it doesn't
displace any piece on the square; pieces can move onto/off of the
square normally.

### Placement

The action `UseTool { tool: MirrorPlate, params: { square, orientation } }`:

- **`square`** — any walkable square on the board, including
  occupied squares. The plate goes underneath; the piece stays on
  top.
- **`orientation ∈ {Slash, Backslash}`** — the diagonal axis of the
  mirror. `Slash` mirrors along the `/` diagonal (NE-SW axis);
  `Backslash` mirrors along the `\` diagonal (NW-SE axis).

A square cannot hold two Mirror Plates (one charge per square).
Placing on an existing plate replaces it (and consumes the new
charge — design choice; alternative is reject).

### Reflection rule

When a telegraphed effect crosses the Plate's square, the effect is
**reflected 90°** along the diagonal axis. Effects in scope:

- **Beams** (e.g. [Siege Engine](siege_engine.md)): an incoming
  beam's direction changes per the mirror axis. `Slash` reflects:
  N↔E, S↔W. `Backslash` reflects: N↔W, S↔E.
- **Leash-lines** (e.g. [Latcher](latcher.md)): the Latcher's leash
  passes through the plate and emerges at 90°, terminating at the
  first non-leash-target square along the new direction. (Practical:
  this can redirect the Latcher's "yank target" calculation.
  Detailed semantics: TBD.)
- **Telegraphed attack zones** (e.g. [The Clock](the_clock.md)'s 3×3
  blast): blasts are not directional in the same sense. The blast
  zone is a fixed area, not a beam. Mirror Plate does **not** modify
  blast zones — only directional effects.

What the Plate does **not** affect:

- Piece movement. A piece walking across the Plate square is
  unaffected — it moves normally.
- Push direction. A push is "movement," not a telegraphed effect.
- [Marcher](marcher.md) walking. Marchers move; they don't telegraph
  a beam.

The Plate's reflection is **persistent for the puzzle** — the Plate
stays on the square until the puzzle ends (or until destroyed by
some specific effect, e.g. a Clock detonation in its zone).

## Telegraph rendering

The Plate is a small diagonal mark on the square — a `/` or `\`
glyph half-filled with a reflective shimmer. The orientation is
unambiguous at a glance.

When the player begins placement, all walkable squares glow as
candidate placement targets. The two orientations are toggled via a
key (rotate). The frontend should overlay a *preview* of how each
known telegraphed effect reroutes with the plate in place — e.g. a
loaded Siege Engine's threat-zone dashes redraw with the reflection
applied. **That preview is what makes Mirror Plate playable** — the
player needs to see the consequence before committing.

## Why it's interesting

Mirror Plate **redirects threats into other threats.** The puzzle
becomes:

- Reflect a Siege Engine's beam onto a Clock to harmlessly destroy
  the bomb.
- Reflect a Latcher's leash onto a different "nearest enemy"
  calculation. (Whether this works depends on how leash routing
  through the Plate is spec'd — see below.)
- Reflect a beam back at the Siege Engine that fired it (a single
  Plate can do this with the right orientation).

The "two charges per puzzle" budget lets the player chain reflections
— but rarely. Most puzzles are solved by one well-placed plate; the
second is for the killer 4-piece combo.

Mirror Plates do **nothing** to non-telegraphed pieces. A puzzle of
pure Marchers is unaffected by Plates (Marchers don't telegraph a
directional effect — they just walk). That asymmetry forces the
player to read which threats are "redirectable" vs which need a
different tool.

## Example puzzle

```
6 . . . . . . . .
5 . . . . . . . .
4 . . . . k . . .         k = player king on e4
3 . . . . . . . .
2 . . . . . . . .
1 S . . . . . . .         S = Siege Engine loaded, dir=E
  a b c d e f g h
```

Engine fires across rank 1. King on e4 — rank 4, safe.

Boring. Variant:

```
6 . . . . . . . .
5 . . . . . . . .
4 S . . k . . . .         engine loaded, dir=E
3 . . . . . . . .
2 . . . . . . . .
1 . . . . . . . .
  a b c d e f g h
```

Engine on a4 fires E. Beam goes a4-b4-c4-d4-e4-... King on d4 dies.
Player has **1 Mirror Plate**. Goal: survive.

Place plate on c4 with orientation `Backslash` (`\`). Beam enters
c4 from west, reflects per `\` axis to south. Beam now goes c4 →
c3 → c2 → c1. King on d4 safe.

But wait — placing on c4 puts the plate *under* the would-be path.
The beam crosses c4 (from west). Reflects 90° via `\`. `\` reflects:
the incoming W-to-E vector (direction E) reflects to direction... 
let me think. A `\` mirror has axis from NW to SE. A horizontal beam
hitting it from the west: the incidence is along E direction.
Reflecting across `\` axis: the E vector mirrors to S. So beam exits
heading S. **Beam now goes c4 → c3 → c2 → c1.** Continues until
blocked or off-board.

Good. King on d4 is no longer in the beam's path.

But: does the *original* beam (a4 → b4) reach c4 to be reflected?
Yes — the beam starts at the engine on a4 and travels east. b4 is
empty, beam passes. c4 has the Plate (player just placed it). Beam
hits c4 from west, reflects to S. Beam continues c4 → c3 → c2 → c1.

King on d4 is *east* of c4. Never touched by the redirected beam.

**Puzzle solved with one Plate placement.**

Harder variant — two beams crossing:

```
6 . . . . . . . .
5 . . . S . . . .         S2 = Siege Engine loaded, dir=S
4 . . . . . . . .
3 . . . k . . . .         king on d3
2 . . . . . . . .
1 S . . . . . . .         S1 = Siege Engine loaded, dir=E
  a b c d e f g h
```

Two engines: S1 on a1 fires E (rank 1), S2 on d5 fires S (file d).
S1's beam: a1-b1-c1-...-h1. Doesn't touch d3.
S2's beam: d5-d4-d3-d2-d1. Hits king on d3.

Player has **2 Mirror Plates**. Goal: survive both turns of firing.
(Both fire this enemy phase.)

Place Plate 1 on d4 with `Slash` (`/`). S2's beam enters d4 from N
heading S. `/` reflects S to W. Beam exits d4 heading W. Goes d4 →
c4 → b4 → a4. King on d3 safe.

Place Plate 2... actually, the king is safe from both beams now.
S1's beam never crossed d3 to begin with. Plate 2 unused (carry it
to next puzzle).

Or: spend Plate 2 to redirect S1's beam back at S2 for a kill?
Place Plate 2 on a1 (same square as S1, but the plate is *under*
S1 — wait, the engine is on a1, the plate goes under). When S1
fires, its beam starts at a1 heading E. Does the beam reflect off
the plate *under itself*? Edge case. Spec: a Plate under the
firing piece reflects the outgoing beam at the source. Beam from S1
with plate on a1 (orientation `Slash`): outgoing direction E
reflects via `/` to N. Beam goes a1-a2-a3-... up to a8. Kills
nothing. (Plate 2 wasted.)

Better Plate 2 placement: on d1 with `Backslash`. Wait, S2's beam
got reflected at d4 — it never reaches d1. And S1's beam reaches
d1, heading E. Plate at d1 `\` reflects E to S. d1 is rank 1, so
"south" is off-board. Beam dies. Doesn't help.

Plate 2 on b1 with `Slash`: S1's beam a1→b1 heading E. `/` at b1
reflects E to N. Beam goes b1→b2→...→b5. Empty. Beam dies. King
unaffected.

Plate 2 on h1 with `Backslash`: S1's beam reaches h1 heading E. `\`
at h1 reflects E to S. Off-board. Beam dies. (Same as default.)

Conclusion: **only Plate 1 was needed.** The puzzle teaches that
sometimes 2 charges is overkill — and the player learns to *not*
overspend.

## Where it shines

- Beam puzzles. Any [Siege Engine](siege_engine.md) configuration
  becomes a Plate puzzle.
- Self-destruct setups: bounce a beam back at its source.
- Combo with [Conduit](conduit.md): a beam entering a Conduit emerges
  from every other Conduit; a Plate at one Conduit exit reflects
  that exit's beam, creating very complex routing.
- Teaches geometric thinking about angles, which is rare in chess.

## Where it's awkward

- The leash-reflection rule for Latcher is ambiguous. A "leash"
  isn't really a beam — it's a line from A to B. Does the Plate
  bend it? Two interpretations:
  - (a) The leash is a fixed line A-to-B in *board space*. A Plate
    on the line bends the *visual* but not the *target*. Useless
    against Latchers.
  - (b) The leash is computed via raycast from Latcher; the
    raycast bounces off Plates. The Latcher's actual "nearest" might
    change. Powerful but complex.
  Spec: pick (b) for richness, accept the complexity.
- A Plate persists for the puzzle. The puzzle designer must remember
  this — a Plate placed on turn 1 is still there on turn 5.
- Plates don't affect movement; players will *want* them to. A
  "blocker" tool would be a separate piece.
- Plates under enemy pieces are valid (the piece stays, the plate
  is under). Visually busy.

## Engine dependencies

- Player tool inventory (shared with Shover).
- Square condition system (already exists — Frozen, Brainrot are
  examples). Mirror Plate is a new condition.
- Telegraph routing primitives: beam emit, leash compute. Both must
  consult the square's conditions for plates.

## New features required

- **`SquareCondition::MirrorPlate { orientation }`.** New variant.
- **Beam routing through plates.** The beam-emit primitive checks
  each crossed square for a Plate condition; if present, reflect
  direction and continue. Cap recursion at e.g. 16 reflections to
  prevent infinite loops between two parallel plates.
- **Leash routing through plates.** The Latcher's nearest-enemy
  computation uses raycasts that bounce off plates. Costlier — only
  active in puzzle-mode variant.
- **Telegraph preview API.** The frontend needs a query: "given
  this board and these telegraphs, what are the actual effect zones
  *with* current plates applied?" This drives the preview UI.

## FEN encoding

Mirror Plate is a square condition, encoded in the existing condition
list syntax (parallel to Frozen, Brainrot):

```
(C=MIRROR_SLASH)
(C=MIRROR_BACKSLASH)
```

(Or whatever the existing condition vocabulary uses for keys. The
exact prefix `C=` is a guess based on the project's existing
conditions — adapt to the real syntax.)

Charge counts live in the tool inventory section, same as Shover:

```
... | T:SHOVER=1,MIRROR=2,FLAG=1 | ...
```

## Open questions

- **Plates affecting plates.** Two plates on the same line, both
  reflecting. The beam bounces between them indefinitely unless we
  cap. Cap at 16 reflections; if cap reached, beam dies.
- **Plate destruction.** Can a [Clock](the_clock.md)'s blast destroy
  a Plate in its zone? Spec: yes — blasts strip square conditions
  (per the Clock doc).
- **Plate under a Conduit.** Order of operations: the Conduit
  reroutes the beam to other Conduits, *and* a Plate on the Conduit
  square reflects locally. Which first? Spec: Conduit first (routes
  to other Conduits), then per-Conduit-exit any plates apply. This
  is implementation-heavy; flag it.
- **Diagonal beams hitting plates.** [Siege Engine](siege_engine.md)
  is 4-directional, but a future Lancer might shoot diagonally. How
  does a `/` plate reflect a diagonal beam? Geometrically, a
  diagonal beam parallel to the plate axis passes through unchanged;
  a perpendicular diagonal beam reflects 90°. Spec'd, but verify.
- **Orientations beyond two.** Could add `Cross` (both axes,
  reflects N↔S and E↔W), `Anti` (cross of slash and backslash), etc.
  Start with 2 orientations; expand if puzzles demand.
- **Friendly fire reflection.** A reflected beam can hit the player's
  own pieces. The redirect is geometric, not allegiance-aware.
  Players will sometimes self-defeat — that's the puzzle.
