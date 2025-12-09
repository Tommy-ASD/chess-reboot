use axum::{
    Json, Router,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

use engine::board::{Board, Coord, fen::fen_to_board};

#[derive(Debug, Deserialize)]
pub struct GetMovesRequest {
    pub board_fen: String,
    pub from: Coord,
}

#[derive(Debug, Serialize)]
pub struct GetMovesResponse {
    pub moves: Vec<Coord>,
}

async fn get_moves_handler(Json(req): Json<GetMovesRequest>) -> Json<GetMovesResponse> {
    let board = fen_to_board(&req.board_fen);
    let moves = board.get_moves(&req.from);
    Json(GetMovesResponse { moves })
}

pub async fn serve_api() {
    let port = 8080;
    let binding_address = "0.0.0.0".to_string() + ":" + &port.to_string();

    let app = Router::new().route("/board/moves", post(get_moves_handler));

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
