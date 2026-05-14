use tracing::{debug, trace, warn};

use crate::{
    board::{
        Board, BoardFlags, Coord, SignalId, TrainTickRate,
        square::{PressureTrigger, Square, SquareCondition, SquareType, TrackDir},
    },
    pieces::{Color, piecetype::PieceType},
};

/// Finds the index of the closing parenthesis that matches the opening
/// parenthesis located at `open_index`.
///
/// This function scans forward from `open_index`, keeping track of nested
/// parentheses. It returns the character index of the `)` that closes the
/// parenthesis at `open_index`.
///
/// # Parameters
///
/// - `s`: The input string to search.
/// - `open_index`: The index of the `'('` character whose matching `')'` we want
///   to locate.
///   The character at `open_index` **must be `'('`**, otherwise the depth
///   accounting will not behave as intended.
///
/// # Returns
///
/// - `Some(index)` — the byte index of the matching `)`
/// - `None` — if no matching closing parenthesis exists (unbalanced or malformed data)
///
/// # Examples
///
/// ```ignore
/// let s = "G(H=0-7,P=g(H=0-0))";
///
/// // The first '(' occurs at index 1
/// assert_eq!(find_matching_paren(s, 1), Some(18));
///
/// // A nested example:
/// let s = "foo(bar(baz),qux)";
/// // '(' at index 3 closes at index 15
/// assert_eq!(find_matching_paren(s, 3), Some(15));
///
/// // Unmatched parentheses:
/// let s = "(abc(def)";
/// assert_eq!(find_matching_paren(s, 0), None);
/// ```
///
/// # Notes
///
/// - The function uses `char_indices` and returns a byte index into the original
///   string.
/// - Nested parentheses are fully supported. Depth increases on `'('` and
///   decreases on `')'`.
/// - The match occurs when the depth returns to zero for the first time *after*
///   processing the opening parenthesis at `open_index`.
///
pub fn find_matching_paren(s: &str, open_index: usize) -> Option<usize> {
    // `open_index` is a *byte* index (callers pass `str::find('(')`).
    // Iterate the suffix and add the offset back, so a multi-byte char
    // before `(` doesn't shift the alignment between `skip` (char count)
    // and `find` (byte count). See engine/src/board/tests.rs for the
    // regression test.
    let suffix = s.get(open_index..)?;
    let mut depth: i32 = 0;

    for (i, ch) in suffix.char_indices() {
        match ch {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth < 0 {
                    return None;
                }
                if depth == 0 {
                    return Some(open_index + i);
                }
            }
            _ => {}
        }
    }

    None
}

fn fen_row_to_squares(row: &str) -> Vec<Square> {
    trace!(row, "fen_row_to_squares");

    let mut squares = Vec::new();
    let mut chars = row.chars().peekable();
    let mut index = 0usize;

    while let Some(&ch) = chars.peek() {
        trace!(index, ?ch, "char");

        // -------------------------------
        // 1. DIGITS → run-length empties (multi-digit, e.g. "10" → 10 empty
        //    squares on a 10-wide board). Consume the full digit run greedily.
        // -------------------------------
        if ch.is_ascii_digit() {
            let mut count: u32 = 0;
            while let Some(&peek) = chars.peek() {
                if let Some(d) = peek.to_digit(10) {
                    count = count.saturating_mul(10).saturating_add(d);
                    chars.next();
                    index += 1;
                } else {
                    break;
                }
            }
            trace!(count, "digit run-length");

            for _ in 0..count {
                squares.push(Square {
                    piece: None,
                    square_type: SquareType::Standard,
                    conditions: vec![],
                });
            }
            continue;
        }

        // -------------------------------
        // 2. EXTENDED BLOCK STARTS WITH '('
        // -------------------------------
        if ch == '(' {
            trace!(index, "extended square begins");

            let mut buf = String::new();
            // `i32` for defensive bounds — see find_matching_paren.
            let mut depth: i32 = 0;

            while let Some(c) = chars.next() {
                buf.push(c);

                match c {
                    '(' => {
                        depth += 1;
                    }
                    ')' => {
                        depth -= 1;
                        if depth <= 0 {
                            // Either correctly balanced (==0) or stray
                            // `)` with no preceding `(` (<0). Either
                            // way, stop scanning this block.
                            trace!(buf, "extended block closed");
                            break;
                        }
                    }
                    _ => {}
                }
            }

            trace!(buf, "parsing extended square");
            squares.push(fen_to_square(&buf));

            index += buf.len();
            continue;
        }

        // -------------------------------
        // 3. STANDARD SINGLE-CHAR PIECE
        // -------------------------------
        trace!(?ch, "standard piece");
        squares.push(fen_to_square(&ch.to_string()));

        chars.next();
        index += 1;
    }

    trace!(count = squares.len(), "row complete");
    squares
}

pub fn board_to_fen(board: &Board) -> String {
    let mut rows = vec![];

    for row in &board.grid {
        let mut fen_row = String::new();
        let mut empty_count = 0;

        for square in row {
            let fen = square_to_fen(square);

            if fen.is_empty() || fen == "()" {
                empty_count += 1;
            } else {
                if empty_count > 0 {
                    fen_row.push_str(&empty_count.to_string());
                    empty_count = 0;
                }
                fen_row.push_str(&fen);
            }
        }

        if empty_count > 0 {
            fen_row.push_str(&empty_count.to_string());
        }

        rows.push(fen_row);
    }

    let grid = rows.join("/");
    let stm = match board.flags.side_to_move {
        Color::White => "w",
        Color::Black => "b",
        // Neutral as side-to-move has no meaning. `make_move`'s phase 3
        // also defends against it (debug_assert + warn + coerce to
        // White). Mirror the warn here so encoding a misuse is loud
        // on both sides of the round-trip.
        Color::Neutral => {
            warn!("side_to_move is Neutral at FEN-encode; coercing to 'w'");
            "w"
        }
    };
    let castling = castle_rights_to_fen(&board.flags);
    let ep = match &board.flags.en_passant_target {
        Some(c) => board.format_coord(c),
        None => "-".to_string(),
    };
    let tr = format_train_tick_rate(&board.flags.train_tick_rate);
    let p = format!("p={}", board.flags.ply_count);
    format!("{grid} {stm} {castling} {ep} {tr} {p}")
}

fn format_train_tick_rate(rate: &TrainTickRate) -> String {
    match rate {
        TrainTickRate::EveryPly => "tr=ply".to_string(),
        TrainTickRate::EveryFullTurn => "tr=full".to_string(),
        // `EveryNPly(1)` is behaviorally identical to `EveryPly`
        // (the modulo gate `ply_count % 1` is always 0). The parser
        // normalizes `tr=1ply` to `EveryPly`; mirror that on the
        // encoder side so the canonical wire form is `tr=ply`.
        TrainTickRate::EveryNPly(1) => "tr=ply".to_string(),
        TrainTickRate::EveryNPly(n) => format!("tr={n}ply"),
    }
}

/// Parse a train-tick rate field. Accepts either bare ("full",
/// "ply", "<n>ply") or "tr=…"-prefixed forms.
///
/// **Rejects `tr=0ply`** (and any equivalent like `0ply`):
/// `EveryNPly(0)` would divide by zero in the modulo gate without
/// the runtime clamp at `maybe_advance_trains`, and a round-trip
/// that serializes the zero back into a FEN would lie about the
/// actual behavior. A 0-ply rate is treated as malformed; callers
/// fall back to the default `EveryFullTurn`.
fn parse_train_tick_rate(s: &str) -> Option<TrainTickRate> {
    let body = s.strip_prefix("tr=").unwrap_or(s);
    let parsed = match body {
        "full" => Some(TrainTickRate::EveryFullTurn),
        "ply" => Some(TrainTickRate::EveryPly),
        other => other
            .strip_suffix("ply")
            .and_then(|n| n.parse::<u8>().ok())
            .filter(|n| *n > 0)
            // Normalize the duplicate-state cases: `tr=1ply` is
            // behaviorally identical to `tr=ply` (the modulo gate
            // `ply_count % 1` is always 0). Canonicalize at parse
            // time so two FENs that describe the same execution
            // round-trip through one canonical form.
            .map(|n| if n == 1 {
                TrainTickRate::EveryPly
            } else {
                TrainTickRate::EveryNPly(n)
            }),
    };
    // Only warn when the input *looks* like a train-tick-rate token
    // (uses the engine's `tr=` prefix). Standard 6-token chess FENs
    // shove the halfmove clock and fullmove number into the flag
    // tail; those are pure-numeric and shouldn't trip a warn for
    // every external client (perft databases, opening books, etc.).
    if parsed.is_none() && s.starts_with("tr=") {
        warn!(s, "could not parse train-tick-rate field; ignoring");
    }
    parsed
}

fn parse_ply_count(s: &str) -> Option<u32> {
    let parsed = s.strip_prefix("p=").and_then(|n| n.parse::<u32>().ok());
    // Same rationale as `parse_train_tick_rate`: only warn when the
    // token is prefixed `p=` (engine convention). A bare integer
    // halfmove clock from a standard FEN shouldn't trigger.
    if parsed.is_none() && s.starts_with("p=") {
        warn!(s, "could not parse ply-count field; defaulting to 0");
    }
    parsed
}

/// Parse an algebraic square ("e3") into a Coord. Needs the board height
/// so the algebraic rank (1 = bottom row) inverts correctly: rank 8 →
/// internal rank 0 for an 8-tall board, rank 12 → 0 for a 12-tall board.
/// The file width bound is checked too — algebraic "i9" on a 6-wide board
/// returns `None`.
fn algebraic_to_coord(s: &str, height: u8, width: u8) -> Option<Coord> {
    // Files use letters a..z (so up to 26-wide boards).
    let bytes = s.as_bytes();
    if bytes.len() < 2 {
        return None;
    }
    let file = bytes[0].checked_sub(b'a')?;
    if file >= width {
        return None;
    }
    let rank_str = std::str::from_utf8(&bytes[1..]).ok()?;
    let algebraic_rank: u8 = rank_str.parse().ok()?;
    if algebraic_rank == 0 || algebraic_rank > height {
        return None;
    }
    Some(Coord {
        file,
        rank: height - algebraic_rank,
    })
}

fn castle_rights_to_fen(flags: &BoardFlags) -> String {
    let mut out = String::new();
    if flags.white_can_castle_kingside {
        out.push('K');
    }
    if flags.white_can_castle_queenside {
        out.push('Q');
    }
    if flags.black_can_castle_kingside {
        out.push('k');
    }
    if flags.black_can_castle_queenside {
        out.push('q');
    }
    if out.is_empty() {
        out.push('-');
    }
    out
}

fn parse_castle_rights(s: &str) -> (bool, bool, bool, bool) {
    if s == "-" {
        return (false, false, false, false);
    }
    let mut wk = false;
    let mut wq = false;
    let mut bk = false;
    let mut bq = false;
    for c in s.chars() {
        match c {
            'K' => wk = true,
            'Q' => wq = true,
            'k' => bk = true,
            'q' => bq = true,
            _ => warn!(?c, "unknown castle-rights char"),
        }
    }
    (wk, wq, bk, bq)
}

pub fn fen_to_board(fen: &str) -> Board {
    debug!(%fen, "fen_to_board");

    // Split off optional flag fields:
    //   <grid> <stm> <castling> <ep> <train_tick> <ply>
    // Plan 09 appends `tr=...` and `p=...`. Both are back-compatible: any
    // FEN without them parses with the defaults (`EveryFullTurn`, 0).
    let mut parts = fen.split_whitespace();
    let grid_part = parts.next().unwrap_or("");
    let stm_part = parts.next();
    let castle_part = parts.next();
    let ep_part = parts.next();
    let train_part = parts.next();
    let ply_part = parts.next();

    let rows: Vec<&str> = grid_part.split('/').collect();
    // Clamp height at 255 — `Coord::rank` is `u8`, so beyond-255 rows
    // can't be addressed, and `Board::height()` would silently truncate
    // to a wrapped value that desyncs from `grid.len()`. Reject the
    // overflow rather than build a state subsequent callers can't use.
    if rows.len() > 255 {
        warn!(
            len = rows.len(),
            "FEN board has >255 rows; truncating to 255 (Coord uses u8)"
        );
    }
    let row_limit = rows.len().min(255);
    let mut grid = vec![];

    for row in rows.into_iter().take(row_limit) {
        let mut squares = fen_row_to_squares(row);
        // Same clamp for row width.
        if squares.len() > 255 {
            warn!(
                len = squares.len(),
                "FEN row has >255 squares; truncating to 255"
            );
            squares.truncate(255);
        }
        grid.push(squares);
    }

    let side_to_move = match stm_part {
        Some("w") | None => Color::White,
        Some("b") => Color::Black,
        Some(other) => {
            warn!(other, "unknown side-to-move byte; defaulting to white");
            Color::White
        }
    };

    let (wk, wq, bk, bq) = match castle_part {
        Some(s) => parse_castle_rights(s),
        // No castling field provided — fall back to all rights set so that
        // a bare grid keeps round-tripping. New games (sent with an explicit
        // castling field) get exactly what the client says.
        None => (true, true, true, true),
    };

    // Algebraic EP parsing needs to know the board's dimensions to invert
    // the rank correctly. Read them off the grid built above.
    let height = grid.len() as u8;
    let width = grid.first().map(|row| row.len() as u8).unwrap_or(0);
    let en_passant_target = match ep_part {
        None | Some("-") => None,
        Some(s) => algebraic_to_coord(s, height, width),
    };

    let train_tick_rate = train_part
        .and_then(parse_train_tick_rate)
        .unwrap_or(TrainTickRate::EveryFullTurn);
    let ply_count = ply_part.and_then(parse_ply_count).unwrap_or(0);

    let flags = BoardFlags {
        side_to_move,
        white_can_castle_kingside: wk,
        white_can_castle_queenside: wq,
        black_can_castle_kingside: bk,
        black_can_castle_queenside: bq,
        en_passant_target,
        train_tick_rate,
        ply_count,
    };

    Board { grid, flags }
}

pub fn square_to_fen(square: &Square) -> String {
    let piece_symbol = square
        .piece
        .as_ref()
        .map(|p| p.symbol())
        .unwrap_or("".to_string());
    let is_standard_square =
        matches!(square.square_type, SquareType::Standard) && square.conditions.is_empty();

    if piece_symbol.len() == 1 && is_standard_square {
        return piece_symbol; // e.g., "P" or "r"
    }

    // Non-standard notation
    let mut parts = vec![];

    if !piece_symbol.is_empty() {
        parts.push(format!("P={}", piece_symbol));
    }

    if !matches!(square.square_type, SquareType::Standard) {
        parts.push(format!("T={}", square.square_type.type_tag()));
        // Each variant only emits the fields it carries. Within a variant,
        // fields appear in the relative order ID → STATE → BRANCHES →
        // TARGETS → OPEN → FIRES so the encoder is deterministic; the
        // parser is order-agnostic (two-pass accumulator below).
        match &square.square_type {
            SquareType::Standard | SquareType::Turret | SquareType::Vent => {}
            SquareType::Switch { targets } => {
                parts.push(format!("TARGETS={}", format_id_list(targets)));
            }
            SquareType::Junction {
                id,
                state,
                branches,
            } => {
                parts.push(format!("ID={}", id));
                parts.push(format!("STATE={}", state));
                parts.push(format!("BRANCHES={}", format_dir_list(branches)));
            }
            SquareType::Gate { id, open } => {
                parts.push(format!("ID={}", id));
                parts.push(format!("OPEN={}", if *open { 1 } else { 0 }));
            }
            SquareType::PressurePlate { targets, fires_for } => {
                parts.push(format!("TARGETS={}", format_id_list(targets)));
                parts.push(format!("FIRES={}", format_pressure_trigger(fires_for)));
            }
            SquareType::Track { direction } => {
                parts.push(format!("D={}", direction.as_str()));
            }
        }
    }

    for cond in &square.conditions {
        parts.push(format!("C={}", cond.as_str()));
    }

    format!("({})", parts.join(","))
}

fn format_id_list(ids: &[SignalId]) -> String {
    let inner = ids
        .iter()
        .map(|id| id.to_string())
        .collect::<Vec<_>>()
        .join(",");
    format!("({inner})")
}

fn parse_id_list(v: &str) -> Vec<SignalId> {
    let Some(inner) = v.strip_prefix('(').and_then(|s| s.strip_suffix(')')) else {
        warn!(v, "malformed id list; expected (...)");
        return vec![];
    };
    if inner.is_empty() {
        return vec![];
    }
    split_top_level(inner)
        .iter()
        .filter_map(|s| match s.parse::<SignalId>() {
            Ok(id) => Some(id),
            Err(e) => {
                warn!(s, ?e, "bad signal id");
                None
            }
        })
        .collect()
}

fn format_dir_list(dirs: &[TrackDir]) -> String {
    let inner = dirs
        .iter()
        .map(|d| d.as_str())
        .collect::<Vec<_>>()
        .join(",");
    format!("({inner})")
}

fn parse_dir_list(v: &str) -> Vec<TrackDir> {
    let Some(inner) = v.strip_prefix('(').and_then(|s| s.strip_suffix(')')) else {
        warn!(v, "malformed dir list; expected (...)");
        return vec![];
    };
    if inner.is_empty() {
        return vec![];
    }
    split_top_level(inner)
        .iter()
        .filter_map(|s| match TrackDir::parse_tag(s) {
            Some(d) => Some(d),
            None => {
                warn!(s, "bad track direction");
                None
            }
        })
        .collect()
}

fn format_pressure_trigger(t: &PressureTrigger) -> String {
    match t {
        PressureTrigger::AnyPiece => "ANY".to_string(),
        PressureTrigger::OnlyColor(Color::White) => "W".to_string(),
        PressureTrigger::OnlyColor(Color::Black) => "B".to_string(),
        // A pressure plate scoped to Neutral pieces would fire only for
        // trains. Encoded as "N" for symmetry.
        PressureTrigger::OnlyColor(Color::Neutral) => "N".to_string(),
    }
}

fn parse_pressure_trigger(v: &str) -> Option<PressureTrigger> {
    match v {
        "ANY" => Some(PressureTrigger::AnyPiece),
        "W" => Some(PressureTrigger::OnlyColor(Color::White)),
        "B" => Some(PressureTrigger::OnlyColor(Color::Black)),
        "N" => Some(PressureTrigger::OnlyColor(Color::Neutral)),
        _ => {
            warn!(v, "unknown pressure trigger");
            None
        }
    }
}

/// Splits a string on **top-level commas**, i.e., commas that are **not**
/// nested inside parentheses.
///
/// This is useful for parsing argument lists or comma-separated constructs
/// where parentheses may contain commas that should *not* act as separators.
///
/// # How it works
/// - Iterates through the input one character at a time.
/// - Tracks the current *parenthesis depth*.
///   - `(` increases depth
///   - `)` decreases depth
/// - A comma is treated as a separator **only when `depth == 0`**.
/// - Everything else is appended to the current buffer.
/// - When a top-level comma is encountered, the accumulated buffer becomes a
///   new part in the result.
/// - After iteration, any remaining buffered text is pushed as the final part.
///
/// # Returns
/// A `Vec<String>` where each element is a trimmed substring extracted from
/// the input, split only at top-level commas.
///
/// # Examples
/// ```ignore
/// let input = "a, b(c, d), e";
/// let parts = split_top_level(input);
///
/// assert_eq!(
///     parts,
///     vec![
///         "a",
///         "b(c, d)",
///         "e",
///     ]
/// );
/// ```
///
/// # Notes
/// - The function treats parentheses literally and does not validate that they
///   are balanced beyond simple depth tracking.
/// - Whitespace around each extracted part is trimmed.
pub fn split_top_level(input: &str) -> Vec<String> {
    trace!(input, "split_top_level");

    let mut parts = Vec::new();
    let mut buf = String::new();
    // `i32` so an unbalanced `)` decrements below zero gracefully
    // instead of underflowing `usize` and panicking in debug builds.
    let mut depth: i32 = 0;

    for (i, ch) in input.chars().enumerate() {
        trace!(i, ?ch, depth, buf, "char");

        match ch {
            '(' => {
                depth += 1;
                buf.push(ch);
            }
            ')' => {
                depth = depth.saturating_sub(1);
                buf.push(ch);
            }
            ',' if depth == 0 => {
                trace!(i, part = %buf.trim(), "top-level comma");
                parts.push(buf.trim().to_string());
                buf.clear();
            }
            _ => {
                buf.push(ch);
            }
        }
    }

    if !buf.is_empty() {
        trace!(part = %buf.trim(), "final part");
        parts.push(buf.trim().to_string());
    }

    trace!(?parts, "split_top_level done");
    parts
}

pub fn fen_to_square(fen: &str) -> Square {
    // Standard empty
    if fen.is_empty() || fen == "()" {
        return Square {
            piece: None,
            square_type: SquareType::Standard,
            conditions: vec![],
        };
    }

    // Extended form
    if fen.starts_with('(') && fen.ends_with(')') {
        let inner = &fen[1..fen.len() - 1];
        let mut piece: Option<PieceType> = None;
        let mut conditions = Vec::new();

        // Variant payload accumulators — buffered through the loop and
        // collapsed into the right `SquareType` by `type_tag` once every
        // field is seen. Lets fields appear in any order.
        let mut type_tag: Option<String> = None;
        let mut id: Option<SignalId> = None;
        let mut state: Option<u8> = None;
        let mut branches: Option<Vec<TrackDir>> = None;
        let mut targets: Option<Vec<SignalId>> = None;
        let mut open: Option<bool> = None;
        let mut fires: Option<PressureTrigger> = None;
        let mut track_dir: Option<TrackDir> = None;

        // Split only at top-level commas (nested-safe)
        let fields = split_top_level(inner);

        for field in fields {
            let mut kv = field.splitn(2, '=');
            let key = kv.next().unwrap_or("").trim();
            let value = kv.next().unwrap_or("").trim();

            match key {
                "P" => {
                    piece = PieceType::symbol_to_piece(value);
                }
                "T" => {
                    type_tag = Some(value.to_string());
                }
                "ID" => match value.parse::<SignalId>() {
                    Ok(v) => id = Some(v),
                    Err(e) => warn!(value, ?e, "bad ID field"),
                },
                "STATE" => match value.parse::<u8>() {
                    Ok(v) => state = Some(v),
                    Err(e) => warn!(value, ?e, "bad STATE field"),
                },
                "BRANCHES" => branches = Some(parse_dir_list(value)),
                "TARGETS" => targets = Some(parse_id_list(value)),
                "OPEN" => match value {
                    "0" => open = Some(false),
                    "1" => open = Some(true),
                    other => {
                        // Reject the whole square — silently defaulting to
                        // `open: true` would mask malformed input and let
                        // an attacker/test author "open" a Gate by typo.
                        warn!(other, "bad OPEN field; expected 0 or 1");
                        open = Some(false);
                    }
                },
                "FIRES" => fires = parse_pressure_trigger(value),
                "D" => match TrackDir::parse_tag(value) {
                    Some(d) => track_dir = Some(d),
                    None => warn!(value, "bad D field"),
                },
                "C" => match value {
                    "FROZEN" => conditions.push(SquareCondition::Frozen),
                    "BRAINROT" => conditions.push(SquareCondition::Brainrot),
                    _ => warn!(value, "unknown square condition"),
                },
                _ => warn!(field, "unknown field"),
            }
        }

        let square_type = match type_tag.as_deref() {
            None => SquareType::Standard,
            Some("STANDARD") => SquareType::Standard,
            Some("TURRET") => SquareType::Turret,
            Some("VENT") => SquareType::Vent,
            Some("SWITCH") => SquareType::Switch {
                targets: targets.unwrap_or_default(),
            },
            Some("JUNCTION") => {
                let branches = branches.unwrap_or_default();
                // Clamp branch count to u8 — anything larger would
                // overflow the `state` field and break the modulo-step
                // in `activate_receiver`. Drop the tail with a warn.
                let branches = if branches.len() > 255 {
                    warn!(len = branches.len(), "junction has >255 branches; truncating");
                    branches.into_iter().take(255).collect()
                } else {
                    branches
                };
                // Normalize `state` to `state % branches.len()` so two
                // FENs (`STATE=0` and `STATE=branches.len()`) describe
                // the same execution state, and `activate_receiver`
                // never sees an out-of-range state.
                let raw_state = state.unwrap_or(0);
                let state = if branches.is_empty() {
                    0
                } else {
                    ((raw_state as usize) % branches.len()) as u8
                };
                SquareType::Junction {
                    id: id.unwrap_or(0),
                    state,
                    branches,
                }
            }
            Some("GATE") => SquareType::Gate {
                id: id.unwrap_or(0),
                open: open.unwrap_or(true),
            },
            Some("PLATE") => SquareType::PressurePlate {
                targets: targets.unwrap_or_default(),
                fires_for: fires.unwrap_or(PressureTrigger::AnyPiece),
            },
            Some("TRACK") => SquareType::Track {
                direction: track_dir.unwrap_or(TrackDir::E),
            },
            Some(other) => {
                warn!(other, "unknown square type");
                SquareType::Standard
            }
        };

        return Square {
            piece,
            square_type,
            conditions,
        };
    }

    // Standard single-character piece
    Square {
        piece: PieceType::symbol_to_piece(fen),
        square_type: SquareType::Standard,
        conditions: vec![],
    }
}
