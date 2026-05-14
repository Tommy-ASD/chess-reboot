# Scar

> A piece that, on each occasion it is checked, permanently records
> the square the checking piece occupied at the moment of the check.
> The list grows in time order and is never erased.

## Inspiration

The classical retrograde question "was this king in check at any
point?" requires the solver to reconstruct the game's check sequence
from circumstantial evidence. Smullyan's puzzles trade in
"if-the-king-had-been-checked-on-move-N-then-…" arguments. The Scar
makes such arguments concrete: the king (or any Scar-bearing piece)
carries an ordered list of every check it has ever survived.

## Mechanic

A Scar-bearing piece is typically a king, though any non-pawn piece
can be designated a Scar. When a check is delivered to the Scar (it
becomes a legal threatening target, the rules require its side to
respond), the engine appends an entry to the Scar's `S` payload:

```
S = [<square_of_checker_at_check>, <square_of_checker_at_check>, …]
```

Entries are ordered: oldest first, newest last. The list is
**append-only** — entries are never removed, even if the checking
piece is later captured, even if the king moves out of check, even
if the game ends.

The square recorded is the square the checking piece occupied at the
moment of the check, *not* where the king moved to in response. This
is what we mean by "where the check came from."

Double checks append two entries in a single event: the two squares
of the two checkers, in canonical order (file then rank ascending),
appended together as a single sub-list to mark them as part of one
double-check event. The FEN encoding distinguishes single from
double via grouping.

A Scar is not lost on capture. If the Scar-bearing piece is captured,
its accumulated list is gone with it. Composition that needs the
list preserved must keep the piece alive.

A Scar on a non-king piece works the same way, with "check" replaced
by "the piece is attacked and its side must respond" — but in
practice only the king has this property in standard chess. For
non-kings, the rule degrades to "appends on attack" if the engine
ever extends the notion. Default Scar usage: kings only.

## The deduction it enables

A position has the white king on g1 with `S=[e8, h5]`. The solver
reasons:

1. The white king has been checked at least twice during the game.
2. The first check came from a piece on e8. The second came from
   a piece on h5.
3. The list is time-ordered. So the e8-check happened first.
4. The piece on e8 at the time was attacking g1 (or wherever the
   white king was at the time of the e8-check). An e8-piece
   attacking the white king is most likely a rook or queen on the
   e-file or 8th-rank (king was somewhere reachable in line) or a
   bishop on a diagonal from e8.
5. The piece on h5 at the time of the second check was likewise
   attacking the white king (wherever the king was at *that*
   moment). An h5 checker against a king on g1 would be a knight
   from h5 (no — knight on h5 attacks f4, f6, g3, g7, not g1) — so
   the king was *not* on g1 at the time of the h5 check. The king
   moved between the two checks.
6. So the white king has made at least one move after the first
   check, possibly more. Combined with castling rights (if any
   remain — the puzzle composer specifies), we can determine
   whether that move was a castling or not.

For instance, if White still has castling rights on g1 (the king
is on the original castling square, having moved at most by
returning), and the only path of arrival at g1 from a check-
escape is through f1 or h1 — castling is impossible after the king
has moved, so the castling-rights flag in FEN already conflicts
unless the king reached g1 *by castling*. But the king has been
checked, and castling out of check is illegal. So the e8-check
must have happened *after* castling. Combined with the time-order
of the Scar entries, this pins the e8-check to a window between
"after castling" and "before the h5-check," and the h5-check to
a window between "after the king moved off g1" and "now."

## Why it's interesting

A single bit ("has been checked") would already be useful but coarse.
The ordered list of *origin squares* is dramatically richer. Each
entry tells the solver three things at once: the square the checker
was on, the time-ordinal of the check (its position in the list),
and the implicit constraint that the king was at some square
attacked by a piece on that origin at that moment.

The Scar is also the only retrograde piece in this set that records
**other pieces'** behavior on itself. The square recorded is not its
own — it's the checker's. The Scar is a witness to its attackers.

## Example puzzle setup

```
. . . . . . . .
. . . . . . . .
. . . . . . . .
. . . . . . . .
. . . . . . . .
. . . . . . . .
. . . . . . . .
. . . . . . . K
```

White king on h1, with `S=[a8]`. The board is empty of other
pieces.

Prove: at some point in the game, a black rook or queen reached a8
and delivered check from there.

Argument: an a8-piece checking the white king required a clear line
from a8 to wherever the white king was at the time. The piece had
to be a queen, rook (along rank 8 if king was on 8th rank, along
a-file if king was on a-file), or bishop (along a diagonal from
a8). The king is currently on h1; for an a8-piece to have checked
it, the king must have been on the a1-h8 diagonal (bishop check) or
on the 8th rank or a-file (rook/queen check). The king is currently
on h1, not on those lines, so the king has moved since the check.

But all pieces of both colors are now gone from the board *except*
the white king. The black piece that delivered the a8-check has
itself been captured at some point, or moved away and was later
captured. Either way, the Scar `S=[a8]` is irrefutable evidence of
both the check and the existence of a black piece on a8 at some
prior moment, despite that piece being absent now.

## Where it shines

- **"Has Black castled?" puzzles.** A king on g8 or c8 with `S=[]`
  (no checks ever) plus a missing rook is suggestive of castling.
  An `S` list, by contrast, *forbids* castling-out-of-check — so
  certain entries in the list pin the castling event to before or
  after.
- **Mate-history puzzles.** "Show that the mate sequence began on
  move N."
- **Reconstruction puzzles.** Given the Scar list, retrace the
  ordered sequence of checking events to determine pawn-structure
  evolution.

## Where it's awkward

Scar lists grow with each check. A drawn-out game with frequent
checks produces a long list. The FEN remains parseable, but the
list becomes hard to read visually. For composition, this is rarely
an issue (puzzles are typically curated short-history positions);
for analysis of real games, it's an unsightly cost.

Discovered checks and double checks each contribute two entries.
The composer must understand the convention or the puzzle reads
as having one extra check too many.

## Engine dependencies

- Existing check-detection (the engine already knows when a side is
  in check; the hook fires when that detection trips).
- Per-piece payload system supporting list-valued payloads.

## New features required

- **Check-event hook.** When a check is detected at the end of a
  move resolution, identify the square(s) of the checker(s) and
  append to the king's `S` payload. Idempotent — the same check
  detected by two passes of the engine should not double-append.
- **List-valued payload encoding in FEN.** Existing FEN supports
  scalar payloads. The Scar requires a list. See encoding below.
- **Editor support.** Composer must be able to set `S` to an
  arbitrary list in the editor for puzzle setup.

## FEN encoding

Piece payload key: `S`. Value: a comma-separated list of square
names. Double-check events use brackets to group two squares.

Examples:

```
(P=K,S=e8)              — king has been checked once, from e8
(P=K,S=e8,h5)           — checked from e8 then from h5
(P=K,S=e8,[f4,h6])      — checked from e8, then double-checked from f4 and h6
(P=K,S=)                — never checked (empty list; explicit)
```

Empty list: emit `S=` (no value after the equals). Round-trip
preserves the explicit emptiness.

Square names use the engine's standard file-rank format (`a1`
through `h8` for an 8x8 board; the engine's variable-board system
extends this for larger boards).

Order within a double-check group: lexicographic by square name (so
the FEN is canonical regardless of detection order).

## Open questions

- **Self-check (pinning the king to itself somehow?).** Not a
  thing in standard chess. Engine should never append a check from
  the king's own square. Confirmed.
- **Discovered checks: which piece is the checker?** The piece newly
  attacking, not the piece that moved. Standard chess rule. Confirmed.
- **A king delivering check via an interaction (e.g., a Goblin-
  empowered king).** Out of scope for the Scar mechanic itself —
  the Scar records *being checked*, not *delivering check*.
- **Should a non-king piece be Scar-eligible?** Default no. The
  engine could be extended to mark any piece as Scar-bearing and
  treat its "being attacked" events as check-equivalents. Deferred
  to a follow-up.
