# Mind Control

> A piece that spends a turn marking one enemy piece as "controlled," then dictates that enemy's next move when it would otherwise act.

## Source

From `engine/src/pieces/ideas.txt:11`:

> Mind control piece

That is the entire entry. Two words, no mechanic, no range, no
counter-play. The rest of this doc is design from scratch — but the
core fantasy is fixed: a piece that hijacks an enemy unit.

## Inspiration

"Mind control" in tabletop games (Magic, Hearthstone, XCOM) is one of
the loudest possible swing mechanics — it doesn't just remove a
threat, it converts it. The original note is gesturing at that
fantasy: a chess piece that, instead of *capturing*, *commandeers*.

The Chess 2 lineage already has soft-commandeering (Goblin
kidnap-and-convert), so a hard one-turn commandeer fits the
established vocabulary. Goblin trades movement freedom for permanent
conversion; Mind Control trades permanence for immediacy.

## Mechanic

The Mind Control piece (call it the **Hypnotist** — see naming below)
moves as a king (1 square, any direction) and cannot capture by
moving. It has one alternative action that consumes a turn:

**Cast** — select one enemy piece within range R of the Hypnotist's
current square. The selected piece is marked as **controlled** with a
duration counter `D` set to 1. The cast costs the Hypnotist's full
turn.

**Resolution** — at the *start* of the controlled piece's owner's next
turn, the controlling player (the Hypnotist's side) chooses that
piece's move from the controlled piece's normal legal-move set. That
move counts as the enemy's full turn. After the move resolves, `D`
decrements; when `D` reaches 0 the mark clears.

Concretely:

- **Range R**: 3 squares Chebyshev (a 7×7 box centred on the
  Hypnotist). Justification: king move + chess-king-radius gives the
  Hypnotist some breathing room, R=3 stops it from being a global
  threat, R=8 (full board) would feel oppressive.
- **Duration D**: 1 turn. Castable again immediately the next turn.
  Longer durations make the mechanic NP-hard to evaluate for the
  defender and trivialise endgames.
- **Cooldown**: optional. **Recommend none** — the Hypnotist trades a
  full turn for the cast already, which is its own cooldown. If
  playtests show it's oppressive, add a 2-turn cooldown encoded as a
  small counter on the Hypnotist.
- **Legality of forced move**: the chosen move must be legal *for the
  controlled piece's owner*. King-safety, brainrot, frozen, walkability
  all apply. The controller cannot, for example, force the enemy king
  into check.
- **Mutual checks**: if the only legal moves for the controlled piece
  would leave the controller's king in check after the move, the
  controller is the one in check — meaning the controller chose the
  move and chose poorly. No special-case.
- **Stalemate interaction**: if the controlled piece has no legal
  moves, the controller must pick a different piece — but the mark is
  already placed. **Recommend**: if the controlled piece has zero legal
  moves at resolution time, the mark fizzles and the controlled
  player picks their own move freely. Avoids a softlock.
- **Capturing the Hypnotist while a mark is live**: the mark persists.
  The controlled piece is still controlled for its next move. (The
  spell was already cast; killing the caster doesn't unmake it.)
  Alternative: capturing fizzles the mark. **Recommend persistence**
  — it makes the cast feel like a real commitment.

### State

Per Hypnotist: nothing. The cast doesn't store data on the caster.

Per controlled piece: a single field `controlled_by: Option<Color>`
plus implicit `D=1`. Since duration is always 1, the field is
boolean-equivalent — "controlled this upcoming turn, by colour X."

## Why it's interesting

Mind Control is the only mechanic in the engine that hands the *move*
to the wrong side. Every other piece — Goblin, Skibidi, even Block
square — modifies the board or the available moves. Mind Control
modifies *whose decision it is*. That's a category change.

It's also the answer to a class of puzzles existing pieces can't
express: "force your opponent to walk into a fork." Today a fork is
something *you* set up; the opponent walks in or doesn't. Mind
Control makes the walk-in the controller's choice.

## Example scenarios

1. **Forced suicide.** Black king on g8, black queen on g7, white
   Hypnotist on e5. White casts on the black queen. Next turn, white
   moves the black queen to h7 (legal — adjacent to king, no check
   yet). The queen is now en prise to a white knight on f6. Black has
   donated their queen.
2. **Defensive sabotage.** White rook is one move from capturing a
   white piece. Black Hypnotist casts on the white rook. Next turn,
   black moves the white rook to a corner. White has lost tempo and
   the rook is now passive.
3. **Anti-check.** White is in check from a black rook on the e-file.
   White's Hypnotist is in range of the black rook. White casts; next
   turn — but white moves first since white is still in check. White
   *cannot resolve the check* by casting, because casting consumes the
   turn that should resolve check. The cast is illegal under the
   king-safety filter. (Important: the cast resolves on the *enemy's*
   next turn, so it doesn't address current check.)

## Where it shines

- **Endgames with few pieces.** Each remaining unit matters more, so
  hijacking one for a turn is a bigger relative swing.
- **Variants with high-value targets** (Bus with a king passenger,
  Locomotive carrying a queen). Forcing the carrier to drive off a
  cliff is a unique threat.
- **Puzzle compositions.** The "force the enemy to move *here*" idiom
  unlocks a puzzle genre that doesn't otherwise exist.

## Where it's awkward

- **Two Hypnotists on one side.** Stacks: cast, cast, cast — three
  enemy turns in a row hijacked. Plausibly fun, plausibly degenerate.
  Suggest each side may field at most one.
- **King control.** Forcing the enemy king into a corner-of-your-choice
  is brutal. Don't ban it, but note that king-safety means you can't
  force the king *into* check, only *toward* check. Still strong.
- **AI/engine cost.** Move search has to consider, on the opponent's
  turn, that the move might be picked by *you*. The branching factor
  doubles in subtrees where a mark is live. Tolerable; not free.

## Engine dependencies

- Move dispatch needs a new "GameMove" variant for cast.
- Turn flow needs a hook: at the start of a player's turn, check
  whether any of their pieces are controlled, and if so, hand the
  move-choice UI / API to the *other* side for that one move.
- King-safety filter applies to forced moves (already general).

## New features required

- `GameMove::CastMindControl { from: Square, target: Square }`.
- Per-piece flag `controlled_by_next_turn: Option<Color>`.
- API/UI: when it's player X's turn and one of X's pieces is marked,
  the move-input flow asks player Y for the move on that piece, then
  control returns to X for any remaining decisions. (X has no other
  decision; the marked piece consumes the turn. So practically: Y
  picks one move, the turn passes.)
- FEN field for the mark.

## FEN encoding

Two pieces of state need to round-trip:

1. **The Hypnotist itself** — just a piece symbol, e.g. `H` (white)
   / `h` (black). No payload. If a cooldown counter is added later it
   would live here: `H(CD=2)`.
2. **The controlled flag** — a payload on the controlled piece:
   `Q(MC=W)` meaning "queen, mind-controlled by white on the next
   resolution." Default absent.

Examples:

```
rnbqkbnr/pppppppp/8/8/8/4H3/PPPPPPPP/RNBQKBNR w KQkq - 0 1
```

A Hypnotist on e3.

```
... 4Q(MC=B) ... w ...
```

A white queen marked for black's control on its owner's (white's)
next turn.

The `MC=W` value records *who* gets to make the move. At resolution
the engine asks that side for input, then strips the payload.

## Resolving the source's open questions

The original note has none — it's two words. Instead, this section
addresses the questions the brevity *forces* us to decide:

- **What does it move like?** King. Justification: the Hypnotist is
  fragile and meant to be positioned carefully; a long-range piece
  with mind control is two power budgets in one. King-mover is the
  default for "support piece with a strong active."
- **Can it capture normally?** No. Capturing trivialises positioning
  — the Hypnotist becomes a queen with a free spell. Forcing the
  cast-or-don't-move binary keeps the piece honest.
- **Range of cast?** 3 squares Chebyshev. Tunable.
- **Duration?** 1 turn. Anything longer turns into a chain-control
  game.
- **Cooldown?** None initially. Add if oppressive.

## Open questions (new)

- **Multiple casts on the same target.** If both sides have a
  Hypnotist and both target the same piece in the same round, whose
  control resolves first? Recommend: order by turn — whichever cast
  happened last is the live one. The earlier mark is overwritten.
- **Promotion under control.** If a pawn is mind-controlled while
  one move from promotion, the controller picks the promotion target.
  Probably fine; flagged for explicitness.
- **Cast on a carried piece.** Bus passengers don't have legal moves
  of their own. Cast should be illegal on carried pieces (no move
  set to draw from). Alternative: cast on the carrier and force a
  drop. Recommend the former for simplicity.
- **Naming.** "Mind Control" is the mechanic, not the piece name.
  Candidates: **Hypnotist**, **Mesmer**, **Charmer**, **Puppeteer**.
  Recommend Hypnotist (chess pieces traditionally have one-word
  occupational names: Knight, Bishop, Rook).
