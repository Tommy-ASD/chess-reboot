# Trellis

> Stationary plant that spawns one neutral vine tile per turn into an adjacent empty square, building permanent walls that block enemies and pass friends.

## Inspiration

The deckbuilder problem: "I have a strong central piece (Skibidi
parked on e4, Bus mid-route) and I need *time* to set it up. The
opponent has tempo, I have a plan." The answer card is a piece that
trades movement for **territory**. Trellis doesn't threaten anything.
It carves the board into a shape where your other pieces work.

The vine tiles are deliberately asymmetric: enemies see walls,
friendlies see paths. This is the same asymmetric-terrain trick the
Track tiles use for trains, applied to a non-train piece.

## Mechanic

- **Movement:** None. Trellis never moves under its own power.
- **Capture:** Captured normally as any piece.
- **Per-turn passive (on Trellis's own move):**
  1. Pick one orthogonally adjacent empty Standard square.
  2. That square becomes a `T=VINE` square owned by Trellis's color.
  3. Increment the vine count.
- **Limit:** Maximum 6 vines per Trellis, tracked in its FEN payload
  as `V=N`. Once `V=6`, Trellis's turn passes without effect.
- **Vine semantics:** `VINE` square type. Walkable by the owning
  color (and that color's neutral allies — trains, etc., are
  configurable). Non-walkable by the opposing color; blocks slider
  paths through it.
- **Vine removal:** A vine is destroyed if (a) any piece of the owning
  color stands on it and is captured *from* it (vine becomes Standard),
  or (b) Trellis dies — all of its vines decay to Standard the same
  turn. The latter is critical for counter-play.

## Why it's interesting

Pure **denial / area control**. Trellis doesn't take pieces, doesn't
deliver checks, doesn't trade. It paints the floor. Skill expression
is in *which* six squares you paint — vine placement is a 6-move
sub-game played in parallel with the main game.

Killing Trellis erases its work. This makes the piece itself a
high-value target, but also low-tempo to attack — by the time the
opponent's rook arrives, the damage is done.

## Combos and counters

**Combo with Bus (passenger funnel):** Park Trellis behind a Bus that
has loaded 3+ passengers. Grow vines in a U-shape: walls on both
sides of the Bus's intended exit lane. The Bus slides forward and
unloads into the safe corridor; the opponent cannot interpose because
the flanking squares are vine-walled. Effectively a one-way deployment
chute. The vine tiles are walkable by the Bus's faction (and by the
unloaded passengers), so the funnel works both directions for friends
and is a hard wall for foes.

**Combo with Goblin (kidnap exfil):** Goblin's identity is "kidnap
and run home." A Trellis grown in a 3-vine line between Goblin and
its home square turns the return trip into a guaranteed escape — no
opposing piece can interpose on the vine-rail. Place Trellis on the
back rank, grow vines forward along Goblin's expected return path,
and the kidnap success rate jumps.

**Counter to Monkey (landing denial):** Monkey's jump-chain requires
empty squares as intermediate landing pads. A Trellis growing six
vines in a 2x3 cluster around a key central square eliminates all
the staging squares Monkey needs. Monkey doesn't get to chain through
vines — they're non-walkable to the opposing color. This is the
hardest counter Trellis offers, and one of the few clean answers to
Monkey-stack openings.

**Counter-play to Trellis itself:** The most important counter is
**don't bother killing it**. Trellis caps at 6 vines, and it can't
threaten anything. If you can route your attack through the *other*
half of the board, Trellis spent its game being useless. The
positional cost is real: you lose half the board to it. But that's
true of any wall.

If you must remove Trellis: a single capture kills all vines. Bishops
and Skibidis are good answers because they can reach the back-rank
parking spot diagonally without touching the vine field. A Skibidi
brainrot pulse aimed at Trellis is brutal — the pulse reaches through
your own vines (you own them) but Trellis can't dodge.

## Example scenarios

**Scenario 1: Funnel deployment.**
White Bus on b1 with 4 passengers. White Trellis on a3.
Turns 1-3: Trellis grows vines on a4, b4, c4 (a horizontal lid).
Turn 4: Bus moves up to b3, then b5 next turn — but the lid blocks
black's interposition along the 4th rank entirely. Black must commit
the queen all the way around to a6 to threaten the Bus, losing two
tempi.

**Scenario 2: Monkey lockout.**
Black Monkey on f6, looking to jump-chain toward white's king on g1.
White Trellis on h1 grows vines: h2, g2, f2, h3, g3, h4.
Monkey's chain dies — every potential landing on the kingside is
vine-walled. The Monkey is alive but inert.

**Scenario 3: Trellis cleared.**
Same position as scenario 1, but black plays Bxa3, capturing Trellis
with a bishop that slipped down the long diagonal. All four vines
(a3 was just captured — er, Trellis itself isn't a vine, but the
three vines a4/b4/c4) revert to Standard the same half-move. White's
funnel collapses. The bishop trade is a textbook "answer the engine,
not the threat."

## Where it shines

- Slow, positional openings where both sides build up before
  committing.
- Variants that disable some classical pieces (no bishops, no
  knights) — Trellis fills the board-control role those pieces
  normally hold.
- Compositions with **Locomotive** trains: vines walling the train
  track keep the train safe en route. Trains can't navigate around
  threats; Trellis pre-clears the route.
- Endgames where the opponent has one big attacker and you need to
  slow it for 6+ moves.

## Where it's awkward

- Fast tactical games. By turn 4, Trellis has 3 vines; by then the
  opponent's already started the assault. Trellis is a turn-12 piece.
- Open positions with multiple attack vectors. Six vines cannot wall
  off the whole board.
- When the opponent has a Bus or other carrier. Carrier pieces don't
  care about wall geometry — they pick up passengers and warp them
  past the wall.
- Self-block. A poorly-placed vine can wall in your own attacker.
  Trellis player must plan the vine pattern in advance.

## Engine dependencies

- `SquareType::Standard` (the base case Trellis transforms).
- A new `SquareType` variant (see below).
- FEN payload syntax for per-piece counters (`V=N` on Trellis's piece
  payload).
- The walkability predicate from plan 10's modifier band — vine
  walkability is color-dependent, so the predicate needs the moving
  piece's color as input. Plan 10 already provides this.

## New features required

- **`SquareType::Vine { owner: Color }`** — new walkability rule:
  walkable iff `moving_piece.color == owner`. Blocks sliders for the
  opposing color. Decays to Standard on owner's Trellis death.
- **Trellis death cleanup** — make_move hook: when a Trellis is
  captured, sweep all `Vine { owner: c }` squares belonging to
  that color back to `Standard`. (If multiple Trellises share a
  color: only sweep the dead Trellis's vines. Requires the vines to
  carry the spawning Trellis's piece ID, not just the color. See
  Open Questions.)
- **Per-Trellis vine count** — stored in the Trellis piece payload as
  `V=N`. Auto-incremented at end of Trellis's turn after a successful
  grow.

## FEN encoding

Trellis piece payload:

```
(P=R,V=3)
```

Where `V=3` means three vines currently grown. Defaults to `V=0` on
spawn.

Vine square:

```
(T=VINE,O=W)
```

Where `O=W` is the owning color (`W` / `B` / `N`). If the
"vine-belongs-to-which-Trellis" question gets resolved with piece IDs
instead of color (see Open Questions), this becomes `(T=VINE,O=W,I=42)`.

## Open questions

- **Vine ownership granularity.** Color or piece-ID? Color is simpler
  and FEN-cheaper. Piece-ID lets you have two Trellises that don't
  share vines — when one dies, only its vines decay. The deckbuilder
  argument: piece-ID is more interesting because it makes "trade off
  the *small* Trellis to keep the *fat* Trellis's wall" a real
  decision. Recommend piece-ID.
- **Vine direction limits.** Should Trellis only grow vines in a
  straight line from itself, or arbitrarily through any adjacent vine?
  Arbitrary growth is more flexible but lets vines reach impossibly
  far. "Must grow into a square adjacent to Trellis or to an existing
  own vine" is the natural rule and matches the plant fiction. Cap is
  still 6 either way.
- **Captures landing on vines.** If a friendly piece on a vine is
  captured, does the vine survive? Current proposal: vine reverts to
  Standard (the foot-traffic destroyed the plant). Alternative: vine
  is hardened, captures don't break it. Reverting feels right —
  encourages the opponent to *attack into* the vine zone.
- **Stacking limit interaction.** Max 6 vines is the design knob.
  Anywhere from 4-8 plays; 6 is the round-number guess. Needs play
  testing in a Skibidi-heavy meta where 6 vines might be too few to
  shape a Skibidi's pulse radius.
- **Interaction with Plague Doctor.** Vine squares are terrain, not
  pieces. Plague Doctor's miasma is a separate terrain layer. Can a
  square be both vine and miasma'd? Probably yes — they don't
  conflict. But the rendering question is real.
