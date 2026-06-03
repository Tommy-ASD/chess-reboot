use axum::{
    Json, Router,
    extract::{
        Path, State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
};
use http::Method;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::sync::broadcast;
use tower_http::cors::CorsLayer;

use engine::board::{
    Coord, GameMove, GameStatus, MoveError,
    fen::{FenError, board_to_fen, fen_to_board},
};
use engine::pieces::Color;

mod game;
use game::{AppState, AuthPlayer, CreateGame, GameActionError, GameState};

#[derive(Debug, Deserialize)]
pub struct GetMovesRequest {
    pub board_fen: String,
    pub from: Coord,
}

#[derive(Debug, Serialize)]
pub struct GetMovesResponse {
    pub moves: Vec<GameMove>,
}

/// JSON error body returned on a 400 when the supplied `board_fen` is
/// structurally malformed. Follows the same client contract as
/// `MakeMoveErrorBody` — a machine-branchable `code` plus a human
/// message — with an echo of the FEN the server actually parsed.
/// Intentionally leaner: no structured `details` payload (the FEN
/// failure is fully described by `code` + `message`).
#[derive(Debug, Serialize)]
struct FenErrorBody {
    code: &'static str,
    message: String,
    /// The FEN string the server received, so a client can confirm what
    /// it sent without re-deriving it from request state.
    fen: String,
}

fn fen_error_code(err: &FenError) -> &'static str {
    match err {
        FenError::EmptyInput => "fen_empty_input",
        FenError::BadRowCount { .. } => "fen_bad_row_count",
        FenError::BadRowWidth { .. } => "fen_bad_row_width",
        FenError::UnknownPieceSymbol(_) => "fen_unknown_piece_symbol",
        FenError::UnbalancedParen { .. } => "fen_unbalanced_paren",
        FenError::BadExtendedSquare { .. } => "fen_bad_extended_square",
        FenError::BadFlagsField(_) => "fen_bad_flags_field",
    }
}

fn fen_error_response(err: FenError, fen: String) -> Response {
    let body = FenErrorBody {
        code: fen_error_code(&err),
        message: err.to_string(),
        fen,
    };
    (StatusCode::BAD_REQUEST, Json(body)).into_response()
}

#[axum::debug_handler]
async fn get_moves_handler(Json(req): Json<GetMovesRequest>) -> Response {
    let board = match fen_to_board(&req.board_fen) {
        Ok(b) => b,
        Err(e) => return fen_error_response(e, req.board_fen),
    };
    let moves = board.get_moves(&req.from);
    Json(GetMovesResponse { moves }).into_response()
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetNewBoardStateRequest {
    pub board_fen: String,
    pub game_move: GameMove,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetNewBoardStateResponse {
    pub new_board_fen: String,
    /// Game status of the position *after* the move was applied,
    /// evaluated for the side now to move: `Check`/`Stalemate` refer to
    /// that player and `Ongoing` means the game continues. Note
    /// `Checkmate { winner }` names the side that *delivered* mate — the
    /// player who just moved, **not** the side now to move; `BrainrotWin
    /// { winner }` and `BrainrotLockout { winner }` (plan 04) carry the
    /// same winner semantics for the two Skibidi terminal cases — a
    /// brainrot stalemate-wall (the loser is frozen with no legal move)
    /// vs. losing your Skibidi to a phase-4 enemy (`BrainrotLockout`,
    /// which ends the game even if the loser still has legal moves).
    /// Folded in so a client gets
    /// check/checkmate/stalemate/brainrot-win with every move without
    /// a follow-up `/board/status` round-trip. Adjacently tagged (see
    /// `GameStatus`): `{"status":"Checkmate","data":{"winner":"White"}}`
    /// means White just gave mate and won.
    pub status: GameStatus,
}

#[derive(Debug, Deserialize)]
pub struct GetStatusRequest {
    pub board_fen: String,
}

#[derive(Debug, Serialize)]
pub struct GetStatusResponse {
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
        MoveError::CompelledByTornado { .. } => "compelled_by_tornado",
        MoveError::ApplyFailed { .. } => "apply_failed",
    }
}

#[axum::debug_handler]
async fn get_new_board_state_handler(
    Json(req): Json<GetNewBoardStateRequest>,
) -> Response {
    let mut board = match fen_to_board(&req.board_fen) {
        Ok(b) => b,
        Err(e) => return fen_error_response(e, req.board_fen),
    };
    let side_to_move = board.flags.side_to_move;
    let game_move = req.game_move.clone();

    match board.make_move(game_move) {
        Ok(()) => {
            let new_board_fen = board_to_fen(&board);
            let status = board.status();
            Json(GetNewBoardStateResponse { new_board_fen, status })
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

/// Ad-hoc game-status query for a client that holds a FEN and just
/// wants to know "is this game over?" without making a move. Same
/// structured-400-on-bad-FEN contract as the other endpoints.
#[axum::debug_handler]
async fn get_status_handler(Json(req): Json<GetStatusRequest>) -> Response {
    let board = match fen_to_board(&req.board_fen) {
        Ok(b) => b,
        Err(e) => return fen_error_response(e, req.board_fen),
    };
    Json(GetStatusResponse { status: board.status() }).into_response()
}

// ---------------------------------------------------------------------
// Online multiplayer handlers (Phase 1: REST backbone over the game store)
// ---------------------------------------------------------------------

/// Map a `GameActionError` to an HTTP response. Move rejections carry the
/// `code` / `message` / `details` fields — that subset of
/// `/board/new_state`'s `MakeMoveErrorBody`, not the full shape: the online
/// client already holds the live `GameState`, so the `side_to_move` /
/// `received` echoes are omitted. A client can branch on `code` for both
/// local and online play.
fn game_error_response(err: GameActionError) -> Response {
    match err {
        GameActionError::NotFound => (
            StatusCode::NOT_FOUND,
            Json(json!({ "code": "game_not_found", "message": "no such game" })),
        )
            .into_response(),
        GameActionError::BadFen(msg) => (
            StatusCode::BAD_REQUEST,
            Json(json!({ "code": "bad_starting_fen", "message": msg })),
        )
            .into_response(),
        GameActionError::NotSeated => (
            StatusCode::FORBIDDEN,
            Json(json!({ "code": "not_seated", "message": "you are not a player in this game" })),
        )
            .into_response(),
        GameActionError::Full => (
            StatusCode::CONFLICT,
            Json(json!({ "code": "game_full", "message": "both seats are already taken" })),
        )
            .into_response(),
        GameActionError::NotYourTurn => (
            StatusCode::FORBIDDEN,
            Json(json!({ "code": "not_your_turn", "message": "it is not your turn" })),
        )
            .into_response(),
        GameActionError::Over => (
            StatusCode::CONFLICT,
            Json(json!({ "code": "game_over", "message": "the game has already ended" })),
        )
            .into_response(),
        GameActionError::Move(move_err) => {
            let body = json!({
                "code": move_error_code(&move_err),
                "message": move_err.message(),
                "details": move_err,
            });
            (StatusCode::BAD_REQUEST, Json(body)).into_response()
        }
        GameActionError::Internal(msg) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "code": "internal", "message": msg })),
        )
            .into_response(),
    }
}

/// `POST /games` — create a game; the caller is seated (white by default).
#[axum::debug_handler]
async fn create_game_handler(
    State(state): State<AppState>,
    player: AuthPlayer,
    Json(req): Json<CreateGame>,
) -> Response {
    match state.create(&player, req) {
        Ok(s) => Json(s).into_response(),
        Err(e) => game_error_response(e),
    }
}

/// `GET /games` — public, joinable games for the lobby.
#[axum::debug_handler]
async fn list_games_handler(State(state): State<AppState>) -> Response {
    Json(state.list_public()).into_response()
}

/// `GET /games/{id}` — a snapshot (the path token may be a game id or a
/// join code). Requires an identity so `your_color` can be filled.
#[axum::debug_handler]
async fn get_game_handler(
    State(state): State<AppState>,
    player: AuthPlayer,
    Path(id): Path<String>,
) -> Response {
    match state.get(&id, Some(player.id)) {
        Some(s) => Json(s).into_response(),
        None => game_error_response(GameActionError::NotFound),
    }
}

/// `POST /games/{id}/join` — seat the caller in the open colour.
#[axum::debug_handler]
async fn join_game_handler(
    State(state): State<AppState>,
    player: AuthPlayer,
    Path(id): Path<String>,
) -> Response {
    match state.join(&id, &player) {
        Ok(s) => Json(s).into_response(),
        Err(e) => game_error_response(e),
    }
}

#[derive(Debug, Deserialize)]
struct SubmitMoveRequest {
    game_move: GameMove,
}

/// `POST /games/{id}/move` — submit a move; the store enforces colour/turn
/// before the engine validates legality.
#[axum::debug_handler]
async fn move_handler(
    State(state): State<AppState>,
    player: AuthPlayer,
    Path(id): Path<String>,
    Json(req): Json<SubmitMoveRequest>,
) -> Response {
    match state.apply_move(&id, &player, req.game_move) {
        Ok(s) => Json(s).into_response(),
        Err(e) => game_error_response(e),
    }
}

/// `POST /games/{id}/resign` — the caller forfeits.
#[axum::debug_handler]
async fn resign_game_handler(
    State(state): State<AppState>,
    player: AuthPlayer,
    Path(id): Path<String>,
) -> Response {
    match state.resign(&id, &player) {
        Ok(s) => Json(s).into_response(),
        Err(e) => game_error_response(e),
    }
}

// ---------------------------------------------------------------------
// Phase 2: WebSocket live updates (server -> client push)
// ---------------------------------------------------------------------

/// Serialize `value` to a WS text frame. `Err(())` (serialize or send
/// failure) tells the caller to drop the connection.
async fn send_json<T: Serialize>(socket: &mut WebSocket, value: &T) -> Result<(), ()> {
    let txt = serde_json::to_string(value).map_err(|_| ())?;
    socket.send(Message::Text(txt.into())).await.map_err(|_| ())
}

/// `GET /ws/games/{id}` — push a `GameState` snapshot on connect and on
/// every change. Read-only / spectator-friendly; moves go over REST.
#[axum::debug_handler]
async fn game_ws_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
    ws: WebSocketUpgrade,
) -> Response {
    match state.subscribe_game(&id) {
        Some((initial, rx)) => ws.on_upgrade(move |socket| game_ws_loop(socket, initial, rx)),
        None => game_error_response(GameActionError::NotFound),
    }
}

async fn game_ws_loop(
    mut socket: WebSocket,
    initial: GameState,
    mut rx: broadcast::Receiver<GameState>,
) {
    if send_json(&mut socket, &initial).await.is_err() {
        return;
    }
    loop {
        tokio::select! {
            update = rx.recv() => match update {
                Ok(state) => {
                    if send_json(&mut socket, &state).await.is_err() { break; }
                }
                // Slow client dropped some snapshots — the next recv carries
                // the latest, so the board still converges; log the skip.
                Err(broadcast::error::RecvError::Lagged(skipped)) => {
                    eprintln!("game ws: client lagged, skipped {skipped} update(s)");
                    continue;
                }
                Err(broadcast::error::RecvError::Closed) => break,
            },
            incoming = socket.recv() => match incoming {
                Some(Ok(Message::Close(_))) | None => break,
                Some(Err(e)) => {
                    eprintln!("game ws: socket error: {e}");
                    break;
                }
                // Push-only channel: ignore anything the client sends.
                Some(Ok(_)) => {}
            },
        }
    }
}

/// `GET /ws/lobby` — push the public-games list on connect and whenever the
/// lobby changes (game created / joined / ended).
#[axum::debug_handler]
async fn lobby_ws_handler(State(state): State<AppState>, ws: WebSocketUpgrade) -> Response {
    ws.on_upgrade(move |socket| lobby_ws_loop(socket, state))
}

async fn lobby_ws_loop(mut socket: WebSocket, state: AppState) {
    let mut rx = state.subscribe_lobby();
    if send_json(&mut socket, &state.list_public()).await.is_err() {
        return;
    }
    loop {
        tokio::select! {
            tick = rx.recv() => match tick {
                Ok(()) | Err(broadcast::error::RecvError::Lagged(_)) => {
                    if send_json(&mut socket, &state.list_public()).await.is_err() { break; }
                }
                Err(broadcast::error::RecvError::Closed) => break,
            },
            incoming = socket.recv() => match incoming {
                Some(Ok(Message::Close(_))) | None => break,
                Some(Err(e)) => {
                    eprintln!("lobby ws: socket error: {e}");
                    break;
                }
                Some(Ok(_)) => {}
            },
        }
    }
}

pub async fn serve_api() {
    let cors = CorsLayer::new()
        .allow_origin("*".parse::<http::HeaderValue>().unwrap()) // allow all — dev only
        .allow_methods([Method::GET, Method::POST])
        .allow_headers([
            http::header::CONTENT_TYPE,
            http::HeaderName::from_static("x-player-id"),
            http::HeaderName::from_static("x-player-name"),
        ]);

    let port = 8080;
    let binding_address = format!("0.0.0.0:{port}");

    let state = AppState::new();
    let app = Router::new()
        // Stateless engine endpoints (local play + legal-move queries).
        .route("/board/moves", post(get_moves_handler))
        .route("/board/new_state", post(get_new_board_state_handler))
        .route("/board/status", post(get_status_handler))
        // Online multiplayer (Phase 1 REST; WS added in Phase 2).
        .route("/games", post(create_game_handler).get(list_games_handler))
        .route("/games/{id}", get(get_game_handler))
        .route("/games/{id}/join", post(join_game_handler))
        .route("/games/{id}/move", post(move_handler))
        .route("/games/{id}/resign", post(resign_game_handler))
        // Live updates (Phase 2): per-game + lobby pushes.
        .route("/ws/games/{id}", get(game_ws_handler))
        .route("/ws/lobby", get(lobby_ws_handler))
        .layer(cors)
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(&binding_address)
        .await
        .unwrap_or_else(|e| panic!("Couldn't bind to {binding_address}: {e}"));
    println!("Serving on {binding_address}");
    ::axum::serve(listener, app)
        .await
        .unwrap_or_else(|e| panic!("Failed to serve on {binding_address}: {e}"));
}

#[tokio::main]
async fn main() {
    serve_api().await;
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Plan-05 audit (B8): the FEN→400 mapping is the plan's stated API
    /// deliverable, and the engine-side `FenError` tests can't reach
    /// the three parser-never-constructs variants
    /// (`BadRowCount`/`BadExtendedSquare`/`BadFlagsField`) — this is
    /// their only coverage. Pure-function test; no HTTP harness needed.
    #[test]
    fn fen_error_code_maps_every_variant() {
        let cases: [(FenError, &str); 7] = [
            (FenError::EmptyInput, "fen_empty_input"),
            (
                FenError::BadRowCount { expected: 8, found: 9 },
                "fen_bad_row_count",
            ),
            (
                FenError::BadRowWidth { row: 0, expected: 8, found: 9 },
                "fen_bad_row_width",
            ),
            (
                FenError::UnknownPieceSymbol("Z".to_string()),
                "fen_unknown_piece_symbol",
            ),
            (
                FenError::UnbalancedParen { in_row: 0 },
                "fen_unbalanced_paren",
            ),
            (
                FenError::BadExtendedSquare {
                    content: "x".to_string(),
                    reason: "r",
                },
                "fen_bad_extended_square",
            ),
            (
                FenError::BadFlagsField("x".to_string()),
                "fen_bad_flags_field",
            ),
        ];
        for (err, code) in cases {
            assert_eq!(fen_error_code(&err), code, "code for {err:?}");
            // `fen_error_response` builds the body from these same two
            // calls (`fen_error_code` + `Display`), so the body's
            // `code`/`message` can't diverge from what's asserted
            // here; we only additionally pin the 400 status and a
            // non-empty `Display` (the JSON shape is a trivial
            // infallible derive, not re-deserialized here).
            let resp = fen_error_response(err.clone(), "the-fen".to_string());
            assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
            assert!(!err.to_string().is_empty(), "empty message for {err:?}");
        }
    }

    /// Plan 06 (audit D): `/board/new_state` must report the status of
    /// the position *after* the move, not before. A regression that
    /// read `status()` on the pre-move board would still pass every
    /// engine-side test (those build boards directly), so guard the
    /// ordering at the handler: a move that delivers mate must come back
    /// `Checkmate`, not `Ongoing`. Direct handler call — no HTTP
    /// harness, consistent with the test above.
    #[tokio::test]
    async fn new_state_status_is_post_move() {
        use engine::board::MoveType;

        let to = |f, r| MoveType::MoveTo(Coord { file: f, rank: r });
        let at = |f, r| Coord { file: f, rank: r };

        // Internal coord system: rank 0 is black's back row, rank 7 is white's.
        // Fool's mate, set up to the position where Black plays Qd8-h4#.
        let mut board = fen_to_board(
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq -",
        )
        .unwrap();
        for (from, dest) in [
            (at(5, 6), to(5, 5)), // f2-f3
            (at(4, 1), to(4, 3)), // e7-e5
            (at(6, 6), to(6, 4)), // g2-g4
        ] {
            board
                .make_move(GameMove { from, move_type: dest })
                .unwrap();
        }
        // Pre-move status of this position is `Ongoing`; the handler
        // must report the *post-move* status of the mating move.
        let req = GetNewBoardStateRequest {
            board_fen: board_to_fen(&board),
            game_move: GameMove {
                from: at(3, 0), // d8
                move_type: to(7, 4), // h4 — Qd8-h4#
            },
        };

        let resp = get_new_board_state_handler(Json(req)).await;
        assert_eq!(resp.status(), StatusCode::OK);

        let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .expect("read response body");
        let body: GetNewBoardStateResponse =
            serde_json::from_slice(&bytes).expect("parse response JSON");
        assert!(
            matches!(
                body.status,
                GameStatus::Checkmate { winner: Color::Black }
            ),
            "expected post-move Checkmate(Black), got {:?}",
            body.status
        );
    }
}

#[cfg(test)]
mod game_tests {
    use super::*;
    use crate::game::{AppState, AuthPlayer, CreateGame, GameResult};
    use engine::board::MoveType;
    use uuid::Uuid;

    fn player(name: &str) -> AuthPlayer {
        AuthPlayer {
            id: Uuid::new_v4(),
            name: name.to_string(),
        }
    }

    fn req(public: bool) -> CreateGame {
        CreateGame {
            name: None,
            public,
            starting_fen: None,
            color: None,
        }
    }

    /// e2-e4 for White (engine coords: white pawns on rank 6, advancing to
    /// lower ranks).
    fn e2e4() -> GameMove {
        GameMove {
            from: Coord { file: 4, rank: 6 },
            move_type: MoveType::MoveTo(Coord { file: 4, rank: 4 }),
        }
    }

    #[test]
    fn create_join_and_first_move() {
        let state = AppState::new();
        let alice = player("Alice");
        let bob = player("Bob");

        let g = state.create(&alice, req(true)).unwrap();
        assert_eq!(g.your_color, Some(Color::White));
        assert!(g.white.is_some() && g.black.is_none());
        assert_eq!(state.list_public().len(), 1, "open public game is listed");

        // Bob joins as Black via the share code.
        let j = state.join(&g.code, &bob).unwrap();
        assert_eq!(j.your_color, Some(Color::Black));
        assert!(
            state.list_public().is_empty(),
            "a full game leaves the lobby"
        );

        let after = state.apply_move(&g.id.to_string(), &alice, e2e4()).unwrap();
        assert_eq!(after.side_to_move, Color::Black, "turn passed to Black");
        assert_eq!(after.ply, 1);
        assert_eq!(
            after.your_color,
            Some(Color::White),
            "the move response echoes the mover's colour"
        );
    }

    #[test]
    fn turn_and_seat_enforcement() {
        let state = AppState::new();
        let alice = player("Alice"); // White
        let bob = player("Bob"); // Black
        let eve = player("Eve"); // not seated
        let g = state.create(&alice, req(false)).unwrap();
        state.join(&g.code, &bob).unwrap();
        let id = g.id.to_string();

        // Black can't move on White's turn; a stranger can't move at all.
        assert!(matches!(
            state.apply_move(&id, &bob, e2e4()),
            Err(GameActionError::NotYourTurn)
        ));
        assert!(matches!(
            state.apply_move(&id, &eve, e2e4()),
            Err(GameActionError::NotSeated)
        ));
        // White can.
        assert!(state.apply_move(&id, &alice, e2e4()).is_ok());
    }

    #[test]
    fn join_full_is_rejected_and_resign_ends_the_game() {
        let state = AppState::new();
        let alice = player("Alice");
        let bob = player("Bob");
        let eve = player("Eve");
        let g = state.create(&alice, req(true)).unwrap();
        state.join(&g.code, &bob).unwrap();
        let id = g.id.to_string();

        // A third player can't take a full game; re-joining as a seated
        // player is idempotent.
        assert!(matches!(
            state.join(&g.code, &eve),
            Err(GameActionError::Full)
        ));
        // Idempotent re-join returns the seated player's snapshot with the
        // correct colour (not just Ok).
        assert_eq!(
            state.join(&g.code, &alice).unwrap().your_color,
            Some(Color::White)
        );

        // A non-seated player can't resign the game.
        assert!(matches!(
            state.resign(&id, &eve),
            Err(GameActionError::NotSeated)
        ));

        // Resigning ends the game with the opponent as winner; further
        // moves are rejected.
        let r = state.resign(&id, &alice).unwrap();
        assert_eq!(
            r.result,
            Some(GameResult::Resignation {
                winner: Color::Black
            })
        );
        assert!(matches!(
            state.apply_move(&id, &alice, e2e4()),
            Err(GameActionError::Over)
        ));
        // Resigning an already-finished game is likewise Over.
        assert!(matches!(
            state.resign(&id, &alice),
            Err(GameActionError::Over)
        ));
    }

    #[test]
    fn create_as_black_seats_host_as_black() {
        let state = AppState::new();
        let alice = player("Alice");
        let g = state
            .create(
                &alice,
                CreateGame {
                    name: None,
                    public: true,
                    starting_fen: None,
                    color: Some(Color::Black),
                },
            )
            .unwrap();
        assert_eq!(g.your_color, Some(Color::Black));
        assert!(g.black.as_ref().is_some_and(|p| p.id == alice.id));
        assert!(g.white.is_none(), "the white seat is left open for the joiner");
    }

    #[test]
    fn join_by_uuid_and_unknown_id_rejected() {
        let state = AppState::new();
        let alice = player("Alice");
        let bob = player("Bob");
        let g = state.create(&alice, req(true)).unwrap();

        // The raw game UUID resolves the same game as the share code.
        let j = state.join(&g.id.to_string(), &bob).unwrap();
        assert_eq!(j.id, g.id);
        assert_eq!(j.your_color, Some(Color::Black));

        // An unknown id/code → NotFound.
        assert!(matches!(
            state.join(&Uuid::new_v4().to_string(), &bob),
            Err(GameActionError::NotFound)
        ));
    }

    #[test]
    fn create_with_bad_starting_fen_rejected() {
        let state = AppState::new();
        let alice = player("Alice");
        let r = state.create(
            &alice,
            CreateGame {
                name: None,
                public: true,
                starting_fen: Some("invalid".to_string()),
                color: None,
            },
        );
        assert!(matches!(r, Err(GameActionError::BadFen(_))));
    }

    #[test]
    fn illegal_move_is_rejected_by_the_engine() {
        let state = AppState::new();
        let alice = player("Alice"); // White
        let bob = player("Bob");
        let g = state.create(&alice, req(true)).unwrap();
        state.join(&g.code, &bob).unwrap();

        // White's e-pawn can't step sideways onto its own f-pawn — the engine
        // rejects it, surfacing as GameActionError::Move (turn already OK).
        let illegal = GameMove {
            from: Coord { file: 4, rank: 6 },
            move_type: MoveType::MoveTo(Coord { file: 5, rank: 6 }),
        };
        assert!(matches!(
            state.apply_move(&g.id.to_string(), &alice, illegal),
            Err(GameActionError::Move(_))
        ));
    }

    #[test]
    fn list_public_hides_private_and_finished_games() {
        let state = AppState::new();
        let alice = player("Alice");

        // Private games are never listed.
        state
            .create(
                &alice,
                CreateGame {
                    name: None,
                    public: false,
                    starting_fen: None,
                    color: None,
                },
            )
            .unwrap();
        assert!(state.list_public().is_empty(), "private game is not listed");

        // A public open game is listed, then leaves the lobby once finished.
        let pub_g = state.create(&alice, req(true)).unwrap();
        assert_eq!(state.list_public().len(), 1);
        state.resign(&pub_g.id.to_string(), &alice).unwrap();
        assert!(
            state.list_public().is_empty(),
            "a finished game leaves the lobby"
        );
    }

    #[test]
    fn stalemate_is_recorded_as_a_draw() {
        let state = AppState::new();
        let alice = player("Alice"); // White
        let bob = player("Bob"); // Black
        // White queen b3, lone kings; Qb3-b6 stalemates Black (no legal move,
        // not in check). Engine coords: rank 0 is the top (black) row.
        let g = state
            .create(
                &alice,
                CreateGame {
                    name: None,
                    public: true,
                    starting_fen: Some("k7/8/8/8/8/1Q6/8/K7 w - -".to_string()),
                    color: None,
                },
            )
            .unwrap();
        state.join(&g.code, &bob).unwrap();

        let qb6 = GameMove {
            from: Coord { file: 1, rank: 5 },
            move_type: MoveType::MoveTo(Coord { file: 1, rank: 2 }),
        };
        let after = state.apply_move(&g.id.to_string(), &alice, qb6).unwrap();
        assert!(
            matches!(after.status, GameStatus::Stalemate),
            "expected stalemate, got {:?}",
            after.status
        );
        assert_eq!(after.result, Some(GameResult::Draw));
        assert!(
            state.list_public().is_empty(),
            "the drawn game leaves the lobby"
        );
    }

    #[test]
    fn snapshot_without_viewer_has_no_your_color() {
        let state = AppState::new();
        let alice = player("Alice");
        let g = state.create(&alice, req(true)).unwrap();
        // Viewerless snapshots (lobby list, WS broadcasts) carry no your_color.
        assert_eq!(state.get(&g.id.to_string(), None).unwrap().your_color, None);
        assert_eq!(state.list_public()[0].your_color, None);
    }

    #[test]
    fn join_seats_white_when_creator_chose_black() {
        let state = AppState::new();
        let alice = player("Alice");
        let bob = player("Bob");
        let g = state
            .create(
                &alice,
                CreateGame {
                    name: None,
                    public: true,
                    starting_fen: None,
                    color: Some(Color::Black),
                },
            )
            .unwrap();
        let j = state.join(&g.code, &bob).unwrap();
        assert_eq!(
            j.your_color,
            Some(Color::White),
            "the joiner takes the open White seat"
        );
        assert!(
            j.black.as_ref().is_some_and(|p| p.id == alice.id),
            "the creator stays Black"
        );
    }

    #[test]
    fn cannot_join_a_finished_game() {
        let state = AppState::new();
        let alice = player("Alice");
        let bob = player("Bob");
        // Alice creates and resigns while alone → finished, one seat open.
        let g = state.create(&alice, req(false)).unwrap();
        state.resign(&g.id.to_string(), &alice).unwrap();
        // A new player can't take the open seat in a finished game.
        assert!(matches!(
            state.join(&g.code, &bob),
            Err(GameActionError::Over)
        ));
        // But the seated player may still re-fetch their snapshot.
        assert!(state.join(&g.code, &alice).is_ok());
    }

    #[test]
    fn create_defaults_for_blank_fen_and_name() {
        let state = AppState::new();
        let alice = player("Alice");
        let g = state
            .create(
                &alice,
                CreateGame {
                    name: Some("   ".to_string()),
                    public: true,
                    starting_fen: Some("  ".to_string()),
                    color: None,
                },
            )
            .unwrap();
        assert!(
            g.fen.starts_with("rnbqkbnr/pppppppp"),
            "blank starting FEN falls back to the standard position"
        );
        assert_eq!(g.name, "Alice's game", "blank name falls back to the default");
    }

    #[test]
    fn duck_chess_two_part_turn_over_the_wire() {
        let state = AppState::new();
        let alice = player("Alice"); // White
        let bob = player("Bob"); // Black
        let g = state
            .create(
                &alice,
                CreateGame {
                    name: None,
                    public: true,
                    starting_fen: Some("4k3/8/8/8/8/8/8/4K3 w KQkq - variants=duck_chess".to_string()),
                    color: None,
                },
            )
            .unwrap();
        state.join(&g.code, &bob).unwrap();
        let id = g.id.to_string();

        // White's PIECE half-move (king e1-e2) keeps White on the move.
        let king = GameMove {
            from: Coord { file: 4, rank: 7 },
            move_type: MoveType::MoveTo(Coord { file: 4, rank: 6 }),
        };
        assert_eq!(
            state.apply_move(&id, &alice, king).unwrap().side_to_move,
            Color::White,
            "a Duck-Chess piece move does not pass the turn"
        );
        // Black still can't move during White's duck phase.
        let bk = GameMove {
            from: Coord { file: 4, rank: 0 },
            move_type: MoveType::MoveTo(Coord { file: 4, rank: 1 }),
        };
        assert!(matches!(
            state.apply_move(&id, &bob, bk),
            Err(GameActionError::NotYourTurn)
        ));
        // White's DUCK half-move completes the turn → flips to Black.
        let duck = GameMove {
            from: Coord { file: 0, rank: 0 },
            move_type: MoveType::PlaceDuck {
                to: Coord { file: 3, rank: 3 },
            },
        };
        assert_eq!(
            state.apply_move(&id, &alice, duck).unwrap().side_to_move,
            Color::Black,
            "the duck half-move passes the turn"
        );
    }

    #[test]
    fn move_broadcasts_a_snapshot_to_subscribers() {
        let state = AppState::new();
        let alice = player("Alice");
        let bob = player("Bob");
        let g = state.create(&alice, req(true)).unwrap();
        state.join(&g.code, &bob).unwrap();
        let id = g.id.to_string();

        // Subscribe (as a WS handler would), then move and confirm the push.
        let (_initial, mut rx) = state.subscribe_game(&id).unwrap();
        state.apply_move(&id, &alice, e2e4()).unwrap();
        let pushed = rx
            .try_recv()
            .expect("a snapshot is broadcast to subscribers on a move");
        assert_eq!(pushed.ply, 1);
        assert_eq!(pushed.side_to_move, Color::Black);
        assert_eq!(
            pushed.your_color, None,
            "broadcast snapshots are viewer-agnostic"
        );
    }

    #[tokio::test]
    async fn auth_player_extractor_requires_a_player_id() {
        use axum::extract::FromRequestParts;
        use axum::http::Request;

        // Missing X-Player-Id → 401.
        let (mut parts, _) = Request::builder().body(()).unwrap().into_parts();
        let err = AuthPlayer::from_request_parts(&mut parts, &())
            .await
            .unwrap_err();
        assert_eq!(err.0, StatusCode::UNAUTHORIZED);

        // Valid id, no name → "Anonymous".
        let id = Uuid::new_v4();
        let (mut parts, _) = Request::builder()
            .header("x-player-id", id.to_string())
            .body(())
            .unwrap()
            .into_parts();
        let p = AuthPlayer::from_request_parts(&mut parts, &())
            .await
            .unwrap();
        assert_eq!(p.id, id);
        assert_eq!(p.name, "Anonymous");

        // A provided name is trimmed.
        let (mut parts, _) = Request::builder()
            .header("x-player-id", id.to_string())
            .header("x-player-name", "  Bob  ")
            .body(())
            .unwrap()
            .into_parts();
        let p = AuthPlayer::from_request_parts(&mut parts, &())
            .await
            .unwrap();
        assert_eq!(p.name, "Bob");
    }

    #[test]
    fn resign_by_black_makes_white_the_winner() {
        let state = AppState::new();
        let alice = player("Alice"); // White
        let bob = player("Bob"); // Black
        let g = state.create(&alice, req(true)).unwrap();
        state.join(&g.code, &bob).unwrap();
        let r = state.resign(&g.id.to_string(), &bob).unwrap();
        assert_eq!(
            r.result,
            Some(GameResult::Resignation {
                winner: Color::White
            })
        );
    }

    #[test]
    fn a_seated_player_may_move_before_the_opponent_joins() {
        let state = AppState::new();
        let alice = player("Alice"); // White, alone
        let g = state.create(&alice, req(true)).unwrap();
        let after = state.apply_move(&g.id.to_string(), &alice, e2e4()).unwrap();
        assert_eq!(after.ply, 1);
        assert_eq!(after.side_to_move, Color::Black);
    }

    #[test]
    fn join_codes_are_uppercase_hex_and_case_insensitive() {
        let state = AppState::new();
        let alice = player("Alice");
        let bob = player("Bob");
        let g = state.create(&alice, req(true)).unwrap();

        // 6 uppercase-hex characters.
        assert_eq!(g.code.len(), 6);
        assert!(
            g.code.chars().all(|c| c.is_ascii_digit() || ('A'..='F').contains(&c)),
            "code is 6 uppercase hex chars: {}",
            g.code
        );
        // Resolution lower-cases the lookup key, and a non-UUID falls through
        // to the code map (→ None when absent).
        assert_eq!(state.join(&g.code.to_lowercase(), &bob).unwrap().id, g.id);
        // Whitespace-padded codes resolve too (resolve trims the token).
        assert_eq!(
            state.get(&format!("  {}  ", g.code), None).unwrap().id,
            g.id
        );
        assert!(state.get("not-a-uuid", None).is_none());
    }

    #[test]
    fn get_reflects_the_viewer_color() {
        let state = AppState::new();
        let alice = player("Alice");
        let bob = player("Bob");
        let eve = player("Eve");
        let g = state.create(&alice, req(true)).unwrap();
        state.join(&g.code, &bob).unwrap();
        let id = g.id.to_string();
        assert_eq!(
            state.get(&id, Some(alice.id)).unwrap().your_color,
            Some(Color::White)
        );
        assert_eq!(
            state.get(&id, Some(bob.id)).unwrap().your_color,
            Some(Color::Black)
        );
        assert_eq!(
            state.get(&id, Some(eve.id)).unwrap().your_color,
            None,
            "a non-seated viewer has no colour"
        );
    }

    #[test]
    fn list_public_excludes_full_games() {
        let state = AppState::new();
        let alice = player("Alice");
        let bob = player("Bob");
        let open = state.create(&alice, req(true)).unwrap();
        let to_fill = state.create(&alice, req(true)).unwrap();
        state.join(&to_fill.code, &bob).unwrap(); // fill the second game
        let listed: Vec<_> = state.list_public().into_iter().map(|g| g.id).collect();
        assert_eq!(listed, vec![open.id], "only the un-filled public game is listed");
    }

    #[test]
    fn ply_increments_across_sequential_moves() {
        let state = AppState::new();
        let alice = player("Alice");
        let bob = player("Bob");
        let g = state.create(&alice, req(true)).unwrap();
        state.join(&g.code, &bob).unwrap();
        let id = g.id.to_string();
        let mv = |ff, fr, tf, tr| GameMove {
            from: Coord { file: ff, rank: fr },
            move_type: MoveType::MoveTo(Coord { file: tf, rank: tr }),
        };
        assert_eq!(state.apply_move(&id, &alice, mv(4, 6, 4, 4)).unwrap().ply, 1); // e2-e4
        assert_eq!(state.apply_move(&id, &bob, mv(4, 1, 4, 3)).unwrap().ply, 2); // e7-e5
        assert_eq!(state.apply_move(&id, &alice, mv(6, 7, 5, 5)).unwrap().ply, 3); // Ng1-f3
    }

    #[test]
    fn black_to_move_fen_enforces_the_right_turn() {
        let state = AppState::new();
        let alice = player("Alice"); // White
        let bob = player("Bob"); // Black, to move first
        // A Black-to-move position (Black can play Ka8-a7).
        let g = state
            .create(
                &alice,
                CreateGame {
                    name: None,
                    public: true,
                    starting_fen: Some("k7/8/8/8/8/1Q6/8/K7 b - -".to_string()),
                    color: None,
                },
            )
            .unwrap();
        state.join(&g.code, &bob).unwrap();
        let id = g.id.to_string();
        assert_eq!(g.side_to_move, Color::Black);
        assert_eq!(g.your_color, Some(Color::White), "the creator is White");

        // White can't move — it's Black's turn (the colour check fires first).
        let wk = GameMove {
            from: Coord { file: 0, rank: 7 },
            move_type: MoveType::MoveTo(Coord { file: 1, rank: 7 }),
        };
        assert!(matches!(
            state.apply_move(&id, &alice, wk),
            Err(GameActionError::NotYourTurn)
        ));
        // Black moves Ka8-a7 → the turn passes to White.
        let bk = GameMove {
            from: Coord { file: 0, rank: 0 },
            move_type: MoveType::MoveTo(Coord { file: 0, rank: 1 }),
        };
        assert_eq!(
            state.apply_move(&id, &bob, bk).unwrap().side_to_move,
            Color::White
        );
    }

    #[test]
    fn black_creator_can_move_before_white_joins() {
        let state = AppState::new();
        let alice = player("Alice"); // Black creator, to move first
        let g = state
            .create(
                &alice,
                CreateGame {
                    name: None,
                    public: true,
                    starting_fen: Some("k7/8/8/8/8/1Q6/8/K7 b - -".to_string()),
                    color: Some(Color::Black),
                },
            )
            .unwrap();
        assert_eq!(g.your_color, Some(Color::Black));
        let bk = GameMove {
            from: Coord { file: 0, rank: 0 },
            move_type: MoveType::MoveTo(Coord { file: 0, rank: 1 }),
        };
        let after = state.apply_move(&g.id.to_string(), &alice, bk).unwrap();
        assert_eq!(after.side_to_move, Color::White);
        assert_eq!(after.ply, 1);
    }

    #[test]
    fn checkmate_by_move_sets_a_decisive_result() {
        let state = AppState::new();
        let alice = player("Alice"); // White, delivers a back-rank mate
        let bob = player("Bob");
        let g = state
            .create(
                &alice,
                CreateGame {
                    name: None,
                    public: true,
                    starting_fen: Some("6k1/5ppp/8/8/8/8/8/4R1K1 w - -".to_string()),
                    color: None,
                },
            )
            .unwrap();
        state.join(&g.code, &bob).unwrap();
        // Re1-e8#.
        let re8 = GameMove {
            from: Coord { file: 4, rank: 7 },
            move_type: MoveType::MoveTo(Coord { file: 4, rank: 0 }),
        };
        let after = state.apply_move(&g.id.to_string(), &alice, re8).unwrap();
        assert!(
            matches!(after.status, GameStatus::Checkmate { winner: Color::White }),
            "expected checkmate, got {:?}",
            after.status
        );
        assert_eq!(
            after.result,
            Some(GameResult::Decisive {
                winner: Color::White
            })
        );
        assert!(state.list_public().is_empty(), "a decided game leaves the lobby");
    }

    #[test]
    fn terminal_starting_fen_opens_already_finished() {
        let state = AppState::new();
        let alice = player("Alice");
        let bob = player("Bob");
        // A FEN that is already checkmate (Black mated, White won).
        let g = state
            .create(
                &alice,
                CreateGame {
                    name: None,
                    public: true,
                    starting_fen: Some("k1R5/8/1K6/8/8/8/8/8 b - -".to_string()),
                    color: None,
                },
            )
            .unwrap();
        assert_eq!(
            g.result,
            Some(GameResult::Decisive {
                winner: Color::White
            }),
            "a mated starting position opens finished"
        );
        assert!(state.list_public().is_empty(), "a finished game is never listed");
        assert!(matches!(
            state.join(&g.code, &bob),
            Err(GameActionError::Over)
        ));
    }

    #[test]
    fn lobby_broadcasts_on_game_events() {
        let state = AppState::new();
        let alice = player("Alice");
        let bob = player("Bob");
        let mut rx = state.subscribe_lobby();
        let g = state.create(&alice, req(true)).unwrap();
        assert!(rx.try_recv().is_ok(), "creating a game pings the lobby");
        state.join(&g.code, &bob).unwrap();
        assert!(rx.try_recv().is_ok(), "joining a game pings the lobby");
        state.resign(&g.id.to_string(), &alice).unwrap();
        assert!(rx.try_recv().is_ok(), "ending a game pings the lobby");
    }

    #[test]
    fn actions_on_an_unknown_game_are_not_found() {
        let state = AppState::new();
        let alice = player("Alice");
        let unknown = Uuid::new_v4().to_string();
        assert!(matches!(
            state.apply_move(&unknown, &alice, e2e4()),
            Err(GameActionError::NotFound)
        ));
        assert!(matches!(
            state.resign(&unknown, &alice),
            Err(GameActionError::NotFound)
        ));
        assert!(state.get(&unknown, Some(alice.id)).is_none());
    }

    #[test]
    fn idempotent_rejoin_reflects_current_game_state() {
        let state = AppState::new();
        let alice = player("Alice");
        let bob = player("Bob");
        let g = state.create(&alice, req(true)).unwrap();
        state.join(&g.code, &bob).unwrap();
        let id = g.id.to_string();
        state.apply_move(&id, &alice, e2e4()).unwrap(); // ply 1, Black to move

        // Re-joining mid-game returns the *current* state, not the start.
        let rejoin = state.join(&g.code, &alice).unwrap();
        assert_eq!(rejoin.ply, 1);
        assert_eq!(rejoin.side_to_move, Color::Black);
        assert_eq!(rejoin.your_color, Some(Color::White));
    }

    #[test]
    fn get_and_subscribe_resolve_by_join_code() {
        let state = AppState::new();
        let alice = player("Alice");
        let g = state.create(&alice, req(true)).unwrap();
        // The read paths accept the share code, not just the UUID.
        assert_eq!(state.get(&g.code, Some(alice.id)).unwrap().id, g.id);
        assert!(state.subscribe_game(&g.code).is_some());
    }

    #[test]
    fn game_error_response_maps_each_variant_to_its_status() {
        use crate::game::GameActionError as E;
        let cases: [(E, StatusCode); 8] = [
            (E::NotFound, StatusCode::NOT_FOUND),
            (E::BadFen("x".to_string()), StatusCode::BAD_REQUEST),
            (E::NotSeated, StatusCode::FORBIDDEN),
            (E::Full, StatusCode::CONFLICT),
            (E::NotYourTurn, StatusCode::FORBIDDEN),
            (E::Over, StatusCode::CONFLICT),
            // The engine-rejection arm builds the richest body (code +
            // message + serialized `details`); exercise its full mapping.
            (
                E::Move(MoveError::NoSourceSquare {
                    from: Coord { file: 0, rank: 0 },
                }),
                StatusCode::BAD_REQUEST,
            ),
            (E::Internal("x".to_string()), StatusCode::INTERNAL_SERVER_ERROR),
        ];
        for (err, status) in cases {
            assert_eq!(game_error_response(err).status(), status);
        }
    }
}
