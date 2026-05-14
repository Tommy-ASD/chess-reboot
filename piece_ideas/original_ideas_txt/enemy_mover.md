# Enemy Mover

> A piece whose legal moves are the union of all moves any enemy piece could make from its current square.

## Source

From `engine/src/pieces/ideas.txt:1`:

> Piece which can move to any spot which enemy pieces can move to

The very first line of the file. The opening salvo. The mechanic is
self-describing: this piece's move set is *parasitic* on the
opponent's piece composition.

## Inspiration

The interesting question this piece asks: *what does it mean for a
piece's move set to be a function of the opponent's army?* Standard
chess pieces have fixed move sets. The closest existing thing is
the Monkey (chains based on board state), but the Monkey's move set
is still defined in piece-relative terms.

The Enemy Mover (call it the **Mimic** — see naming below) inverts
this: its identity is "whatever you have, I can move like that."
It's an anti-asymmetry piece. The more varied your opponent's
army, the more flexible the Mimic becomes.

## Mechanic

On each move, the Mimic's legal moves are computed as the **union**
of the legal-move sets of every enemy piece *imagined to be sitting
on the Mimic's current square*.

### Computation

For each enemy piece type P currently on the board:

1. Imagine a P (of the Mimic's colour) sitting on the Mimic's
   square.
2. Compute that imagined piece's legal-move set on the current
   board.
3. Add those moves to the Mimic's move set.

The final Mimic move set = union of all such per-type move sets,
minus any moves filtered by king-safety on the Mimic's side.

### Which enemies count?

Two reasonable interpretations:

- **Distinct enemy *piece types*.** If the enemy has 3 knights, the
  Mimic gets knight-moves *once*. The Mimic asks "what *kinds* of
  pieces does my opponent have?"
- **Each enemy piece instance.** If the enemy has 3 knights, the
  Mimic gets knight-moves *3 times*. Same set, no behavioural
  difference (union absorbs duplicates).

**Recommend: distinct piece types.** Computationally cheaper, and
semantically cleaner ("I copy what my opponent has, not how much
of it").

### Carriers and nested pieces

Bus passengers, Goblin kidnap-targets, Locomotive passengers — are
those counted as "enemy pieces the Mimic can mimic"?

**Recommend: yes, but only at the top level.** The Bus itself
contributes Bus-moves. The passenger queen inside contributes
*nothing* to the Mimic. Justification:

- Passengers are inert; they aren't *currently* legal-move sources
  on the board.
- Recursive descent through carriers makes the move-gen rule
  expensive and hard to teach.
- The simple version is: "anything that could currently make a
  legal move on the enemy's turn."

### Empty enemy set / extinction

If the enemy has no pieces at all (extreme edge case — the king is
the last piece, or the king has been captured ending the game),
the Mimic has *no legal moves*.

If the enemy has *only the king*, the Mimic gets king-moves (1
square, any direction). This is the floor: as long as one enemy
piece remains, the Mimic can move.

Edge case: if the only enemy piece is the king and king-moves all
lead to king-safety violations (the Mimic-side king would be in
check), the Mimic has no legal moves — same as any other piece
that has no king-safe moves. Stalemate considerations apply
normally.

### Does the Mimic consume threats?

I.e. does mimicking an enemy queen's moves require the queen to
*exist*?

**Recommend: yes, mimicry requires the source to exist at the
moment of move resolution.** If the enemy queen is captured between
"select move" and "resolve move," the Mimic's move set is
re-computed at resolution. In practice, since moves are atomic
(one move per turn), this never matters within a single turn — the
recompute happens at the *start* of each Mimic turn, using the
current board.

So: the Mimic's move set is *frozen* at the moment the Mimic player
selects their move. The set is computed from the live board on the
Mimic player's turn.

### Captures

The Mimic captures normally on any move that lands on an enemy
piece. The captured piece is removed; if it was the *last* of its
type, the Mimic *loses access to that move-pattern next turn*.
Anti-mimic warfare: trade your varied pieces to flatten the
Mimic's options.

### King-safety

The Mimic is bound by standard king-safety. Mimicked moves that
would leave the Mimic-side king in check are filtered.

The Mimic *itself* can be checking the enemy king on a mimicked
move — totally legal, it's a real piece that really delivers check.

### Castling, en passant, promotion — mimicking weird moves

These are corner cases worth nailing down:

- **Castling.** Mimicking enemy king-moves: does the Mimic castle?
  **No.** Castling is a king-specific compound move with origin/path
  conditions (king and rook unmoved, etc.) that don't transfer to
  a non-king piece. Mimic gets king's one-square moves, not
  castling.
- **En passant.** Mimicking enemy pawn-moves: does the Mimic
  en-passant? **No.** En passant is specific to pawns and depends
  on the prior-move state of an enemy pawn. The Mimic isn't a
  pawn; the compound move doesn't apply.
- **Promotion.** Mimicking enemy pawn pushes to the back rank:
  does the Mimic promote? **No.** Promotion is a pawn-specific
  end-state. The Mimic remains a Mimic.
- **Two-square pawn push.** Mimicking pawn-moves from a square
  not on the pawn's starting rank: **no double push**. The
  imagined pawn isn't on rank 2; double-push doesn't apply.

General rule: the Mimic gets the *raw movement geometry* of each
mimicked piece, not the special-move triggers tied to that piece's
identity or history.

## Why it's interesting

The Mimic's strength is *exactly inverse* to the simplicity of the
opponent's army. Against a pure-pawn opponent, the Mimic is a
glorified pawn. Against an opponent with queens, knights, bishops,
rooks, Goblins, Skibidis — the Mimic moves like *all of them*.

This creates a new strategic dimension: **piece-type discipline**.
Pruning your own army's variety (trading down to fewer types)
suppresses the enemy Mimic. Standard chess doesn't reward type
homogenisation; the Mimic invents the incentive.

## Example scenarios

1. **Early game.** All 16 enemy pieces on the board. The Mimic is
   effectively a queen-knight-pawn hybrid — moves like a queen
   (from queen+bishop+rook), jumps like a knight, has 1-square
   captures forward-diagonal (from the geometry of pawn capture,
   though without double-push or en-passant). Extraordinarily
   strong.
2. **Mid-game.** Enemy has traded down to queen + 4 pawns + king.
   Mimic moves like queen + king + pawn-push-1 + pawn-capture-1.
   Still strong, but the rook-mover loss matters.
3. **End-game.** Enemy has only the king. Mimic moves like a king
   (1 square, any direction). Reduced to weakest "real piece."

## Where it shines

- **Mixed-army positions.** Maximum mimic surface.
- **Asymmetric variants.** A side with varied pieces grants their
  opponent's Mimic that variety.
- **As a counter-piece.** If your strategy involves keeping piece
  variety, the Mimic punishes it.

## Where it's awkward

- **UI complexity.** The Mimic's legal-move highlights are a union
  set — could be the entire board. Visually overwhelming.
- **Move-gen cost.** O(unique enemy types) generator calls per
  Mimic move-gen. Cheap in absolute terms (~6 types max), but a
  new pattern in the engine.
- **Teaching.** "Your piece moves like all my pieces" is a mouthful.
  Once understood, it's natural; first explanation is rough.
- **Self-reference.** Mimic vs Mimic — what happens? Each mimics
  enemy pieces. If the *only* enemy piece is the Mimic, what does
  the Mimic mimic? **Recommend: mimicking another Mimic grants the
  Mimic's *current* move set, computed once.** This is recursive
  but with a clear base case (look at the *other* Mimic's set
  *as it stands*, no further recursion). Documented.

## Engine dependencies

- Move-gen API for arbitrary piece-on-square: needed for the
  imagine-piece-here computation. Likely already exists as the
  per-piece `moves()` method; the Mimic loops over enemy piece
  types and calls each with the Mimic's square.
- Enumeration of piece types on the board: needed to know which
  enemy types are alive. A board-scan helper.
- King-safety filter (existing, applies on top).

## New features required

- `Piece::Mimic` enum case.
- `Piece::Mimic::moves(board, square)` that:
  1. Enumerates distinct enemy piece types currently on the board.
  2. For each type T, computes `T(colour=mimic_colour).moves(board, square)`.
  3. Unions all results.
  4. Applies king-safety filter.
- Edge handling for Mimic-vs-Mimic (terminate recursion at one
  level — mimic the *other* Mimic's current move set as-of-now).

## FEN encoding

The Mimic is **stateless** — its move set is fully computed from
the live board. No payload needed.

Symbol: `M` (white) / `m` (black). Note: lowercase `m` is currently
free; uppercase `M` is also free. The Monkey uses different letters
(check existing assignments) — if `M` collides, fall back to
`MIMIC`.

Example: a Mimic on e4 is just `4M3` in that rank (or however the
existing encoder formats single-letter pieces).

No state, no parens, no payload. Round-trip is trivial.

## Resolving the source's open questions

The source says:

> Piece which can move to any spot which enemy pieces can move to

One implicit question that's worth flagging: *moves from where?* The
phrasing "spots which enemy pieces can move to" could mean:

- (a) Spots enemy pieces *can currently reach* on the board (i.e.
  every enemy's actual current move targets — the Mimic can jump
  to *anywhere any enemy could go this turn*). Powerful, weird,
  and depends on whose-turn semantics.
- (b) Spots the Mimic could reach if it *moved like an enemy piece*
  from its current square. (Mimicry of move-pattern, not of
  destination.)

**Recommend (b).** Reasoning:

- (a) is a teleport-anywhere piece (basically: "go where an enemy
  pawn could push to" = "any rank-3 square if any pawn is on rank
  2"). Hard to defend against, hard to balance.
- (b) is the natural "mimic moves" reading — the Mimic moves *like*
  the opponent's pieces, applied to its own square. Much cleaner.
- (b) preserves chess geometry. A Mimic on a8 with a bishop in the
  enemy army can move diagonally from a8 — same as a bishop
  would. Intuitive.

Other implicit questions, answered above:

- **Distinct types or per-instance?** Distinct types.
- **Carriers descended into?** No.
- **King-safety?** Yes.
- **Special moves (castling, e.p., promotion)?** No — geometry only.

## Open questions (new)

- **Mimic-vs-Mimic recursion.** Recommended one-level recursion
  (use the other Mimic's current set, no infinite descent). Could
  alternatively define as "Mimic-vs-Mimic = king-mover" (the
  weakest possible mimic, since with only-Mimics in play there's
  nothing to copy). Either is defensible.
- **Does the Mimic show a hint of its current move-pattern?** UI
  could display "moves like: Q, N, B" on hover. Suggested for
  the frontend, out of engine scope.
- **Promotion target.** A pawn promoting to a Mimic — does it
  immediately get its move set computed for the new turn? Yes,
  no special handling; promotion is just piece-type replacement.
- **AI evaluation.** Engine eval for a Mimic is hard — its move
  set varies turn-by-turn. Probably evaluate as
  `max(value(P) for P in enemy_types)` as a heuristic. Out of
  scope.
- **Naming.** "Enemy Mover" is descriptive but bad. Candidates:
  **Mimic**, **Echo**, **Mirror**, **Chameleon**, **Imitator**.
  Recommend Mimic.
