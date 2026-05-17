# Plan 06: API evolution

The API is `api/src/main.rs` — a small axum service with two endpoints.
After plans 01-03 land, the engine will have richer state and the API
needs to keep up.

## Current state

```
POST /board/moves         { board_fen, from } -> { moves }
POST /board/new_state     { board_fen, game_move } -> { new_board_fen }
```

Both are stateless — the client owns the game state and passes FEN on
each call. `get_new_board_state_handler` now returns `400 Bad Request`
with the engine's error string when `make_move` fails (recently
fixed).

## What needs to change

### Driven by plan 01 (turn system)

If FEN gets extended to include side-to-move (recommended in plan 01),
both endpoints' request shape stays the same — clients just include
the new byte. If not, both requests need an explicit `side_to_move`
field.

### Driven by plan 02 (king-safety / game-over)

A new endpoint for game status:

```
POST /board/status        { board_fen } -> { status: GameStatus }
```

Where `GameStatus` is the enum from plan 02 (Ongoing / Checkmate /
Stalemate / BrainrotWin). Serde JSON of the enum is fine — adjacently
tagged works.

> **Update — shipped.** The landed enum is `Ongoing` /
> `Check { side_to_move }` / `Checkmate { winner }` / `Stalemate` —
> there's an extra `Check` variant this plan didn't anticipate and
> **no `BrainrotWin`** (deferred to plan 04, per the doc-comment on
> `Board::status()`). It derives serde with
> `#[serde(tag = "status", content = "data")]`, e.g.
> `{"status":"Checkmate","data":{"winner":"White"}}` /
> `{"status":"Ongoing"}`. Folded into `/board/new_state` (status of the
> position *after* the move) **and** exposed as `POST /board/status`
> `{ board_fen } -> { status }`, which reuses the same
> structured-400-on-bad-FEN contract. Shape pinned by
> `game_status_json_shape` in the engine test suite.

Alternatively, fold status into `/board/new_state`'s response:

```
{ new_board_fen, status }
```

The latter is slightly nicer (status comes with every move
automatically), but the dedicated endpoint is useful for clients that
load a FEN and want to know "is this game over?".

Recommendation: do both. Fold status into `/new_state` response, and
add `/status` as a separate endpoint for ad-hoc queries.

### Driven by plan 05 (FEN errors)

`FenError` should serialize to a JSON error response with a structured
shape, not a bare string:

```json
{ "error": "BadRowWidth", "row": 3, "expected": 8, "found": 9 }
```

Use `serde` derive on `FenError` and `IntoResponse` for axum.

> **Update — plan 05 shipped a different (intentional) shape.** The
> landed contract is `FenErrorBody { code, message, fen }` (e.g.
> `{"code":"fen_bad_row_width","message":"Row 0 is …","fen":"…"}`),
> mapped by a hand-written `fen_error_code(&FenError)`. `FenError`
> itself is **not** `serde`-derived and there is no `IntoResponse`;
> the shape deliberately mirrors `MakeMoveErrorBody` so clients branch
> on a stable string `code`. If richer per-variant fields (`row`,
> `expected`, `found`) are wanted later, **extend `FenErrorBody`** —
> do not replace it with a serde-flattened `FenError`, since clients
> now depend on `code`. The 400-on-bad-FEN deliverable is done; only
> the optional per-variant detail fields remain.

### Driven by plan 03 (promotion etc.)

Promotion needs a way to pick the target piece. If `MoveType::Promotion`
gets a `PromotionTarget` field (per plan 03), clients send it as part
of the `game_move`. No API shape change beyond what serde gives.

## Bigger design questions

### Stateless vs stateful

Today the client owns the FEN. As game features grow (move history,
clock, multiplayer), the API will probably want to own a game.

```
POST /games                            -> { game_id, fen }
GET  /games/{id}                       -> { fen, status, history }
POST /games/{id}/moves                 { game_move } -> { fen, status }
POST /games/{id}/resign                -> { status }
```

This is a much larger change. **Not for now.** The stateless model is
fine while there's no clock, no multiplayer, no persistence. Worth
flagging as the future destination so we don't paint into a corner.

### Server-side validation vs client-side

Today the API trusts the FEN sent by the client. That's fine for a
toy. If/when this becomes multiplayer, the server has to be the
authority — it can't accept arbitrary FEN from a player, only valid
move-by-move transitions.

That implies: API owns game state, plus a "make move" endpoint that
takes a move (not a FEN) and applies it to the canonical game state.
Same design path as "stateless vs stateful" above.

### Serde JSON for full Board state

Plan 01 mentions extending FEN to include side-to-move + castle rights.
That works for ~chess data. As fairy-piece state grows (Goblin home
square, Skibidi phase, Bus passengers, future custom state), the FEN
format gets harder to keep readable. Consider a JSON board format as
a parallel option:

```json
{
  "grid": [...],
  "flags": { "side_to_move": "white", "castles": {...} }
}
```

Already half-true — `Coord`, `MoveType`, `GameMove` derive
`Serialize`/`Deserialize`. `Board` and `BoardFlags` don't yet. Adding
those derives is one or two lines per type.

Recommendation: add the derives now (it's free); leave the API on FEN
for the moment; introduce a JSON board format the moment it stops being
trivial to extend the FEN.

> **Update — derives shipped.** `Board`, `BoardFlags`, `LastMove`,
> `Square`, `SquareType`, `SquareCondition`, `PressureTrigger`,
> `PieceType` and the full piece graph (standard + fairy + chess2,
> incl. carriers with nested `PieceType`) now derive
> `Serialize`/`Deserialize`. `Coord` / `Color` / `TrackDir` /
> `TrainHeading` / `MoveType` / `GameMove` already did. Lossless JSON
> round-trip is smoke-tested by `serde_json_board_roundtrip` (standard
> position, nested-bus, loco+cart+passenger, tornado condition,
> capture-bearing `last_move`). A 23-type sweep rather than the "one or
> two lines" estimated above — mechanical, and `serde_json` is a
> dev-dependency only (the round-trip test), no new runtime dep. The
> API stays on FEN; the JSON board format itself is still deferred.
>
> **Constraint — a `Deserialize`d `Board` is structurally
> unvalidated.** Unlike the FEN path, derived `Deserialize` is a free
> constructor: it can build a ragged grid or out-of-range fairy state
> (Skibidi `phase`, Carriage `chain_index`, Junction `state`) that the
> FEN parser would reject. It is memory-safe (all grid access is
> bounds-checked) but engine-invalid. Only FEN-sourced boards are
> trusted today and there is **no** `Board`-from-JSON ingress (the API
> takes `board_fen`, never a serialized `Board`), so this is latent,
> not active. Any future JSON-board ingress must validate (or use
> `#[serde(try_from)]`) before the board reaches engine logic.

## CORS, auth, etc.

The API allows all origins (`CorsLayer::new().allow_origin("*")`),
which is appropriate for local dev and dangerous for anything else.
Flag for whenever this goes anywhere near a real deployment.

## Sequencing

1. ~~Add `Serialize`/`Deserialize` to `Board` and `BoardFlags`.~~
   **Done** — extended across the whole `grid`/`flags` graph (23
   types); lossless JSON round-trip smoke-tested. See the "derives
   shipped" note above. The JSON *board format* (vs FEN on the wire) is
   still intentionally deferred.
2. ~~After plan 02 lands: include `GameStatus` in `/new_state` response
   and add `/board/status` endpoint.~~ **Done** — both shipped; see the
   "shipped" note under "Driven by plan 02" above for the real enum
   shape + JSON tagging.
3. ~~After plan 05 lands: switch error responses to structured
   `FenError` JSON.~~ **Done** — plan 05 shipped `FenErrorBody
   { code, message, fen }` + 400 (see the note under "Driven by plan
   05" above; only optional per-variant detail fields remain).
4. Whenever multiplayer / persistence / clocks come up: redesign as
   stateful with game IDs. **Still open** — explicitly out of scope
   (see "Stateless vs stateful" above).
