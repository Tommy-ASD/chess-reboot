# Latcher

> [ENEMY] Draws a visible leash to its nearest enemy each turn, then
> yanks that target one square toward itself. No damage — pure
> repositioning. Turn the leash against itself.

## Inspiration

The Hook in Hoplite. The grappler enemies in Into the Breach's later
islands. Any puzzle game where the threat is **placement**, not
damage — the enemy moves *you*, not your hitpoints. Latcher exists to
keep the player's pieces *off* the squares they want to be on.

## Mechanic

A Latcher carries one piece of telegraph state: `target_square`,
recomputed at the start of every enemy resolution phase based on
current board state. The Latcher's `target` is:

1. The closest piece (Chebyshev distance) that is **not Neutral** and
   not the Latcher itself.
2. If multiple tie at the same distance, the one earliest in canonical
   board iteration order (file then rank ascending).
3. If no eligible piece exists on the board, `target` is `None` and
   the Latcher does nothing this turn.

On the enemy resolution phase:

1. **Recompute target.** Done at phase start, simultaneously for all
   Latchers (so two Latchers can each target each other's victims
   without ordering ambiguity).
2. **Telegraph.** The leash-line from Latcher to target is rendered.
   (This happened all of the previous turn too — the player has been
   reading it.)
3. **Yank.** The target piece is moved one square along the
   straight-line vector from target toward Latcher. The vector is
   first normalized to one of the 8 compass directions (the one
   closest to the true vector; ties resolved clockwise from N). If
   the destination square is walkable and empty, the target moves
   there. If it's blocked, the target does not move (the yank
   "tugs but slips").

The Latcher itself does not move. It is a stationary repositioning
engine. Captureable by any normal piece.

A Latcher does not damage. It cannot capture a target — only relocate
it. The puzzle is: what square does the Latcher want you in, and
where does that square sit relative to other threats?

## Telegraph rendering

The piece sprite shows a coiled rope or chain. The telegraph is a
dashed leash line from the Latcher's center to the `target_square`'s
center. The line is one-square-thick and translucent — it doesn't
obscure pieces under it.

The leash updates **live** during the player's turn: as the player
moves a piece, the Latcher's `target` recomputes and the leash
snaps to the new nearest enemy. This is critical — it lets the
player *choose* what the Latcher will pull. Move your bishop closer
than your king, and the Latcher latches onto the bishop.

A small directional indicator at the target end of the leash shows
**which square the target will land on** next enemy phase (the
vector-normalized destination). That's the actual telegraph — the
player needs to read "my piece moves *there* unless I prevent it."

## Why it's interesting

The Latcher inverts the relationship between threat and threatened.
Most pieces ask "where will the enemy go?" The Latcher asks "where
will *I* go?" The player's own positioning becomes the enemy
telegraph.

The "heavier neutral adjacent" trick is the key insight. If the
player places a Neutral piece (e.g. a heavy stationary block-piece,
or another Latcher's intended target) adjacent to the Latcher and
that piece is closer than anything else, the Latcher pulls *it*. But
neutrals are excluded from targeting — so the actual lever is
placing a non-neutral piece (yours) closer than the king. The
Latcher pulls the bait, not the king.

Two Latchers near each other create a feedback loop: each tries to
target the closest non-neutral, which might be the same piece.
That piece gets yanked once per Latcher in sequence — moved twice in
one phase. Resolution order matters intensely here.

## Example puzzle

```
6 . . . . . .
5 . . k . . .          k = player king
4 . . . . . .
3 . . . . . .
2 . . . . L .          L = Latcher
1 . . . . . .
  a b c d e f
```

The king on c5 is the only non-neutral piece. Latcher on e2 targets
the king. The vector from c5 toward e2 is roughly SE. Normalized to
8-compass: SE. Next enemy phase, king moves c5 → d4.

Player has **1 Shover** charge. Goal: prevent the king from
*ever* being adjacent to the Latcher.

Turn 1 (player options):

- **Move king west:** king c5 → b5. New vector b5→e2 is roughly
  ESE, normalized to E. Next yank: b5 → c5. King gets pulled back
  to start. No progress.

- **Use Shover:** Shover knight-leaps to a square; the adjacent
  piece gets pushed away from Shover's landing. Place Shover so it
  pushes the Latcher.

  Suppose the player has a Shover piece (call it shovel) somewhere
  off-board, available to deploy via the tool action. A Shover
  action targets an empty square reachable by knight-leap and pushes
  the adjacent-to-landing piece directly away. Say Shover lands on
  d3 (knight-leap from wherever). The square adjacent to d3 in line
  to e2 is e2 itself (no — d3 and e2 are not adjacent in line; d3→e2
  is NE-ish). Adjusting: Shover lands on f3. From f3, e2 is adjacent
  (SW). Pushing e2 away from f3 means pushing e2 in the NE-from-f3
  direction... actually away from f3 is the opposite, so SW. e2 →
  d1. Latcher is now on d1.

  New vector: c5 → d1 is roughly SE. Normalized: SSE → S. Next yank:
  c5 → c4. King moves one south.

  Did this help? Yes: the king is now further from the Latcher,
  and importantly the king is not yet adjacent to the Latcher.

This puzzle is small but it demonstrates the lever: **the Latcher's
threat is a function of geometry, and the player edits the geometry
to redirect the yank to a survivable square.**

A meatier version: add a [Siege Engine](siege_engine.md) facing W on
rank 1. Now the Latcher wants to pull the king *into* the engine's
beam zone. The player's job is to ensure the yank-destination is
*not* rank 1.

```
6 . . . . . .
5 . . k . . .
4 . . . . . .
3 . . . . . .
2 . . . . L .
1 S . . . . .         S = Siege Engine, dir=E, state=Loaded
  a b c d e f
```

The engine fires next phase along rank 1. The Latcher yanks the king
toward e2 (the SE direction). The king's normalized SE step from c5
is d4 — still safe. But if the player moves the king to e4, the
Latcher's vector normalizes to S, yanking the king e4 → e3 — closer
to the loaded engine's beam path. **Don't move toward the Latcher.**

The puzzle is "do nothing on turn 1." The king's c5 is the only
configuration where the yank lands safely. The Siege Engine fires
along rank 1, killing nothing. The Latcher yanks c5→d4. The king
survives. The Latcher's leash is the telegraph that *teaches* you
which moves are safe.

## Where it shines

- Forces the player to think about their own piece positions as
  threats, not just as units.
- Pairs viciously with [Siege Engine](siege_engine.md): the Latcher
  pulls you *into* the beam.
- Combo with [The Clock](the_clock.md): Latcher pulls a piece
  adjacent to a 1-turn-from-detonate Clock. Sometimes the puzzle is
  letting the Latcher pull a *sacrifice* piece next to the Clock to
  absorb the explosion.

## Where it's awkward

- The leash's target recomputes live during the player's turn. Some
  players will find this confusing — "wait, the leash moved when I
  moved my piece?" Documentation matters.
- Two Latchers yanking the same target in one phase is
  order-of-resolution sensitive. Spec it.
- A lone Latcher with nothing on the board does nothing — a degenerate
  puzzle. Always seed the position with at least one yank-target.
- The "vector normalization" rule for which-direction-to-yank has
  edge cases (perfectly diagonal: which way wins the tie?). The
  "clockwise from N" tiebreak is a free design choice — pick one and
  document it.

## Engine dependencies

- `Color::Neutral`.
- Signal payload for `target_square` (recomputed each phase, but
  also serialized for save/load consistency).
- Telegraph resolution phase.
- A piece-translation primitive that respects walkability.
- Board-distance query (Chebyshev).

## New features required

- **`Latcher` piece kind.** `Piece::Latcher`. No persistent dir/state
  payload — the target is recomputed each phase.
- **Live target recomputation hook.** The frontend needs a query:
  "given the current board, what is each Latcher's target?" This
  drives the leash rendering during the player's turn.
- **Normalized vector function.** A primitive that, given two
  squares, returns the closest of 8 compass directions. Reusable for
  other piece designs (any "pull toward / push from" mechanic).
- **Telegraph resolution ordering.** All Latchers recompute targets
  simultaneously, then yank in canonical board order. Two Latchers
  yanking the same piece: it moves twice (sequentially); second move
  evaluates from the post-first-move position.

## FEN encoding

Latcher's `target_square` is recomputed and not strictly required for
FEN — but encoding it makes the saved position deterministic without
re-running the recompute logic on load. Optional payload:

```
(P=LATCHER,C=NEUTRAL)               # target auto-computed on load
(P=LATCHER,C=NEUTRAL,T=c5)          # target cached as c5
```

The `T=` payload (target square, in algebraic notation) is optional.
If absent, the engine recomputes on first reference. If present, the
loaded value is honored *unless* the target square is empty, in
which case it falls back to recompute. Lenient parse.

## Open questions

- **Pull through walls.** Does the leash respect line-of-sight? A
  Latcher with a Block between it and the target — does it still
  yank? Spec says yes (geometry only). A line-of-sight variant
  would make terrain matter more.
- **Multi-yank.** Should a Latcher yank by one square, or by
  Chebyshev-distance squares? Spec says one. Multi-yank is a
  different puzzle entirely (more aggressive, less tactical).
- **Yank cap.** If the target is already adjacent to the Latcher,
  does the yank do nothing, or does the target swap squares with
  the Latcher? Spec says nothing (the normalized vector points at
  the Latcher's square, which is occupied — yank is blocked).
- **What counts as "non-neutral"?** Pure colors only (White, Black),
  or also "puzzle-colored" pieces if such a concept exists? Settle
  before shipping.
