# Sediment

> Every capture leaves a sediment token on the capture square,
> stacked in capture order. The board literally accumulates a fossil
> record of what died where.

## Inspiration

Classical retrograde puzzles ask "what was captured here?" The
solver counts missing pieces and reasons by elimination. The
Sediment mechanic eliminates the inference — the board *shows* what
died, in order, on each square where any death occurred. The
fossil record is on the surface.

## Mechanic

Sediment is a square-level mechanic, not a piece-level one. Any
square on the board may carry a **sediment stack**: an ordered list
of tokens, each token recording a single capture event.

A sediment token is a tuple: `(piece_type, piece_color, ply)`. The
piece_type and piece_color identify *what* was captured; the ply
identifies *when*.

Update rule:
- On any capture, append a token `(captured_piece_type,
  captured_piece_color, current_ply)` to the destination square's
  sediment stack.
- Tokens are *not* removed by any subsequent gameplay. They are
  permanent until an editor stroke removes them.
- A pawn promotion is not a capture; it does not generate a
  sediment token.
- En passant: the captured pawn dies on its own square (the square
  it last occupied), not on the capturing pawn's destination. The
  sediment token is deposited on the captured pawn's square (the
  square *en passant* "looks at").

Stacks are ordered. The bottom of the stack is the oldest token;
the top is the newest.

There is no piece called "Sediment." The mechanic is environmental.
It is in this set because it transforms every capture into evidence
— and the evidence is the kind that retrograde puzzles consume.

## The deduction it enables

A position has sediment stacks:

- d4: `[(N, b, 8), (R, b, 21)]`
- f5: `[(B, w, 12)]`

The solver reads:

1. On ply 8, a black knight died on d4. The capturer was a white
   piece (since the dead piece was black, the move was a white
   move, and ply 8 is move 4, White's move).

   Wait — ply 8 is *Black's* 4th move (plies 1-2 are move 1, etc.,
   so ply 8 is move 4 — and since White moves first, plies 1, 3,
   5, 7 are White's moves and plies 2, 4, 6, 8 are Black's
   moves). Ply 8 is Black's 4th move. But a black piece cannot
   capture a black piece. Contradiction.

   Resolution: the puzzle's ply convention or the sediment record
   is misread. Standardize: ply N means the Nth half-move; odd
   plies are White's, even plies are Black's. Ply 8 is Black's
   4th move — meaning a black piece moved on ply 8. The token
   `(N, b, 8)` says a black knight died on ply 8. A black piece
   dying on a black move means the black piece *moved into
   capture* — i.e., it was captured by a white piece *during the
   resolution of the black move.* This is impossible in standard
   chess. The token is therefore evidence of an *illegal* sediment
   placement, or the convention is "ply N = the move that caused
   the capture, and the captured piece's color must be opposite
   the mover."

Restandardize: ply N is the ply on which the capture *was made*.
The mover is the captor; the captured is the opposite color. So
the token `(N, b, 8)` means: on ply 8 (Black's 4th move), a black
knight died — wait, but on Black's move, a Black piece moves, and
the captor is therefore Black, and the captured must be opposite,
i.e., White. So the token must be `(N, w, 8)` for ply 8 to be
consistent with a black move.

Let me redo with consistent reading: on ply 8, a *black* knight
died. A black knight dying means a *white* piece captured. Ply 8
is *Black's* 4th move. Therefore a black piece moved on ply 8.
But the captor was *White*. Contradiction.

The fix is convention: **the ply in the sediment token is the
ply on which the captured piece died, which is the ply of the
*capturing move*.** A black piece dies when a white piece moves
onto it. A white move is an odd ply. So `(N, b, 8)` is *internally
inconsistent*: ply 8 (an even ply, Black's move) cannot have
killed a black piece.

This is itself a retrograde reading: **the puzzle position with
inconsistent sediment is an illegal position**. The Smullyan
tradition is full of these. The Sediment mechanic catches
illegality automatically.

For a *legal* example, replace `8` with `9`:

- d4: `[(N, b, 9), (R, b, 21)]`
- f5: `[(B, w, 12)]`

Now: ply 9 is White's 5th move. A black knight died on d4 on
White's 5th move — consistent. Ply 21 is White's 11th move; a
black rook died on d4 on that move — also consistent. The
token order says the knight died first (ply 9 < ply 21), and the
stack is bottom-to-top.

So d4 has been a capture square *twice*. The black knight that
died there can be matched to the black knight that is now missing
from the position; same for the rook. The exact captures are
pinned: White killed a black knight on ply 9 and a black rook on
ply 21, both on d4. The composer can now construct the rest of
the puzzle knowing these two events are unavoidable.

## Why it's interesting

Sediment is the *most* concrete retrograde evidence in the set.
Where a Witness only shows the trail of a Witness, Sediment shows
every capture by every piece. Where a Scar shows checks against a
specific king, Sediment shows the fate of every dead piece.

The cost is verbosity: a long game produces many tokens. For
composition, the composer typically *seeds* a position with a
small number of pre-existing sediment tokens (i.e., the position
is presented as "this game has been played, with these captures
already" and the puzzle begins from there). The sediment is the
historical record the puzzle is *about*.

A stack of depth two or more on the same square is a particularly
useful tool: it tells a *small story* about that square. Two
deaths on d4 means d4 has been contested twice. The pieces and
their colors and the ply order constrain what the pawn structure
around d4 must look like.

## Example puzzle setup

```
. . . . . . . k
. . . . . . . .
. . . . . . . .
. . . . . . . .
. . . X . . . .
. . . . . . . .
. . . . . . . .
K . . . . . . .
```

Two kings and a sediment stack on d4: `[(P, b, 3), (P, w, 4),
(N, b, 5)]`.

Reading:
- Ply 3 (White's 2nd move): a black pawn died on d4. White's 2nd
  move killed a black pawn on d4.
- Ply 4 (Black's 2nd move): a white pawn died on d4. Black's 2nd
  move killed a white pawn on d4.
- Ply 5 (White's 3rd move): a black knight died on d4.

Prove: in the first three moves, White's pawn structure went
through specific changes.

White's first move was a pawn move (because by ply 3 = White's 2nd
move, there is a white pawn capturing on d4, but a white piece can
only capture a black pawn on d4 if a white piece is *attacking* d4
— and on move 2, the only feasible attacker is a pawn on c3 or e3,
which requires White's move 1 to have been c2-c3 or e2-e3 followed
by the move 2 pawn capture, or White's move 1 was an immediate
pawn push to c4/e4 with the move 2 capture). Etc. The fossil
record forces a near-unique opening sequence.

## Where it shines

- **Missing-piece puzzles.** "Which piece died?" — the token
  states which. The Sediment mechanic eliminates an entire genre
  of inferential reasoning by making it explicit.
- **Capture-sequence puzzles.** "In what order were captures
  made?" — the stack order is the answer.
- **Pawn-structure reconstruction.** Sediment on the 4th, 5th,
  6th ranks constrains pawn-capture lines.
- **Illegal-position detection.** A sediment token whose ply
  parity conflicts with its color (as in the worked example
  above) is direct evidence of illegality.

## Where it's awkward

Boards accumulate sediment over time. A full-length game with
dozens of captures yields a heavily annotated board. Composition
typically curates a small set of tokens (one to four squares with
shallow stacks) to keep the puzzle readable.

The "en passant captures land sediment on the captured pawn's
square, not the captor's destination" rule is the right rule (the
pawn died there) but is a subtle gotcha for composers.

The FEN payload grows; a stack of five tokens on one square is a
real chunk of text on that square. Beyond ~5 tokens per square the
encoding becomes painful. Curated composition keeps stacks
shallow.

## Engine dependencies

- Existing square-payload system.
- Existing capture detection in the move applier.
- Existing ply counter.

## New features required

- **Sediment stack payload on squares.** A new square-level payload
  key holding an ordered list of tokens.
- **Capture hook.** When a capture occurs, append a token to the
  appropriate square's stack. The "appropriate square" is the
  capture-resolution square: for normal captures, the destination
  of the capturing move; for en passant, the square the captured
  pawn last occupied.
- **FEN encoding.** Stack-of-tuples payload. See below.
- **Editor support.** Composers must be able to seed sediment
  stacks at puzzle setup.

## FEN encoding

Square payload key: `SED`. Value: a list of token-strings, each
token of the form `<piece_letter><color>@<ply>`.

Examples:

```
(SED=Pb@3)              — one token: black pawn died here on ply 3
(SED=Pb@3,Pw@4,Nb@5)    — three tokens, bottom-up order
(SED=Bw@12)             — single white bishop death, ply 12
(SED=Pb@5,T=BLOCK)      — wait, no — a Block square can't accumulate
                          sediment (no piece ever dies there). The parser
                          should warn on this combination.
```

Piece letters: standard (P, N, B, R, Q, K plus the engine's fairy
letters CH, TE, AN, PA, GO, SK, etc.).

Order in the list: bottom-up = chronological. Index 0 is the
oldest token, last index is the newest. Round-trip rule: preserve
order exactly. Do not re-sort.

Empty stack: the `SED` key is absent. There is no `SED=` form.

## Open questions

- **Sediment on a non-walkable square.** Can sediment ever land on
  a Block / Turret / Vent? In normal play, no — pieces can't be
  captured on squares they can't land on. Editor placement could
  contrive it; the parser should warn but not reject (lenient).
- **Sediment from promoted pieces.** A pawn promotes to a queen,
  then is captured. The sediment token records `Q`, not `P` — it
  records *what was captured*, not *what was originally born as*.
  This is the cleaner rule and matches retrograde reasoning ("what
  died here, in its current form").
- **Sediment from a Color::Neutral piece.** Tokens carry a color
  field. Use `n` for neutral. Allowed.
- **Sediment-square interactions with terrain conditions.** A
  Frozen square can still accumulate sediment (the freeze
  prevents *future* moves of the piece on top, not past deaths).
  No special case.
- **Sediment + signal substrate.** A sediment token does not emit
  signals. It is inert.
