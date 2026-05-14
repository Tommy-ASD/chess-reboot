# Apocrypha

> A piece that defends itself preemptively by killing the
> counterfactual attacker — whoever *would have* captured it next
> turn, dies now instead. [INFORMATION / COUNTERFACTUAL]

## The law it breaks

Chess effects propagate forward: a piece exists, an opponent
moves to capture it, the opponent's piece arrives, the capture
resolves. The Apocrypha breaks this forward chain by reaching
into a counterfactual one ply ahead. Its turn does not produce a
move; it produces an *answer to a question that was never
asked* — "if I had stood still, who would have captured me?" —
and that hypothetical attacker dies in the present.

The break is informational: the Apocrypha consults a branch of
the game tree that will not be played, and acts on it. The
branch *would have happened*; it is not random; it is fully
determined by the current position. The piece collapses a
counterfactual into a fact.

## Mechanic

Movement primitive: the Apocrypha optionally takes a king-step
displacement (or stays still — see below). Mobility is
secondary; the killing happens regardless.

Turn flow per Apocrypha move:

1. **Optional displacement.** Owner may move the Apocrypha one
   square in any direction to an empty square (no piece-on-piece
   capture by the Apocrypha itself — its threats are
   counterfactual, never geometric). Owner may also pass: keep
   the Apocrypha in place. Either consumes the turn.
2. **Counterfactual lookahead.** The engine constructs a
   *hypothetical* board for the opponent's next ply by:
   - Reverting the Apocrypha to its pre-step position (or
     leaving it, if the owner passed).
   - Enumerating *all* opponent legal moves on that board.
   - Filtering to moves whose destination is the Apocrypha's
     square (i.e., moves that would capture it).
3. **Counterfactual resolution.**
   - If exactly one such move exists, the piece that *would*
     have made it dies on its current square. Removed from the
     board. The would-have-been capture never occurs.
   - If multiple such moves exist (multiple attackers), all of
     them die. The Apocrypha is a one-ply
     mass-counterfactual-killer.
   - If no such move exists, nothing dies. The Apocrypha's turn
     was a normal step (or a pass) with no counterfactual
     effect.
4. **Apply.** The Apocrypha completes its step (if any). The
   counterfactual deaths are committed to the board.

**Pinning interactions.** Counterfactual attackers must be
*legal* attackers in the hypothetical — pieces pinned to their
own king, for example, do not threaten the Apocrypha and are not
killed.

**Sliders and blockers.** A bishop with a clear ray to the
Apocrypha threatens it. A bishop with a blocker (own or
opposing) on the ray does not. The hypothetical respects all
normal blocker rules.

**Discovered attacks.** If the opponent has a move that would
*discover* an attack on the Apocrypha (e.g., move a knight
revealing a rook ray), the *revealed attacker* is the
counterfactual threat, not the discoverer. The rook dies; the
knight is untouched.

**Self-displacement effect.** If the Apocrypha *moves* on its
turn, the counterfactual is evaluated on the *post-move* board.
Moving Apocrypha to a square where no opponent attacks the new
square produces no kill. Moving Apocrypha *into* an attack
produces a kill of the attacker. Players can use the Apocrypha
to deliberately step into attacked squares to kill the attacker.

## Why it's interesting

The chess novelty: the Apocrypha cannot be attacked by isolated
weak pieces. Any piece that approaches it dies on its own
square, on the Apocrypha's turn. The piece is *anti-fragile to
single-attacker positions* — only multi-attacker forks survive
against it (since all forkers die in one shot, but the
*Apocrypha* still has to commit a single legal move; a fork
with a defended attacker may still threaten via overload).

The conceptual elegance: the engine already does one-ply
lookahead (for legal-move generation under check). The Apocrypha
reuses that machinery but inverts the consumer — it asks "which
moves would be made *against me*" rather than "which moves can
*I* make."

## Example scenarios

- **The harmless approach.** Black knight on c3 moves to threaten
  the White Apocrypha on e5. On White's next turn (the Apocrypha
  passes), the engine asks: who, on Black's next ply, would
  capture e5? Answer: the c3 knight (Nxe5). The c3 knight dies
  on c3. White's Apocrypha did not move; the knight is gone.
- **The fork that fails halfway.** Black queen on d8 and bishop
  on g7 both threaten the Apocrypha on e5 (queen via the
  d8-e5 diagonal, bishop via g7-e5). On the Apocrypha's turn,
  it passes. The engine finds both threatening moves
  (Qxe5, Bxe5). Both pieces die — queen on d8, bishop on g7.
  One Apocrypha turn, two enemy losses.
- **The deliberate suicide-bait.** Apocrypha steps from f6 to
  e5, into the c3 knight's threat range. The post-move
  counterfactual: would Black capture the *new* Apocrypha
  position on e5? Yes, by Nxe5. The c3 knight dies. The
  Apocrypha effectively *teleports* into attacks to kill
  attackers.

## Where it shines

- Defensive endgames where the Apocrypha sits on a key square
  and any approach costs the approacher their piece.
- Positions where the opponent has only one logical attacker —
  the Apocrypha forces them to never commit it.
- Variants with high piece density — every threatening piece is
  exposed.

## Where it's awkward

- **The pass-as-move issue.** "Passing" is not normally a legal
  move in chess. The Apocrypha must be allowed to consume a
  turn without moving, which conflicts with stalemate rules in
  some variants. Solution: the Apocrypha's "pass" is a special
  move type, `MoveKind::Apocrypha(None)`, distinct from a true
  pass. Stalemate is unaffected because the Apocrypha always
  has at least this move available.
- **Double-Apocrypha.** Two Apocryphas on one side — only one
  acts per turn, but their counterfactuals reference each
  other's existence. Default: each Apocrypha's lookahead uses
  the current board (other Apocryphas present); no recursion.
- **Forced sequence.** If the only legal move for the opponent
  is to capture the Apocrypha, and the Apocrypha kills the
  capturer, the opponent has *no other moves* and the game
  stalemates. Rare but real. Document and accept.
- **Pinned attackers.** A pinned piece does not legally threaten
  the Apocrypha and is not killed. Players will misread this as
  a bug.
- **Promotion-square threats.** A pawn that *would* promote on
  the Apocrypha's square is a counterfactual attacker. Default:
  the pawn dies before promoting. The would-be-promoted piece
  is never created.

## Engine dependencies

- One-ply opponent-move enumeration (exists, for check
  detection).
- Legal-move filtering through pin/check predicates (exists).

## New features required

- **Counterfactual evaluation primitive.** Given a board and a
  target square, return the set of opponent moves that would
  capture the target. The engine has the components (legal-move
  enumeration + destination filter); needs a unified call site.
- **Multi-kill effect type.** A single Apocrypha move can
  produce N >= 0 piece removals across the board, distinct from
  the move's mover-displacement. `GameMove` needs an effect
  list, not just `from`/`to`/`captured`.
- **Pass-as-move type.** `MoveKind::ApocryphaPass` recorded in
  the move list distinct from a regular displacement.
- **Counterfactual-kill annotation.** The move log should
  surface "Apocrypha kills knight on c3 by counterfactual" so
  players can audit the engine's choice.

## FEN encoding

Apocrypha piece-id `A`. No payload needed — the counterfactual
is recomputed from the position on every turn, and depends only
on board state visible to both sides.

```
(P=A)
```

A position with one White Apocrypha on e5:

```
... A on e5 ... (no payload)
```

The simplicity is intentional: the break is in the *evaluation*,
not the *state*. The Apocrypha has no internal memory.

## Determinism notes

- The counterfactual is a pure function of the current board.
  Same position, same kill set.
- Opponent-move enumeration is the same machinery used for
  legal-move generation today: ordered, exhaustive, fully
  defined.
- Multi-attacker case: all attackers die simultaneously, in one
  effect block. No "first attacker only" ambiguity.
- No randomness, no temporal state, no hidden information. The
  counterfactual could be computed by either player by hand.
- Discovered-attack resolution: deterministic via the standard
  legal-move enumerator, which respects pins and king-in-check.

## Open questions

- **Counterfactual against an opponent's Apocrypha.** When the
  opponent has *their own* Apocrypha, the counterfactual's
  "opponent next ply" is itself an Apocrypha turn. Does that
  Apocrypha's counterfactual kill apply in the hypothetical?
  Default: no — counterfactuals do not recurse. The
  hypothetical evaluates moves on the *current* board without
  triggering further counterfactual effects.
- **Counterfactual against check.** If the opponent is in
  check on the hypothetical board (because the Apocrypha's own
  position blocks an escape), only check-resolving moves are
  legal. The Apocrypha's counterfactual filters within that
  legal set.
- **Two Apocryphas on the same side moving in sequence.** Only
  one Apocrypha acts per turn (each is a separate piece).
  Counterfactuals between them are non-issues because they're
  on the same side — same-side pieces don't threaten each
  other.
- **King-Apocrypha.** Could a king be the Apocrypha? Probably
  not — the king's check rules and the counterfactual rules
  interact too painfully. Restrict Apocrypha to non-king
  pieces.
