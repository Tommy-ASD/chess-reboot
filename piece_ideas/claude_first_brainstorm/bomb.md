# Bomb

> Moves like a Knight. When captured, removes everything on the
> adjacent 8 squares — including the capturer.

## Inspiration

Plan 04's Goblin needs a "captured-piece post-effect" slot: when a
Goblin is captured, the kidnapped passenger drops back onto the
Goblin's square. That same slot — *on-self-capture hook* — is
exactly what the Bomb needs. One generic hook serves both pieces.

The chess problem it answers: captures are unilaterally good for
the capturer in standard chess. The Bomb makes capturing risky.
The opponent has to ask: "is this Knight-shaped piece worth
capturing, given that the capture might cost my Knight, my Queen
on an adjacent square, and a Pawn shielding my King?" Captures
become a calculation, not a reflex.

## Mechanic

Movement set: identical to Knight (`L`-jump, all 8 offsets, jumps
over intermediate pieces).

State: none.

Special property — **Detonation.** When the Bomb is *captured* by
an enemy move, the post-capture hook fires before the next turn:

1. The Bomb is removed (standard capture).
2. The Bomb's square + all 8 neighbour squares are scanned.
3. Every piece on those 9 squares — including the capturer that
   just arrived — is removed from the board.
4. Pieces on neutral side (e.g., Locomotive, Carriage) are also
   removed.
5. Tiles (Block walls, Track, Switches, Gates, etc.) are *not*
   destroyed — only pieces.
6. Pieces removed by Detonation count as deaths for
   death-observation hooks (Reanimator banking, Vampire absorb
   if any Vampire was the capturer — see open question).

The capturer's player still completes their turn, but their move
results in zero pieces gained (capturer's own piece died in the
blast) and a 9-square crater.

Pieces with passengers / graveyards (Bus, Reanimator) lose their
carried state when they die in the blast — standard
carrier-death rules.

Detonation does *not* fire when:
- The Bomb is removed by mechanism other than capture (e.g.,
  Bomb runs over a closed Gate that opens beneath it — n/a since
  Gates don't crush pieces, but the principle stands).
- The Bomb is removed by Goblin kidnap (the Bomb is taken to
  Goblin's home, doesn't explode there — unless Goblin-kidnap
  *is* defined as a capture; clarify).

## Why it's interesting

Three reasons:

1. **Capture-as-cost-bearer.** The Bomb is a piece you *want*
   the opponent to capture if your own pieces are clear of the
   blast radius. Pre-positioning around your own Bomb is a real
   tactical layer.

2. **Asymmetric capture incentive.** Captures usually favor the
   capturer. The Bomb makes capture sometimes favor the
   defender. This breaks the implicit chess assumption that
   *more captures = more advantage*.

3. **Reuses the on-self-capture hook.** Plan 04's Goblin needs
   exactly this hook. Bomb piggybacks on the existing
   infrastructure rather than inventing a new one.

## Example scenarios

**Sacrifice gambit.** White Bomb on e5, with white pieces on
d4, d5, f5 (all out of blast radius — actually d4/d5/f5 are
*in* blast radius from e5 since they're all adjacent. Re-do:
white pieces on d2, h5 — neither adjacent to e5 → safe). Black
Queen on a5 captures Bomb on e5 by long-range slide. Detonation
fires: all pieces on d4, e4, f4, d5, e5 (Bomb), f5, d6, e6, f6
are removed. Black Queen on e5 dies. Any black pieces on those
8 neighbour squares die. Black piece count drops by 1 (Queen) +
N (whatever else was there). White lost a Bomb. Net: black
loses Queen + collateral for one Bomb. Likely a winning
sacrifice for white.

**Bomb chain.** Two white Bombs adjacent: Bomb-A on e4, Bomb-B
on e5. Black captures Bomb-A on e4. Detonation: e4 + 8
neighbours die. Among the 8 neighbours is e5 — Bomb-B. Bomb-B
dies. **Does Bomb-B's death also trigger Detonation?** This is
the cascade question. Recommend: no. Detonation triggers only
on *capture*, not on death-by-blast. (Open question, see
below.) If yes, the cascade can wipe huge chunks of the
board.

**Bomb feint.** White Bomb on a1, no other white pieces nearby.
Black has nothing within blast radius. Black captures Bomb for
free. Detonation removes only the Bomb and the capturer. Net:
1-for-1 trade. Bomb worth slightly less than a Knight
materially, but the *threat* of pre-positioning made it a
distraction asset.

## Where it shines

- Tactical positions with cluster-density of enemy pieces.
- Sacrificial play. The Bomb encourages it.
- Compositions where the player can pre-position around their
  own Bomb.
- Variants where King-safety relies on close-quarters defenders
  — a Bomb near the opponent's King is terrifying.

## Where it's awkward

- Friendly fire risk is constant. New players will lose pieces
  to their own Bomb detonations.
- Hard to evaluate. A position with a Bomb is conditional on
  every possible capture path.
- Computer evaluators will need to search beyond captures (since
  captures of Bombs may cascade); branching factor inflates.
- The Knight-move primitive on a Bomb is occasionally useful
  for repositioning, but the Bomb's strategic value is in
  *standing still as a threat*. Tempo on moving a Bomb is
  often wasted.

## Engine dependencies

- Knight-movement primitive.
- An "on-self-capture" hook (planned for Goblin in plan 04).
  This hook fires after the capture is applied, with the
  Bomb's pre-capture square as context.
- The death-observation pipeline (Reanimator hooks, Vampire
  absorb, etc.) — Detonation removes pieces *via this
  pipeline*, so all hooks fire correctly.
- Tile vs. piece distinction (Block tiles persist; pieces die).

## New features required

- `Piece::Bomb` with no state.
- A `BombCapture` struct implementing
  `engine/src/movement/stack/capture.rs::CaptureModifier`, registered
  with the `CaptureStack` (see `default_capture_stack`). The plan-10
  refactor settled on external `CaptureModifier` registration rather
  than a `Piece::on_capture` trait method — both because
  `CaptureModifier` returns declarative `BoardOp`s (more loggable than
  a `&self` mutation) and because handlers need captor info, not just
  victim. The `BombCapture::apply` body matches on
  `event.victim`'s `PieceType::Bomb(_)` variant, enumerates the 8
  neighbours around `event.victim_coord` (NOT `event.captor_coord` —
  AOE centers on the bomb's old square), and emits a
  `BoardOp::Compose([RemovePiece, RemovePiece, ...])`. The dispatcher
  applies the ops on the post-relocation board. Each removed piece
  does NOT recursively fire its own capture event in v1 — chained
  detonation is an open question. (See `GoblinDropVictimCapture` in
  the same file for the canonical example of a CaptureModifier.)
- Decide cascade behavior (see open questions).
- Tests: Bomb captured by Knight, capturer dies; Bomb captured
  by long-range piece, only the immediate-radius dies;
  Bomb-chain cascade (depending on resolution); Bomb captured
  by Vampire (does Vampire absorb Bomb's "moveset" = Knight?);
  FEN round trip.

## FEN encoding

Piece symbol: `B` (Bomb). Conflicts with Bishop (`B`). Use
`BMB` or `Bm` to disambiguate.

```
(P=BMB)              # white Bomb
(P=bmb)              # black Bomb
```

No state. The Bomb's "fuse" is purely "did you capture me yet."

## Open questions

- **Bomb-chain cascade.** Does a Bomb that dies in another
  Bomb's blast also detonate? Two designs:
  - **No cascade (recommended).** Detonation triggers only on
    *capture by an enemy move*. Death-by-blast doesn't qualify.
    Simple to reason about. Bomb chains require separate
    captures.
  - **Cascade.** Any Bomb death triggers Detonation. Allows
    spectacular chain reactions and clears huge regions. Risk:
    cascade across a board sprinkled with Bombs is hard to
    bound and could trivialize positions.
  v1: no cascade. Allow a future "Cascading Bomb" variant.
- **Vampire absorbs Bomb.** Vampire captures Bomb. Standard
  Vampire-capture rules: Bomb (Knight-mover) added to
  `absorbed`. Then Detonation fires: Vampire dies. Bomb's
  Knight-move was added to a Vampire that just died — useless,
  cleanup discards. Mostly: the Vampire took the same hit any
  capturer would. Worth a test.
- **Bomb capturing Bomb.** White Bomb moves (Knight-jump),
  lands on enemy Bomb. White Bomb captures black Bomb. Black
  Bomb's Detonation: 8 neighbours + black Bomb's old square.
  White Bomb arrived *at* black Bomb's square = inside the
  blast radius. White Bomb dies. Net: trade. Fine.
- **Bomb captured during a multi-step move.** E.g., Bus
  passenger-drop ends adjacent to Bomb, then captures it as a
  second step? The plan 04 Goblin-style hooks should fire
  after the *whole* move resolves. So Detonation fires *after*
  the Bus has finished moving. The Bus might be adjacent to
  the Bomb (in blast radius); Bus dies.
- **Bomb on a square with a `SquareCondition`.** Frozen Bomb:
  Frozen pieces can be captured normally? Yes — Frozen only
  prevents the *frozen piece* from moving. Capture still
  works. So a Frozen Bomb still detonates when captured.
- **Goblin kidnaps Bomb.** Is kidnap a capture? Per plan 04:
  yes, Goblin treats the victim as captured. So Detonation
  fires when Goblin grabs the Bomb. Goblin and 8 neighbours
  die. Net: Goblin loss, Bomb gone. Probably a good rule —
  Bombs counter Goblins.
- **Train runs over Bomb.** Plan 09's Locomotive walks onto
  Bomb-occupied Track tile (Bomb is on a Track? Unusual but
  legal — Track is walkable). Locomotive captures Bomb?
  Trains are neutral; "capture by neutral" is novel. v1:
  neutral pieces destroying enemy pieces should trigger
  Detonation same as any capture. Train + adjacent pieces all
  die. Worth confirming with plan 09's death pipeline.
- **Pinned Bomb.** Bomb can't move (would expose king). Can
  still be captured. Detonation still fires.
- **Bomb's square = King's square.** Bomb cannot be on the
  King's square (one piece per square). Skip.
