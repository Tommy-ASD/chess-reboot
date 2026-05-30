use axum::{
    Json, Router,
    body::Bytes,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
};
use http::Method;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use tower_http::cors::CorsLayer;

use engine::board::{
    Board, Coord, GameMove, GameStatus, MoveError,
    fen::{board_to_fen, fen_to_board},
};
use engine::pieces::Color;

#[derive(Debug, Deserialize)]
pub struct GetMovesRequest {
    pub board_fen: String,
    pub from: Coord,
}

#[derive(Debug, Serialize)]
pub struct GetMovesResponse {
    pub moves: Vec<GameMove>,
}

/// Legal moves for the piece at `from`. Returns king-safety-filtered
/// moves (`legal_moves`), not raw geometry (`get_moves`) — so every move
/// the client is offered here is one `/board/new_state` will accept.
#[axum::debug_handler]
async fn get_moves_handler(Json(req): Json<GetMovesRequest>) -> Json<GetMovesResponse> {
    let board = fen_to_board(&req.board_fen);
    let moves = board.legal_moves(&req.from);
    Json(GetMovesResponse { moves })
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetNewBoardStateRequest {
    pub board_fen: String,
    pub game_move: GameMove,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetNewBoardStateResponse {
    pub new_board_fen: String,
    /// Game status *after* the move is applied, from the perspective of
    /// the side now to move. Lets the client render checkmate / stalemate
    /// / check without a follow-up `/board/status` round-trip.
    pub status: GameStatus,
}

/// JSON error body returned on 4xx. Designed to be self-contained: a
/// client can log/display this without keeping track of what it sent.
#[derive(Debug, Serialize)]
struct MakeMoveErrorBody {
    /// Short identifier for the failure category (mirrors the
    /// `MoveError` `code` tag). Useful for client-side branching.
    code: &'static str,
    /// Human-readable explanation. Suitable to surface verbatim.
    message: String,
    /// Full structured `MoveError` — all fields the engine produced.
    /// Clients that want richer rendering (e.g. highlight the source
    /// square, list legal alternatives) read these.
    details: MoveError,
    /// Whose turn it actually was on the received board, so the client
    /// doesn't have to re-parse the FEN to find out.
    side_to_move: Color,
    /// Echo of the request payload — easy to confirm the server saw what
    /// the client thinks it sent (CORS / proxy / serialization issues).
    received: GetNewBoardStateRequest,
}

fn move_error_code(err: &MoveError) -> &'static str {
    match err {
        MoveError::NoSourceSquare { .. } => "no_source_square",
        MoveError::NoPieceAtSource { .. } => "no_piece_at_source",
        MoveError::WrongTurn { .. } => "wrong_turn",
        MoveError::PieceCannotMakeMove { .. } => "piece_cannot_make_move",
        MoveError::WouldLeaveKingInCheck { .. } => "would_leave_king_in_check",
        MoveError::ApplyFailed { .. } => "apply_failed",
    }
}

#[axum::debug_handler]
async fn get_new_board_state_handler(
    Json(req): Json<GetNewBoardStateRequest>,
) -> Response {
    let mut board = fen_to_board(&req.board_fen);
    let side_to_move = board.flags.side_to_move;
    let game_move = req.game_move.clone();

    match board.make_move(game_move) {
        Ok(()) => {
            let status = board.status();
            let new_board_fen = board_to_fen(&board);
            Json(GetNewBoardStateResponse {
                new_board_fen,
                status,
            })
            .into_response()
        }
        Err(err) => {
            let body = MakeMoveErrorBody {
                code: move_error_code(&err),
                message: err.message(),
                side_to_move,
                received: req,
                details: err,
            };
            (StatusCode::BAD_REQUEST, Json(body)).into_response()
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct GetStatusRequest {
    pub board_fen: String,
}

#[derive(Debug, Serialize)]
pub struct GetStatusResponse {
    pub status: GameStatus,
}

/// Report the game status for a position without making a move — for a
/// client that loads a FEN and wants to know "is this game over?". Status
/// is from the perspective of the side to move in the supplied FEN.
#[axum::debug_handler]
async fn get_status_handler(Json(req): Json<GetStatusRequest>) -> Json<GetStatusResponse> {
    let board = fen_to_board(&req.board_fen);
    Json(GetStatusResponse {
        status: board.status(),
    })
}

// ===========================================================================
// Stateful game store (plan 06 step 4)
//
// The stateless `/board/*` endpoints above trust whatever FEN the client
// sends. The `/games/*` endpoints below make the *server* the authority: it
// owns each game's canonical `Board`, applies only engine-validated moves,
// and tracks move history + resignation. State is in-memory only — no
// persistence, no auth. Game IDs are a monotonic counter, which is fine for
// local dev; a real deployment would want unguessable IDs + per-player auth.
// ===========================================================================

/// Standard chess opening position. `POST /games` with no body starts here.
const STANDARD_START_FEN: &str =
    "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

/// One server-owned game.
struct GameRecord {
    board: Board,
    /// Moves applied so far, in order. The canonical `board` is the result
    /// of replaying these onto the starting position.
    history: Vec<GameMove>,
    /// `Some(winner)` once a player resigns. Overrides the board-derived
    /// status; `Board::status()` can never produce `Resigned`.
    resigned_winner: Option<Color>,
}

impl GameRecord {
    /// Session status: a recorded resignation wins over the board-derived
    /// status; otherwise defer to the engine.
    fn status(&self) -> GameStatus {
        match self.resigned_winner {
            Some(winner) => GameStatus::Resigned { winner },
            None => self.board.status(),
        }
    }

    /// Is the game finished? Guards against moves / double-resignation.
    /// Delegates the terminal-vs-ongoing classification to the engine
    /// (`GameStatus::is_terminal`) so it stays correct as new statuses land.
    fn is_over(&self) -> bool {
        self.status().is_terminal()
    }
}

/// In-memory registry of games, shared across requests via `State<Arc<_>>`.
#[derive(Default)]
struct GameStore {
    games: Mutex<HashMap<u64, GameRecord>>,
    next_id: AtomicU64,
}

/// Why a `/games/{id}` operation couldn't complete.
#[derive(Debug)]
enum GameError {
    NotFound,
    /// The game is already finished — no further moves / resignations.
    GameOver,
    /// The submitted move was rejected by the engine.
    IllegalMove(MoveError),
    /// A resignation named a color that can't be a player (`Neutral`).
    InvalidColor,
}

impl GameStore {
    /// Insert a fresh game starting from `board`. Returns its id.
    fn create(&self, board: Board) -> u64 {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed) + 1;
        self.games.lock().expect("game store mutex").insert(
            id,
            GameRecord {
                board,
                history: Vec::new(),
                resigned_winner: None,
            },
        );
        id
    }

    /// Run `f` against a game under the lock. `NotFound` if absent.
    fn read<T>(&self, id: u64, f: impl FnOnce(&GameRecord) -> T) -> Result<T, GameError> {
        let games = self.games.lock().expect("game store mutex");
        games.get(&id).map(f).ok_or(GameError::NotFound)
    }

    /// Validate + apply a move to the canonical board. The move is applied
    /// to a clone first, so a rejected move never corrupts the stored game.
    fn apply_move(&self, id: u64, mv: GameMove) -> Result<(), GameError> {
        let mut games = self.games.lock().expect("game store mutex");
        let rec = games.get_mut(&id).ok_or(GameError::NotFound)?;
        if rec.is_over() {
            return Err(GameError::GameOver);
        }
        let mut next = rec.board.clone();
        next.make_move(mv.clone()).map_err(GameError::IllegalMove)?;
        rec.board = next;
        rec.history.push(mv);
        Ok(())
    }

    /// Record a resignation. `color` is the side giving up (defaulting to
    /// the side to move); the winner is its opponent.
    fn resign(&self, id: u64, color: Option<Color>) -> Result<(), GameError> {
        let mut games = self.games.lock().expect("game store mutex");
        let rec = games.get_mut(&id).ok_or(GameError::NotFound)?;
        if rec.is_over() {
            return Err(GameError::GameOver);
        }
        let resigning = color.unwrap_or(rec.board.flags.side_to_move);
        // `Neutral` is never a player (trains / unaligned actors); resigning
        // as Neutral would persist a nonsensical `Resigned { winner: Neutral }`.
        if resigning == Color::Neutral {
            return Err(GameError::InvalidColor);
        }
        rec.resigned_winner = Some(resigning.opposite());
        Ok(())
    }
}

impl IntoResponse for GameError {
    fn into_response(self) -> Response {
        match self {
            GameError::NotFound => (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": "game_not_found" })),
            )
                .into_response(),
            GameError::GameOver => (
                StatusCode::CONFLICT,
                Json(serde_json::json!({ "error": "game_over" })),
            )
                .into_response(),
            GameError::IllegalMove(err) => {
                let body = GameMoveErrorBody {
                    code: move_error_code(&err),
                    message: err.message(),
                    details: err,
                };
                (StatusCode::BAD_REQUEST, Json(body)).into_response()
            }
            GameError::InvalidColor => (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": "invalid_color" })),
            )
                .into_response(),
        }
    }
}

/// Structured body for a move rejected on the stateful path.
#[derive(Debug, Serialize)]
struct GameMoveErrorBody {
    code: &'static str,
    message: String,
    details: MoveError,
}

/// Full view of a game's current state — the body of every `/games` success.
#[derive(Debug, Serialize)]
pub struct GameView {
    pub game_id: u64,
    pub fen: String,
    pub status: GameStatus,
    pub history: Vec<GameMove>,
}

impl GameView {
    fn of(id: u64, rec: &GameRecord) -> Self {
        GameView {
            game_id: id,
            fen: board_to_fen(&rec.board),
            status: rec.status(),
            history: rec.history.clone(),
        }
    }
}

#[derive(Debug, Default, Deserialize)]
pub struct CreateGameRequest {
    /// Optional starting position. Defaults to the standard opening.
    #[serde(default)]
    pub board_fen: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SubmitMoveRequest {
    pub game_move: GameMove,
}

#[derive(Debug, Default, Deserialize)]
pub struct ResignRequest {
    /// Which side resigns. Defaults to the side to move.
    #[serde(default)]
    pub color: Option<Color>,
}

/// `POST /games` — create a game. Optional JSON body `{ "board_fen": ... }`;
/// an empty body starts from the standard opening. Responds `201 Created`.
#[axum::debug_handler]
async fn create_game_handler(State(store): State<Arc<GameStore>>, body: Bytes) -> Response {
    let req: CreateGameRequest = match parse_optional_body(&body) {
        Ok(req) => req,
        Err(resp) => return resp,
    };

    let fen = req.board_fen.as_deref().unwrap_or(STANDARD_START_FEN);
    let id = store.create(fen_to_board(fen));
    let view = store
        .read(id, |rec| GameView::of(id, rec))
        .expect("freshly created game exists");
    // RFC 9110 §15.3.2: a 201 SHOULD point at the newly-created resource.
    (
        StatusCode::CREATED,
        [(http::header::LOCATION, format!("/games/{id}"))],
        Json(view),
    )
        .into_response()
}

/// `GET /games/{id}` — current FEN, status, and move history.
#[axum::debug_handler]
async fn get_game_handler(State(store): State<Arc<GameStore>>, Path(id): Path<u64>) -> Response {
    match store.read(id, |rec| GameView::of(id, rec)) {
        Ok(view) => Json(view).into_response(),
        Err(err) => err.into_response(),
    }
}

/// `POST /games/{id}/moves` — validate + apply a move to the canonical game.
/// Body: `{ "game_move": GameMove }`.
#[axum::debug_handler]
async fn submit_move_handler(
    State(store): State<Arc<GameStore>>,
    Path(id): Path<u64>,
    Json(req): Json<SubmitMoveRequest>,
) -> Response {
    match store.apply_move(id, req.game_move) {
        Ok(()) => {
            let view = store
                .read(id, |rec| GameView::of(id, rec))
                .expect("game exists after a successful move");
            Json(view).into_response()
        }
        Err(err) => err.into_response(),
    }
}

/// `POST /games/{id}/resign` — record a resignation. Optional body
/// `{ "color": "White" | "Black" }`; defaults to the side to move.
#[axum::debug_handler]
async fn resign_handler(
    State(store): State<Arc<GameStore>>,
    Path(id): Path<u64>,
    body: Bytes,
) -> Response {
    let req: ResignRequest = match parse_optional_body(&body) {
        Ok(req) => req,
        Err(resp) => return resp,
    };

    match store.resign(id, req.color) {
        Ok(()) => {
            let view = store
                .read(id, |rec| GameView::of(id, rec))
                .expect("game exists after resignation");
            Json(view).into_response()
        }
        Err(err) => err.into_response(),
    }
}

/// `400` for a malformed JSON request body.
fn invalid_request(err: &serde_json::Error) -> Response {
    (
        StatusCode::BAD_REQUEST,
        Json(serde_json::json!({ "error": "invalid_request", "message": err.to_string() })),
    )
        .into_response()
}

/// Parse an optional JSON request body: an empty body yields `T::default()`,
/// a malformed body yields a `400 invalid_request`. Shared by the endpoints
/// that accept an empty body to select a default (`POST /games`, `/resign`).
fn parse_optional_body<T: Default + serde::de::DeserializeOwned>(
    body: &Bytes,
) -> Result<T, Response> {
    if body.is_empty() {
        Ok(T::default())
    } else {
        serde_json::from_slice(body.as_ref()).map_err(|e| invalid_request(&e))
    }
}

/// Build the full router (all routes + CORS) backed by `store`. Extracted
/// from `serve_api` so HTTP-layer tests can drive it via
/// `tower::ServiceExt::oneshot` without binding a socket.
fn build_app(store: Arc<GameStore>) -> Router {
    let cors = CorsLayer::new()
        .allow_origin("*".parse::<http::HeaderValue>().unwrap()) // allow all — dev only
        .allow_methods([Method::GET, Method::POST])
        .allow_headers([http::header::CONTENT_TYPE]);

    Router::new()
        // Stateless: the client owns the FEN.
        .route("/board/moves", post(get_moves_handler))
        .route("/board/new_state", post(get_new_board_state_handler))
        .route("/board/status", post(get_status_handler))
        // Stateful: the server owns the game (plan 06 step 4).
        .route("/games", post(create_game_handler))
        .route("/games/{id}", get(get_game_handler))
        .route("/games/{id}/moves", post(submit_move_handler))
        .route("/games/{id}/resign", post(resign_handler))
        .layer(cors)
        .with_state(store)
}

pub async fn serve_api() {
    let port = 8080;
    let binding_address = "0.0.0.0".to_string() + ":" + &port.to_string();

    let app = build_app(Arc::new(GameStore::default()));

    let listener = tokio::net::TcpListener::bind(&binding_address)
        .await
        .expect(&format!("Couldn't bind to port {port}"));
    println!("Serving on {binding_address}");
    ::axum::serve(listener, app)
        .await
        .expect(&format!("Failed to serve on port {port}"));
}

#[tokio::main]
async fn main() {
    serve_api().await;
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::{Body, to_bytes};
    use axum::http::Request;
    use engine::board::MoveType;
    use tower::ServiceExt;

    /// White e-pawn push e2->e3 (rank 0 = top, so white pawns sit on rank 6).
    fn e2_e3() -> GameMove {
        GameMove {
            from: Coord { file: 4, rank: 6 },
            move_type: MoveType::MoveTo(Coord { file: 4, rank: 5 }),
        }
    }

    fn standard_game() -> (GameStore, u64) {
        let store = GameStore::default();
        let id = store.create(fen_to_board(STANDARD_START_FEN));
        (store, id)
    }

    #[test]
    fn create_starts_ongoing_with_empty_history() {
        let (store, id) = standard_game();
        let view = store.read(id, |r| GameView::of(id, r)).unwrap();
        assert_eq!(view.game_id, id);
        assert_eq!(view.status, GameStatus::Ongoing);
        assert!(view.history.is_empty());
    }

    #[test]
    fn ids_are_unique_and_monotonic() {
        let store = GameStore::default();
        let a = store.create(fen_to_board(STANDARD_START_FEN));
        let b = store.create(fen_to_board(STANDARD_START_FEN));
        assert_ne!(a, b);
        assert!(b > a);
    }

    #[test]
    fn legal_move_updates_board_and_history() {
        let (store, id) = standard_game();
        store.apply_move(id, e2_e3()).expect("e2-e3 is legal");
        let view = store.read(id, |r| GameView::of(id, r)).unwrap();
        assert_eq!(view.history.len(), 1);
        // Side flipped to black; pawn now on e3.
        assert!(
            view.fen
                .starts_with("rnbqkbnr/pppppppp/8/8/8/4P3/PPPP1PPP/RNBQKBNR b"),
            "unexpected fen: {}",
            view.fen
        );
    }

    #[test]
    fn illegal_move_is_rejected_and_leaves_game_untouched() {
        let (store, id) = standard_game();
        // A pawn moving sideways is illegal.
        let bogus = GameMove {
            from: Coord { file: 4, rank: 6 },
            move_type: MoveType::MoveTo(Coord { file: 5, rank: 6 }),
        };
        let before = store.read(id, |r| board_to_fen(&r.board)).unwrap();
        assert!(matches!(
            store.apply_move(id, bogus),
            Err(GameError::IllegalMove(_))
        ));
        let (after_fen, history_len) = store
            .read(id, |r| (board_to_fen(&r.board), r.history.len()))
            .unwrap();
        assert_eq!(before, after_fen, "a rejected move must not mutate the board");
        assert_eq!(history_len, 0, "a rejected move must not grow history");
    }

    #[test]
    fn operations_on_missing_game_are_not_found() {
        let store = GameStore::default();
        assert!(matches!(
            store.apply_move(999, e2_e3()),
            Err(GameError::NotFound)
        ));
        assert!(matches!(store.resign(999, None), Err(GameError::NotFound)));
    }

    #[test]
    fn resign_sets_winner_and_blocks_further_play() {
        let (store, id) = standard_game();
        // White (side to move) resigns by default -> Black wins.
        store.resign(id, None).expect("can resign an ongoing game");
        let status = store.read(id, |r| r.status()).unwrap();
        assert_eq!(status, GameStatus::Resigned {
            winner: Color::Black
        });
        // No moves once the game is over, and no double-resignation.
        assert!(matches!(
            store.apply_move(id, e2_e3()),
            Err(GameError::GameOver)
        ));
        assert!(matches!(store.resign(id, None), Err(GameError::GameOver)));
    }

    #[test]
    fn explicit_resigning_color_hands_win_to_opponent() {
        let (store, id) = standard_game();
        store.resign(id, Some(Color::Black)).unwrap();
        let status = store.read(id, |r| r.status()).unwrap();
        assert_eq!(status, GameStatus::Resigned {
            winner: Color::White
        });
    }

    // ---- HTTP-layer tests: drive the real router via `oneshot`, exercising
    // ---- status-code mapping, body shapes, and the /board/* endpoints.

    /// One request against the router. Clones `app` so the same `GameStore`
    /// is shared across calls within a test. Returns (status, parsed body).
    async fn send(
        app: &Router,
        method: &str,
        uri: &str,
        body: Option<serde_json::Value>,
    ) -> (StatusCode, serde_json::Value) {
        let builder = Request::builder().method(method).uri(uri);
        let req = match body {
            Some(v) => builder
                .header("content-type", "application/json")
                .body(Body::from(v.to_string()))
                .unwrap(),
            None => builder.body(Body::empty()).unwrap(),
        };
        let resp = app.clone().oneshot(req).await.unwrap();
        let status = resp.status();
        let bytes = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
        let json = if bytes.is_empty() {
            serde_json::Value::Null
        } else {
            serde_json::from_slice(&bytes).unwrap()
        };
        (status, json)
    }

    fn e2_e3_body() -> serde_json::Value {
        serde_json::json!({
            "game_move": {
                "from": { "file": 4, "rank": 6 },
                "move_type": { "kind": "MoveTo", "target": { "file": 4, "rank": 5 } }
            }
        })
    }

    #[tokio::test]
    async fn http_create_get_move_lifecycle() {
        let app = build_app(Arc::new(GameStore::default()));

        let (status, body) = send(&app, "POST", "/games", None).await;
        assert_eq!(status, StatusCode::CREATED);
        assert_eq!(body["game_id"], 1);
        assert_eq!(body["status"], "Ongoing");
        assert_eq!(body["history"].as_array().unwrap().len(), 0);

        let (status, body) = send(&app, "POST", "/games/1/moves", Some(e2_e3_body())).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["status"], "Ongoing");
        assert_eq!(body["history"].as_array().unwrap().len(), 1);

        let (status, body) = send(&app, "GET", "/games/1", None).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["history"].as_array().unwrap().len(), 1);
    }

    #[tokio::test]
    async fn http_create_sets_location_header() {
        let app = build_app(Arc::new(GameStore::default()));
        let req = Request::builder()
            .method("POST")
            .uri("/games")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::CREATED);
        assert_eq!(
            resp.headers()
                .get(http::header::LOCATION)
                .unwrap()
                .to_str()
                .unwrap(),
            "/games/1"
        );
    }

    #[tokio::test]
    async fn http_missing_game_is_404() {
        let app = build_app(Arc::new(GameStore::default()));
        let (status, body) = send(&app, "GET", "/games/999", None).await;
        assert_eq!(status, StatusCode::NOT_FOUND);
        assert_eq!(body["error"], "game_not_found");
        let (status, _) = send(&app, "POST", "/games/999/moves", Some(e2_e3_body())).await;
        assert_eq!(status, StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn http_resign_then_move_is_409() {
        let app = build_app(Arc::new(GameStore::default()));
        send(&app, "POST", "/games", None).await;
        // White is to move; default resign -> Black wins.
        let (status, body) = send(&app, "POST", "/games/1/resign", None).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["status"]["Resigned"]["winner"], "Black");
        let (status, body) = send(&app, "POST", "/games/1/moves", Some(e2_e3_body())).await;
        assert_eq!(status, StatusCode::CONFLICT);
        assert_eq!(body["error"], "game_over");
    }

    #[tokio::test]
    async fn http_illegal_move_is_400_structured() {
        let app = build_app(Arc::new(GameStore::default()));
        send(&app, "POST", "/games", None).await;
        // A pawn moving sideways is illegal.
        let bogus = serde_json::json!({
            "game_move": {
                "from": { "file": 4, "rank": 6 },
                "move_type": { "kind": "MoveTo", "target": { "file": 5, "rank": 6 } }
            }
        });
        let (status, body) = send(&app, "POST", "/games/1/moves", Some(bogus)).await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(body["code"], "piece_cannot_make_move");
        assert!(body["message"].is_string());
        assert!(body["details"].is_object());
    }

    #[tokio::test]
    async fn http_custom_fen_checkmate_then_blocks() {
        let app = build_app(Arc::new(GameStore::default()));
        // Fool's-mate-pre position, black to move.
        let create = serde_json::json!({
            "board_fen": "rnbqkbnr/pppp1ppp/8/4p3/6P1/5P2/PPPPP2P/RNBQKBNR b KQkq - 0 2"
        });
        let (status, view) = send(&app, "POST", "/games", Some(create)).await;
        assert_eq!(status, StatusCode::CREATED);
        let id = view["game_id"].as_u64().unwrap();

        // Qd8-h4#.
        let mate = serde_json::json!({
            "game_move": {
                "from": { "file": 3, "rank": 0 },
                "move_type": { "kind": "MoveTo", "target": { "file": 7, "rank": 4 } }
            }
        });
        let (status, body) = send(&app, "POST", &format!("/games/{id}/moves"), Some(mate)).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["status"]["Checkmate"]["winner"], "Black");

        // Resigning a finished (checkmated) game -> 409.
        let (status, _) = send(&app, "POST", &format!("/games/{id}/resign"), None).await;
        assert_eq!(status, StatusCode::CONFLICT);
    }

    #[tokio::test]
    async fn http_malformed_body_and_neutral_resign_are_400() {
        let app = build_app(Arc::new(GameStore::default()));

        // Malformed JSON to create (Bytes extractor -> our `invalid_request`).
        let req = Request::builder()
            .method("POST")
            .uri("/games")
            .header("content-type", "application/json")
            .body(Body::from("{not valid json"))
            .unwrap();
        let resp = app.clone().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
        let bytes = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
        let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(v["error"], "invalid_request");

        // Resigning as Neutral -> 400 invalid_color.
        send(&app, "POST", "/games", None).await;
        let (status, body) =
            send(&app, "POST", "/games/1/resign", Some(serde_json::json!({ "color": "Neutral" })))
                .await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(body["error"], "invalid_color");
    }

    #[tokio::test]
    async fn http_board_moves_returns_legal_only() {
        let app = build_app(Arc::new(GameStore::default()));
        // Pinned bishop: white king e1, white bishop e2, black rook e8.
        // The bishop can only move along diagonals, all of which expose the
        // king to the rook -> zero legal moves (raw get_moves would list some).
        let body = serde_json::json!({
            "board_fen": "4r3/8/8/8/8/8/4B3/4K3 w - - 0 1",
            "from": { "file": 4, "rank": 6 }
        });
        let (status, v) = send(&app, "POST", "/board/moves", Some(body)).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(
            v["moves"].as_array().unwrap().len(),
            0,
            "pinned bishop must have no legal moves"
        );
    }

    #[tokio::test]
    async fn http_board_status_and_new_state_status() {
        let app = build_app(Arc::new(GameStore::default()));

        // /board/status over a checkmated position.
        let cm = serde_json::json!({
            "board_fen": "rnb1kbnr/pppp1ppp/8/4p3/6Pq/5P2/PPPPP2P/RNBQKBNR w KQkq - 1 3"
        });
        let (status, v) = send(&app, "POST", "/board/status", Some(cm)).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(v["status"]["Checkmate"]["winner"], "Black");

        // /board/new_state returns the post-move status alongside the FEN.
        let mv = serde_json::json!({
            "board_fen": "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
            "game_move": {
                "from": { "file": 4, "rank": 6 },
                "move_type": { "kind": "MoveTo", "target": { "file": 4, "rank": 5 } }
            }
        });
        let (status, v) = send(&app, "POST", "/board/new_state", Some(mv)).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(v["status"], "Ongoing");
        // The returned FEN must reflect the applied move (pawn now on e3,
        // side flipped to black) — not merely echo the input.
        assert!(
            v["new_board_fen"]
                .as_str()
                .unwrap()
                .starts_with("rnbqkbnr/pppppppp/8/8/8/4P3/PPPP1PPP/RNBQKBNR b"),
            "new_board_fen did not reflect the move: {}",
            v["new_board_fen"]
        );
    }

    #[tokio::test]
    async fn http_board_new_state_illegal_move_is_400() {
        let app = build_app(Arc::new(GameStore::default()));
        // Illegal: a pawn moving sideways from the opening position.
        let mv = serde_json::json!({
            "board_fen": "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
            "game_move": {
                "from": { "file": 4, "rank": 6 },
                "move_type": { "kind": "MoveTo", "target": { "file": 5, "rank": 6 } }
            }
        });
        let (status, body) = send(&app, "POST", "/board/new_state", Some(mv)).await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
        // The stateless error body carries the side_to_move + received echo
        // (distinct from the trimmed stateful /games move-error body).
        assert_eq!(body["code"], "piece_cannot_make_move");
        assert!(body["message"].is_string());
        assert!(body["details"].is_object());
        assert_eq!(body["side_to_move"], "White");
        assert!(body["received"].is_object());
    }

    #[tokio::test]
    async fn http_board_new_state_reports_post_move_status() {
        let app = build_app(Arc::new(GameStore::default()));
        // A legal move that *ends* the game: black plays Qd8-h4# (fool's
        // mate). Pins that `/board/new_state` computes `status` on the
        // POST-move board (a pre-move read would still say Ongoing).
        let mv = serde_json::json!({
            "board_fen": "rnbqkbnr/pppp1ppp/8/4p3/6P1/5P2/PPPPP2P/RNBQKBNR b KQkq - 0 2",
            "game_move": {
                "from": { "file": 3, "rank": 0 },
                "move_type": { "kind": "MoveTo", "target": { "file": 7, "rank": 4 } }
            }
        });
        let (status, v) = send(&app, "POST", "/board/new_state", Some(mv)).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(v["status"]["Checkmate"]["winner"], "Black");
    }
}
