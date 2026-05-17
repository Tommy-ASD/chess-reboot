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
    Coord, GameMove, GameStatus, MoveError,
    fen::{FenError, board_to_fen, fen_to_board},
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
    /// player who just moved, **not** the side now to move. Folded in so
    /// a client gets check/checkmate/stalemate with every move without a
    /// follow-up `/board/status` round-trip. Adjacently tagged (see
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
        .route("/board/status", post(get_status_handler))
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
