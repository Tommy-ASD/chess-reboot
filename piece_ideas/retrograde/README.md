# Retrograde pieces

Forensic chess. Eight pieces whose **state on the board is testimony
to the past**. None of them are designed for competitive play. Every
one of them exists to make a class of retrograde-analysis puzzles
possible, in the Smullyan tradition.

## The thesis

In a competitive game, a piece's state is what it can *do next*: where
it can move, what it threatens, whether it has the right to castle.
The past is collapsed into a few flags (castling rights, en-passant
target, ply counter).

In a retrograde puzzle, the player is given a position and asked to
prove something about the past. *Who moved last? Has Black castled?
What was captured on f4?* The classical answer relies on
piece-counting, pawn-structure logic, and the implicit constraints of
legal play.

The retrograde piece set turns this implicit reasoning into **explicit
evidence**. Each piece writes its own forensic trail directly onto the
board. The puzzle becomes deduction over a record, not over silence.

## What makes state into evidence

Three properties:

1. **Visibility.** The state is in FEN. No hidden information. Both
   players, and the puzzle reader, can see it.
2. **Permanence.** Evidence does not retract. A Sediment token, once
   placed, stays. A Scar, once acquired, accumulates. Even pieces that
   *update* their state (Tether, Antipode) overwrite a previous record
   — they never lose the *fact that the record exists.*
3. **Determinism.** Given a sequence of legal moves, the resulting
   evidence is unique. The puzzle composer can rely on it. The solver
   can reason backward from it.

These three together are why the FEN encodings in each file are
designed with care. The evidence has to round-trip and has to be
inspectable by hand.

## Competitive vs forensic

A Goblin is a competitive piece — its design question is *what should
it do on its turn?* A Witness is a forensic piece — its design
question is *what trail does it leave for the solver to read?* The
two design modes share an engine but pull in opposite directions:

- Competitive pieces compress past into present (efficient state).
- Forensic pieces expand past into present (verbose state).

The retrograde set is unapologetically verbose. A Chainwalker's `C12`
flag, a Scar's accumulated list of check origins, a Sediment stack
five tokens deep — these are features, not warts.

## Why all eight are FEN-state-rich

Each piece adds at least one new FEN payload form. Several add
*square-level* state that lives independently of the piece sitting on
the square (Witness notches, Sediment fossil stacks). The engine
already accepts parenthesized payloads on squares and pieces — these
designs extend that surface, they do not break it.

Round-trip is non-negotiable. A puzzle saved and reloaded must
present an identical evidence record.

## Index

| File | One-line pitch |
|---|---|
| [witness.md](witness.md) | Pawn-like piece that scratches a directional notch onto every square it leaves. |
| [chainwalker.md](chainwalker.md) | Carries a strictly-increasing move counter visible in FEN. |
| [tether.md](tether.md) | Paired pieces that record the Chebyshev distance between them at their last move. |
| [scar.md](scar.md) | Accumulates a list of squares from which it has been checked. |
| [antipode.md](antipode.md) | Mirrors its twin one tempo late; freezes with a ply-stamp when the twin dies. |
| [parole.md](parole.md) | May only revisit a square once an enemy has visited it in between; carries the bar list. |
| [sediment.md](sediment.md) | Capture squares retain a stacked fossil record of everything that died on them. |
| [promotee.md](promotee.md) | A promoted piece remembers the file and ply of the pawn promotion that produced it. |

## A note on composition

These pieces are tools. A retrograde puzzle generally uses one, maybe
two of them — a position rich in evidence is also a position rich in
constraint, and constraint is what makes a unique solution possible.
Stacking all eight in one position would produce a puzzle that solves
itself.

The composer's craft is to choose the *one* piece whose evidence
exactly suffices to pin down the past being asked about. The engine
provides the vocabulary. The puzzle provides the question.
