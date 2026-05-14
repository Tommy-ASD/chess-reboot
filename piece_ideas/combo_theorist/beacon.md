# Beacon

> Any friendly piece sharing a rank or file with Beacon may teleport-swap with it — once per piece per game, irreversibly.

## Inspiration

The deckbuilder problem: "I need to deploy a key piece **across the
board** in a single move." Most fairy pieces solve this by being
long-range sliders. Beacon solves it by being a **fixed teleport
anchor** — the relic that lets you warp other relics.

Think Slay the Spire's "Letter Opener" / "Toolbox" relic
philosophy: a piece whose value is what it does *for other pieces*.
Or Magic's "Flicker"-style cards: the piece itself is unimpressive,
the destination it enables is the wincon.

The crucial constraint: **one-shot per friendly piece**. A Bus that
already swapped with Beacon can never swap with it again. This
limits the combo to a single critical move per attacker, and makes
each warp a *commitment*.

## Mechanic

- **Movement (Beacon's own turn):** Beacon moves one square like a
  king. Slow, vulnerable, no captures.
- **Passive ability (any friendly piece's turn):** A friendly piece
  that:
  - Shares the same rank OR same file as Beacon, AND
  - Has not yet used a Beacon-swap in this game (flag `B=0`),
  - May spend its turn to **teleport-swap with Beacon**.
- **Swap semantics:** The friendly piece's square becomes Beacon's
  square. Beacon's square becomes the friendly piece's old square.
  Both pieces relocate instantly. No captures occur.
- **One-shot enforcement:** Each friendly piece carries a `B=0/1`
  flag. After it Beacon-swaps, `B=1` forever. FEN-tracked per
  piece. Pieces never reset; capture-and-respawn doesn't apply
  (chess doesn't respawn anyway, but variants might — see Open
  Questions).
- **No carry semantics.** A Bus with passengers swapping with
  Beacon: the Bus teleports, **the passengers stay on the Bus**.
  The Bus is treated as a single unit. (This is the whole point of
  the combo.)
- **No swap into illegal squares:** if the swap would put either
  piece on a non-walkable square, the swap is illegal. (Beacon's
  starting square is presumed walkable since Beacon is there.)
- **Multiple Beacons:** each Beacon is independent. A piece can
  swap with Beacon A and later swap with Beacon B (each has its
  own `B` flag for the moving piece? Or one global flag? See Open
  Questions.)

## Why it's interesting

**Tempo / deployment.** Beacon is a fixed warp anchor. Its existence
makes every rank and file it touches a teleport corridor for one
of your pieces. The opponent must respect *the entire cross-shape*
formed by Beacon's row and column.

The one-shot constraint is the design lock. Beacon is too strong if
infinite — every piece warps in turn 1, you have an army at the
back rank turn 3. The one-shot rule means each warp is a major
strategic decision: which piece, which turn, which destination?

The piece is also defensively strong without doing anything: as
long as Beacon is alive, the threat of a teleport-swap forces the
opponent to defend the *entire rank* Beacon sits on. Kill Beacon
and the corridor collapses.

## Combos and counters

**Combo with Bus (instant deployment cannon):** Bus loads 4
passengers on a1. White Beacon parked on h8. Bus and Beacon share
neither rank nor file — but white plays Bus a1-h1 first (rook-like
move). Now Bus and Beacon share the h-file. Next turn, Bus uses
its Beacon-swap: Bus is now on h8, Beacon is on h1. Bus unloads
4 passengers onto g8, h7, etc. Instant back-rank assault.

This is the wincon Beacon exists for. Bus + Beacon is the
canonical combo. The cost: Bus's `B=1` is set, so Bus cannot use
this Beacon again. But the game is usually decided by the back-
rank invasion.

**Combo with Goblin (kidnap-and-warp escape):** Goblin captures a
piece deep in enemy territory and starts the kidnap-return. Long
trip, high interception risk. If Beacon is parked on Goblin's home
rank, and Goblin's captured-piece-square shares the file with
Beacon, Goblin can swap with Beacon as its next move. Hostage and
all teleport home in one ply. Goblin's `B=1` is consumed; you only
get one of these per game per Goblin.

**Combo with Skibidi (warped pulse):** Skibidi's brainrot pulse is
dangerous because it has a wide AoE. Pre-position Beacon at the
opposite end of the board. Skibidi crosses the half-board normally
over several turns; when a key target appears, Skibidi swaps with
Beacon to the *back* of the action, pulses from behind the
opponent's pieces, catching them in the back-arc of the pulse.

**Counter to Monkey (back-rank pivot):** Monkey's jump-chain is
linear. A Monkey threatening from the south can be answered by
swapping a defender into the action from the north via Beacon.
This isn't quite a "hard counter" — it's a tempo answer that lets
you tele-defend across the board. Useful when your defenders are
out of position.

**Counter-play to Beacon (assassination):** Beacon's death ends
all future warps. Pieces with `B=1` are stuck (can never warp
again, and Beacon is gone), but pieces with `B=0` lose the
*option* permanently. A disciplined opponent will spend 2-3 tempi
to kill Beacon early, locking the opponent into conventional
deployment.

**Hard counter — Plague Doctor:** Beacon's swap-grant is an
ability the swapping piece uses. If the swapping piece is on
miasma terrain (Plague Doctor), it cannot use its ability that
turn — meaning the warp is denied. Lay miasma along Beacon's
rank/file and the corridor is silenced.

**Counter — Echo:** Beacon doesn't capture, so Echo doesn't
directly threaten it. But Beacon is also fragile and unable to
flee. Capture Beacon with any piece, and that piece is not
compelled (Beacon isn't Echo). However, if Echo and Beacon share
a rank, and a friendly piece warps to Beacon's square through
Echo's threat... no, Beacon-swap teleports, doesn't move through
intermediate squares. Echo doesn't apply unless Beacon is captured
*through* Echo somehow. Edge case is rare.

## Example scenarios

**Scenario 1: The h-file cannon.**
White Bus on a1 with 4 passengers. White Beacon on h6.
Turn 1: Bus a1-h1 (rook slide along rank 1).
Turn 3: Bus uses Beacon-swap. Bus is now on h6, Beacon on h1.
Turn 5: Bus unloads at h7, h8 (drops 2 passengers immediately).
Turn 7: Bus drops remaining 2 passengers as it shuffles.
Net effect: by turn 7, 4 passengers on the back rank. Black has
typically not had time to defend; the game is decided.

**Scenario 2: Goblin warp-home.**
Black Goblin captures white queen on d5 at turn 8. Black Beacon
on d1. Turn 9: Goblin uses Beacon-swap (they share d-file).
Goblin teleports to d1 with the kidnapped queen. Queen is now
fully captured (kidnap completes after the home-return). Black
is up a queen by turn 10, Beacon is on d5 in enemy territory
and will probably die — but the queen is worth it.

**Scenario 3: Wasted swap.**
White panic-swaps a rook with Beacon turn 2, thinking it's the
combo. The rook teleports to a defensive square — fine, but the
rook's `B=1` is now set. Three turns later, white wants to swap
the rook into a more useful spot. Cannot. The one-shot cost is
real.

## Where it shines

- Bus-centric builds where back-rank deployment is the win path.
- Goblin builds where kidnap-return is too slow.
- Wide boards where conventional movement is too slow. Beacon
  trivializes distance.
- Variants with Track tiles where Locomotives are slow to position.
  Beacon can swap a defender to the train's location instantly
  to threaten / repair / block.

## Where it's awkward

- Once Beacon dies, all unused warps die with it. A Beacon killed
  before any warps fire is wasted material.
- The one-shot is brutal in long games. Pieces hoarding their warp
  often never use it. Players over-plan and miss the window.
- Compositions where no piece can profitably teleport. If your
  army is short-range and clustered, Beacon's value is zero.
- Aggressive openings. Beacon needs setup time to position on a
  useful rank/file. Two-turn-attack metas eat it alive.

## Engine dependencies

- Rank/file alignment check (already present in slider movegen).
- Per-piece FEN payload `B=0/1` flag.
- Per-piece move-generation option: "Beacon-swap with B_id" appears
  as a legal move when alignment + flag conditions hold. The move
  resolves by swapping piece positions in board state.
- Beacon piece tracking — the engine needs to know which Beacon
  the swap targets (in the case of multiple Beacons, see Open
  Questions).

## New features required

- **Per-piece `B` flag.** One bit per piece. FEN-encoded as `B=1`
  if the piece has used its warp, omitted (default 0) otherwise.
- **Beacon-swap move type.** A new legal-move kind that swaps two
  pieces. Distinct from a normal move; the engine's move
  representation needs a "swap" variant.
- **Movegen pass for Beacon-eligible swaps.** For each friendly
  piece with `B=0`, check rank/file alignment with all friendly
  Beacons. Emit a swap move per eligible Beacon. This is O(P*B)
  per turn but P*B is small.
- **Beacon-death cleanup:** When Beacon dies, no special handling.
  Already-`B=1` pieces stay `B=1`; `B=0` pieces stay `B=0` but
  there are no Beacons left to swap with, so the option simply
  never appears in their legal moves. Clean.

## FEN encoding

Beacon piece:

```
(P=B)
```

(No per-Beacon state — every Beacon is identical until killed.)

Pieces that have used their warp:

```
(P=R,B=1)
```

A rook that already warped with a Beacon. Default `B=0` is
omitted.

If multiple-Beacon disambiguation is needed (rare), Beacon could
carry a tag: `(P=B,ID=1)`, and the swapping piece records which
Beacon it consumed: `(P=R,B=ID:1)`. Recommend NOT supporting this
— see Open Questions.

## Open questions

- **Multiple Beacons: shared or per-Beacon flag?** Two camps:
  - **Shared flag:** A piece warps once, ever, regardless of which
    Beacon. Simple. Encourages a single critical warp.
  - **Per-Beacon flag:** A piece warps once *per Beacon*. With
    three Beacons, a piece can warp three times in a game (to
    three different anchors). Strong; encourages Beacon-rich
    builds.
  Recommend **shared flag** for simplicity and game balance.
- **Swap that puts a piece in check.** Standard legality rule:
  Beacon-swap is illegal if it ends with your king in check.
  Normal movegen filter. No new rule.
- **Swap with a piece being attacked.** Can a king Beacon-swap
  out of check? It's a legal "move" if the resulting position is
  not in check. Yes, king can warp out. King's `B=1` is set —
  king has one panic-warp per game. This is *strong*. Recommend
  excluding kings from Beacon-swap. They can be the *Beacon's*
  rank/file mate, but not the *swapper*. Otherwise kings become
  un-checkmate-able. Strong design constraint.
- **Beacon as a swap target with an enemy attacker on its
  square.** Not a real edge — Beacon is on its own square. The
  swap goes between Beacon and a friendly piece on the same
  rank/file. The attacker isn't involved.
- **Castling interaction.** Castling has its own one-shot per
  king. Does Beacon-swap count as the king's castling-equivalent?
  Recommend: separate flags. Beacon-swap and castling are
  independent. (If kings are excluded per the prior rule, moot.)
- **Promotion timing.** A pawn that promotes inherits `B=0`?
  Recommend the promoted piece is a new piece-state, so `B=0`.
  But the *original* pawn's `B` flag, if it had one, is lost.
  Pawns realistically never warp (low value, short-range), so the
  edge is theoretical.
