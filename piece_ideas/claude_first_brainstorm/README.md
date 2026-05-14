# Claude First Brainstorm — fairy piece pitches

Opening pitch for new fairy pieces. Eleven ideas, grouped not by
theme but by which engine system each one hooks into. The through-line:
**mechanically novel pieces that lean on infrastructure the engine
already has**, rather than one-off pieces that require their own
private subsystem.

## Design philosophy

Each piece justifies itself against three criteria:

1. **Reuses an existing substrate.** Signal wiring, train tracks, the
   `Bus.passengers` carrier shape, `Color::Neutral`, the
   square-conditions stack — these already exist. A piece that
   reuses one of them is cheap to land and forces the substrate to
   prove its generality.
2. **Has a clear chess-novelty beat.** Not "knight but with a hat" —
   a mechanical property that changes how the position is read.
3. **State is FEN-serializable and deterministic.** No hidden RNG, no
   unobservable accumulators. If a piece carries memory, the FEN
   shows that memory.

## Categories

The eleven pieces split into four loose buckets:

**Terrain-makers** paint new squares onto the board mid-game,
turning king safety and pawn cover into a moving target.

- [architect.md](architect.md) — paints `Block` walls.
- [engineer.md](engineer.md) — paints `Track` rails.

**Signal-system actors** interact with the switch/gate/plate wiring
established in plan 08.

- [conductor.md](conductor.md) — throws switches remotely.

**Accumulators** carry state across moves — the `Bus.passengers`
pattern generalized to other resources.

- [vampire.md](vampire.md) — absorbs movesets from captures.
- [reanimator.md](reanimator.md) — banks dead friendlies for resurrection.

**Geometry-benders** change how movement reads off the board.

- [magnet.md](magnet.md) — passively pulls adjacent enemies.
- [mirror.md](mirror.md) — replays the opponent's last move's shape.
- [dancer.md](dancer.md) — Queen restricted to squares adjacent to friends.

**Threat-shape oddities** that don't fit a single bucket.

- [sniper.md](sniper.md) — moves like a King, captures at range without moving.
- [bomb.md](bomb.md) — Knight-mover, explodes on capture.
- [quantum.md](quantum.md) — one piece, two coordinates, collapses on observation.

## How to read these

Each file follows the same skeleton: pitch, inspiration, mechanic,
why it's interesting, scenarios, where it shines, where it's awkward,
engine dependencies, new features required, FEN encoding, open
questions. The "engine dependencies" and "new features required"
sections are the operational heart — they say what already exists
versus what would need building.

The pitches deliberately vary in cost. `architect`, `engineer`, and
`dancer` are nearly free given existing infrastructure. `quantum`,
`vampire`, and `reanimator` each need a meaningful new field. The
goal of this brainstorm is breadth, not pre-selection — future Claude
picks which ones survive to a real plan.

## Index

| File | One-liner | Cost |
|---|---|---|
| [architect.md](architect.md) | King-mover; paints Block tile in lieu of moving. | Cheap |
| [engineer.md](engineer.md) | King-mover; paints Track tile in lieu of moving. | Cheap |
| [conductor.md](conductor.md) | King-mover; can fire any Switch from anywhere. | Small |
| [vampire.md](vampire.md) | King-mover; absorbs captured pieces' movesets. | Medium |
| [reanimator.md](reanimator.md) | Banks dead friendlies; resurrects on cooldown. | Medium |
| [magnet.md](magnet.md) | Stationary; pulls adjacent enemies each opp turn. | Small |
| [mirror.md](mirror.md) | King OR replay opponent's last move shape. | Medium |
| [dancer.md](dancer.md) | Queen, but only to squares adjacent to a friend. | Cheap |
| [sniper.md](sniper.md) | King-mover; captures at line-of-sight without moving. | Small |
| [bomb.md](bomb.md) | Knight; on-capture removes adjacent 8 squares. | Small |
| [quantum.md](quantum.md) | One piece, two coords, collapses on adjacency. | Heavy |
