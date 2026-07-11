// src/presets.ts
//
// Shared FEN preset catalogue for the play surface (main.ts) and the
// read-only FEN renderer (fen_page.ts). Kept in one place so the two
// pages never drift out of sync.
//
// Presets whose grids are bare (no trailing flag block) lean on the
// engine's parser to fill in default flag fields (stm=w, castling=KQkq,
// ep=-, tr=full, p=0). The curated playable positions carry their flags
// explicitly. The editor's own PRESETS in `editor_page.ts` use the
// canonical full form; all round-trip through the engine identically.

export type FenPreset = { name: string; fen: string };

export const FEN_PRESETS: FenPreset[] = [
  { name: "Empty Board", fen: "8/8/8/8/8/8/8/8" },
  { name: "Standard Chess", fen: "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR" },
  // Curated "default board configurations" — each is a full, playable
  // starting position (both kings, side-to-move has legal moves,
  // status Ongoing), validated against the engine
  // (`cargo run -p engine --example validate_positions`). FENs are the
  // engine's canonical round-trip form. Flags are included where they
  // matter (tr=ply so trains roll every ply; variants=duck_chess to arm
  // the duck rules); the rest round-trip identically to bare grids.
  {
    // Rooks→Goblins (corners), knights→Monkeys, bishops→Skibidi (q-side)
    // + Stormcaller (k-side); mirrored, standard K/Q + pawn walls.
    name: "Fairy Army",
    fen: "(P=g(H=0-0))msqkwm(P=g(H=7-0))/pppppppp/8/8/8/8/PPPPPPPP/(P=G(H=0-7))MSQKWM(P=G(H=7-7)) w - - tr=full p=0",
  },
  {
    // Terrain rampart bisects rank 5 (TURRET flanks, BLOCK/VENT core),
    // leaving b/g-file chokepoints; each side's k-side knight is a Monkey
    // that can vault the wall.
    name: "Rampart Run",
    fen: "rnbqkbmr/pppppppp/8/(T=TURRET)1(T=BLOCK)(T=VENT)(T=VENT)(T=BLOCK)1(T=TURRET)/8/8/PPPPPPPP/RNBQKBMR w - - tr=full p=0",
  },
  {
    // Two phase-2 Skibidis face off on the d-file; each already freezes
    // its four orthogonal (empty) neighbors — pump to phase 3 to expand
    // the Brainrot zone into enemy lines.
    name: "Brainrot Standoff",
    fen: "r2wk2r/ppp2ppp/3(P=s(PHASE=2))4/8/8/3(P=S(PHASE=2))4/PPP2PPP/R2WK2R w KQkq - tr=full p=0",
  },
  {
    // Signal substrate: a wall of 3 closed gates (IDs 1-3) between the
    // armies; knights sit on switches wired to toggle them, a White-only
    // plate on d3 auto-opens gate 1, and junction 9 tallies every pulse.
    name: "Signal Contraption",
    fen: "r2k3r/pp4pp/8/(T=GATE,ID=1,OPEN=0)(T=GATE,ID=2,OPEN=0)(T=GATE,ID=3,OPEN=0)2(T=JUNCTION,ID=9,STATE=0,BRANCHES=(N,E))2/(P=N,T=SWITCH,TARGETS=(1,2,9))5(P=n,T=SWITCH,TARGETS=(2,3,9))1/3(T=PLATE,TARGETS=(1,9),FIRES=W)4/PP4PP/R2K3R w KQkq - tr=full p=0",
  },
  {
    // 10x10 yard: a loco (hauling a knight+pawn) plus two carts loops a
    // 7x4 rail circuit every ply between two full armies framed by BLOCK
    // bumpers. Invincible train — time your center play around it.
    name: "Railyard Roundhouse",
    fen: "(T=BLOCK)rnbqkbnr(T=BLOCK)/1pppppppp1/10/1(P=LOCO(ID=1,H=F,P=(N,P)),T=TRACK,D=E)(T=TRACK,D=E)(T=TRACK,D=E)(T=TRACK,D=E)(T=TRACK,D=E)(T=TRACK,D=E)(T=TRACK,D=S)2/1(P=CART(ID=1,I=2),T=TRACK,D=N)5(T=TRACK,D=S)2/1(P=CART(ID=1,I=1),T=TRACK,D=N)5(T=TRACK,D=S)2/1(T=TRACK,D=N)(T=TRACK,D=E)(T=TRACK,D=E)(T=TRACK,D=E)(T=TRACK,D=E)(T=TRACK,D=E)(T=TRACK,D=N)2/10/1PPPPPPPP1/(T=BLOCK)RNBQKBNR(T=BLOCK) w - - tr=ply p=0",
  },
  {
    // Duck Chess (no check/mate — king capture wins): both knights are
    // Monkeys and the duck starts pre-placed on d5, blocking the center.
    name: "Duck & Monkeys",
    fen: "rmbqkbmr/pppppppp/8/3(DUCK)4/8/8/PPPPPPPP/RMBQKBMR w KQkq - tr=full p=0 variants=duck_chess",
  },
  // Oversized, FULLY SYMMETRIC positions — each is invariant under a
  // vertical flip + colour swap (White's army is the mirror image of
  // Black's, file for file), so the two sides are congruent and neither is
  // favoured. Generated + engine-validated; canonical round-trip form.
  {
    // 16x16 grand classical: rooks/knights/bishops around a Monkey pair,
    // one back rank + a full pawn wall, over a wide open battlefield.
    name: "Colossus 16x16",
    fen: "rnbmbnrkqrnbmbnr/pppppppppppppppp/16/16/16/16/16/16/16/16/16/16/16/16/PPPPPPPPPPPPPPPP/RNBMBNRKQRNBMBNR w - - tr=full p=0",
  },
  {
    // 16x16 fairy showcase: Goblins in all four corners, Stormcallers +
    // Skibidis + Monkeys across the ranks, and a Bus anchoring each flank
    // of the pawn wall.
    name: "Fairy Colosseum",
    fen: "(P=g(H=0-0))mbrwnskqsnwrbm(P=g(H=15-0))/(P=bus)pppppppppppppp(P=bus)/16/16/16/16/16/16/16/16/16/16/16/16/(P=BUS)PPPPPPPPPPPPPP(P=BUS)/(P=G(H=0-15))MBRWNSKQSNWRBM(P=G(H=15-15)) w - - tr=full p=0",
  },
  {
    // 17x17 living fortress. A central wall (self-mirroring middle row) of
    // Turret towers + a BLOCK curtain pierced by three GATE sally-ports
    // (outer two barred, center open), wired to SWITCH tiles either side
    // that toggle all three. Manned by a forward garrison — Monkeys that
    // vault the wall, Skibidis barring the closed gates with brainrot, a
    // Stormcaller at the open port — with Goblins, Buses, Stormcallers and
    // the rest of the court spread across the rear ranks. Fight for the
    // gates.
    name: "Great Wall 17x17",
    fen: "(P=g(H=0-0))1r1(P=bus)1bqk1b1(P=bus)1r1(P=g(H=16-0))/1n1m1w2n2w1m1n1/1ppppppppppppppp1/17/17/17/6(T=SWITCH,TARGETS=(1,2,3))3(T=SWITCH,TARGETS=(1,2,3))6/2m1s3w3s1m2/(T=TURRET)(T=BLOCK)(T=BLOCK)(T=BLOCK)(T=GATE,ID=1,OPEN=0)(T=BLOCK)(T=BLOCK)(T=BLOCK)(T=GATE,ID=2,OPEN=1)(T=BLOCK)(T=BLOCK)(T=BLOCK)(T=GATE,ID=3,OPEN=0)(T=BLOCK)(T=BLOCK)(T=BLOCK)(T=TURRET)/2M1S3W3S1M2/6(T=SWITCH,TARGETS=(1,2,3))3(T=SWITCH,TARGETS=(1,2,3))6/17/17/17/1PPPPPPPPPPPPPPP1/1N1M1W2N2W1M1N1/(P=G(H=0-16))1R1(P=BUS)1BQK1B1(P=BUS)1R1(P=G(H=16-16)) w - - tr=full p=0",
  },
  {
    // 24x24 — even larger. Fairy+standard back rank (Goblins, Stormcallers,
    // Skibidis, Monkeys) + a 24-pawn wall on a vast open field.
    name: "Mega Field 24x24",
    fen: "rnbmbn(P=g(H=6-0))wbnskqsnbw(P=g(H=17-0))nbmbnr/pppppppppppppppppppppppp/24/24/24/24/24/24/24/24/24/24/24/24/24/24/24/24/24/24/24/24/PPPPPPPPPPPPPPPPPPPPPPPP/RNBMBN(P=G(H=6-23))WBNSKQSNBW(P=G(H=17-23))NBMBNR w - - tr=full p=0",
  },
  { name: "Goblin Test", fen: "(P=g(H=0-0))nbqkbn(P=g(H=7-0))/pppppppp/8/8/8/8/PPPPPPPP/(P=G(H=0-7))NBQKBN(P=G(H=7-7))" },
  { name: "Vent Test", fen: "(T=VENT)7/8/8/8/8/8/8/8" },
  { name: "Frozen Test", fen: "(C=FROZEN)7/8/8/8/8/8/8/8" },
];
