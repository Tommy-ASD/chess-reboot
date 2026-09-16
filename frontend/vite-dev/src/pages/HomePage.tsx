// src/pages/HomePage.tsx
//
// The front door. In order: what Chess 2 is (with a board you can poke),
// the variant gallery, games happening right now, community variants, and
// the piece guide.

import { useMemo, useState } from "preact/hooks";
import { useLocation } from "preact-iso";
import { Board } from "../components/board/Board";
import { PieceSprite } from "../components/board/PieceSprite";
import { VariantCard } from "../components/VariantCard";
import { parseFEN } from "../core/fen";
import type { Coord } from "../core/types";
import { PIECES, pieceForSymbol } from "../content/pieces";
import { OFFICIAL_VARIANTS, describeFen } from "../content/variants";
import { savedVariants, toVariant } from "../content/userVariants";
import { useLobby } from "../online/useLobby";
import type { GameState } from "../api/online";

/// A display-only showcase: every custom mechanic visible at once.
const SHOWCASE_FEN =
  "(P=g(H=0-0))3(C=BRAINROT)1kr/pp1(C=BRAINROT)(P=s(PHASE=2))(P=p,C=BRAINROT)pp/2n1(C=BRAINROT)3/3Mp3/5(C=TORNADO:3)2/2(P=G(H=0-7,P=n))3W1/PPP2PPP/(P=BUS(P=(N,P)))3K2R w - - tr=full p=0";

const CONDITION_NOTES: Record<string, { name: string; note: string }> = {
  BRAINROT: { name: "Brainrot", note: "A Skibidi's brainrot. Pieces caught in it are frozen and can't move." },
  TORNADO: { name: "Tornado", note: "Summoned by a Stormcaller. Anyone who can move onto it must, and whatever lands there is trapped." },
};

export function HomePage() {
  return (
    <>
      <Hero />
      <VariantsSection />
      <LiveSection />
      <CommunitySection />
      <PiecesStrip />
    </>
  );
}

function Hero() {
  const squares = useMemo(() => parseFEN(SHOWCASE_FEN), []);
  const [picked, setPicked] = useState<Coord>({ file: 2, rank: 5 });
  const sq = squares[picked.rank]?.[picked.file];
  const def = sq?.piece ? pieceForSymbol(sq.piece) : undefined;
  const condition = sq?.conditions.map((c) => CONDITION_NOTES[c.split(":")[0]]).find(Boolean);

  return (
    <section class="hero">
      <div class="container hero-grid">
        <div class="hero-copy">
          <p class="eyebrow">Chess, remixed</p>
          <h1 class="hero-title">Chess, with pieces that kidnap, freeze and ride the bus.</h1>
          <p class="hero-lede">
            Play classic chess or one of its stranger variants with a friend, in seconds. No account, no download. Send a link and
            you're playing.
          </p>
          <div class="hero-actions">
            <a class="btn btn-primary btn-lg" href="/play/classic">
              Play a friend
            </a>
            <a class="btn btn-secondary btn-lg" href="#variants">
              Browse variants
            </a>
          </div>
          <p class="hero-note muted">
            New here? <a href="/pieces">Meet the pieces</a> first. Every one has a board to try it on.
          </p>
        </div>

        <div class="hero-showcase">
          <Board squares={squares} selected={picked} onSquareClick={setPicked} label="Showcase board: tap a piece to learn about it" />
          <div class="hero-callout panel" aria-live="polite">
            {def ? (
              <>
                <PieceSprite symbol={sq!.piece!} class="hero-callout-sprite" />
                <div>
                  <strong>{def.name}</strong>
                  <p class="muted">{def.tagline}</p>
                </div>
                <a class="btn btn-ghost btn-sm" href={`/pieces/${def.id}`}>
                  Rules
                </a>
              </>
            ) : condition ? (
              <div>
                <strong>{condition.name}</strong>
                <p class="muted">{condition.note}</p>
              </div>
            ) : (
              <p class="muted">Tap a piece on the board to meet it.</p>
            )}
          </div>
        </div>
      </div>
    </section>
  );
}

function VariantsSection() {
  return (
    <section class="section" id="variants">
      <div class="container">
        <header class="section-head">
          <div>
            <h2 class="section-title">Pick a variant</h2>
            <p class="section-lede">Start with Classic, or jump straight into something new.</p>
          </div>
        </header>
        <div class="variant-grid">
          {OFFICIAL_VARIANTS.map((v) => (
            <VariantCard key={v.id} variant={v} href={`/play/${v.id}`} />
          ))}
        </div>
      </div>
    </section>
  );
}

function LiveSection() {
  const lobby = useLobby();
  const { route } = useLocation();
  const [code, setCode] = useState("");

  return (
    <section class="section section-alt" id="live">
      <div class="container">
        <header class="section-head">
          <div>
            <h2 class="section-title">Happening now</h2>
            <p class="section-lede">Join an open game, watch one in progress, or enter a code a friend sent you.</p>
          </div>
          <form
            class="code-form"
            onSubmit={(e) => {
              e.preventDefault();
              const c = code.trim();
              if (c) route(`/game/${encodeURIComponent(c)}`);
            }}
          >
            <label class="sr-only" for="join-code">
              Game code
            </label>
            <input id="join-code" class="input input-mono" placeholder="Game code" value={code} maxLength={40} onInput={(e) => setCode(e.currentTarget.value)} autoComplete="off" />
            <button class="btn btn-secondary" type="submit" disabled={!code.trim()}>
              Join
            </button>
          </form>
        </header>

        {lobby.status === "offline" ? (
          <div class="notice notice-warn">
            <strong>Can't reach the game server.</strong> The rules engine runs there, so online and same-device games are unavailable until it's back.
          </div>
        ) : (
          <div class="live-grid">
            <GameList
              title="Open games"
              games={lobby.open}
              loading={lobby.status === "loading"}
              empty="Nobody's waiting right now. Pick a variant above and open a game to anyone."
              action="Join"
              hrefFor={(g) => `/game/${g.id}`}
              describe={(g) => `${hostName(g)} is waiting · plays ${g.white ? "White" : "Black"}`}
            />
            <GameList
              title="In progress"
              games={lobby.live}
              loading={lobby.status === "loading"}
              empty="No public games in progress."
              action="Watch"
              hrefFor={(g) => `/game/${g.id}?watch=1`}
              describe={(g) => `${g.white?.name || "?"} vs ${g.black?.name || "?"} · move ${Math.ceil(g.ply / 2) || 1}`}
            />
          </div>
        )}
      </div>
    </section>
  );
}

function hostName(g: GameState): string {
  return (g.white ?? g.black)?.name || "Anonymous";
}

function GameList({ title, games, loading, empty, action, hrefFor, describe }: {
  title: string;
  games: GameState[];
  loading: boolean;
  empty: string;
  action: string;
  hrefFor: (g: GameState) => string;
  describe: (g: GameState) => string;
}) {
  return (
    <div class="panel game-list">
      <h3 class="panel-title">
        {title} {games.length > 0 && <span class="count">{games.length}</span>}
      </h3>
      {loading ? (
        <p class="muted">Loading…</p>
      ) : games.length === 0 ? (
        <p class="muted">{empty}</p>
      ) : (
        <ul class="game-rows">
          {games.map((g) => (
            <li key={g.id} class="game-row">
              <div class="game-row-text">
                <strong>{g.name || describeFen(g.fen)}</strong>
                <span class="muted">{describe(g)}</span>
              </div>
              <a class="btn btn-secondary btn-sm" href={hrefFor(g)}>
                {action}
              </a>
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}

function CommunitySection() {
  const mine = savedVariants.value;
  return (
    <section class="section" id="community">
      <div class="container">
        <header class="section-head">
          <div>
            <h2 class="section-title">Community variants</h2>
            <p class="section-lede">
              Build a position in the Lab, save it as a variant and share the link. Anyone who opens it can play it.
            </p>
          </div>
          <a class="btn btn-secondary" href="/lab">
            Open the Lab
          </a>
        </header>
        {mine.length === 0 ? (
          <div class="empty-state panel">
            <h3 class="panel-title">No community variants on this device yet</h3>
            <p class="muted">
              Variants you save in the Lab show up here. A public gallery is on the way; for now, share your variant's link and
              whoever opens it can play.
            </p>
            <a class="btn btn-primary" href="/lab">
              Make a variant
            </a>
          </div>
        ) : (
          <div class="variant-grid">
            {mine.map((v) => (
              <VariantCard key={v.id} variant={toVariant(v)} href={`/play/${v.id}`} />
            ))}
          </div>
        )}
      </div>
    </section>
  );
}

function PiecesStrip() {
  const featured = PIECES.filter((p) => p.family === "chess2" || p.family === "trains");
  return (
    <section class="section section-alt">
      <div class="container">
        <header class="section-head">
          <div>
            <h2 class="section-title">Meet the pieces</h2>
            <p class="section-lede">Each one has a small board where you can try it before a real game.</p>
          </div>
          <a class="btn btn-secondary" href="/pieces">
            Piece guide
          </a>
        </header>
        <div class="piece-strip">
          {featured.map((p) => (
            <a key={p.id} class="piece-chip" href={`/pieces/${p.id}`}>
              {p.symbols && <PieceSprite symbol={p.symbols.white} />}
              <span>
                <strong>{p.name}</strong>
                <span class="muted">{p.tagline}</span>
              </span>
            </a>
          ))}
        </div>
      </div>
    </section>
  );
}
