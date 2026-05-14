# Shover

> [TOOL] Player piece. Knight-leaps to an empty square; the piece
> adjacent to the landing in line gets pushed one square directly
> away. One charge per puzzle. Arrangement, not killing.

## Inspiration

The Push Mech in Into the Breach. The shove action in Hoplite. The
displacement primitive of every grid puzzle game. The Shover is the
**player's positioning verb** — a hammer that *moves* enemies
without destroying them. Its existence is what makes the other
[TOOL] pieces interesting; every puzzle has at least one piece that
needs to be *somewhere else*.

## Mechanic

The Shover is a tool, not a permanent board piece. It is held in the
player's tool inventory and consumed when played. (See "Engine
dependencies" — this requires the player-inventory feature, new.)

### Action: Shove

The player spends the Shover's charge to perform one Shove action.
The action takes two parameters:

1. **Landing square** `L` — a square the Shover "leaps to."
2. **Target direction** `D` — one of the 8 compass directions from
   `L`. The piece on `L+D` (if any) is the push target.

Constraints on `L`:

- `L` must be reachable from any square by a knight-leap pattern.
  In practice, since the Shover doesn't have a current board
  square (it's an inventory item), the constraint is *that the
  player chose `L` from a precomputed knight-jump set*. The exact
  source-square depends on variant rules — the simplest is: any
  knight-move offset from any of the player's existing pieces. The
  most-restrictive version: only from one specific source piece
  marked "Shover-staging."
- `L` must be empty and walkable.

Constraints on `D` and the push:

- The square `L+D` is the **push target**. If that square is empty,
  the Shove has no effect on a piece (only the leap happens). If
  occupied, the piece at `L+D` is pushed to `L+2D`.
- The push destination `L+2D` must be walkable and empty. If
  blocked, the push fails — the piece stays put, but the Shover
  charge is still consumed. ("The shove glanced off.") A variant
  rule could refund the charge on failure; default is no refund.
- The pushed piece is moved without capture semantics — even if
  `L+2D` would normally be a capturable enemy, the push doesn't
  trigger capture; it's blocked instead. To capture, use a normal
  capture move.

### Resolution

1. Find or instantiate a Shover marker on `L`. (Visually: a
   knight-leap arc animates to `L`.)
2. Compute `L+D` and apply the push as above.
3. Remove the Shover marker from `L` (consumed). The square `L` is
   empty again.

The Shover is **not a permanent piece on the board**. It "appears,
shoves, vanishes" in one action. Think of it as a thrown action
rather than a placed unit.

## Telegraph rendering

The Shover, being a tool not a piece, has no on-board telegraph in
its default state — it lives in the player's tool tray (a UI
element). When the player begins a Shove action, the UI highlights:

- All valid `L` squares (knight-jump destinations).
- After `L` is chosen, all 8 valid `D` directions, with the would-be
  push target piece highlighted.
- The push destination `L+2D` shown with a "pushed-to" indicator.

The player commits, the action resolves, the Shover marker vanishes.

## Why it's interesting

The Shover is **the universal answer to position puzzles.** Every
[ENEMY] piece in this category has a "push solves it" interaction:

- [Siege Engine](siege_engine.md): push it to a different file
  before it fires.
- [The Clock](the_clock.md): push the bomb away from your king.
- [Domino](domino.md): push a piece *out of* a Domino's front-square
  to prevent triggering the chain. Or push a piece *into* the front
  to deliberately fire the chain.
- [Latcher](latcher.md): push the Latcher one square to change its
  yank vector.
- [Marcher](marcher.md): push the Marcher off its 2×2 orbit.

But the charge is finite (one per puzzle, default). The Shover
forces the player to ask **"of all the things I could push, which
*must* I push?"** Choosing the right one is the puzzle.

The knight-leap restriction is critical. A free-aim push would be
boring — the player could solve any puzzle by shoving any piece any
time. Knight-leap means the *geometry* of the puzzle determines
which pushes are even available. Sometimes the bomb is unreachable
by knight-leap from any player piece, and the Shover is mute. That's
a hard puzzle.

## Example puzzle

```
6 . . . . . . . .
5 . . . . . . . .
4 . . . . . . . .
3 . . k . . . . .         k = player king
2 . . . . . . . .
1 . . . 1 . . . .         '1' = Clock, countdown=1 (detonates this enemy phase)
  a b c d e f g h
```

Player has **1 Shover** charge. Goal: survive enemy phase.

Clock's 3×3 zone centered on d1: c0-e0 (off-board), c1-e1, c2-e2.
King on c3 — *outside* the zone (c3 is rank 3, zone is ranks 1-2).
King safe? Yes. No action needed.

But suppose:

```
6 . . . . . . . .
5 . . . . . . . .
4 . . k . . . . .         king on c4 — one square inside zone
3 . . . . . . . .
2 . . . . . . . .
1 . . . 1 . . . .         Clock on d1 — but blast zone is c0-e2, doesn't include c4
  a b c d e f g h
```

Re-check the blast zone of Clock on d1: 3×3 centered on d1 means
c0-e0 (off-board), c1-e1, c2-e2. King on c4 is well outside. Safe.

Let me put the king actually in danger:

```
6 . . . . . . . .
5 . . . . . . . .
4 . . . . . . . .
3 . . . k . . . .         king on d3
2 . . . . . . . .
1 . . . 1 . . . .         Clock on d1, countdown=1
  a b c d e f g h
```

Blast zone: c0-e0, c1-e1, c2-e2. King on d3 is outside (rank 3 vs
ranks 0-2). Safe.

I keep failing to put the king in the blast zone. Let me try:

```
6 . . . . . . . .
5 . . . . . . . .
4 . . . . . . . .
3 . . . . . . . .
2 . . . k . . . .         king on d2 — INSIDE blast (c1-e1, c2-e2 zone)
1 . . . 1 . . . .         Clock on d1, countdown=1
  a b c d e f g h
```

King on d2. Blast zone of Clock on d1 is c0-e2. d2 is inside. King
dies unless action taken.

Shover knight-leap landings from the king on d2 (knight moves):
b1, b3, c4, e4, f1, f3. Of these, which are empty (let's assume
all)? All. Which knight-landing puts the player in position to push
the Clock?

To push the Clock on d1: Shover must land **adjacent** to d1 (so
`L+D=d1` for some `D`). Knight-leap landings from any of the king's
knight-moves... wait, the Shover's `L` is a knight-leap from a
player piece's square, *not* the king's specifically. The only
player piece here is the king. So `L` ∈ {b1, b3, c4, e4, f1, f3}.

Adjacent to d1: c1, c2, d2, e2, e1. Are any of these knight-leap
landings from the king? Knight-leaps from d2 are b1, b3, c4, e4,
f1, f3. None of these are adjacent to d1.

**No Shover landing can push the Clock.** The Shover is mute in
this configuration.

The player's only option: **move the king.** King d2 → c3 (legal
king move, ranks-files agnostic). King now on c3 — outside the
blast zone. Clock detonates harmlessly. Shover unused (carry to next
puzzle).

**Lesson:** Shover isn't always the answer. Sometimes it's a
preserved resource for a future turn.

Now a configuration where Shover IS the answer:

```
6 . . . . . . . .
5 . . . . . . . .
4 . . . k . . . .         king on d4
3 . . . . . . . .
2 . . . . . . . .
1 . . . 1 . . . .         Clock on d1
  a b c d e f g h
```

King on d4, Clock on d1 countdown=1. Blast zone c0-e2. King at d4 is
safe (rank 4, blast ranks 0-2). No action needed.

But add a [Siege Engine](siege_engine.md) loaded on a4 facing east:

```
6 . . . . . . . .
5 . . . . . . . .
4 S . . k . . . .         S = Siege Engine loaded, dir=E
3 . . . . . . . .
2 . . . . . . . .
1 . . . 1 . . . .         Clock countdown=1
  a b c d e f g h
```

Siege Engine fires across rank 4 next phase. King on d4 dies in the
beam. King must move out of rank 4 OR the engine must be displaced.

Knight-leaps from king on d4: b3, b5, c2, c6, e2, e6, f3, f5.
None are adjacent to a4 (a4's adjacents: a3, a5, b3, b4, b5). The
knight-leaps that land *adjacent to a4*: b3, b5. Yes — those work.

Shover lands on b3. Push direction toward a4 from b3 is NW.
`L+D = a4`. Push target = Siege Engine on a4. Pushed to `L+2D` =
NW twice from b3 = `0,5` (off-board). Push fails. (Or pushes off
the board — designer choice. Spec: blocked.)

Shover lands on b5. Push direction toward a4 from b5 is SW.
`L+D = a4`. Push target = Siege Engine. Pushed to `L+2D` = SW twice
from b5 = "off the board"-ish (a3? Let me compute: from b5, SW is
a4. SW again is "below-rank-4 and left of a" → off-board). Blocked.

Damn. Try a different `D`:

Shover lands on b3. Push direction `E` from b3 means `L+D=c3`. c3
is empty. No piece to push. Shover's leap happens but nothing
moves. Charge wasted.

Shover lands on b3, direction `N`: `L+D=b4`. b4 is empty. No push.

Shover lands on b3 with `D=NW`: push the Siege Engine NW. But NW
from a4 is off-board. Blocked.

**There's no Shover solution here.** The puzzle needs a *different*
tool ([Anchor Flag](anchor_flag.md) to freeze the engine, or
[Mirror Plate](mirror_plate.md) to redirect the beam). Lesson:
Shover excels at *some* problems; pair it with a Flag/Plate for
coverage.

A clean Shover puzzle:

```
4 S . . k . . . .         Siege Engine loaded, dir=E
3 . n . . . . . .         n = player knight on b3
2 . . . . . . . .
1 . . . . . . . .
```

Knight on b3 provides additional knight-leap origins for the Shover.
Knight-leaps from b3: a1, a5, c1, c5, d2, d4. From these, find one
adjacent to a4: a5 (adjacent: a4, a6, b4, b5, b6). Yes, a5 is
adjacent to a4.

Shover lands on a5. `D=S`: `L+D=a4` (Siege Engine). Push target =
engine, pushed to `L+2D=a3`. a3 is empty. **Push succeeds.** Engine
moves a4 → a3. Now the loaded engine fires next phase along rank 3,
not rank 4. The beam goes a3-east, hitting the player knight on b3
(killed) but NOT the king on d4. King survives.

**The puzzle is "what additional piece do I need to enable the
Shover?"** Pre-positioned player pieces extend the Shover's reach.

## Where it shines

- Universal positioning verb. Combines with every enemy.
- Forces the player to compute knight-jump geometry — a different
  spatial puzzle than standard chess threats.
- Single-charge-per-puzzle limit makes timing critical.

## Where it's awkward

- Knight-leap origins depend on existing player pieces. A puzzle
  with only a king on the board has very limited Shover access. The
  puzzle designer must place support pieces.
- "Push direction" parameterization is a lot of UI surface. Players
  used to chess "click source, click destination" may bounce off the
  three-click pattern (landing → direction → push target preview →
  confirm).
- Push-fails-on-blocked is one of those rules players will forget.
  Indicate clearly in the UI before commit.
- Push interactions with [Conduit](conduit.md): does a piece pushed
  *into* a Conduit teleport out the other Conduit? Spec says no
  (Conduit only routes telegraphed effects, not movement). Players
  will *want* it to teleport.

## Engine dependencies

- Player tool inventory (new feature — see below).
- Push primitive (shared with Clock's push interaction).
- Knight-leap reachability query (already exists for actual knights).

## New features required

- **Player tool inventory.** A per-side resource: list of tool
  charges. `Shover: 1`, `MirrorPlate: 2`, `AnchorFlag: 1`, etc.
  Stored as part of game state. FEN-serializable (new top-level
  section, suggested `T:` for "tools," distinct from existing `T=`
  square-type tag).
- **Tool action move type.** A new `GameMove` variant `UseTool {
  tool, params }`. Each tool defines its own params struct.
- **Push primitive.** Function `try_push(board, from, dir) ->
  Result`. Moves the piece at `from` to `from+dir` if walkable+empty.
- **Knight-leap origin set.** Function `knight_leap_landings(board,
  side) -> Vec<Square>` returning all squares reachable by knight
  offset from any of `side`'s pieces.
- **Puzzle-mode variant flag.** A variant flag enabling
  charge-based tools and goal-state win conditions. Without this
  flag, Shover-style tools are unavailable (free-play chess
  shouldn't have one-shot displacement powers).

## FEN encoding

The Shover itself doesn't live on the board, so no FEN piece
encoding is needed. The charge count is part of the tool inventory,
encoded in a new FEN section. Proposed syntax appended after the
existing fields:

```
... | T:SHOVER=1,MIRROR=2,FLAG=1 | ...
```

The exact delimiter is bikeshed — see plan stub. The shover line
above means "1 Shover charge, 2 Mirror Plate charges, 1 Anchor Flag
charge."

## Open questions

- **Origin restriction.** Knight-leap from *any* player piece, or
  from a specific staging square? Spec says "any player piece" for
  flexibility. A more restrictive variant (from a single source
  piece) makes puzzles harder to design but more constrained — a
  pro for some designers.
- **Multi-charge.** Default is 1 charge per puzzle. A 2-charge
  Shover is much more powerful — possibly too much. Probably keep
  default at 1.
- **Push vs throw.** Does pushing capture? Spec says no (push
  blocks on occupied). A "throw" variant could capture-on-landing.
- **Tool refunds.** If push fails (target blocked), is the charge
  refunded? Default: no. (Penalize sloppy clicks.) A more forgiving
  variant: refund.
- **What about pushing terrain?** Some squares have terrain features
  (Frozen, Track, etc.). A "push that terrain off" verb doesn't
  exist; Shover only pushes pieces. Confirm.
- **Friendly push.** Can the Shover push the player's own pieces?
  Default: yes (it's just a push, not enemy-only). Useful for
  positioning.
