# NPC

> A piece that walks forward by itself every turn — the controller
> cannot command it, only design the board around it.

## Inspiration

"NPC" is short for "non-player character," the meme of someone who
goes through life on rails, reacting predictably to stimuli with no
internal agency. The chess version *is* an NPC: it walks the same
direction every turn, pivots when it hits a wall, and ignores
everything its controller wants.

Strip the paint: the mechanic is **an autonomous traversal piece.**
Trains (Locomotive + Carriage) already move autonomously on Track
tiles, but they're rail-locked. The NPC walks the *open board* on
its own — no track required. This produces a moving obstacle/asset
that both players know exactly where it's going next, and have to
plan around. Pressure plates, signal interactions, and goal-tile
puzzles become first-class with NPCs in play.

## Mechanic

### State

The NPC carries a per-piece state field `D` — its current facing
direction, in `{N, E, S, W}`. Default `D=N` for white NPCs (faces
"up" the board), `D=S` for black NPCs (faces "down"). Diagonal
facing is not supported in the base version.

### Auto-advance

At the **start** of the NPC's controller's turn, **before the
controller has a chance to make any move**:

1. The NPC attempts to step **one square in the direction of
   `D`**.
2. The step target square is evaluated:
   - **Empty walkable square:** NPC moves there. Done.
   - **Enemy piece:** NPC **captures** the enemy. Standard
     capture. Done.
   - **Friendly piece:** Step blocked. **Pivot** (see below).
   - **Non-walkable square (off-board, Block, closed Gate,
     etc.):** Step blocked. **Pivot.**

### Pivot

When blocked, rotate `D` **clockwise by 90°**: `N → E → S → W → N`.
Then attempt the step again with the new direction.

If the new direction is also blocked, pivot again. The NPC may
pivot up to **3 times** in one turn (i.e. try all 4 directions).
If all 4 directions are blocked, the NPC does not move this turn
and its `D` resets to the value it had at start of turn.

Each successful step (or capture) consumes the NPC's auto-action
for the turn. The NPC is fully autonomous; it does not interact
with the controller's normal move action.

### Controller agency

The controller **does not select moves for the NPC.** The NPC's
auto-step happens at the start of each of the controller's turns
and resolves before the controller's regular move.

This means: each turn, the controller's NPC moves once, *then*
the controller selects one regular move with any of their other
pieces. The NPC is "free" — it doesn't consume the controller's
turn-action.

If the controller has only NPCs and no other pieces, every turn
is just the NPC auto-advance(s) firing — the controller has no
move to make. The game continues. (Stalemate-like? See open
questions.)

### Multiple NPCs

If a controller has multiple NPCs, all of them auto-advance at
the start of the turn. The resolution order is **left-to-right,
top-to-bottom by board position** (deterministic). Each NPC's
step is resolved fully before the next NPC's step is evaluated.

This matters when two NPCs would collide: the earlier-resolved
NPC moves first, the second NPC sees the updated board.

### Captures *of* the NPC

The NPC is captured normally — any enemy piece can land on its
square and remove it. Captured NPCs are gone. The Goblin can
kidnap them and return them home, in which case the NPC resumes
walking from its home square with its original `D`.

### Square interactions

NPCs trigger pressure plates, ride track tiles, fall into Ohio,
etc. — same as any other piece. The autonomous movement does
not exempt them from board effects.

- **Pressure plates:** stepping on a plate fires its signal.
  NPCs make signal puzzles deterministic — the NPC arrives on a
  given turn, fires the plate, the signal cascade resolves.
- **Ohio tile:** ending on Ohio rotates the NPC. A rotated NPC's
  direction is rotated too — `D=N` at `R=1` becomes `D=E` for
  purposes of auto-step. The NPC's facing visually rotates with
  the piece.
- **Frozen:** Frozen NPC does not auto-advance this turn. `D`
  unchanged.

## Why it's interesting

1. **Deterministic ambient hazard.** Every turn, both players
   know exactly what the NPC will do. The controller "owns" the
   NPC but cannot redirect it. The opponent can plan a defense
   knowing exactly when and where the NPC will arrive.
2. **Puzzle-first piece.** The NPC is the ideal piece for hand-
   crafted positions. A pressure plate at the far end of the
   board with an NPC walking toward it is a deterministic timer
   — the NPC arrives in exactly N turns.
3. **Forced position pressure.** The controller cannot un-move
   an NPC. They have to *prepare* the NPC's path. If the path
   leads to a bad square, the controller must rearrange the
   board to redirect it before the NPC gets there.
4. **Captures-on-rails.** The NPC will capture whatever sits in
   its path. Opposing players can deliberately park weak pieces
   in the NPC's path to soak the capture — interesting
   sacrificial pattern.
5. **Cheap actions.** The NPC's auto-step doesn't consume the
   controller's main move. A controller with an NPC effectively
   gets 1.5 actions per turn — the NPC's step plus a regular
   move. Strong piece. Balance is in the lack of agency.

## Example scenarios

1. **Standard walk.** White NPC on a2, `D=N`. White's turn:
   NPC auto-steps to a3. White picks a regular move with a
   different piece. Next turn: NPC steps to a4. Etc.
2. **Pivot at the wall.** White NPC on a7, `D=N`. White's turn:
   NPC tries to step to a8 — but a8 has a friendly rook (own
   piece). Pivot to `D=E`. Try b7: empty, NPC moves to b7.
   `D` is now `E`. Next turn: NPC tries c7. And so on.
3. **NPC captures into trap.** White NPC on e4, `D=N`. Black
   knight on e5. NPC auto-steps onto e5, capturing the knight.
   White is happy — free capture. Black's bishop on h2 was
   waiting; if e5 is defended by a black piece, the NPC will
   be captured back next turn. The "free" capture is rarely
   actually free.
4. **Plate-timing puzzle.** Variant board: white NPC on a1,
   `D=N`, pressure plate on a5 wired to a gate at e8. White
   has 4 turns until the NPC arrives at a5 and opens the gate.
   White's regular moves should focus on positioning to
   exploit the open gate when the NPC arrives.
5. **Cornered NPC.** White NPC in a corner with friendly
   pieces north and east, edge south and west. All 4
   directions blocked. NPC does not move; `D` unchanged.

## Where it shines

- **Puzzle compositions** — the NPC's determinism makes it a
  reliable timer.
- **Pressure-plate/signal variants** — automatic plate-
  triggering without controller cost.
- **Variants where the NPC IS the win condition** — "get your
  NPC to the back rank" goal mode.
- **Maps with Track + Block walls** — NPCs navigating a maze.

## Where it's awkward

- **No controller agency** — players who want to micromanage
  hate this piece.
- **Stalemate-by-NPC** — if a player has only NPCs and they
  can't move (boxed in), the player's turn produces nothing.
  Need a rule: does the game end? Recommend: NPCs without legal
  steps simply pass; the controller plays a regular move (or
  resigns).
- **Pivot determinism** — clockwise pivot is a choice. A
  variant could randomize, but randomness is disallowed.
  Counterclockwise pivot is a different piece. Pick one and
  commit.
- **Capture loops** — two NPCs facing each other will walk
  toward each other and one will capture the other on the
  next turn. Deterministic and fine.

## Engine dependencies

- **Per-piece FEN payload** for `D`.
- **Turn-start auto-action hook** — fires before controller's
  regular move action. Skibidi's brainrot evaluation is a
  similar hook point.
- **Movement primitive** for "step one square in direction D"
  — trivial.
- **Pivot logic** — small state machine, fully synchronous.
- **Pressure plate trigger** — already exists (plan 08).
- **Goblin kidnap** — must preserve `D` on the captive.

## New features required

- **NPC piece type with stateful turn-start action.** Plan stub:
  introduce a "stateful piece autonomous action" hook that
  runs before normal move generation. Gooner uses the same
  surface.
- **Controller turn structure update.** A controller's turn is
  now: (1) autonomous piece actions, (2) regular move action.
  The engine's turn pipeline must support this two-phase model.
- **Pivot direction constant.** Document clockwise pivot as
  spec-fixed.

## FEN encoding

Symbol: `NP` for white NPC, `np` for black.

Payload: facing direction `D=<N|E|S|W>`.

```
(P=NP,D=N)      # white NPC, facing north (default)
(P=NP,D=E)      # white NPC, facing east (after a pivot)
(P=np,D=S)      # black NPC, facing south (default for black)
```

Default for white: `D=N`. Default for black: `D=S`. Encoder omits
if matches default; decoder tolerates explicit defaults.

## Open questions

- **Diagonal facing?** Variant where `D` can be one of 8
  directions. Doubles the state space and the pivot complexity.
  Probably not for v1.
- **Stalemate semantics.** A controller with no moveable pieces
  *other than NPCs* — is the turn complete after just the
  NPC's auto-step? Most permissive: yes. Strictest: no, must
  make a regular move or it's stalemate. Recommend permissive.
- **Multiple NPCs collision priority.** Left-to-right,
  top-to-bottom is one choice. Original-FEN-order is another.
  Pick one. Recommend board-position-based for determinism
  across turns.
- **Goblin returning NPC home.** Does `D` reset to the original
  starting direction, or preserve the pre-kidnap value?
  Recommend: preserve. The NPC remembers.
- **NPC riding the Costco Guy.** Possible if NPC value is ≥
  Costco Guy. While inside the carrier, NPC does not auto-
  advance (it has no board square). On unboarding, it resumes.
  Fine.
- **Castling-eligible NPC.** No. NPC is not a king or rook.
- **NPC promotion target?** A pawn promoting to NPC is weird.
  The NPC's value vs the queen is probably much less. Plays
  into the meme well: "promoting to NPC = giving your own
  pawn no agency." Variant choice.
