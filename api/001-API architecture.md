# API architecture

An axum service (`api/src/main.rs`) with two families of endpoints:

- **Stateless `/board/*`** — the client owns the state and passes a FEN on
  every call; the server parses it, runs the engine, returns a result.
  Good for tools and one-off queries.
- **Stateful `/games/*`** — the *server* owns the game (plan 06 step 4):
  it holds the canonical `Board`, applies only validated moves, and tracks
  history + resignation. In-memory only — no persistence, no auth, game IDs
  are a monotonic counter.

Served on `0.0.0.0:8080`. CORS is `allow_origin("*")` — fine for local
dev, **not** for any real deployment.

## Stateless endpoints (`/board/*`)

All three are `POST` with a JSON body and a JSON response.

### `POST /board/moves` — legal moves for a square

```
req:  { "board_fen": <fen>, "from": Coord }
resp: { "moves": [GameMove, ...] }
```

Returns king-safety-filtered moves (`Board::legal_moves`), **not** raw
geometry — every move returned here is one `/board/new_state` will
accept. A pinned piece with no safe move returns `{ "moves": [] }`.

### `POST /board/new_state` — apply a move

```
req:  { "board_fen": <fen>, "game_move": GameMove }
resp (200): { "new_board_fen": <fen>, "status": GameStatus }
resp (400): error body  (see "Error body" below)
```

`status` is computed *after* the move, from the perspective of the side
now to move — so a mating move returns `{"Checkmate":{"winner":...}}`
without a follow-up call.

### `POST /board/status` — status of a position

```
req:  { "board_fen": <fen> }
resp: { "status": GameStatus }
```

For a client that loads a FEN and just wants to know "is this game
over?". Status is from the perspective of the side to move in the FEN.

## Stateful endpoints (`/games/*`)

The server owns each game. Every success returns the same shape,
`GameView`:

```
{ "game_id": u64, "fen": <fen>, "status": GameStatus, "history": [GameMove, ...] }
```

`{id}` is a `u64`; a non-numeric path segment is rejected by the router
with a plain-text `400` before reaching the handler (not the JSON error
shapes below).

### `POST /games` — create a game

```
req:  (empty)  |  { "board_fen": <fen> }   // empty body = standard opening
resp (201): GameView                       // history: []
resp (400): { "error": "invalid_request" } // malformed JSON body
```

### `GET /games/{id}` — fetch a game

```
resp (200): GameView
resp (404): { "error": "game_not_found" }
```

### `POST /games/{id}/moves` — apply a move

```
req:  { "game_move": GameMove }
resp (200): GameView        // status reflects the move (may be Checkmate / Stalemate)
resp (400): move error      // engine rejected the move (see Error body)
resp (404): { "error": "game_not_found" }
resp (409): { "error": "game_over" }      // game already finished
```

The move is validated against the canonical board and applied only on
success — a rejected move never mutates the stored game.

### `POST /games/{id}/resign` — resign

```
req:  (empty)  |  { "color": "White" | "Black" }   // empty = side to move resigns
resp (200): GameView        // status: { "Resigned": { "winner": <opponent> } }
resp (400): { "error": "invalid_color" }    // resigning as Neutral (or "invalid_request" on a malformed body)
resp (404): { "error": "game_not_found" }
resp (409): { "error": "game_over" }
```

## Shared JSON shapes

- **`Coord`**: `{ "file": u8, "rank": u8 }`. Rank 0 is the top row of the
  FEN grid (black's back rank); white's back rank is `height - 1`.
- **`GameMove`**: `{ "from": Coord, "move_type": MoveType }`.
- **`MoveType`**: *adjacently* tagged — `#[serde(tag = "kind", content = "target")]` —
  so a variant's payload (if any) nests under a `"target"` content key:
  - `{ "kind": "MoveTo", "target": Coord }` (newtype payload *is* the Coord)
  - `{ "kind": "MoveIntoCarrier", "target": Coord }`
  - `{ "kind": "Promotion", "target": { "target": Coord, "into": "Queen" } }` —
    the promotion piece field is `into` (Q/R/B/N), nested under the content `target`
  - `{ "kind": "Castle", "target": { "side": "Kingside" } }`
  - `{ "kind": "EnPassant", "target": { "target": Coord, "captured": Coord } }`
  - `{ "kind": "ThrowSwitch", "target": { "switch": Coord } }`
  - `{ "kind": "PieceInCarrier", "target": { "piece_index": u8, "move_type": MoveType } }` (recursive)
  - `{ "kind": "PhaseShift" }` — unit variant; **no** `target` key
  (Authoritative source: [`engine/src/board/mod.rs`](../engine/src/board/mod.rs).)
- **`GameStatus`**: externally tagged. Unit variants are bare strings
  (`"Ongoing"`, `"Stalemate"`); data variants wrap an object
  (`{ "Check": { "side_to_move": "White" } }`,
  `{ "Checkmate": { "winner": "Black" } }`,
  `{ "Resigned": { "winner": "White" } }`). `Resigned` only comes from the
  stateful resign path — `Board::status()` never produces it.

The whole `Board` (grid + flags + every piece and square-type payload)
also derives `Serialize`/`Deserialize` (plan 06 step 1) — the wire format
stays FEN for now, but a JSON board format is a drop-in when FEN stops
being trivial to extend.

## Error body (`400 Bad Request`)

When `make_move` rejects a move, the response is a self-contained JSON
object:

```
{
  "code": <short category, e.g. "would_leave_king_in_check">,
  "message": <human-readable explanation>,
  "details": MoveError,        // full structured engine error
  "side_to_move": Color,       // whose turn it was on the received board
  "received": <echo of the request payload>
}
```

`code` mirrors the `MoveError` discriminant so clients can branch without
parsing `message`. The stateful `POST /games/{id}/moves` returns the same
`code` / `message` / `details` (without the `received` / `side_to_move`
echo). (FEN *parse* errors are not yet structured — the parser is
currently lenient and infallible; that's
[plan 05](../plans/05-fen-hardening.md).)

## Conventions

- **Content type.** The endpoints using the typed JSON extractor —
  `POST /board/moves`, `/board/new_state`, `/board/status`, and
  `POST /games/{id}/moves` — require `Content-Type: application/json` and
  reject its absence with `415 Unsupported Media Type`. `POST /games` and
  `POST /games/{id}/resign` read the raw body, so they accept an empty or
  any-typed body (that's how an empty body selects the default).
- **Error-body shapes vary; branch on the status code first.** Move
  rejections return a structured body (`{ code, message, details, ... }`);
  not-found / conflict / invalid-color / invalid-request return
  `{ "error": ... }`. But a malformed-JSON body on a typed endpoint, or a
  non-numeric `{id}`, is rejected by the framework with a **plain-text**
  `400` / `415` — *not* the JSON shapes above.
- **Idempotency.** Moves are not idempotent (replaying advances the game).
  Resigning an already-finished game returns `409`, so treat a `409` after
  a retried resign as "already done", not a hard failure.
- **History replay.** `history` replays onto the game's *creation* FEN
  (the standard opening unless a `board_fen` was supplied at create), not
  necessarily the standard start. `GameView.fen` is always the current
  position, so a client never needs to replay it itself.
- **201 `Location`.** `POST /games` sets `Location: /games/{id}` on the
  `201` response (the id is also in the body as `game_id`).
