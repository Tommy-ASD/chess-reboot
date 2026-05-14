# Promotee

> A piece created by pawn promotion that records, on itself, the file
> of the originating pawn and the ply on which the promotion happened.
> The two values together pin down a specific past move.

## Inspiration

Classical chess hides one of the most informative events in the
game: promotion. After a pawn promotes, the new piece — a queen,
say — is *indistinguishable* from a queen that started the game.
This is occasionally a real cost in retrograde reasoning: the
solver wants to know whether the queen on e8 was *born* there or
*arrived* there. Without external annotation, the answer requires
counting and elimination.

The Promotee variant of any promotable piece (queen, rook, bishop,
knight) carries its own birth record. The new piece *remembers*
that it was a pawn, and remembers exactly which pawn.

## Mechanic

When a pawn promotes, the resulting piece is *not* a fresh queen
(or rook/bishop/knight) — it is a Promotee. The Promotee is a piece
that, in all gameplay respects, moves and acts identically to its
promotion target. Functionally it is a queen (if the player chose
queen), etc. The *only* difference from an "original" queen is its
FEN payload: a record of birth.

The birth record is a tuple `(file, ply)`:
- **file:** the file the originating pawn was on at the moment of
  promotion. (For an 8x8 board, one of `a`..`h`. For variable
  boards, one of the board's file labels.)
- **ply:** the ply on which the promotion happened. (Half-move
  counter.)

Both values are set at promotion and are immutable for the
remainder of the piece's life. Capture destroys the record; there
is no transfer.

A Promotee can never be created by editor placement *without*
specifying both values. A puzzle composer placing a Promotee
declares the birth record explicitly.

The promotee tag distinguishes a promotion-born queen from a born-
on-d1 queen. *Both* exist in the engine. The starting position has
all-original queens; only post-promotion queens are Promotees.

## The deduction it enables

A position has a white queen on h8 with payload `f@13`.

The solver reads:

1. The queen on h8 was originally a white pawn on the f-file.
2. The promotion occurred on ply 13.
3. Ply 13 is White's 7th move.

From the file and the side, the original pawn was White's f-pawn.
White's f-pawn started on f2. To promote, it must have reached f8
(or some 8th-rank square — but a pawn on the f-file can only
promote on f8, unless it captured along the way to a different
file, in which case the *promotion square* is whatever file it
ended on. But the Promotee records the file *at promotion* — the
file of the pawn on the move it promoted. So the pawn promoted on
the f-file — i.e., on f8.)

Wait — the file recorded is the file *at the moment of promotion*.
If the pawn captured during its march and changed files (a typical
pawn-promotion trajectory), the recorded file is the *final* file
(the file where it actually promoted). So `f@13` means "promoted
on the f-file."

But which pawn? The white f-pawn? Not necessarily — a white pawn
that started on, say, the e-file could have captured onto the f-
file at some point and then promoted on f8. The file alone does
not pin the *original* file; it pins the *promotion* file.

So the Promotee's evidence is: "a white pawn promoted on the f-
file on ply 13." Combined with the puzzle's other pawn-structure
evidence, the solver can usually pin which original pawn it was —
but the Promotee by itself does not.

Continuing the deduction:

4. Ply 13 is White's 7th move. So a white pawn was on f7 at the
   end of ply 11 (the move before), played from f7 to f8 on ply
   13 with promotion. (Or captured on f8 — same conclusion.) Or
   played from e7 or g7 to f8 with capture-and-promotion.
5. Whichever the case, on ply 13 a Black piece was *captured* on
   f8 (if the promotion was a capture) or *no* capture happened
   (if it was a quiet promotion-push). The Sediment token (if
   any) on f8 distinguishes.
6. The Promotee + the (absent or present) Sediment token together
   pin the move completely: file, square, ply, capture-or-not,
   captured-piece (if any).

## Why it's interesting

A queen carries information about her birth. This is the only
piece in the retrograde set whose evidence is *fundamentally about
identity* rather than about motion or interaction. The Promotee
answers "who are you?" rather than "what have you done?"

The composer's leverage: a Promotee in the puzzle's presented
position is direct, irrefutable evidence that a promotion event
occurred. The ply is exact. The promoting file is exact. The
mover's side is exact (the Promotee's color is the mover's color).
This is a remarkable amount of evidence packed into a four-
character payload.

Pairing a Promotee with a Sediment stack on its promotion square
yields a *complete* reconstruction of the promotion event: what
file the pawn was on (Promotee), what ply it promoted on
(Promotee), what (if anything) it captured (Sediment).

## Example puzzle setup

```
. . . . . . . k
. . . . . . . .
. . . . . . . .
. . . . . . . .
. . . . . . . .
. . . . . . . .
. . . . . . . .
K . . . . . . Q
```

White Promotee queen on h1 with `f@13`.

Wait — a white queen ends up on h1? Queens promote on the 8th rank
(from a white pawn's perspective). A white Promotee on h1 means
the queen has moved at least once since promotion (from f8 to h1).

The puzzle: prove the white queen made at least one move after
promotion.

Argument: the Promotee promoted on f8 (recorded file = f). The
queen is currently on h1. f8 != h1. So the queen has moved. (At
least one move, possibly more.)

Combined with a Chainwalker showing some count, the puzzle pins
the exact number of post-promotion queen moves: total ply count
minus promotion ply minus Chainwalker count.

## Where it shines

- **Promotion-was-here puzzles.** Direct identification of which
  pieces are promoted vs original.
- **Pawn-trajectory reconstruction.** The promotion file + Sediment
  on the promotion square together pin the pawn's terminal moves.
- **Move-count puzzles.** "How many moves has the white queen
  made?" — knowable when the queen is a Promotee with known birth
  ply.
- **"Was it a capture?"** — pairs naturally with Sediment on the
  promotion square.

## Where it's awkward

The original-piece vs Promotee distinction must be maintained
through the entire engine. Every queen, rook, bishop, knight is
potentially-a-Promotee at the type-system level. This is a wide
surface for a small evidence gain — but the gain is *exact* and
*permanent*.

A pawn that under-promotes (to knight, bishop, rook) creates a
Promotee of that target type. The FEN encoding must cover all four
promotion targets; the mechanic is the same.

The composer choosing to place a Promotee in the puzzle must
specify the birth record. There is no default. This is a slight
ergonomic cost in the editor.

## Engine dependencies

- Existing promotion mechanic.
- Per-piece payload system.
- Ply counter (already present for halfmove-clock / move-number).

## New features required

- **Promotee marker.** A flag (or "is-promoted" boolean) per
  promotable piece. The flag enables the `f@ply` payload.
- **Promotion hook.** At promotion time, the engine sets the
  Promotee flag and records the originating file (file of the
  pawn just before its promoting move) and current ply.
- **FEN encoding.** New payload form, see below.
- **Editor support.** When the composer places a queen (or rook
  etc.), an optional "promotion-born" toggle with file/ply
  fields.

## FEN encoding

Piece payload key: `R` (for "record" — birth record). Value:
`<file>@<ply>`.

Examples:

```
(P=Q,R=f@13)            — white queen, promoted on f-file, ply 13
(P=Q,R=h@27,F=COLOR:b)  — black queen, promoted on h-file, ply 27
(P=N,R=a@8)             — white knight (under-promotion), a-file, ply 8
(P=R,R=d@31)            — white rook (under-promotion), d-file, ply 31
(P=B,R=g@19,F=COLOR:b)  — black bishop (under-promotion)
```

The absence of `R` means the piece is *original* (not a Promotee).
A queen on the board with no `R` payload is the original queen
that started on d1 (or d8 for black).

Round-trip rule: emit `R` if and only if the piece is a Promotee.
Never emit `R=` (empty). The presence of the key carries information.

File label: use the engine's file naming for the board in question
(letters for 8x8; the engine's variable-board file labels
otherwise). Ply is a non-negative integer.

## Open questions

- **Is a Promotee distinguishable in gameplay from an original?**
  No. It moves and captures identically. The distinction is purely
  evidential.
- **Can a Promotee re-promote?** No — Promotees are not pawns.
  The mechanic ends with the original promotion event.
- **What if a pawn promotes via a capture onto a different file?**
  The recorded file is the file of the pawn *after* the promoting
  move (i.e., the file where it ended up — the promotion square's
  file). This is what the solver sees and reasons about.
- **What if a pawn promotes via a non-capture push?** Same rule —
  the file is its current file (which equals its starting file
  for a pure-push promotion).
- **Under-promotion to a non-standard piece (e.g., a fairy
  piece).** Allowed if the variant permits it. The Promotee
  payload attaches to whatever piece type the pawn promoted to.
  The engine's promotion menu may need extending; the Promotee
  payload extension is orthogonal.
- **Editor-placed Promotee with an absurd ply (e.g., `f@1`).**
  Allowed but warned. A promotion on ply 1 is impossible by
  legal play (a white pawn cannot promote on its first move).
  The editor surfaces the warning; the FEN parses regardless.
