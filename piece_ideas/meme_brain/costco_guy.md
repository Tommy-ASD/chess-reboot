# Costco Guy

> A 2-passenger carrier that only boards equal-or-greater material —
> the inverted Bus: instead of ferrying pawns around, you're
> hauling your queen.

## Inspiration

The "we got [X] at home" meme — a parent telling their kid no, we
already have something equivalent at the house, used ironically to
denote a worse off-brand version of something. The Costco Guy is
that joke applied to piece transport: his "fancy cargo" must
already be valuable to qualify as cargo.

Strip the paint: the mechanic is **a value-inverted Bus.** The
existing Bus is a 5-passenger rook that scoops up pawns and minor
pieces — a logistics piece for the cheap stuff. Costco Guy
inverts the rule: he can only carry pieces *worth more than him.*
This produces a fundamentally different gameplay function. The
Bus moves your scrubs around; Costco Guy is your queen's
chauffeur. The mechanic forces interesting tradeoffs around when
to interrupt a powerful piece's normal movement for transport.

## Mechanic

### Base form

The Costco Guy is a single piece with a small base movement: it
**shuffles one square in any of 8 king directions** when empty,
once per turn. No captures. Material value: roughly 2 — somewhere
between pawn and knight.

It cannot board onto another piece by moving onto its square.
Boarding is initiated as a separate action — see "boarding"
below.

### Boarding

At the start of the controller's turn, the controller may declare
a **board action** instead of a move action. To board:

1. Select the Costco Guy.
2. Select an **adjacent friendly piece** (king-radius 1).
3. The selected piece must satisfy: **material value ≥ Costco
   Guy's own material value.** Pieces below this threshold
   cannot board. (Pawns: no. Knights: yes. Bishops: yes. Rooks:
   yes. Queens: yes. King: see open questions.)
4. If a slot is open (Costco Guy carries up to **2 passengers**
   total), the adjacent piece moves onto the Costco Guy's
   square — it is now a passenger. The Costco Guy's square is
   still a single square on the board; passengers are hidden
   *inside* the piece, encoded in its FEN payload.

A board action consumes the controller's turn. The Costco Guy
does not move during a board turn.

### Unboarding

Symmetric: as a turn-action, declare an **unboard**. Select a
passenger and an adjacent empty walkable square. The passenger
exits onto that square. Passengers exit in any order. Unboarding
consumes the turn.

### Movement (passengers > 0)

The Costco Guy's movement mode changes depending on passenger
count:

- **0 passengers:** shuffle 1 square (king-style, no captures).
- **1 passenger:** **king-mover** (1 square in any direction,
  *with* captures using whatever the carrier's normal capture
  rule is — see open questions).
- **2 passengers (full):** **rook movement** — any number of
  squares horizontally or vertically, standard rook captures.

The rationale: more passengers = more horsepower = more reach.
This mirrors the Bus's mechanic of being slower when empty.

### Captures by Costco Guy

The Costco Guy captures with the body of the piece, not the
passengers. Captures behave like the carrier's current movement
mode — empty = no captures, 1 passenger = king-radius capture,
2 passengers = rook captures along ray.

When the Costco Guy is captured, **all passengers are also
captured.** This is a key strategic constraint: hauling your
queen in a Costco Guy means a single trade can lose you the
queen plus the carrier. The opponent must reach the Costco
Guy with a sufficient threat.

### Passenger contributions

While riding, passengers do not move, do not contribute attacks,
do not block lines of sight (they are inside the carrier). They
exist in FEN only. Castling rights and any piece-specific state
(e.g., a kidnapped Mewing's `J` counter) persist on the
passenger while it rides.

### Friendly pickup at boarding

Boarding is a friendly act — the passenger's controller must be
the same as the Costco Guy's controller. No enemy boarding. No
neutral boarding.

## Why it's interesting

1. **Risk-concentrated mobility.** Moving a queen normally
   exposes it to one threat at a time, with a path the opponent
   can read. Loading it into a Costco Guy and moving the carrier
   instead exposes the entire bundle to one threat. The carrier
   is a high-stakes mobility tool.
2. **Threshold-based gating.** The "equal or greater value"
   rule means a Costco Guy is **useless for pawns and weak
   minors.** It's the opposite of the Bus. Players with a lot
   of low-value pieces can't use it; players who developed
   strong pieces can.
3. **Stack-vs-spread tradeoff.** Concentrating two strong pieces
   into one square reduces the controller's per-piece attack
   surface but increases the per-square loss risk. A different
   kind of position-management problem than chess normally
   offers.
4. **Inverts Bus's role.** Bus and Costco Guy together cover
   the full value spectrum: Bus does low-value transport, Costco
   does high-value transport. A variant with both is a logistics
   playground.

## Example scenarios

1. **Queen shuttle.** Costco Guy on c3, white queen on c4. White's
   turn: board the queen. Costco Guy now has 1 passenger and
   moves like a king. Next turn: move 1 square to d3. Next turn:
   board white's bishop on e3 (also ≥ Costco Guy's value).
   Now full — rook movement. Slide to d7. Unboard queen on d8 —
   queen is now on the 8th rank delivered by truck.
2. **Pawn rejection.** Costco Guy adjacent to a friendly pawn.
   Controller attempts to board the pawn. Move illegal —
   pawn's value < Costco Guy's. The Costco Guy is stuck
   shuffling unless a stronger piece comes adjacent.
3. **Lossy intercept.** Costco Guy with queen + rook passengers
   is on the 4th rank. Enemy queen captures the Costco Guy.
   White loses queen + rook + Costco Guy. White is now down
   the equivalent of 16+ points of material in a single move.
   Catastrophic.
4. **Disembark to launch attack.** Costco Guy with queen
   passenger slides to a key square on the 7th rank, unboards
   the queen. The queen now has back-rank mate threats from a
   square it could never have reached on its own.

## Where it shines

- **Tight closed positions** where the queen and rook have no
  legal squares to develop normally — the Costco Guy is a
  smuggler.
- **Variants with reduced piece counts** where you want big
  pieces to move quickly across the board.
- **Endgame king transport** if king can ride (see open
  questions).

## Where it's awkward

- **Concentrated risk** — a single tactical mistake loses an
  enormous amount of material.
- **Empty Costco Guy** is barely a piece — material value 2
  shuffling around 1 square per turn.
- **Vulnerable to long-range pieces** — a queen or rook can
  threaten a Costco Guy from across the board, forcing
  defensive disembarkation.
- **Material-value lookups** — every boarding check requires
  the engine to know the material value of every piece type.
  Custom pieces (Skibidi, Goblin, etc.) need value assignments.
  Already needed elsewhere; just a dependency.

## Engine dependencies

- **Bus's passenger system** — this is the same mechanic with
  a different capacity (2 vs 5) and a different inclusion
  predicate (value ≥ self vs value ≤ self... well, vs always).
  Pull out the carrier infrastructure.
- **Material value table** — every piece type has a canonical
  value. Engine probably has this somewhere for evaluation;
  expose it.
- **Board action vs move action** — controller selects which
  action to take per turn. Bus already has this.

## New features required

- **Inclusion-predicate-parameterized carrier.** Refactor Bus's
  carrier into a generic `Carrier` trait or struct with
  parameters: capacity, predicate, mobility-by-load. Costco Guy
  and Bus instantiate it differently.
- **Material-value-aware predicates.** Boarding predicate
  consults the piece's material value. Plan stub: add a
  `material_value()` method on piece type, called by
  predicates.

## FEN encoding

Symbol: `CO` for Costco Guy (lowercase `co` for black).

Payload: list of passenger piece codes, comma-separated.

```
(P=CO)                  # empty Costco Guy
(P=CO,X=Q)              # carrying a queen
(P=CO,X=Q,R)            # carrying queen + rook
(P=co,X=Q,N)            # black Costco Guy, carrying queen + knight
```

Passenger codes follow the same single-letter convention as the
Bus's passenger list.

## Open questions

- **Can the king ride?** The king's value is conventionally
  infinite. Allowing king transport sidesteps king-safety
  logic in interesting ways. Probably yes, with a variant
  rule that the king-in-carrier is still in check if the
  carrier is attacked. Open design question.
- **What's the Costco Guy's material value exactly?** Set the
  boarding threshold. 2 feels right (bare carrier is weaker
  than a knight). The value drives which pieces can board —
  too low and everything qualifies; too high and only the
  queen qualifies.
- **Capture mode for 1-passenger Costco Guy.** Does the
  carrier capture with its own attacks, or with the
  passenger's attacks somehow? Simplest: carrier's attacks
  only, ignore passenger. The passenger contributes mobility
  but not firepower.
- **Stacked Costco Guys.** Can a Costco Guy board another
  Costco Guy? Probably yes — value comparison should permit
  it. Carries-passengers-itself nested transport is weird but
  determinable.
- **Goblin and Costco Guy interaction.** If a Goblin kidnaps a
  Costco Guy with passengers, do the passengers go with the
  Goblin? Recommended: yes, kidnap takes the whole carrier
  including passengers. Goblin returns them home together.
- **Promotion of carried pawns.** N/A — pawns can't be
  carried.
