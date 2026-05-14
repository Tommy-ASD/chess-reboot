# The Owl-Faced Judge

> "Tally up. Tally up. The number is wrong."

## Character

He has the body of a man and the face of an owl — round, pale, with
eyes set too far forward. He wears a black robe that drags. He carries
no gavel; the tally itself is the punishment. He arrived on the board
one evening when no one was watching and he has not introduced himself
to anyone.

He moves slowly. He counts. He counts the pieces on the rank he
currently occupies, and he counts the pieces on the rank that has the
most enemies in it, and when those two numbers do not match he is
unsettled, and he migrates. When they match — when the rank he stands
on is the rank of greatest enemy density — he settles. And then he
considers the oldest piece in that rank, and he removes it. Not in
malice. Not in justice, either. He is keeping a tally that no one
asked him to keep, and the tally requires a piece to be subtracted.

The pieces have stopped trying to understand him. They have begun to
move in flocks, against their better judgment, to dilute the densities.

## Mechanic

**Movement.** King-pattern (one square, any of eight directions). He
captures by landing on enemy pieces — normal capture. He may not
castle.

**Auto-migration.** At the end of every full turn (both sides have
moved), the engine computes:
- `densest_enemy_rank`: the rank (row) containing the most enemy
  pieces relative to the Judge.
- If the Judge is *not* currently on that rank, he migrates one step
  toward it on the next available turn — this is *advisory*, not
  forced; the controlling player chooses the Judge's move, but the
  engine displays the densest rank as a guide.

(See Open questions for whether migration should be forced.)

**The cull condition.** At the end of any turn (either side's), if:
- The Judge is on a rank where the *enemy-piece count* exceeds the
  *enemy-piece count on every other rank* (strict majority — ties do
  not fire),

then the cull fires:
- The engine identifies the *oldest* enemy piece on the Judge's rank,
  by *placement-turn* — the FEN-tracked move-counter value at which
  each piece arrived on its current square. Ties broken by lowest
  file.
- That piece is removed from the board. No capture animation; the
  piece simply leaves. The Judge does not move.

Pieces moved from one square to another have their `placement_turn`
updated to the turn of the move — so a piece that has been *sitting*
on a square for many turns becomes older than a piece that recently
arrived, regardless of game-time existence.

**Friendly pieces.** The Judge does not cull friendly pieces. He
counts only enemy pieces for the density check, and culls only enemy
pieces from his rank.

**Once per turn.** The cull fires at most once per full turn (per
side's move). If a cull empties the densest rank such that another
rank now has the most pieces, the *next* turn's check is recomputed
fresh; no chain-culling within a single turn.

**Multiple Judges.** Each Judge of a colour computes independently.
Two Judges of the same colour on the same rank both fire — the rank's
two oldest enemies are removed. Two Judges of opposite colour on the
same rank each fire on their own enemies (which may be the same set of
pieces from their respective viewpoints, but the targets differ).

**State.** Per-piece: `placement_turn`. This is added to every
piece's FEN payload (a global engine change, not Judge-specific).

## Why it's interesting

The Judge introduces *density-aversion* into positional play. Players
will avoid clustering pieces on the same rank because the rank with
the most pieces eats the oldest. The pawn structure — usually a
linear chain along one rank — becomes a liability. Castling, which
puts the king and a rook on the same back rank, becomes risky.

Because *oldest piece* is determined by recency of movement, players
can defensively re-shuffle their pieces (move them and back, or shift
them sideways) to *refresh* the placement-turn counter. This creates a
new kind of busywork tempo: the *refresh move*, a move whose sole
purpose is to prevent your immovable backline piece from being the
oldest.

The combined effect: the Judge punishes both *concentration* and
*stagnation*. Active distributed play is rewarded; static masses of
pieces are eaten.

## Example scenarios

1. **Back-rank cull.** Black has castled long. Black's back rank has
   king, rook, bishop, knight, pawn (after a pawn moved backward via
   a fairy mechanic) — five pieces. White's Judge migrates onto the
   black back rank. End of turn: the oldest piece is the rook (it
   has not moved since the game started). The rook is removed. The
   king is now exposed.

2. **The refresh dance.** White, foreseeing a Judge approach, moves
   their bishop to a different square and then back over the next two
   turns. The bishop's `placement_turn` is now recent. The Judge
   arrives. The oldest piece on the back rank is now a pawn that
   white had ignored. The pawn is taken. White breathes out.

3. **Mutual stalemate.** Both sides have spread their pieces evenly
   across all ranks. No rank has a strict majority — every cull check
   fails. The Judge migrates aimlessly but never fires. The game
   proceeds normally, except that one piece on each side is
   permanently pacing.

## Where it shines

- Long strategic games. The Judge's effect accumulates.
- Positions with heavy back-rank congestion — endgames after
  castling, especially.
- Asymmetric setups where one side cannot easily distribute (e.g. a
  side with mostly slow pieces).

## Where it's awkward

- Opening play. The Judge's mechanic is silent until densities form,
  which usually requires several moves.
- Sparse positions. With six pieces total on the board, the densest
  rank may have only one piece — strict majority is easy, but the
  cull is trivial and predictable.
- The `placement_turn` requirement adds bookkeeping to every piece on
  the board, which has FEN-size and migration implications (see
  New features).

## Engine dependencies

- King move generation (existing).
- Rank-occupancy counter (trivial; sums pieces per rank by colour).
- Per-piece `placement_turn` state stored in FEN payload. (Major
  engine change; affects every piece type.)
- Turn-counter (existing or trivial).
- End-of-turn hook for the cull.

## New features required

- **Per-piece placement-turn tracking.** Every piece carries a
  `placement_turn` integer in its FEN payload. Set on initial
  placement (turn 0 for setup pieces, turn N for promoted pawns or
  spawned pieces). Updated on every move to the turn-counter value at
  the move.
- **Rank-density helper.** `enemy_pieces_on_rank(rank, side)` —
  trivial sum.
- **End-of-turn cull hook.** New end-of-turn engine step that
  consults all Judges, computes densities, fires culls.
- **Migration hint (optional UI).** The frontend may visualise the
  densest rank as a target indicator.

## FEN encoding

The Judge himself:
```
(P=J,T=JUDGE)     # P=J for Judge
```

Every piece on the board carries a `placement_turn` payload field:
```
(P=R,N=14)        # rook placed on its current square at turn 14
(P=P,N=0)         # pawn placed at setup
```

The `N` field is engine-wide, not Judge-specific. Its presence is
mandatory once any Judge is on the board; arguably, it should be
mandatory in *all* variants for uniformity. Recommend mandatory
globally; defaults to 0 if absent in legacy FEN.

## Open questions

- **Forced migration.** Should the Judge be *forced* to move toward
  the densest rank, or merely advised? Forcing reduces player agency
  (the Judge is barely playable as a normal piece) but increases
  thematic fidelity. Recommend: advisory. The Judge is the player's
  piece; the player decides.
- **Tied densities.** Spec says strict majority required for cull.
  Should ties cause *both* tied ranks to cull? Recommend no — strict
  only, by character (the tally is *wrong* when it does not match;
  ties don't satisfy the trigger).
- **Newly placed pieces and the tie-break.** A piece placed on the
  same turn as another — `placement_turn` ties — the lowest file
  breaks. Confirm that this is deterministic and FEN-replayable.
- **Promotion and placement-turn.** A pawn that promotes — does the
  resulting piece reset its `placement_turn`? Recommend yes — the
  piece is *new*; the queen has only existed since the promotion turn.
- **The Judge culling himself.** Impossible — he only culls *enemy*
  pieces.
- **Mass migration to avoid culls.** Players will redistribute
  pieces. This is the intended outcome. The mechanic is positional
  pressure, not a one-shot trap.
- **King exemption.** Should the king be exempt from culls?
  Recommend yes — the cull cannot remove the king. If the king is the
  oldest piece on the rank, the cull fires on the *next-oldest* enemy
  piece on the rank. (Otherwise the Judge becomes a checkmate
  mechanism, which is too strong for the flavour.)
