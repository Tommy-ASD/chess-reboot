# Conduit

> [NEUTRAL] Topology piece. Any telegraphed attack-zone entering one
> Conduit emerges from every other Conduit on the board next turn.
> Players and enemies share them. Capture a Conduit to change
> routing.

## Inspiration

Portal tiles. The teleporter mechanic from Imbroglio and 868-HACK.
The network-routing puzzle from any "redirect this signal" game.
Conduits are the **graph-edge piece** — they don't act, they reshape
what connects to what. The board's topology becomes a puzzle
variable.

## Mechanic

A Conduit is a neutral, stationary piece. Multiple Conduits can
exist on the board simultaneously; they form an implicit network
(every Conduit is connected to every other Conduit).

### Effect on telegraphed effects

When a telegraphed effect (beam, blast, leash) enters a Conduit's
square, it emerges from **every other Conduit on the board** in the
*same direction it was traveling*.

Worked example with two Conduits:

- Conduit A on c1, Conduit B on g5.
- [Siege Engine](siege_engine.md) on a1 fires E. Beam: a1→b1→c1.
- Beam enters Conduit A heading E. Beam is re-emitted from Conduit
  B (the only "other Conduit"), heading E.
- Beam continues: g5→h5. Off-board. Beam dies.

The original entry-conduit "consumes" the beam — it does not also
continue forward. The beam is rerouted entirely, not duplicated.

With three Conduits:

- Conduits A (c1), B (g5), C (e8).
- Beam enters A heading E. Emerges from BOTH B and C heading E.
- B's beam: g5→h5, off-board.
- C's beam: e8 — off-board (no square to the east at rank 8... actually
  e8 is fine, beam continues e8→f8→g8→h8 east on rank 8).
- Two beams from a single source — the topology *multiplied* the
  threat.

### Effect on movement (player and enemy walking)

Conduits do **not** route piece movement. A piece walking onto a
Conduit square is fine — the piece occupies the Conduit; the Conduit
doesn't teleport it.

This is the asymmetry that makes Conduits puzzle-shaped: they route
*telegraphed* effects but not *moves*. Players can capture a Conduit
by landing on it normally; doing so removes the Conduit from the
board, changing the routing topology.

### Capture

A Conduit can be captured by any normal piece (player or enemy). It
has no special armor. Capturing removes the Conduit from the network:

- If three Conduits exist (A, B, C), and A is captured, the network
  becomes {B, C}. Beams entering B now only emerge from C, and vice
  versa.
- If two Conduits exist (A, B), and A is captured, the network has
  one node {B}. A beam entering B... there's no other Conduit to
  emerge from. **Spec: with only one Conduit on the board, beams
  entering it disappear entirely** (the beam went into a one-ended
  pipe). Alternative spec: beams pass through unchanged. Pick "beam
  disappears" — creates strategic capture decisions.
- If zero Conduits, no routing. Beams behave normally everywhere.

### Conduit-to-Conduit beam loops

A beam emerging from Conduit B might immediately enter Conduit C
(if B and C are adjacent). The beam re-routes from every OTHER
Conduit — which now includes A (the original entry). Risk of
infinite loop. Cap at 16 redirections; if exceeded, beam dies.

## Telegraph rendering

The Conduit is a circular "portal" sprite, glowing softly with a
unique color per puzzle instance. All Conduits on the same board
share the same color or pulse in sync — visually conveying "these
are linked."

When a beam telegraph touches a Conduit, the previewed beam path
splits and is drawn emerging from every other Conduit, with arrows
showing direction. The player reads the full routed beam path at
once, including all branches.

For player-placeable Conduits (variant rule TBD — see Open
Questions), placement UI previews the network's new topology before
commit.

## Why it's interesting

Conduits make **the board's connectivity into a puzzle variable.**
Most chess pieces interact with a static board. Conduits dynamically
reroute the relationships between distant squares.

The capture mechanic is exquisite. The player faces a choice:

- **Capture a Conduit** to break a routing chain (e.g. prevent a
  beam from emerging at a deadly location). One-time topology edit.
- **Leave it in place** to use it offensively (route a beam into
  a useful kill location).

The network grows quadratically in complexity. Two Conduits =
one route. Three = three pairwise routes (or one one-to-many
emission). Four = six. Each puzzle is a small graph problem.

Combinations:

- [Mirror Plate](mirror_plate.md) on a Conduit's emission point:
  redirect the rerouted beam.
- [Siege Engine](siege_engine.md) firing into a Conduit network:
  one beam becomes many.
- [The Clock](the_clock.md): the 3×3 blast is *not* directional, so
  Conduits don't route blasts. (Spec'd this way for sanity.)
- [Latcher](latcher.md): the leash is directional. Does it route?
  Spec: yes — a leash entering a Conduit emerges from every other
  Conduit. The leash's "target" recomputes based on the routed
  endpoint. Latcher with Conduits is wild.

## Example puzzle

```
6 . . . . . . . .
5 . . . . . . . .
4 . . . k . . . .         k = player king
3 . . . . . . . .
2 . . . . . . . .
1 S . O . . . O .         S = Siege Engine loaded dir=E, O = Conduit
  a b c d e f g h
```

Engine on a1 fires E rank 1. Beam: a1→b1→c1. Conduit at c1.

Without Conduits, beam continues to d1, e1, f1, g1, h1. King on d4
unaffected.

With Conduits at c1 and g1: beam enters c1 heading E. Emerges from
g1 heading E. g1→h1, off-board. Beam dies.

King on d4: never touched. **Same outcome** as without Conduits in
this puzzle — but the player needs to verify by reading the network.

Now move the king into a more dangerous configuration:

```
6 . . . . . . . .
5 . . . . . . . .
4 . . . . . . . .
3 . . . . . . . k         king on h3
2 . . . . . . . .
1 S . O . . . O .
  a b c d e f g h
```

Beam enters c1, emerges from g1 heading E. g1→h1, off-board. King
on h3 safe.

But add a third Conduit at h4:

```
6 . . . . . . . .
5 . . . . . . . .
4 . . . . . . . O         O3 = Conduit on h4
3 . . . . . . . k
2 . . . . . . . .
1 S . O . . . O .
  a b c d e f g h
```

Three Conduits: c1, g1, h4. Beam enters c1 heading E. Emerges from
every other = g1 AND h4. Two beams:

- From g1 heading E: g1→h1, off-board. Dies.
- From h4 heading E: h4→i4 — off-board (board is 8 wide). Dies.

King on h3 still safe.

Hmm, the directional preservation means beams emerging from
non-edge-adjacent Conduits often go off-board immediately. The
puzzles get interesting when the **emission direction matches the
target's location**.

```
8 . . . . . . O .         O3 = Conduit on g8
7 . . . . . . . .
6 . . . . . . k .         king on g6
5 . . . . . . . .
4 . . . . . . . .
3 . . . . . . . .
2 . . . . . . . .
1 S . O . . . . .         S = engine dir=N, Conduit on c1
  a b c d e f g h
```

Wait — engine `dir=N` fires north up file a. That's the engine's
beam path: a1-a2-...-a8. The Conduit on c1 is on file c, not in the
beam. Beam doesn't hit the Conduit network.

Re-set:

```
8 . . . . . O . .         O3 = Conduit on f8
7 . . . . . . . .
6 . . . . . k . .         king on f6
5 . . . . . . . .
4 . . . . . . . .
3 . . . . . . . .
2 . . . . . . . .
1 . . . . S . O .         S = engine on e1 dir=N, Conduit on g1
  a b c d e f g h
```

Engine on e1 fires N (up file e). Beam: e1→e2→e3→e4→e5→e6→e7→e8.
King on f6 is on file f, not in beam path. Beam continues to e8,
off-board. King safe.

Conduits on g1 and f8 — beam never crosses either. Network
irrelevant in this configuration.

Try a different engine direction:

```
8 . . . . . O . .         O3 on f8
7 . . . . . . . .
6 . . . . . k . .         king on f6
5 . . . . . . . .
4 . . . . . . . .
3 . . . . . . . .
2 . . . . . . . .
1 . . . . . S O .         S = engine on f1 dir=N, Conduit on g1
  a b c d e f g h
```

Engine fires N up file f. Beam: f1→f2→...→f6 (king). King dies.

But: Conduit on f8 is on the same file. Beam reaches f6 first
(king's square — king dies before reaching f8). The Conduit doesn't
save the king.

Put the king elsewhere and engineer a Conduit-routing puzzle:

```
8 . . . . . . . .
7 . . . . . . . .
6 . . . . . . . .
5 . . . . O . . .         Conduit on e5
4 . . . k . . . .         king on d4
3 . . . . . . . .
2 . . . . . . . .
1 . . . O S . . .         Conduit on d1, engine on e1 dir=N
  a b c d e f g h
```

Engine on e1 fires N up file e. Beam: e1→e2→e3→e4→e5 (Conduit). Beam
enters e5 heading N. Other Conduits: d1.

Beam emerges from d1 heading N. d1→d2→d3→d4 (king). **King dies.**

The Conduit rerouted the beam into the king's column.

Player tools: **1 Mirror Plate.** Goal: survive.

Option: capture a Conduit. The king is 1 square from d1 (king on
d4, Conduit on d1). King can't reach d1 in 1 move (3 squares away).
Can't capture this turn.

Place Plate at e5 with `Slash` (`/`): when beam enters e5 heading
N, reflects to E. Beam continues e5→f5→g5→h5, off-board. **But
does the reflection happen before or after the Conduit routing?**
Spec'd in [mirror_plate.md](mirror_plate.md): "Conduit first, then
plates on emission points." So at e5: Conduit routes first. Beam
emerges from d1 heading N. The plate on e5 doesn't reflect (the
beam's path *through* e5 was consumed by routing).

Wait, that's the question. Re-read the spec: "any beam crossing a
Plate reflects 90°." If the beam enters e5 (Conduit + Plate), does
the Plate reflect *before* the Conduit reroutes? Order matters.

**Decision needed:** spec'd as "Conduit first, then plates on
emission points." So plate on entry-conduit is ignored. The beam
enters the Conduit, gets routed, emerges from d1, and plates on
d1 (if any) would then reflect the emerging beam.

With Plate on e5: no effect (entry-conduit). King dies.

With Plate on d1 `\`: beam emerges from d1 heading N. At d1 the
plate is *on the Conduit's square*. Does it reflect the *emerging*
beam? Spec: yes — emission-conduit plates reflect. Plate `\` reflects
N→W. Beam from d1 heading W: d1→c1→b1→a1, off-board. King safe.

So the answer is **Plate on d1 with `\`.** But d1 is the Conduit
square — placing a Plate on it requires the rules allowing plate +
conduit on same square. Spec'd: yes (Plate is a square condition,
not a piece; Conduit is a piece. Condition + piece coexist).

**Puzzle solved with one Plate.** The trick is reading the Conduit
routing and finding the *emission* point to redirect.

## Where it shines

- Network puzzles. Any puzzle with 2+ Conduits creates a routing
  problem on top of the regular threat analysis.
- The asymmetry between movement and effect routing creates
  unexpected solutions ("I can capture this Conduit because moving
  onto it doesn't teleport me").
- Composes with every directional telegraph piece.

## Where it's awkward

- High cognitive load. Players must trace beams through the entire
  Conduit graph. A 4-Conduit network has 12 routing paths to
  consider.
- The "every other Conduit" rule emits multiple beams from one
  entry — which often go off-board uselessly. Most Conduit
  placements waste most of their emissions. Designer must intend
  this.
- Blast (non-directional) routing is undefined. Spec says: blasts
  don't route. But players will expect the bomb to teleport through
  the network. Manage expectations in tutorial.
- Capturing a Conduit changes routing mid-puzzle; the player's
  threat analysis must update. Visual rerendering is important.

## Engine dependencies

- Neutral piece slot for the Conduit.
- Telegraph routing primitives: beam emit, leash compute.
- Per-turn graph construction (all Conduits' positions cached for
  routing queries).

## New features required

- **`Conduit` piece kind.** `Piece::Conduit`. Neutral. No payload —
  identity is positional.
- **Beam-routing-through-conduit primitive.** Modify beam-emit:
  when crossing a Conduit, terminate the current beam and spawn
  new beams from every other Conduit on the board, in the same
  direction. Track redirection count, cap at 16.
- **Leash-routing-through-conduit primitive.** Same logic for
  Latcher's leash raycast.
- **Conduit-graph cache.** A per-board cache `Vec<Square>` of
  Conduit positions, rebuilt when a Conduit is added/removed.
  Avoids O(n²) scan per beam.

## FEN encoding

```
(P=CONDUIT,C=NEUTRAL)
```

No payload. The "network" is implicit from positions. No order, no
ID — every Conduit on the board is equivalent.

A future variant might add named networks: `(P=CONDUIT,C=NEUTRAL,N=A)`,
where only Conduits with the same `N` value route to each other.
This enables multi-network puzzles. Defer to a later plan.

## Open questions

- **Single-Conduit behavior.** What does a lone Conduit do when a
  beam enters it? Spec: beam disappears (one-ended pipe). Alternative:
  beam passes through unchanged. Both defensible. Pick "disappears"
  for richer capture-strategy.
- **Network naming.** Mentioned above. Default: one network for
  all Conduits. A `N=` tag could enable multi-network. Bikeshed
  later.
- **Emergence direction.** Spec: same direction as entry. Alternative:
  Conduits have an `out=` direction; beams emerge in that direction
  regardless. Probably too restrictive — keep "same direction."
- **Conduit-on-Conduit.** Two Conduits adjacent: beam through A
  emerges from B heading same direction. The next square is B's
  emission — but B is itself a Conduit. Does emergence re-route?
  Spec: yes, the loop-cap handles infinite cases. Worth careful
  testing.
- **Beam splitting.** With three Conduits, one entry produces two
  emergences. Two emergences could both enter Conduits again,
  producing further splits. Total beam count grows. Cap by total
  beam count too (e.g. max 32 active beam segments per phase) to
  prevent worst-case explosion.
- **Movement-routing variant.** A "Wormhole" variant of Conduit
  that routes piece movement (player walks onto Conduit, emerges
  from another Conduit). Probably a *separate* piece, not a Conduit
  variant. Naming: "Wormhole" or "Portal."
- **Player-placeable Conduits.** Currently no [TOOL] places a
  Conduit. A "Conduit Anchor" tool could exist — places a Conduit
  for the duration of the puzzle. Cool but unnecessary for v1.
