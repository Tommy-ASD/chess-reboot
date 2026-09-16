// src/pages/OnlineGamePage.tsx
//
// An online game at `/game/:id` (id or share code). Opening the link:
//
//   * already seated        → straight into the game
//   * a seat is free        → an invitation card (set your name, then join),
//                             unless `?watch` asks to spectate
//   * full / finished       → spectate
//
// While seated with no opponent yet, the board shows the invite link.

import { useEffect, useMemo, useState } from "preact/hooks";
import { useLocation, useRoute } from "preact-iso";
import { Board } from "../components/board/Board";
import { useBoardPlay } from "../components/board/useBoardPlay";
import { MoveList, PiecePanel, PlayerBar, PromotionPicker, StatusBanner } from "../components/game/GameParts";
import { CopyField } from "../components/ui/Controls";
import { displayName, showCoordinates, showMoveHints } from "../app/prefs";
import * as online from "../api/online";
import type { GameState } from "../api/online";
import { parseFEN } from "../core/fen";
import { describeResult, describeStatus } from "../core/status";
import type { Color, Orientation } from "../core/types";
import { describeFen } from "../content/variants";
import { openOnlineGame, type OnlineGame } from "../online/session";
import { showError } from "../ui/toast";

type Phase =
  | { kind: "loading" }
  | { kind: "error"; message: string }
  | { kind: "invite"; state: GameState }
  | { kind: "game"; state: GameState };

export function OnlineGamePage() {
  const { params, query } = useRoute();
  const id = params.id;
  const watch = "watch" in query;
  const [phase, setPhase] = useState<Phase>({ kind: "loading" });

  useEffect(() => {
    let alive = true;
    setPhase({ kind: "loading" });
    online
      .getGame(id)
      .then((state) => {
        if (!alive) return;
        const seatOpen = !state.result && (!state.white || !state.black);
        setPhase(state.your_color || watch || !seatOpen ? { kind: "game", state } : { kind: "invite", state });
      })
      .catch((err: Error) => alive && setPhase({ kind: "error", message: err.message }));
    return () => {
      alive = false;
    };
  }, [id, watch]);

  switch (phase.kind) {
    case "loading":
      return (
        <div class="container page">
          <p class="muted">Loading game…</p>
        </div>
      );
    case "error":
      return <GameError message={phase.message} />;
    case "invite":
      return <InviteCard state={phase.state} onJoined={(state) => setPhase({ kind: "game", state })} onWatch={() => setPhase({ kind: "game", state: phase.state })} />;
    case "game":
      return <LiveGame key={phase.state.id} initial={phase.state} />;
  }
}

function GameError({ message }: { message: string }) {
  const notFound = /not.?found/i.test(message);
  return (
    <div class="container page narrow">
      <div class="panel empty-state">
        <h1 class="page-title">{notFound ? "Game not found" : "Couldn't open this game"}</h1>
        <p class="muted">
          {notFound
            ? "The link or code may be mistyped, or the game has ended and the server has restarted since. Games aren't kept forever yet."
            : message}
        </p>
        <a class="btn btn-primary" href="/">
          Back to variants
        </a>
      </div>
    </div>
  );
}

function InviteCard({ state, onJoined, onWatch }: { state: GameState; onJoined: (s: GameState) => void; onWatch: () => void }) {
  const [joining, setJoining] = useState(false);
  const host = state.white ?? state.black;
  const yourColor: Color = state.white ? "Black" : "White";
  const squares = useMemo(() => parseFEN(state.fen), [state.fen]);
  const variantName = state.name || describeFen(state.fen);

  const join = async () => {
    setJoining(true);
    try {
      onJoined(await online.joinGame(state.id));
    } catch (err) {
      showError(err);
      setJoining(false);
    }
  };

  return (
    <div class="container page">
      <div class="invite-layout">
        <div class="setup-board">
          <Board squares={squares} orientation={yourColor === "Black" ? "black" : "white"} label="Game position" />
        </div>
        <div class="panel invite-card">
          <p class="eyebrow">You're invited</p>
          <h1 class="page-title">
            {host?.name || "Someone"} wants to play {variantName}
          </h1>
          <p class="page-lede">
            You'll play <strong>{yourColor}</strong>. {state.ply > 0 ? `The game is already at move ${state.ply}.` : ""}
          </p>
          <label class="field">
            <span class="field-label">Your name</span>
            <input
              class="input"
              value={displayName.value}
              maxLength={24}
              placeholder="Anonymous"
              autoComplete="nickname"
              onInput={(e) => (displayName.value = e.currentTarget.value)}
            />
          </label>
          <div class="button-row">
            <button type="button" class="btn btn-primary btn-lg" disabled={joining} onClick={join}>
              {joining ? "Joining…" : `Join as ${yourColor}`}
            </button>
            <button type="button" class="btn btn-ghost" onClick={onWatch}>
              Just watch
            </button>
          </div>
          <p class="muted">
            New to {variantName}? <a href="/pieces" target="_blank" rel="noopener">Read the piece guide ↗</a>
          </p>
        </div>
      </div>
    </div>
  );
}

function LiveGame({ initial }: { initial: GameState }) {
  const { route } = useLocation();
  const session = useMemo<OnlineGame>(() => openOnlineGame(initial), [initial.id]);
  useEffect(() => () => session.close(), [session]);

  const s = session.state.value;
  const my = session.myColor;
  const canMove = session.isMyTurn.value;
  const play = useBoardPlay({ fen: s.fen, canMove, submit: session.move });
  const [flipped, setFlipped] = useState(false);

  const base: Orientation = my === "Black" ? "black" : "white";
  const orientation: Orientation = flipped ? (base === "white" ? "black" : "white") : base;
  const topColor: Color = orientation === "white" ? "Black" : "White";
  const bottomColor: Color = topColor === "White" ? "Black" : "White";
  const seat = (c: Color) => (c === "White" ? s.white : s.black);
  const winner = s.result && s.result.kind !== "Draw" ? s.result.winner : null;
  const statusText = s.result ? describeResult(s.result, s.status) : describeStatus(s.status);
  const opponentMissing = my !== null && !seat(my === "White" ? "Black" : "White");
  const over = Boolean(s.result);
  const rows = play.squares?.length ?? 8;
  const link = `${window.location.origin}/game/${s.id}`;
  const variantName = s.name || describeFen(s.fen);

  const act = (fn: () => Promise<unknown>) => () => {
    fn().catch(showError);
  };

  const bar = (c: Color) => (
    <PlayerBar color={c} name={seat(c)?.name || (seat(c) ? "Anonymous" : "Waiting for a player…")} toMove={!over && s.side_to_move === c && Boolean(seat(c))} isYou={my === c} winner={winner === c}>
      {!over && s.draw_offer === c && <span class="chip chip-soft">offers a draw</span>}
    </PlayerBar>
  );

  if (!play.squares) return <GameError message="This game's position couldn't be read." />;

  return (
    <div class="container page game-page">
      <div class="game-layout">
        <div class="game-board-col">
          {bar(topColor)}
          <Board
            squares={play.squares}
            orientation={orientation}
            selected={play.selected}
            targets={showMoveHints.value ? play.targets : []}
            lastMove={play.lastMove}
            emptyTargets={play.duckPlacing && canMove}
            coords={showCoordinates.value}
            onSquareClick={play.onSquareClick}
            label={`${variantName} board`}
            class="game-board"
          >
            {opponentMissing && !over && (
              <div class="board-overlay">
                <div class="overlay-card invite-overlay">
                  <h2 class="panel-title">Waiting for your opponent</h2>
                  <p class="muted">Send this link to a friend. The game starts as soon as they open it.</p>
                  <CopyField label="Invite link" value={link} />
                  <p class="muted small">
                    Or they can enter code <strong class="mono">{s.code}</strong> on the home page.
                    {s.public ? " It's also listed under “Happening now”." : ""}
                  </p>
                </div>
              </div>
            )}
            {play.promotion && play.selectedPiece && (
              <PromotionPicker moves={play.promotion} pawn={play.selectedPiece} onPick={play.play} onCancel={play.cancelPromotion} />
            )}
          </Board>
          {bar(bottomColor)}
        </div>

        <aside class="game-side">
          <header class="game-side-head">
            <div>
              <p class="eyebrow">
                <ConnectionDot state={session.connection.value} /> {my ? `Online · you play ${my}` : "Spectating"}
              </p>
              <h1 class="game-title">{variantName}</h1>
            </div>
            <button type="button" class="btn btn-ghost btn-sm" onClick={() => setFlipped(!flipped)}>
              Flip board
            </button>
          </header>

          <StatusBanner text={statusText} />
          {!over && my && !opponentMissing && (
            <p class={`turn-callout${canMove ? " is-mine" : ""}`}>{canMove ? (play.duckPlacing ? "Your move: place the duck" : "Your move") : "Waiting for your opponent to move…"}</p>
          )}
          <PiecePanel play={play} canMove={canMove} rows={rows} />

          {my && (
            <div class="button-row">
              {!over && s.draw_offer !== null && s.draw_offer !== my ? (
                <>
                  <button type="button" class="btn btn-primary" onClick={act(session.offerDraw)}>
                    Accept draw
                  </button>
                  <button type="button" class="btn btn-secondary" onClick={act(session.declineDraw)}>
                    Decline
                  </button>
                </>
              ) : (
                !over &&
                !opponentMissing && (
                  <button type="button" class="btn btn-secondary" disabled={s.draw_offer === my} onClick={act(session.offerDraw)}>
                    {s.draw_offer === my ? "Draw offered" : "Offer draw"}
                  </button>
                )
              )}
              {!over && (
                <button
                  type="button"
                  class="btn btn-danger-ghost"
                  onClick={() => {
                    if (confirm("Resign this game?")) act(session.resign)();
                  }}
                >
                  Resign
                </button>
              )}
              {over && !s.rematch && (
                <button type="button" class="btn btn-primary" onClick={act(async () => route(`/game/${(await session.rematch()).id}`))}>
                  Rematch
                </button>
              )}
            </div>
          )}
          {s.rematch && (
            <a class="btn btn-primary btn-block" href={`/game/${s.rematch}`}>
              Go to the rematch →
            </a>
          )}
          {over && (
            <a class="btn btn-ghost btn-block" href="/">
              Back to variants
            </a>
          )}

          <MoveList moves={session.history.value} />

          {!opponentMissing && my && !over && (
            <details class="panel share-details">
              <summary>Share this game</summary>
              <CopyField label="Link (opens as spectator once both seats are taken)" value={link} />
            </details>
          )}
        </aside>
      </div>
    </div>
  );
}

function ConnectionDot({ state }: { state: OnlineGame["connection"]["value"] }) {
  const label = state === "live" ? "Connected" : state === "reconnecting" ? "Reconnecting" : "Connecting";
  return <span class={`conn-dot conn-${state}`} title={label} aria-label={label} role="img" />;
}
