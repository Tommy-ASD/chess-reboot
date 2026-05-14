# Lightcone

> A piece whose threat propagates outward at one square per turn
> from its last-known position; what's outside the cone cannot
> be attacked, what's inside cannot escape. [LOCALITY /
> INFORMATION]

## The law it breaks

Chess threats are instantaneous: a piece attacks every square in
its move geometry the moment it occupies its current square. The
Lightcone refuses this. Its threat is *causal* — information
about where it now sits has to travel outward at a finite speed
of one square per turn. A piece more than `k` squares away from
the Lightcone's last move cannot yet *know* about the Lightcone,
and so cannot be captured by it. Equally, the Lightcone cannot
attack pieces it has not yet "informed."

This is the chess analogue of a Minkowski lightcone: the set of
squares that could have received the news of the Lightcone's
move in `k` turns is a diamond of radius `k`. Inside it, the
Lightcone is real. Outside, it is rumour.

## Mechanic

State per Lightcone instance, stored in FEN:

- `pos: Square` — current position (as for any piece).
- `last_move_turn: u32` — the ply on which the Lightcone last
  *moved*. Zero or absent if it has never moved (initial
  placement counts as "moved on turn 0").

Derived per turn:

- `k = current_turn - last_move_turn` — the cone radius.
- `cone(pos, k) = { sq : chebyshev_distance(pos, sq) <= k }` —
  the set of squares "informed" of the Lightcone's position.
  (Chebyshev, not Manhattan: information travels with the king's
  step. This makes the cone a square diamond.)

Movement primitive: the Lightcone slides like a queen but only
within its current cone — `dest` must satisfy
`chebyshev_distance(pos, dest) <= k`. When `k = 0` (just
moved), the Lightcone *cannot move at all*. It is welded in
place for one turn after each move.

Threat geometry — the key mechanic:

- **Outgoing.** The Lightcone threatens (and may capture) only
  squares inside `cone(pos, k)` that are also reachable by its
  geometric move pattern (queen rays). A piece on a queen-ray
  square outside the cone is geometrically targeted but
  causally unreachable — the Lightcone cannot capture it.
- **Incoming.** Opposing pieces may capture the Lightcone only
  if they are themselves inside `cone(pos, k)`. A bishop on
  the far side of the board, with a clear ray to the
  Lightcone's square, *cannot* capture it: the bishop does not
  yet know the Lightcone is there. Captures wait for the cone
  to reach the bishop.
- **Pieces inside the cone are "pinned to the cone."** They
  cannot leave it. A piece on a cone-boundary square that
  attempts to step outside is rejected — the cone is a soft
  walls. (Alternative: pieces are free to leave but lose
  contact and become safe; this is the looser variant. The
  pinned-to-cone variant produces sharper play.)

Cone growth: every turn (any player's), `k` increments by 1. On
the turn the Lightcone moves, `k` resets to 0 and the new
`last_move_turn` is recorded.

Cone maximum: optionally cap `k` at some `K_MAX` (e.g., the
board diameter). After `K_MAX`, the Lightcone behaves as a
normal queen — the cone has fully expanded. Recommend
`K_MAX = max(FILES, RANKS) - 1`.

## Why it's interesting

The chess novelty: a Lightcone in the corner is a sleeping
threat. For four turns, half the board ignores it. On turn
five, it's a queen with global reach — but only if it hasn't
moved. Players have a tempo-shaped decision: keep the Lightcone
*still* to grow its threat, or *move* it for tactical gain and
reset the cone to zero. The piece's geometric power is
exchanged with its informational reach.

The conceptual elegance: information has a speed. The engine
encodes "what a piece knows" as a turn counter and a metric
ball. The break is dry: a distance check against a counter.

## Example scenarios

- **The sleeper.** White Lightcone on a1, last moved on turn 1.
  By turn 5, `k = 4` and the cone covers files a-e, ranks 1-5.
  Black has a queen on h8 — outside the cone, untouchable. On
  turn 8, `k = 7` and the cone covers the whole board. Black's
  queen, which had been planning to defend, suddenly cannot
  cross the e-file without entering the cone where the
  Lightcone has direct queen-rays.
- **The tempo trade.** Black Lightcone on d4, `k = 5`,
  controlling most of the board. White threatens it. Black
  moves the Lightcone one square to e4 to dodge — `k` resets
  to 0. The next turn, almost nothing on the board threatens or
  is threatened by the Lightcone. Black has bought one safe
  turn at the cost of five turns of accumulated pressure.
- **The cone wall.** White Lightcone on e4 has `k = 3`. Black
  rook on h4 wants to step to h1, exiting the cone (h1 has
  Chebyshev distance 4 from e4). Move rejected — the rook is
  pinned-to-cone. Black must either capture the Lightcone or
  wait.

## Where it shines

- Long games. The cone is a clock that rewards patience.
- Positions with material distributed across the board: the
  Lightcone is a slow-burn fence that suddenly closes.
- Variants with multiple Lightcones: overlapping cones produce
  complex no-go regions that grow turn by turn.

## Where it's awkward

- **Initial-setup tempo.** A Lightcone placed at the start with
  `last_move_turn = 0` already has `k = 0` on turn 1. On turn
  10 it has `k = 9` — likely already capping the board. The
  cone-cap saves this, but the early-game tempo is weird.
- **Move-or-not pressure.** The Lightcone's owner must
  sometimes *want* not to move. Engines and players that
  default to "find a useful move every turn" will mis-evaluate.
- **Threat detection cost.** Every piece's legality depends on
  the Lightcone's cone. For a position with `L` Lightcones,
  every move filter does `L` distance checks. Cheap, but
  pervasive.
- **Cone-leaving rule.** "Pinned to cone" or "free to leave"
  is a design fork. The strict version (pinned) is more
  dramatic but produces stalemates more easily. The lax
  version is gentler but loses the inevitability.
- **The Lightcone never moves itself out of danger.** If `k`
  grows so large that the Lightcone is inside every opponent's
  cone, it becomes a free target. Counter-intuitive: the
  *stillest* Lightcone is also the *easiest* to capture.

## Engine dependencies

- Per-piece FEN payload (exists).
- Turn counter (exists).
- Move-legality predicate stack — every move generator filters
  through a global check (planned in plan 10).

## New features required

- **Per-piece "moved this turn" timestamp.** Different from
  has-moved-ever (used for castling). The Lightcone needs the
  exact ply number of its last move. Suggest a generic
  `last_move_turn` field on the piece payload, usable by
  future time-aware pieces.
- **Cone predicate.** A function
  `cone_contains(lightcone, sq, current_turn) -> bool` queried
  by every threat / move generator that has to filter by the
  cone.
- **Move-rejection rule for cone-leaving pieces.** Plan-10-style
  predicate that veto any move whose destination is outside any
  cone the source piece is currently inside. Easy to slot in.
- **Threat-generation filter.** When enumerating Lightcone's
  attacks, intersect queen-rays with the cone.
- **Optional `K_MAX` cap.** Configurable per variant.

## FEN encoding

Lightcone piece-id `L`. Payload stores `last_move_turn`:

```
(P=L,LMT=12)
```

- `LMT` — ply number of last move. If absent, treat as 0
  (Lightcone has effectively been still since the game began).

Cone radius `k` is *not* stored; it's derived as
`current_turn - LMT`. Storing the derived value would be
redundant and a desync hazard.

Example: White Lightcone on e4, last moved on turn 7, current
turn 12:

```
... (P=L,COL=W,LMT=7) on e4 ...
```

`k = 5`. Cone covers Chebyshev radius 5 from e4.

## Determinism notes

- The cone is a pure function of `pos`, `LMT`, and the current
  turn — all FEN-visible.
- Both players can compute the cone identically. There is no
  hidden information.
- The cone-leaving rule produces deterministic move legality:
  given a position, the move generator produces the same
  legal-move set every time.
- Multiple Lightcones interact via intersection of their
  cones; the predicate is order-independent (set membership).
- Initial-placement convention (`LMT = 0` or absent) is fixed
  and documented; no implicit per-side defaults.

## Open questions

- **Pinned-to-cone or free-to-leave?** Recommend pinned for
  the canonical variant; the lax variant is a configurable
  option.
- **Multiple Lightcones on one side.** Their cones overlap.
  Do their *outgoing* attacks combine? Default: each
  Lightcone's attacks are filtered by its own cone only. Not
  the union.
- **Lightcone captured by a piece outside its cone.** This is
  forbidden — the capturer must be inside the cone. But what
  about *en passant*? A pawn that would en-passant the
  Lightcone must itself be inside the cone, including the
  pawn's pre-move square. Default: yes.
- **Lightcone in check.** If the king is inside an enemy
  Lightcone's cone *and* attacked along a queen-ray, that's
  check. If the king is outside the cone, no check, even with
  a clear ray. Probably the most counter-intuitive consequence;
  flag in the UI.
- **K_MAX choice.** Board diameter is the natural cap, but a
  variant might use a smaller cap to keep the Lightcone
  permanently regional.
