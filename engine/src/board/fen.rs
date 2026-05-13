use tracing::{debug, trace, warn};

use crate::{
    board::{
        Board, BoardFlags, Coord, SignalId,
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
    let mut depth = 0;

    for (i, ch) in s.char_indices().skip(open_index) {
        match ch {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
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
            let mut depth = 0usize;

            while let Some(c) = chars.next() {
                buf.push(c);

                match c {
                    '(' => {
                        depth += 1;
                    }
                    ')' => {
                        depth -= 1;
                        if depth == 0 {
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
    };
    let castling = castle_rights_to_fen(&board.flags);
    let ep = match &board.flags.en_passant_target {
        Some(c) => board.format_coord(c),
        None => "-".to_string(),
    };
    format!("{grid} {stm} {castling} {ep}")
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

    // Split off optional flag fields: <grid> <stm> <castling> <ep> [...]
    let mut parts = fen.split_whitespace();
    let grid_part = parts.next().unwrap_or("");
    let stm_part = parts.next();
    let castle_part = parts.next();
    let ep_part = parts.next();

    let rows: Vec<&str> = grid_part.split('/').collect();
    let mut grid = vec![];

    for row in rows {
        grid.push(fen_row_to_squares(row));
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

    let flags = BoardFlags {
        side_to_move,
        white_can_castle_kingside: wk,
        white_can_castle_queenside: wq,
        black_can_castle_kingside: bk,
        black_can_castle_queenside: bq,
        en_passant_target,
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
    }
}

fn parse_pressure_trigger(v: &str) -> Option<PressureTrigger> {
    match v {
        "ANY" => Some(PressureTrigger::AnyPiece),
        "W" => Some(PressureTrigger::OnlyColor(Color::White)),
        "B" => Some(PressureTrigger::OnlyColor(Color::Black)),
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
    let mut depth = 0usize;

    for (i, ch) in input.chars().enumerate() {
        trace!(i, ?ch, depth, buf, "char");

        match ch {
            '(' => {
                depth += 1;
                buf.push(ch);
            }
            ')' => {
                depth -= 1;
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
            Some("JUNCTION") => SquareType::Junction {
                id: id.unwrap_or(0),
                state: state.unwrap_or(0),
                branches: branches.unwrap_or_default(),
            },
            Some("GATE") => SquareType::Gate {
                id: id.unwrap_or(0),
                open: open.unwrap_or(true),
            },
            Some("PLATE") => SquareType::PressurePlate {
                targets: targets.unwrap_or_default(),
                fires_for: fires.unwrap_or(PressureTrigger::AnyPiece),
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
