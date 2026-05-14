# Sigma

> A piece that refuses to move while any friend stands next to it,
> and rewards prolonged isolation with permanent reach.

## Inspiration

The "sigma male" meme — the lone-wolf archetype that *insists* it
doesn't need anyone, posted ironically by people on the internet
who very much do. The pose is "I am self-sufficient." The piece
takes the pose at face value.

Strip the paint: the mechanic is **anti-cluster scoring.** Almost
every chess heuristic rewards piece coordination (defended pieces,
overlapping influence, pawn chains). A piece that *punishes* the
controller for putting friendly material next to it is a clean
inversion. It rewards spearhead-style outposts and creates a
genuine tension with king safety, since the king's defensive
cluster is exactly the place the Sigma will go inert.

## Mechanic

A long-range slider — by default, **Sigma moves like a queen with
range 2** (any of the 8 directions, up to 2 squares). Range grows
via the grindset counter, below.

### Adjacency lock

At the start of the controller's turn, look at the 8 king-adjacent
squares around the Sigma. If *any* of them contains a friendly
piece (any color matching the Sigma's), the Sigma cannot move
this turn. The controller may still move other pieces. The Sigma
is not stunned (it still attacks, still defends, still blocks
sliders), it simply has zero legal destinations.

Enemy adjacency does not lock. Capturing into Sigma's adjacency
does not unlock retroactively — the lock is sampled at turn start.

### Grindset counter

A non-negative integer, stored as `G=<n>` in FEN. Default `0`.

At the **end** of the controller's turn, evaluate again. If at
that moment the Sigma's 8-neighborhood contains no friendly pieces
*and* no friendly piece moved into adjacency during that turn,
increment `G` by 1. Otherwise no change. `G` never decreases.

The Sigma's slider range is `2 + G`. So:

- `G=0` → range 2 (the default).
- `G=3` → range 5.
- `G=7` → effectively unlimited on an 8×8 board.

Range applies to all 8 ray directions uniformly. The piece is a
range-capped queen.

### Captures

Standard slider capture — the Sigma captures by sliding onto an
enemy square. Adjacency lock applies; if locked, no captures
either.

### Special interactions

- **Promotion target.** A pawn promoting to Sigma starts at
  `G=0`. Common case.
- **Castling.** Sigma is not a king or rook; it does not
  participate in castling. Friendly pieces involved in castling
  may end adjacent to a Sigma — normal lock semantics apply
  starting the following turn.
- **Goblin kidnap.** If the Sigma is kidnapped and later returned
  to the board by the Goblin, its `G` resurfaces with whatever
  value it had at the moment of kidnap. Counter is preserved on
  the piece, not the square.
- **Frozen condition.** A Frozen Sigma cannot move regardless of
  adjacency. While Frozen, `G` still increments if the isolation
  condition is met at end of turn — the meme demands that
  hibernation counts as grinding.

## Why it's interesting

Three nontrivial tensions:

1. **Cluster vs. reach tradeoff.** The natural way to develop a
   piece in chess is to bring it out near other pieces for mutual
   support. The Sigma punishes exactly this and pays for it with
   board-spanning reach later. Players have to commit to *either*
   keeping it as a support piece (low `G`, short range) or
   sacrificing development cohesion to feed its counter.
2. **Asymmetric defense.** Approaching a high-`G` Sigma is hard —
   it covers a third of the board. But you don't need to capture
   it; you just need to nudge a friendly piece next to it, which
   the controller will desperately want to do for protection. Any
   enemy piece they capture *into* adjacency creates a friendly
   replacement (the captured-piece's own former defender), which
   re-locks the Sigma. The meta-game becomes "force the cluster."
3. **Endgame asymmetry.** In endgames where material is thin,
   isolation is cheap and `G` runs up. A long-lived Sigma in a
   sparse endgame is brutal. Conversely, in a packed midgame
   it's near-useless. The piece has a clear life cycle.

## Example scenarios

1. **Lone outpost.** Sigma on d5 with no friendly piece within
   king-radius. End of turn: `G` increments. The controller's
   problem is that any piece they develop to d4, e4, c4, etc.
   shuts it down. They might leave the Sigma stranded on
   purpose, using it as a 5-square-range zone-control piece
   from move 12 onward.
2. **Cluster lock.** Sigma on g2, friendly king on g1, rook on
   h1, knight on f1. Sigma is locked every turn until the king
   moves. Useful if the controller wants the Sigma immobile
   anyway — but means it cannot be deployed defensively.
3. **Adversarial nudge.** Black's bishop captures a white pawn
   on e3 next to white's Sigma on d3. White's turn starts: the
   bishop is enemy, not friendly — no lock. White moves the
   Sigma away. Black's next move places a *black* piece next to
   Sigma's new square — still no lock. The lock only triggers
   when *white* puts friends adjacent. Hostile pieces don't
   help the enemy cluster the Sigma.

## Where it shines

- **Endgames** with sparse boards — high `G` Sigma is a one-piece
  attacking force.
- **Variants with carved boards** (using `Block` walls from plan
  12) — natural isolation pockets reward sigma placement.
- **Anti-fortress play** — once `G` is high, fortress-style
  blockades crack under range.

## Where it's awkward

- **Opening play** — the Sigma sits in the back rank and either
  blocks development or locks itself. Many players will
  effectively skip the piece for the first 10 moves.
- **King-safety conflict** — the player whose king is castled
  will find their Sigma permanently locked if it shelters
  behind the same pawn cluster.
- **Counter farming** — a player who deliberately strands their
  Sigma on the rim and refuses to engage it pumps `G` for free.
  Mitigated by: the piece is still capturable, and a high-`G`
  Sigma exposed on the rim is a juicy target.

## Engine dependencies

- **Per-piece FEN payload** for the `G` counter, same syntax as
  Skibidi's phase and Bus passengers.
- **Turn-start hooks** for the adjacency lock check. Skibidi
  already has the precedent of a piece evaluating its
  neighborhood at the start of its controller's turn.
- **Turn-end hooks** for the increment check. Goblin's "return
  home" trigger fires at end-of-turn; same hook point.
- **Slider movement** with configurable range. Bus's rook-style
  movement already parameterizes path length; reuse that.
- **Variable board size awareness** — `G` should not be capped to
  8 because larger boards exist. Cap at `max(width, height) - 1`
  as a sanity bound.

## New features required

- **Range-parameterized slider primitive.** Build a `slider_with_range(dirs, max_steps)`
  helper if one doesn't exist. Plan 10's movement stack is a
  natural home.
- **Piece-state turn-end increment hook.** Generic enough to be
  shared with Mewing's jaw counter. Worth pulling into a small
  "stateful pieces" subsystem.
- **Adjacency-lock predicate.** Returns `Vec<Move> = vec![]` when
  the predicate is true. Cheap special-case until plan 10's
  modifier stack absorbs it.

## FEN encoding

Symbol: `SI` for Sigma (lowercase `si` for black). Two-letter
codes already exist in the engine for fairy pieces; check
existing two-letter conventions before locking this in.

Payload: a single integer `G`.

```
(P=SI,G=0)      # fresh Sigma
(P=SI,G=4)      # mid-grind, range 6
(P=si,G=11)     # black Sigma, range effectively unlimited
```

Default `G=0` if the field is omitted.

## Open questions

- **Should `G` cap?** Unbounded growth is fine on paper but
  invites silly states. A cap at `2 * board_dim` is reasonable.
- **Does promotion of a friendly pawn adjacent to Sigma break
  isolation?** Probably yes — the promoted piece occupies an
  adjacent square at end of turn, so the increment is skipped
  that turn. Worth a test.
- **Two Sigmas adjacent to each other.** They are friends. Both
  are locked. Both fail the end-of-turn increment. Mutual
  paralysis. Probably correct and funny.
- **Sigma adjacent to a neutral train.** Neutral is not friendly;
  the lock should not trigger. Confirm in test.
