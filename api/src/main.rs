use axum::{
    Json, Router,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::post,
};
use http::Method;
use serde::{Deserialize, Serialize};
use tower_http::cors::CorsLayer;

use engine::board::{
    Coord, GameMove, MoveError,
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

#[axum::debug_handler]
async fn get_moves_handler(Json(req): Json<GetMovesRequest>) -> Json<GetMovesResponse> {
    let board = fen_to_board(&req.board_fen);
    let moves = board.get_moves(&req.from);
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
            let new_board_fen = board_to_fen(&board);
            Json(GetNewBoardStateResponse { new_board_fen }).into_response()
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

pub async fn serve_api() {
    let cors = CorsLayer::new()
        .allow_origin("*".parse::<http::HeaderValue>().unwrap()) // allow all — dev only
        .allow_methods([Method::GET, Method::POST])
        .allow_headers([http::header::CONTENT_TYPE]);

    let port = 8080;
    let binding_address = "0.0.0.0".to_string() + ":" + &port.to_string();

    let app = Router::new()
        .route("/board/moves", post(get_moves_handler))
        .route("/board/new_state", post(get_new_board_state_handler))
        .layer(cors);

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
