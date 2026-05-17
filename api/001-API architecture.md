# API architecture

Stateless axum service — the client owns game state and passes a board
FEN on every call. All endpoints are `POST`. Dev base URL is
`http://localhost:8080` (binds `0.0.0.0:8080`). A malformed FEN returns
`400` with a structured `FenErrorBody { code, message, fen }`.

- `POST /board/moves` — `{ board_fen, from }` → `{ moves }`. Legal
  moves for the piece on the `from` square.
- `POST /board/new_state` — `{ board_fen, game_move }` →
  `{ new_board_fen, status }`. Applies the move; an illegal move
  returns `400` with `MakeMoveErrorBody`. `status` is the `GameStatus`
  of the position *after* the move.
- `POST /board/status` — `{ board_fen }` → `{ status }`. Ad-hoc game
  status for a held FEN, no move applied.

`GameStatus` is adjacently tagged, e.g.
`{"status":"Checkmate","data":{"winner":"White"}}` or
`{"status":"Ongoing"}`.
