use crate::{
    board::{
        Board, BoardFlags,
        square::{Square, SquareCondition, SquareType},
    },
    pieces::piecetype::PieceType,
};

fn fen_row_to_squares(row: &str) -> Vec<Square> {
    // println!("=== fen_row_to_squares BEGIN ===");
    // println!("row = \"{}\"", row);

    let mut squares = Vec::new();
    let mut chars = row.chars().peekable();
    let mut index = 0usize;

    while let Some(&ch) = chars.peek() {
        // println!("char[{}] = '{}'", index, ch);

        // -------------------------------
        // 1. DIGIT → run-length empties
        // -------------------------------
        if ch.is_ascii_digit() {
            let count = ch.to_digit(10).unwrap();
            // println!("  -> digit found: {} → pushing {} empty squares", ch, count);

            for _ in 0..count {
                squares.push(Square {
                    piece: None,
                    square_type: SquareType::Standard,
                    conditions: vec![],
                });
            }
            chars.next();
            index += 1;
            continue;
        }

        // -------------------------------
        // 2. EXTENDED BLOCK STARTS WITH '('
        // -------------------------------
        if ch == '(' {
            // println!("  -> extended square begins at index {}", index);

            let mut buf = String::new();
            let mut depth = 0usize;

            while let Some(c) = chars.next() {
                // println!(
                //     "    collecting '{}', depth={} → buffer=\"{}\"",
                //     c, depth, buf
                // );

                buf.push(c);

                match c {
                    '(' => {
                        depth += 1;
                        // println!("      '(' increases depth → {}", depth);
                    }
                    ')' => {
                        depth -= 1;
                        // println!("      ')' decreases depth → {}", depth);

                        if depth == 0 {
                            // println!("      extended block closed → \"{}\"", buf);
                            break;
                        }
                    }
                    _ => {}
                }
            }

            // println!("  -> parsing extended square: {}", buf);
            squares.push(fen_to_square(&buf));

            index += buf.len();
            continue;
        }

        // -------------------------------
        // 3. STANDARD SINGLE-CHAR PIECE
        // -------------------------------
        // println!("  -> standard piece '{}'", ch);
        squares.push(fen_to_square(&ch.to_string()));

        chars.next();
        index += 1;
    }

    // println!("Row result ({} squares): {:?}", squares.len(), squares);
    // println!("=== fen_row_to_squares END ===\n");

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

    dbg!();
    println!("Parsing board from FEN: {}", fen);

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
    // println!("--- split_top_level BEGIN ---");
    // println!("input = \"{}\"", input);

    let mut parts = Vec::new();
    let mut buf = String::new();
    let mut depth = 0usize;

    for (i, ch) in input.chars().enumerate() {
        // println!(
        //     "char[{}] = '{}'  depth = {}  buf = \"{}\"",
        //     i, ch, depth, buf
        // );

        match ch {
            '(' => {
                depth += 1;
                // println!("  -> '(' encountered, increasing depth to {}", depth);
                buf.push(ch);
            }
            ')' => {
                // println!("  -> ')' encountered, decreasing depth from {}", depth);
                depth -= 1;
                buf.push(ch);
                // println!("     new depth = {}", depth);
            }
            ',' if depth == 0 => {
                // println!(
                //     "  -> TOP-LEVEL COMMA at index {}, pushing part: \"{}\"",
                //     i,
                //     buf.trim()
                // );

                parts.push(buf.trim().to_string());
                buf.clear();
            }
            _ => {
                buf.push(ch);
            }
        }
    }

    if !buf.is_empty() {
        // println!("END OF STRING, pushing final part: \"{}\"", buf.trim());
        parts.push(buf.trim().to_string());
    }

    // println!("FINAL PARTS = {:?}", parts);
    // println!("--- split_top_level END ---\n");

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
                    piece = PieceType::symbol_to_piece(value);
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
