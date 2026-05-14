# meme_brain

Pieces with names borrowed from TikTok and Gen-Z internet meme
culture. The aesthetic is loud; the mechanics are not. Each piece
takes the meme as a *constraint* — what would a chess piece called
"Mewing" actually have to do to earn the name — and answers it with
a mechanic that pulls its weight on the board.

The Skibidi piece (already in-engine) is the precedent: a four-phase
brainrot stunner whose absurd branding doubles as a mnemonic for a
genuinely novel area-of-effect mechanic. These ideas extend that
pattern. The names are unhinged. The rules tables are not.

## Why the meme branding earns its keep

1. **Mnemonic load.** "Mogger inhibits weaker enemies in king-radius"
   is faster to teach than "Inhibitor radius-1 conditional on
   material-value comparison." The meme carries the rule.
2. **Aesthetic coherence.** The engine already commits to absurdism
   (Skibidi, Goblin, Costco trains). A category that goes harder
   in this direction holds the line.
3. **Search-space pressure.** Asking "what would Sigma do in chess"
   forces a non-standard constraint (refuses-to-move-near-friends)
   that a more sober naming process would never propose. The meme
   is a generator, not just a label.

The MECHANIC section of each file is dry-precise — FEN-serializable
state, deterministic resolution, no hidden info, no randomness. The
INSPIRATION section says what the meme is and why it maps onto the
rule. Read both; the mechanics stand alone.

## Index

| File | Piece | One-line |
|------|-------|----------|
| [sigma.md](sigma.md)                       | Sigma             | Refuses to move adjacent to friends; isolation grants permanent range. |
| [ohio.md](ohio.md)                         | Ohio (tile)       | Designated square permutes the moveset of any piece that ends a turn on it. |
| [mewing.md](mewing.md)                     | Mewing            | Pawn that locks in by standing still; promotes to a 3-square king with anti-promotion aura. |
| [gooner.md](gooner.md)                     | Gooner            | Locks onto nearest enemy in line-of-sight, shoves through friendlies toward it. |
| [costco_guy.md](costco_guy.md)             | Costco Guy        | Inverted Bus — only carries passengers of equal-or-greater material value. |
| [npc.md](npc.md)                           | NPC               | Auto-advances forward each turn; controller cannot command it. |
| [italian_brainrot.md](italian_brainrot.md) | Italian Brainrot  | 3-step zigzag mover; emits a board-wide Switch-toggle signal on capture. |
| [mogger.md](mogger.md)                     | Mogger            | Passive — any enemy of strictly lower value in king-radius is frozen for one turn. |

## Common dependencies

Most pieces in this category lean on three engine systems already
in flight:

- **Variable per-piece FEN payloads** — the parenthesized `(K=V,...)`
  syntax established for Skibidi phase, Goblin captives, Bus
  passengers.
- **The signal substrate** (plan 08) — Italian Brainrot's
  `tralala` pulse and Ohio's permutation hook both speak in
  Switch/SignalId terms.
- **Square conditions** (Frozen, Brainrot) — Mogger's inhibit, the
  Mewing aura, and Ohio's permutation all reuse the conditions
  pipeline that Skibidi paved.

No piece in this category needs hidden state, RNG, or simultaneous
move resolution. All eight are clean fits for the deterministic
engine.
