# Weird Physics

Eight pieces, each deliberately violating one assumption that
ordinary chess takes for granted. The break is always
mechanical, never cosmetic. The weirdness is always a finite
struct, never randomness. The information is always public.

## The design manifesto

Chess has implicit physics. Pieces are local — they exist at one
square. Causes precede effects — a move happens *then* a capture
resolves. Identity is singular — one piece is one entity.
Material is conserved — count goes down on capture, up on
promotion, never sideways. Threats are instantaneous —
geometric, not propagating. Information is total — the board
state names everything that exists, everything that can happen.

Strip any of these out and the game changes shape.

The pieces here each break exactly one. None break two —
combined breaks compound into incoherence (and are interesting
to consider only as variant compositions, not as single pieces).
None break "the rules" in a soft fictional sense — the break is
in the FEN, in the move generator, in the legality predicate.

The constraint that earned its keep over and over: **all state
FEN-serializable, deterministic, no hidden information, no
randomness.** A finite struct, a pure function, a turn counter,
maybe a circular buffer. The "weirdness" is the *interaction
rule*, not the *information model*.

Hard sci-fi, not fantasy. Each piece is conceptually anchored:
Lightcone (relativity), Tesseract (extra dimension), Apocrypha
(counterfactuals as physical events), Eternal Return (causal
loops with selective memory), Prophet (retrocausal constraint),
Paradox (committed-future), Twin (non-local identity), Mitosis
(scheduled granularity-shift). Egan, Chiang, Tenet, Primer,
Annihilation — the register is post-cybernetic, not magical.

## The break table

| Piece          | Law broken                  | Mechanical core                                       |
|----------------|-----------------------------|-------------------------------------------------------|
| Paradox        | Time / causality            | Move-destination locks 2 ply before move declaration  |
| Twin           | Identity / non-locality     | One piece on two mirrored squares; either dies, both die |
| Lightcone      | Locality / information      | Threat radius expands at 1 square per turn from last move |
| Apocrypha      | Information / counterfactual| Kills the piece that *would have* captured it next ply |
| Mitosis        | Conservation / identity     | Fissions and fuses on a scheduled phase counter       |
| Tesseract      | Dimensionality              | Lives on a hidden parallel layer; drops at landings   |
| Eternal Return | Time / causality            | Capture rewinds the piece K turns; other captures stay |
| Prophet        | Information / retrocausal   | Writes a future board fact; piece freezes or Prophet dies |

## One-line index

- [paradox.md](paradox.md) — sealed-future commitment with a two-ply deadline.
- [twin.md](twin.md) — distributed identity, mirrored across the centre.
- [lightcone.md](lightcone.md) — light-speed threat propagation from last move.
- [apocrypha.md](apocrypha.md) — counterfactual defence by killing the would-be attacker.
- [mitosis.md](mitosis.md) — scheduled splitting and recombination of one piece into many halves.
- [tesseract.md](tesseract.md) — second-board existence with rare drops onto landings.
- [eternal_return.md](eternal_return.md) — capture rewinds the piece's last K turns; tempo dies, material survives.
- [prophet.md](prophet.md) — future-fact commitment that freezes obedient pieces or kills the Prophet.

## Common engine threads

Several of these pieces want overlapping infrastructure: a
per-piece deque of recent positions (Eternal Return), a per-
piece turn-counter (Mitosis, Lightcone, Paradox), a one-ply
lookahead consumer (Apocrypha), a parallel-grid coordinate
(Tesseract), a structured prophecy struct (Prophet), a deferred
move-resolution (Paradox). The natural plan-stub bundling:

1. **Per-piece typed payload extension.** Push beyond the
   existing signal/track payloads into structured per-piece
   state with versioned fields. Most pieces here need it.
2. **One-ply counterfactual lookahead.** Apocrypha needs it as
   a first-class engine primitive. Other future pieces will
   reuse it.
3. **Move-list undo / replay.** Eternal Return is the only
   strict consumer, but a clean undo stack helps testing and
   serialisation across the board.
4. **Hidden parallel grid.** Tesseract is the use case, but
   the same coordinate-pair trick generalises to dimension-N
   pieces if any later category wants them.
5. **Turn-start prophecy / scheduled-effect hook.** Already
   half-present via signals and conditions. Generalising to
   "fire predicate, apply effect" gives Prophet, future
   countdown pieces, and any later mechanism that wants
   per-ply timer behaviour.

## Style note for readers

The plans here describe pieces, not implementations — they pick
the mechanic and explain why it's worth shipping. Each ends in
*Open questions* because every physics-breaker has at least one
unresolved edge case. The plan documents are the answer to "is
this worth implementing?", not "how do I implement it?" Most
require new substrate to land first.

No emoji. No padding. The mechanics are sharp; the speculative
register is reserved for the *Why it's interesting* sections,
where the conceptual elegance earns its line.
