# YOU (predicate)

> "...IS YOU." The bound SUBJECT becomes the win-condition holder.
> Capturing any piece marked YOU ends the game.

## Role in the grammar

YOU is a **PREDICATE**. It fills the slot to the right of an IS. Its
semantic effect: the bound SUBJECT-types are flagged "royal" for the
duration of the clause's validity. Capture of any royal piece on a side
ends that game with the opposing side as winner.

In standard chess, the king is implicitly YOU. With `VariantId::Baba`
active and a YOU-clause present, the implicit royalty of the king is
**replaced** by the explicit YOU declaration. The king is no longer
special — whatever the clause names is the new king.

## Inspiration

Baba Is You's YOU tile is the player's identity. "BABA IS YOU" means the
player controls the Baba object. Without YOU, there is no player and the
level is unwinnable.

The chess version translates "the player" to "the win condition target."
Removing all YOU declarations from a chess board means there is no king,
no win condition, and the game can only end by mutual agreement or by
exhausting the rule-piece supply (futile). v1 patches this by *defaulting*
to king-royalty when no YOU clause exists. See "Default behavior."

## Mechanic

### Effect on bound pieces

When `SUBJECT(X) IS YOU` is a valid clause, every piece of type X on the
board is flagged royal. Royal pieces:

- **End the game when captured.** Capture of any royal piece (by either
  side) results in `GameStatus::Win { winner: capturer }`. This bypasses
  checkmate logic entirely — the game ends on the actual capture, not on
  inescapable threat.
- **Replace king royalty.** If `SUBJECT(KNIGHT) IS YOU` is active and
  `SUBJECT(KING) IS YOU` is not (and the king is not in the categorically-
  YOU set), the king is NOT royal. Capturing the king just removes a
  piece. The game continues until a knight is captured.
- **Multiple YOU types coexist.** `KNIGHT IS YOU` and `BISHOP IS YOU`
  together: capturing any knight OR any bishop ends the game. Each
  individual royal-piece-capture is a win condition.
- **No "check" detection.** Baba doesn't compute check anyway — the
  variant runs without king-safety filtering, like Duck Chess. YOU
  pieces can walk into attacks; the game ends on actual capture.

### Color awareness

YOU is symmetric across colors by default. `KNIGHT IS YOU` makes all
knights royal — both white's and black's. Capturing a black knight ends
the game with white as winner; capturing a white knight ends it with
black as winner.

For asymmetric YOU, use a color-qualified SUBJECT: `WHITE_KNIGHT IS
YOU` — only white knights are royal. (See subject.md for the payload
types.) This lets puzzles construct "white must protect their knight"
positions without symmetrically endangering black knights.

### Default fallback

If the parser finds **no valid YOU clause anywhere on the board**, the
engine falls back to "the king is YOU" for both sides. This is the
implicit-default rule. Same as standard chess.

If the parser finds a YOU clause for *one side only* (`WHITE_KNIGHT IS
YOU` but no black-side YOU clause), the white side has knights-as-royal
and the black side has its king as royal (the default).

This means players can transfer their own royalty without affecting the
opponent.

### Effect persists clause-by-clause

When a YOU clause dissolves, the royal flag is removed. **If at that
moment the side has no other royal piece**, the king's implicit royalty
returns immediately. There is never a state where a side has no royal
piece (and could not lose).

## Composition rules

- **YOU ∩ WALL**: piece is royal and impassable. Loses by leaper-capture
  only (gliders can't reach). The classic "immovable target" trap.
- **YOU ∩ NOT YOU**: NOT wins. The piece is not royal. If this strips
  the side's only royal piece, default-king royalty reapplies.
- **YOU on multiple SUBJECTs via AND**: union of types. Capturing any
  of them ends the game.
- **YOU vs default king**: an explicit `KING IS NOT YOU` clause strips
  the king's implicit royalty. Without any other YOU clause naming the
  side's pieces, the default-king fallback re-fires, contradicting the
  NOT. v1: the NOT-clause wins, the king is *not* royal, AND the
  default-fallback does not fire (because there *is* a valid YOU-clause
  in play — the negative one — and the engine considers explicit
  declarations to suppress the fallback).
- This means `KING IS NOT YOU` with no other YOU-clauses produces a
  side with **no royal piece**. That side cannot lose by piece-capture.
  It can still lose by other means (no other v1 means exists; effectively
  this side cannot lose, leading to draw/stalemate fallback after move
  exhaustion). Documented edge case — see open questions.

## Why it's interesting

YOU is the **win-condition** rewriter. Puzzles can ask:

- "Capture the rook" instead of "capture the king" — by making the rook
  YOU instead.
- "Transfer YOU to a buried piece" — when your king is exposed, change
  the win-condition to a piece that's well-defended.
- "Make your opponent's win-condition smaller" — by bringing more of
  their piece-types under a YOU clause, multiplying their failure modes.

The third is mean. `WHITE_PAWN IS YOU` for the black-perspective puzzle:
black wants every white pawn captured because each is a win condition;
white desperately defends pawns they would normally trade freely.

## Example sentences

```
[SUBJECT KNIGHT] [IS] [YOU]
  → knights of both colors are royal. The first knight captured ends
    the game with the capturer as winner.

[SUBJECT KING] [IS] [NOT] [YOU]
  → the king is no longer royal. With no other YOU-clause, the side has
    no royal piece and cannot lose by piece-capture.

[SUBJECT WHITE_PAWN] [IS] [YOU]
  → only white pawns are royal. White wants to defend all eight;
    black wins by capturing any one.
```

## Example puzzle

```
 r . . . k . . .
 . . . . . . . .
 . . . . . . . .
 . [SUBJECT WHITE_PAWN] [IS] [YOU] .
 . . . . . . . .
 . . . . . . . .
 . . . . . . . .
 . . . . K . . .
```

White has king on e1, black has rook on a8 and king on e8. The clause
"WHITE_PAWN IS YOU" sits on rank 5.

There are no pawns on the board.

The clause has no effect (vacuous truth — no white pawns to bind).
But the default-fallback rule says: if there's no YOU-clause for white's
pieces, white's king is royal. Here, there IS a YOU-clause naming white
pawns, but white owns no pawns, so v1 considers this... what? See open
questions.

The intended ruling: the clause-validity check requires the SUBJECT-type
to *exist on the board*. With no white pawns, the clause is treated as
inactive, and default-king fallback fires. White's king is royal.

The puzzle becomes a standard chess problem with a vestigial clause.
The puzzle solver's task is to recognize the clause is inert. Pedagogical.

## Where it shines

- Win-condition-transfer puzzles. The mechanical heart of Baba in chess
  form.
- Asymmetric puzzles where one side's win condition is dramatically
  different from the other's.
- "Save your own king" puzzles where the solution is to make the king
  NOT YOU.

## Where it's awkward

- **The "no royal piece" state.** Producing it via grammar makes the
  side unloseable. v1 documents this as a puzzle-construction concern
  (don't let the player reach this state unintentionally) rather than
  fixing it with engine logic.
- **YOU and Color::Neutral.** Could a Neutral piece be YOU? `IS IS YOU`
  would make IS-tokens royal, capturing one would end the game. v1:
  no — YOU applies only to chess pieces, the same restriction as WALL.
- **YOU during promotion.** A pawn that promotes to a knight while
  `KNIGHT IS YOU` is active: the new knight is immediately royal. If
  this is the only YOU clause active and the king is now non-royal, the
  side's royal piece just changed mid-move. Document the timing: parser
  reruns after the promotion applies.

## Engine dependencies

- `VariantId::Baba`.
- Win-condition logic that consults the royal-piece set instead of
  hardcoded king-capture.
- King-safety filter must be skipped (same as Duck Chess plan 11). Baba
  positions never compute check; they only compute capture.

## New features required

- **`PredicateKind::You` variant.**
- **Royal-piece set** in the rule-effect registry, queried by
  `make_move` after every capture.
- **Default-fallback resolver:** "If no valid YOU clause names a piece
  on side S, side S's king is royal."
- **`GameStatus::Win { winner }`** — already proposed in plan 11; reuse.

## FEN encoding

```
(R=YOU)
```

No payload.

## Open questions

1. **"SUBJECT names a type with zero pieces" = vacuous clause?** v1 says
   yes — the clause is valid grammatically but registers no effects.
   Default-fallback then fires if no *effective* YOU clause names the
   side's pieces.
2. **YOU and promotion choice.** When promotion can target multiple types,
   the player picks which. If some choices are YOU and some aren't, the
   player can choose to make their new piece royal or not. Probably fine
   strategically; ensure the UI surfaces this.
3. **YOU and en-passant capture.** If a pawn is `IS YOU` (white pawn) and
   black captures it en-passant, the captured-pawn's square is the e.p.
   target square, not the capturing pawn's destination. Make sure the
   win-condition check fires on en-passant captures.
4. **YOU and self-capture.** Can white capture their own knight if knights
   are YOU? Standard chess says no (no friendly-fire captures). v1 keeps
   that restriction. But certain modifiers (atomic, locust) capture
   adjacent pieces of any color — does an atomic-self-capture of a royal
   piece end the game with the *enemy* as winner? v1: yes. Document.
5. **No YOU clauses and side has no king.** Can occur in puzzle setups
   (king removed from FEN). v1: that side has no royal piece, treat as
   already-lost — game ends at FEN-load with the other side as winner.
