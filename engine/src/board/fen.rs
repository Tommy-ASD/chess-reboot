use crate::{
    board::{
        Board, BoardFlags,
        square::{Square, SquareCondition, SquareType},
    },
    pieces::piecetype::PieceType,
};

fn fen_row_to_squares(row: &str) -> Vec<Square> {
    let mut squares = vec![];
    let mut chars = row.chars().peekable();

    while let Some(&ch) = chars.peek() {
        if ch.is_digit(10) {
            let count = ch.to_digit(10).unwrap();
            for _ in 0..count {
                squares.push(Square {
                    piece: None,
                    square_type: SquareType::Standard,
                    conditions: vec![],
                });
            }
            chars.next();
        } else if ch == '(' {
            // Extended square: find the closing ')'
            let mut fen_piece = String::new();
            let mut depth = 0;
            while let Some(c) = chars.next() {
                fen_piece.push(c);
                if c == '(' {
                    depth += 1;
                }
                if c == ')' {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                }
            }
            squares.push(fen_to_square(&fen_piece));
        } else {
            // Normal piece
            squares.push(fen_to_square(&ch.to_string()));
            chars.next();
        }
    }

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

    rows.join("/")
}

pub fn fen_to_board(fen: &str) -> Board {
    let rows: Vec<&str> = fen.split('/').collect();
    let mut grid = vec![];

    for row in rows {
        grid.push(fen_row_to_squares(row));
    }

    // Default flags for now
    let flags = BoardFlags {
        white_can_castle_kingside: true,
        white_can_castle_queenside: true,
        black_can_castle_kingside: true,
        black_can_castle_queenside: true,
        en_passant_target: None,
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
        parts.push(format!("T={}", square.square_type.as_str()));
    }

    for cond in &square.conditions {
        parts.push(format!("C={}", cond.as_str()));
    }

    format!("({})", parts.join(","))
}

fn split_top_level(input: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut buf = String::new();
    let mut depth = 0;

    for ch in input.chars() {
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
                parts.push(buf.trim().to_string());
                buf.clear();
            }
            _ => buf.push(ch),
        }
    }

    if !buf.is_empty() {
        parts.push(buf.trim().to_string());
    }

    parts
}

fn parse_nested_piece(sym: &str) -> Option<PieceType> {
    // If nested (begins with "(" or something like G(...)) we treat
    // the entire string as a full fen_to_square() call.
    if sym.contains('(') {
        let sq = fen_to_square(sym);
        return sq.piece; // piece extracted from nested square
    }

    // Otherwise treat as normal piece symbol
    PieceType::symbol_to_piece(sym)
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
        let mut square_type = SquareType::Standard;
        let mut conditions = Vec::new();

        // Split only at top-level commas (nested-safe)
        let fields = split_top_level(inner);

        for field in fields {
            let mut kv = field.splitn(2, '=');
            let key = kv.next().unwrap_or("").trim();
            let value = kv.next().unwrap_or("").trim();

            match key {
                "P" => {
                    piece = parse_nested_piece(value);
                }
                "T" => {
                    square_type = match value {
                        "TURRET" => SquareType::Turret,
                        "VENT" => SquareType::Vent,
                        _ => {
                            println!("Unknown square type {value}");
                            SquareType::Standard
                        }
                    }
                }
                "C" => match value {
                    "FROZEN" => conditions.push(SquareCondition::Frozen),
                    _ => println!("Unknown square condition {value}"),
                },
                _ => println!("Unknown field {field}"),
            }
        }

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
