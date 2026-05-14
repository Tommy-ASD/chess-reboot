# Folk Horror

> Pieces that do not want to be played with. They have come to the board for their own reasons.

## Philosophy

The meme-brain pieces are loud. They interrupt. They want attention and
they get it by being absurd. The folk-horror pieces are the opposite
axis: they are quiet, and the quiet is the problem.

Every piece in this category is a **character with a motive**. The
Lantern-Eyed Widow is looking for what was taken from her. The
Bargainer at Crossroads is waiting for someone to make a deal. The
Drowned Miller's Wife is lonely under the wheel. None of them want to
win. They want what they want, and the game is the thing that happens
around them while they pursue it.

The chess rules follow from the wanting. The Widow's lantern marks
pieces because her eyes see what was taken; the Hollow Bride drains
movement from pieces she passes over because she is looking for her
groom and finding only strangers; the Bell-Ringer of Last Parish tolls
on the count of eight because that is how many bells the parish has,
and he is only counting. The mechanic is the character; the character
is the mechanic.

## Tone as mechanic

The atmosphere is load-bearing. A piece called "Owl-Faced Judge" that
removes the oldest piece on the most-populated rank reads as a
calculation problem. A piece called "Owl-Faced Judge" that is *keeping
a tally none of you agreed to* reads as a haunting that happens to be
expressible in chess notation. Same mechanic. Different game.

This category leans hard into prose because that is where the work
gets done. Every piece file has a `Character` section that the engine
will never read. That section is not decoration — it is the
specification for what the mechanic must feel like at the table, and
the test by which a tuning change ("what if the toll happened every
six turns?") gets accepted or rejected.

## Contrast with `meme_brain`

| Axis | meme_brain | folk_horror |
|------|------------|-------------|
| Volume | Loud, interruptive | Quiet, atmospheric |
| Pacing | Burst chaos | Slow dread |
| Player relationship | Antagonistic to seriousness | Indifferent to outcome |
| Win condition pressure | High (must respond now) | Low (the dread accumulates) |
| Reads like | A meme | A fairy tale |

The two categories are designed to coexist. A board with both a
Skibidi and a Hollow Bride feels like a haunted carnival; that is the
correct outcome.

## Index

- [`lantern_eyed_widow.md`](lantern_eyed_widow.md) — back-rank shuffler whose sight-line marks her quarry.
- [`bargainer_at_crossroads.md`](bargainer_at_crossroads.md) — stationary trader of pieces for movement.
- [`hollow_bride.md`](hollow_bride.md) — bishop who drains the pieces she passes over, searching for her groom.
- [`miller_who_sold_his_shadow.md`](miller_who_sold_his_shadow.md) — knight that diagonal-movers pass through.
- [`owl_faced_judge.md`](owl_faced_judge.md) — slow piece that culls the oldest from the busiest rank.
- [`boy_who_followed_the_geese.md`](boy_who_followed_the_geese.md) — willless pawn that walks toward whoever last moved.
- [`drowned_millers_wife.md`](drowned_millers_wife.md) — water-only glider who summons company from the bank.
- [`bell_ringer_of_last_parish.md`](bell_ringer_of_last_parish.md) — motionless ringer counting to eight, again and again.

## Constraints inherited from the engine

All state FEN-serializable. Deterministic. No randomness, no hidden
information. Where these pieces need a turn-counter or a marked-set
they carry it in their FEN payload — see each file's `FEN encoding`
section for the exact syntax.
