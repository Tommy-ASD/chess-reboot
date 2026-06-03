//! Online multiplayer: the in-memory game store, the player-identity seam,
//! and the engine-backed move/turn logic.
//!
//! The engine stays the legality authority — every move is applied by
//! `Board::make_move` on the game's FEN. This module only adds the session
//! state (who is white/black, the live FEN + history + status), colour/turn
//! enforcement, and a per-game broadcast channel that the Phase-2 WebSocket
//! endpoint forwards to connected clients. **No database**: the store lives
//! in the process; it is the single seam where persistence would later
//! attach. Likewise [`AuthPlayer`] is the single seam where a real account
//! system would later validate identity.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use axum::extract::FromRequestParts;
use axum::http::StatusCode;
use axum::http::request::Parts;
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;
use uuid::Uuid;

use engine::board::{
    GameMove, GameStatus, MoveError,
    fen::{board_to_fen, fen_to_board},
};
use engine::pieces::Color;

/// Standard chess starting position (white to move, full castling).
pub const STANDARD_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq -";

pub type GameId = Uuid;
pub type PlayerId = Uuid;

/// A seated player. `name` is a display label only — identity is `id`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PlayerSlot {
    pub id: PlayerId,
    pub name: String,
}

/// Why a game ended. `None` on a live `Game` means in progress.
#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(tag = "kind")]
pub enum GameResult {
    /// Decided on the board (checkmate / king-capture / brainrot win).
    Decisive { winner: Color },
    /// A player resigned; `winner` is the other side.
    Resignation { winner: Color },
    /// Drawn (stalemate).
    Draw,
}

fn result_from_status(status: &GameStatus) -> Option<GameResult> {
    match status {
        GameStatus::Checkmate { winner }
        | GameStatus::Win { winner }
        | GameStatus::BrainrotWin { winner }
        | GameStatus::BrainrotLockout { winner } => Some(GameResult::Decisive { winner: *winner }),
        GameStatus::Stalemate => Some(GameResult::Draw),
        GameStatus::Ongoing | GameStatus::Check { .. } => None,
    }
}

/// Canonical server-side game. Lives in the store; never serialized
/// directly — clients receive [`GameState`] snapshots.
#[derive(Debug, Clone)]
pub struct Game {
    pub id: GameId,
    pub code: String,
    pub name: String,
    pub public: bool,
    pub white: Option<PlayerSlot>,
    pub black: Option<PlayerSlot>,
    pub fen: String,
    pub side_to_move: Color,
    pub history: Vec<GameMove>,
    pub status: GameStatus,
    pub result: Option<GameResult>,
}

impl Game {
    /// The colour `player` is seated as in this game, if any.
    fn color_of(&self, player: PlayerId) -> Option<Color> {
        if self.white.as_ref().is_some_and(|p| p.id == player) {
            Some(Color::White)
        } else if self.black.as_ref().is_some_and(|p| p.id == player) {
            Some(Color::Black)
        } else {
            None
        }
    }

    /// True once both seats are filled.
    fn is_full(&self) -> bool {
        self.white.is_some() && self.black.is_some()
    }

    /// A viewer-facing snapshot. `viewer` (if seated) gets `your_color`;
    /// pass `None` for spectators and broadcast pushes (each client already
    /// knows its own colour from create/join).
    pub fn snapshot(&self, viewer: Option<PlayerId>) -> GameState {
        GameState {
            id: self.id,
            code: self.code.clone(),
            name: self.name.clone(),
            public: self.public,
            white: self.white.clone(),
            black: self.black.clone(),
            fen: self.fen.clone(),
            side_to_move: self.side_to_move,
            ply: self.history.len(),
            status: self.status.clone(),
            result: self.result.clone(),
            your_color: viewer.and_then(|v| self.color_of(v)),
        }
    }
}

/// Serializable game snapshot sent to clients (REST responses + WS pushes).
#[derive(Debug, Clone, Serialize)]
pub struct GameState {
    pub id: GameId,
    pub code: String,
    pub name: String,
    pub public: bool,
    pub white: Option<PlayerSlot>,
    pub black: Option<PlayerSlot>,
    pub fen: String,
    pub side_to_move: Color,
    pub ply: usize,
    pub status: GameStatus,
    pub result: Option<GameResult>,
    /// The requesting player's colour, if seated. `None` for spectators /
    /// broadcast pushes.
    pub your_color: Option<Color>,
}

/// Body for `POST /games`.
#[derive(Debug, Deserialize)]
pub struct CreateGame {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub public: bool,
    /// Optional non-standard start (e.g. a Duck-Chess / variant FEN).
    #[serde(default)]
    pub starting_fen: Option<String>,
    /// Colour the host wants; defaults to White.
    #[serde(default)]
    pub color: Option<Color>,
}

/// Failure modes for the game actions. The HTTP layer maps these to codes.
#[derive(Debug)]
pub enum GameActionError {
    /// No game for the given id/code.
    NotFound,
    /// The supplied starting FEN didn't parse.
    BadFen(String),
    /// The caller isn't seated in this game.
    NotSeated,
    /// Both seats are already taken.
    Full,
    /// Not the caller's turn (wrong colour, or not their move).
    NotYourTurn,
    /// The game has already ended.
    Over,
    /// The engine rejected the move (turn already validated).
    Move(MoveError),
    /// The stored FEN failed to parse — should never happen.
    Internal(String),
}

/// The player identity attached to every authenticated request.
///
/// **Account seam.** Today this trusts a client-generated UUID in the
/// `X-Player-Id` header (+ an optional display name in `X-Player-Name`).
/// This extractor is the *single* place a real account system would later
/// validate a session token / JWT and resolve it to a stable account id —
/// every handler already takes `AuthPlayer`, so no call site changes when
/// that lands.
#[derive(Debug, Clone)]
pub struct AuthPlayer {
    pub id: PlayerId,
    pub name: String,
}

impl<S: Send + Sync> FromRequestParts<S> for AuthPlayer {
    type Rejection = (StatusCode, String);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let id = parts
            .headers
            .get("x-player-id")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| Uuid::parse_str(s.trim()).ok())
            .ok_or((
                StatusCode::UNAUTHORIZED,
                "missing or invalid X-Player-Id header".to_string(),
            ))?;
        let name = parts
            .headers
            .get("x-player-name")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "Anonymous".to_string());
        Ok(AuthPlayer { id, name })
    }
}

struct GameEntry {
    game: Game,
    /// Per-game live-update channel. Phase 2's `/ws/games/{id}` subscribes;
    /// store mutations push a fresh snapshot. Sends with no subscribers are
    /// harmless (`Err` ignored).
    tx: broadcast::Sender<GameState>,
}

#[derive(Default)]
struct Store {
    games: HashMap<GameId, GameEntry>,
    /// Uppercase join-code → game id, for join-by-code / shareable links.
    codes: HashMap<String, GameId>,
}

/// Shared app state: the in-memory game store + a lobby-changed channel.
#[derive(Clone)]
pub struct AppState {
    inner: Arc<Mutex<Store>>,
    /// Pinged whenever the public lobby changes (game created / joined /
    /// ended). Phase 2's `/ws/lobby` forwards it so clients re-fetch the
    /// list. Held here (not per-game) because it's global.
    lobby_tx: broadcast::Sender<()>,
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

impl AppState {
    pub fn new() -> Self {
        let (lobby_tx, _) = broadcast::channel(64);
        AppState {
            inner: Arc::new(Mutex::new(Store::default())),
            lobby_tx,
        }
    }

    /// Recover from a poisoned lock rather than propagate: a panic mid-
    /// mutation shouldn't take the whole server down (hobby-grade; revisit
    /// alongside persistence).
    fn lock(&self) -> std::sync::MutexGuard<'_, Store> {
        self.inner.lock().unwrap_or_else(|e| e.into_inner())
    }

    /// Resolve a path token (a game UUID *or* a join code) to a game id.
    fn resolve(store: &Store, key: &str) -> Option<GameId> {
        if let Ok(id) = Uuid::parse_str(key.trim())
            && store.games.contains_key(&id)
        {
            return Some(id);
        }
        store.codes.get(&key.trim().to_uppercase()).copied()
    }

    /// A fresh, unused 6-char join code (uppercase hex from a v4 UUID's
    /// randomness — ~16M space, collision-checked).
    fn fresh_code(store: &Store) -> String {
        loop {
            let code = Uuid::new_v4().simple().to_string()[..6].to_uppercase();
            if !store.codes.contains_key(&code) {
                return code;
            }
        }
    }

    pub fn create(&self, host: &AuthPlayer, req: CreateGame) -> Result<GameState, GameActionError> {
        let fen = req
            .starting_fen
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| STANDARD_FEN.to_string());
        let board = fen_to_board(&fen).map_err(|e| GameActionError::BadFen(e.to_string()))?;
        let side_to_move = board.flags.side_to_move;
        let status = board.status();

        let host_slot = PlayerSlot {
            id: host.id,
            name: host.name.clone(),
        };
        let (white, black) = match req.color {
            Some(Color::Black) => (None, Some(host_slot)),
            _ => (Some(host_slot), None),
        };

        let id = Uuid::new_v4();
        let mut store = self.lock();
        let code = Self::fresh_code(&store);
        // Per-game live-update buffer. Generous so a briefly-slow client
        // receives intermediate snapshots rather than a `Lagged` skip; a
        // client that still falls behind recovers to the latest state on its
        // next recv (see the WS loop's `Lagged` arm), so its board is never
        // left wrong — only intermediate frames are dropped.
        let (tx, _) = broadcast::channel(128);
        let game = Game {
            id,
            code: code.clone(),
            name: req
                .name
                .filter(|n| !n.trim().is_empty())
                .unwrap_or_else(|| format!("{}'s game", host.name)),
            public: req.public,
            white,
            black,
            fen,
            side_to_move,
            history: Vec::new(),
            status,
            result: None,
        };
        let snap = game.snapshot(Some(host.id));
        store.codes.insert(code, id);
        store.games.insert(id, GameEntry { game, tx });
        drop(store);
        let _ = self.lobby_tx.send(());
        Ok(snap)
    }

    /// Public, joinable (not full, not over) games — the lobby list.
    pub fn list_public(&self) -> Vec<GameState> {
        let store = self.lock();
        store
            .games
            .values()
            .filter(|e| e.game.public && e.game.result.is_none() && !e.game.is_full())
            .map(|e| e.game.snapshot(None))
            .collect()
    }

    pub fn get(&self, key: &str, viewer: Option<PlayerId>) -> Option<GameState> {
        let store = self.lock();
        let id = Self::resolve(&store, key)?;
        store.games.get(&id).map(|e| e.game.snapshot(viewer))
    }

    /// Subscribe to a game's live updates: the current snapshot plus a
    /// receiver for every subsequent change. `None` if the game is unknown.
    /// Used by the `/ws/games/{id}` push loop.
    pub fn subscribe_game(
        &self,
        key: &str,
    ) -> Option<(GameState, broadcast::Receiver<GameState>)> {
        let store = self.lock();
        let id = Self::resolve(&store, key)?;
        let entry = store.games.get(&id)?;
        Some((entry.game.snapshot(None), entry.tx.subscribe()))
    }

    /// Subscribe to lobby-changed pings; the `/ws/lobby` loop re-lists on
    /// each one.
    pub fn subscribe_lobby(&self) -> broadcast::Receiver<()> {
        self.lobby_tx.subscribe()
    }

    pub fn join(&self, key: &str, player: &AuthPlayer) -> Result<GameState, GameActionError> {
        let mut store = self.lock();
        let id = Self::resolve(&store, key).ok_or(GameActionError::NotFound)?;
        let entry = store.games.get_mut(&id).ok_or(GameActionError::NotFound)?;
        // Idempotent: already seated → hand back your snapshot (even after
        // the game has ended — a seated player may still re-fetch it).
        if entry.game.color_of(player.id).is_some() {
            return Ok(entry.game.snapshot(Some(player.id)));
        }
        // A new player can't take a seat in a finished game — mirrors the
        // game-over guard in `apply_move` / `resign`.
        if entry.game.result.is_some() {
            return Err(GameActionError::Over);
        }
        if entry.game.is_full() {
            return Err(GameActionError::Full);
        }
        let slot = PlayerSlot {
            id: player.id,
            name: player.name.clone(),
        };
        if entry.game.white.is_none() {
            entry.game.white = Some(slot);
        } else {
            entry.game.black = Some(slot);
        }
        let your = entry.game.snapshot(Some(player.id));
        let _ = entry.tx.send(entry.game.snapshot(None));
        drop(store);
        let _ = self.lobby_tx.send(());
        Ok(your)
    }

    pub fn apply_move(
        &self,
        key: &str,
        player: &AuthPlayer,
        mv: GameMove,
    ) -> Result<GameState, GameActionError> {
        let mut store = self.lock();
        let id = Self::resolve(&store, key).ok_or(GameActionError::NotFound)?;
        let entry = store.games.get_mut(&id).ok_or(GameActionError::NotFound)?;
        // Mutate inside a scope so the `&mut game` borrow ends before we
        // touch `entry.tx`.
        let (your, bcast) = {
            let g = &mut entry.game;
            if g.result.is_some() {
                return Err(GameActionError::Over);
            }
            let my_color = g.color_of(player.id).ok_or(GameActionError::NotSeated)?;
            if my_color != g.side_to_move {
                return Err(GameActionError::NotYourTurn);
            }
            let mut board =
                fen_to_board(&g.fen).map_err(|e| GameActionError::Internal(e.to_string()))?;
            board.make_move(mv.clone()).map_err(GameActionError::Move)?;
            g.fen = board_to_fen(&board);
            g.side_to_move = board.flags.side_to_move;
            g.status = board.status();
            g.history.push(mv);
            if let Some(res) = result_from_status(&g.status) {
                g.result = Some(res);
            }
            (g.snapshot(Some(player.id)), g.snapshot(None))
        };
        let _ = entry.tx.send(bcast);
        let ended = your.result.is_some();
        drop(store);
        if ended {
            let _ = self.lobby_tx.send(());
        }
        Ok(your)
    }

    pub fn resign(&self, key: &str, player: &AuthPlayer) -> Result<GameState, GameActionError> {
        let mut store = self.lock();
        let id = Self::resolve(&store, key).ok_or(GameActionError::NotFound)?;
        let entry = store.games.get_mut(&id).ok_or(GameActionError::NotFound)?;
        let (your, bcast) = {
            let g = &mut entry.game;
            if g.result.is_some() {
                return Err(GameActionError::Over);
            }
            let my_color = g.color_of(player.id).ok_or(GameActionError::NotSeated)?;
            g.result = Some(GameResult::Resignation {
                winner: my_color.opposite(),
            });
            (g.snapshot(Some(player.id)), g.snapshot(None))
        };
        let _ = entry.tx.send(bcast);
        drop(store);
        let _ = self.lobby_tx.send(());
        Ok(your)
    }
}
