# Combo Theorist

> Pieces as **nodes in a synergy graph**. Identity is what they enable, gate, or punish — never what they do alone.

## Premise

Read these the way you read a Magic: the Gathering set, a Slay the Spire
relic pool, or an Inscryption sigil. A piece by itself is a stat line.
A piece in the right *deck* is a wincon. The interesting design space
is not "what does this thing do on move 1?" — it's "what does it
*enable* on move 8 given what else is on the board?"

The eight pieces in this directory are deliberately weak in isolation.
They earn their place by:

- **Tempo** — buying or stealing half-moves (Quartermaster, Echo).
- **Denial** — making squares or actions unavailable (Lien, Trellis,
  Plague Doctor).
- **Escalation** — getting stronger the longer they live (Hourglass,
  Tithe Collector).
- **Alternate win paths** — sidestepping checkmate as the loss
  condition entirely (Tithe Collector, Beacon).

Every entry names at least two existing-piece synergies and one hard
counter. If the file can't articulate the combo, the piece doesn't
belong in the set.

## Why this matters for a fairy engine

The Chess 2 piece pool is already rich: Goblin kidnaps, Skibidi
brainrots, Bus carries, Monkey jump-chains, Locomotive + Carriage run
neutral trains. With that many actors on the board, the *interactions*
become the interesting part. A solo-power piece (yet another slider,
yet another leaper) adds noise. A combo piece changes which existing
pieces are viable.

The deckbuilder lens treats variant selection as deck construction.
`variants: Vec<VariantId>` becomes a decklist. Pick eight from the
pool, see what synergies emerge, see which counters answer them. This
directory is a draft set designed for that table.

## Constraints (shared with the engine plans)

- All state FEN-serializable. The `(K=V,...)` payload syntax handles
  per-square counters, flags, and countdowns.
- Deterministic. No randomness — "demands a tithe" is a forced binary
  choice, not a coin flip.
- The `Color::Neutral` faction exists; some pieces here mint neutral
  artifacts (vines, miasma, sand).
- Signal substrate (Switch/Junction/Gate/Plate) and terrain
  (Frozen/Brainrot/Block/Track) are reusable primitives — several of
  these pieces extend the terrain layer rather than inventing fresh
  systems.

## The index

| Piece | One-line role | Primary axis |
| --- | --- | --- |
| [Trellis](trellis.md) | Stationary plant, grows vine tiles | Denial / area control |
| [Echo](echo.md) | Captured-piece compels its captor's next move | Tempo / sacrifice trigger |
| [Quartermaster](quartermaster.md) | Lends +1 range to adjacent friendlies | Tempo / amplifier |
| [Lien](lien.md) | Captures lock the square for 3 turns | Denial / countdown |
| [Hourglass](hourglass.md) | Sand counters; capture rewinds plies | Escalation / panic button |
| [Beacon](beacon.md) | One-time rank/file teleport-swap per friendly | Tempo / deployment |
| [Plague Doctor](plague_doctor.md) | Captures leave ability-suppressing miasma | Denial / silencer |
| [Tithe Collector](tithe_collector.md) | Every 3 turns: pay or it promotes | Escalation / alt wincon |

## Reading order

If you only have time for two: **Echo** and **Plague Doctor**. Echo
because it inverts the value of being captured; Plague Doctor because
it's the answer card the ability-stacked builds need. The other six
fill out the meta around them.
