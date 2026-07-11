//! Throwaway validator for candidate starting positions.
//!
//! Reads FENs from stdin, one per line (blank lines and `#` comments
//! skipped). For each, reports whether it parses, its dimensions, the
//! reported `status()`, and how many legal moves the side-to-move has.
//! A good "playable" starting position parses, is `Ongoing`, and has
//! at least one legal move.
//!
//! Run: `printf '%s\n' "<fen1>" "<fen2>" | cargo run -p engine --example validate_positions`

use std::io::Read;

use engine::board::fen::{board_to_fen, fen_to_board};
use engine::pieces::Color;

fn main() {
    let mut input = String::new();
    std::io::stdin().read_to_string(&mut input).unwrap();

    let mut all_ok = true;
    for (i, raw) in input.lines().enumerate() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        match fen_to_board(line) {
            Err(e) => {
                all_ok = false;
                println!("[{i}] PARSE-FAIL: {e}\n      fen: {line}");
            }
            Ok(board) => {
                let side = board.flags.side_to_move;
                let status = board.status();
                let dims = format!("{}x{}", board.width(), board.height());

                let coords: Vec<_> = board
                    .all_pieces()
                    .into_iter()
                    .filter(|(_, p)| p.get_color() == side)
                    .map(|(c, _)| c)
                    .collect();
                let legal: usize = coords.iter().map(|c| board.legal_moves(c).len()).sum();

                let wk = board.find_king(Color::White).is_some();
                let bk = board.find_king(Color::Black).is_some();

                let roundtrip = board_to_fen(&board);
                let ok = matches!(status, engine::board::GameStatus::Ongoing)
                    && legal > 0
                    && wk
                    && bk;
                if !ok {
                    all_ok = false;
                }
                println!(
                    "[{i}] {} dims={dims} status={status:?} side={side:?} legal_moves={legal} WK={wk} BK={bk}",
                    if ok { "OK  " } else { "WARN" }
                );
                println!("      roundtrip: {roundtrip}");
            }
        }
    }

    if !all_ok {
        std::process::exit(1);
    }
}
