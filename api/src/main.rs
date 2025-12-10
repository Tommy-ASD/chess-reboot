use axum::{
    Json, Router,
    routing::{get, post},
};
use http::Method;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;

use engine::board::{
    Board, Coord, GameMove,
    fen::{board_to_fen, fen_to_board},
};

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

#[axum::debug_handler]
async fn get_new_board_state_handler(
    Json(req): Json<GetNewBoardStateRequest>,
) -> Json<GetNewBoardStateResponse> {
    let mut board = fen_to_board(&req.board_fen);
    board.make_move(req.game_move);
    let new_board_fen = board_to_fen(&board);
    Json(GetNewBoardStateResponse { new_board_fen })
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
