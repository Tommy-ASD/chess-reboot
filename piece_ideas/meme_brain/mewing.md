# Mewing

> A pawn-shaped piece that locks in by standing still — four turns
> of patience promote it into a 3-square king with an anti-promotion
> aura.

## Inspiration

"Mewing" is the meme practice of holding your tongue against your
palate to allegedly reshape your jawline. The bit is that you're
not doing anything — you're just sitting there, "locking in." The
piece literalizes that: stillness is action.

Strip the paint: the mechanic is **a deliberate-tempo investment
piece.** Chess doesn't have many pieces that reward *not moving.*
The closest are pawns sitting on outposts to defend, but their
non-movement is incidental. Here, non-movement is the *only* way
the piece accrues power. It is a self-funding investment with an
explicit four-turn countdown, fully visible to both players.

## Mechanic

### Base form: jaw counter

The Mewing piece starts in pawn-shaped form. It has a per-piece
state `J` ("jaw") — an integer in `{0, 1, 2, 3, 4}`. Default `J=0`.

**Movement (J < 4):** Forward 1 square only. **Does not capture
forward, does not capture diagonally, does not capture at all.**
The piece is fully passive in its base form. It can be captured
normally by enemy pieces; it cannot capture back.

**End-of-turn jaw increment:** At end of the controller's turn,
if the Mewing piece did *not* move during that turn (the
controller chose to move other pieces, or had no choice), `J`
increments by 1, capped at 4. If the piece moved (or was moved by
external effect — kidnap, Gooner shove), `J` resets to 0.

The piece does not have to remain on the same square — it only
has to not be the piece that was moved. A Mewing piece can be
shoved by a Gooner or kidnapped by a Goblin; both reset `J`.
Being captured ends the piece outright.

### Transformation: J=4

When `J` reaches 4 at end of turn, the piece transforms in place
into its **"locked-in" form** on the next turn. Transformation is
permanent. The piece's type changes; the `J` field is discarded.

**Locked-in movement:** Moves like a king, but with range 3.
Specifically: all 8 directions, up to 3 squares per turn. Sliders
in the sense that the path must be clear, but stops at the first
piece (capture if enemy, blocked if friendly). Same path rules as
a queen with range 3.

**Locked-in captures:** Standard slider captures.

**Locked-in aura:** While a locked-in Mewing is on the board, no
**enemy** pawn-shaped piece may promote on any square adjacent to
the Mewing (king-radius). The pawn's controller may push it onto
the adjacent rank, but the promotion is denied — the piece
remains a pawn on the back rank. If the pawn moves off the
adjacent square, it can promote normally.

This is a square condition applied to the 8 squares around the
Mewing, refreshed each turn the Mewing exists. Call the condition
`PromotionLock` — it tags squares as "promotion forbidden here."

### Notes

- A Mewing piece can be created via promotion of a normal pawn
  (if the variant allows Mewing as a promotion target) — it
  starts at `J=0`.
- Two Mewing pieces are independent. Each has its own `J`.
- The aura stacks trivially: two locked-in Mewings adjacent to
  the same promotion square both lock it; removing one still
  leaves the other locking.

## Why it's interesting

1. **Tempo as resource.** The Mewing turns the abstract concept
   "I have an extra move available" into a concrete investment.
   The controller pays four full turns to upgrade the piece. The
   opponent has those four turns to prevent it.
2. **Pre-committed escalation.** A Mewing at `J=3` is a clear,
   visible threat that promotes next turn. The opponent *must*
   respond or accept the upgrade. This pre-commitment creates
   sharper positions than abstract piece value.
3. **The aura mirrors pawn play.** A locked-in Mewing in the
   center of the board denies promotion squares to enemy pawns
   — this is the chess equivalent of "blocking promotion lanes"
   that doesn't currently exist as a mechanic. Pawn-heavy
   variants get a new strategic actor.
4. **Asymmetric reset condition.** Being moved by *your own
   choice* resets `J`. Being moved by external effect (Gooner,
   Goblin) also resets — which means an opponent who can shove
   you with a Gooner has a way to undo your patience without
   capturing the piece. Interesting attack vector.

## Example scenarios

1. **Standard mew.** White's Mewing sits on c3. White moves
   other pieces for four turns. On turn five, white declares
   the Mewing locked-in. It now moves three squares per turn in
   any direction, and any black pawn trying to promote on b8,
   c8, or d8 (if Mewing migrates) is denied.
2. **Gooner counter.** Black's Mewing on f6 is at `J=3`. White's
   Gooner locks onto it and on white's next turn shoves the
   Mewing one square south to f5. `J` resets to 0. Black has
   lost three turns of patience.
3. **Capture-baiting.** White's Mewing on e4 is at `J=4`. Black
   captures it before it can transform. The transformation does
   not happen — captured pieces are removed before the start of
   the next turn, where transformation would fire. Black has
   paid a piece-worth of material to deny the locked-in form;
   was it worth it?
4. **Aura denial.** Black has two pawns on the 7th rank ready
   to promote next turn. White's locked-in Mewing sits on a
   square adjacent to both promotion squares. Black must
   choose: capture the Mewing (with what?), redirect the pawns
   (loses tempo), or promote-but-not-really (pawns stay pawns).
5. **Stalemate weirdness.** Mewing at `J=2`, no legal moves
   (boxed in by friendly pieces). End of turn: `J` increments
   to 3, because the Mewing did not move. Stillness counts even
   when forced. Probably correct; meme demands it.

## Where it shines

- **Pawn-rich middlegames** — the aura is most valuable when
  many enemy pawns are advancing.
- **Slow, positional play** — the Mewing rewards a player who
  is willing to invest tempo on the long game.
- **Variants without too many disrupting pieces** — Goblin and
  Gooner partially counter the Mewing, but a small board with
  Mewings as a featured piece works well.

## Where it's awkward

- **Sharp tactical games** — four turns is a lot in a tactical
  middlegame; the Mewing rarely gets to transform.
- **Crowded positions** — a Mewing in a packed midgame just
  sits and does nothing useful. The base form is genuinely
  weak: no captures, one-square push only.
- **Capture vulnerability at `J=3`** — once the counter
  approaches 4, the Mewing is a giant target. Players will
  often capture it at the cost of a minor piece.
- **Promotion target sanity.** Allowing pawns to promote into
  Mewing means a controller can sometimes get a `J=4`
  transformation for the cost of a promotion turn — except no,
  promotion creates `J=0`. Fine.

## Engine dependencies

- **Per-piece FEN payload** for `J`. Identical to Skibidi/Sigma.
- **End-of-turn hook** to evaluate "did this piece move?". The
  engine already tracks the move list per turn — this is
  straightforward.
- **Type transformation** at end-of-turn when `J=4`. Promotion
  pipeline is the closest existing precedent.
- **Square conditions** — `PromotionLock` is a new condition
  but uses the same vector as `Frozen` and `Brainrot`.
- **Aura refresh hook** — at start of Mewing controller's turn,
  re-apply `PromotionLock` to all 8 adjacent squares. Skibidi's
  brainrot radius is the precedent.
- **Promotion attempt check** — promotion code must consult
  square's `PromotionLock` condition before transforming a
  pawn. New surface but localized.

## New features required

- **`PromotionLock` square condition.** Stub plan: add to
  `SquareCondition` enum, list-of-condition vector already
  exists per square. Conditions are dropped at start of next
  Mewing-controller turn and re-added.
- **Piece transformation primitive.** Generalize promotion's
  type-change logic so non-pawn pieces can transform too.
  Useful for future pieces.
- **Stillness detection.** Boolean per-piece "did this piece
  move this turn." Trivial.
- **Locked-in Mewing as a distinct piece type.** Two enum
  variants: `Mewing { jaw: u8 }` and `MewingLocked`. The
  transformation just swaps the variant.

## FEN encoding

Symbol: `ME` for base-form Mewing, `MK` ("Mewing King") or `LM`
for locked-in. Lowercase for black.

Base form payload: integer `J`.

```
(P=ME,J=0)      # fresh Mewing
(P=ME,J=3)      # one turn from transformation
(P=me,J=4)      # transforming next turn (rare snapshot state)
(P=MK)          # locked-in form, no payload
```

Default `J=0` if omitted.

For the `PromotionLock` square condition, reuse the existing
condition list syntax — `(C=PROMOLOCK)` alongside other
conditions.

## Open questions

- **Should `J` reset on Brainrot or Frozen?** A Frozen Mewing
  cannot move — does that count as not-moving and thus
  increment `J`? Probably yes (the meme requires it). Worth a
  test.
- **Range of the locked-in form.** 3 is a guess. 2 might be
  more balanced; 4 might be too strong. Variant tuning.
- **Aura scope.** King-radius (1) feels right. King-radius 2
  (radius extending to all squares within 2) is much more
  oppressive — probably wrong.
- **Multiple Mewings transforming on the same turn.** All
  transform independently. No interaction issue.
- **Promotion of a captured Mewing via Goblin.** Goblin returns
  the captive home. If the captive was a Mewing at `J=3`, its
  `J` field survives the kidnap (mechanic is consistent with
  Sigma's `G`). On return-to-board turn, the kidnapper moved
  the captive, so `J` resets to 0. Consistent.
- **Aura on the Mewing's own square.** The Mewing itself
  occupies the center of its aura. Promotion on the Mewing's
  square is irrelevant (Mewing already occupies it, no pawn
  can be there). Edge case; no special handling needed.
