use crate::board::{Board, Coord, GameMove};

impl Board {
    /// Attempts to execute a move on the board.
    /// Returns Ok(()) if successful, Err(...) if illegal.
    pub fn make_move(&mut self, game_move: GameMove) -> Result<(), String> {
        let board_before = self.clone();

        let from = &game_move.from;
        let to = &game_move.to;

        // validate existence of source square + piece
        let piece = {
            let square = self
                .get_square_at(from)
                .ok_or_else(|| format!("No square at {:?}", from))?;

            square
                .piece
                .clone()
                .ok_or_else(|| format!("No piece at {:?}", from))?
        };

        // validate legality of the move
        if !self.is_valid_move(from, to) {
            return Err(format!("Illegal move: {:?} -> {:?}", from, to));
        }

        // mutate board: remove piece from original square
        // this logic will change later on with new pieces
        {
            let from_sq = self
                .get_square_mut(from)
                .ok_or_else(|| format!("No square at {:?}", from))?;

            from_sq.piece = None;
        }

        // handle capture or landing on new square
        // again, logic will change with new pieces
        {
            let to_sq = self
                .get_square_mut(to)
                .ok_or_else(|| format!("No square at {:?}", to))?;

            // Whatever piece is there → captured automatically
            to_sq.piece = Some(piece);
        }

        println!("Move executed: {:?} -> {:?}", from, to);

        // 5. Special movement hooks (stub)
        self.handle_post_move_effects(&board_before, game_move)?;

        Ok(())
    }

    fn handle_post_move_effects(
        &mut self,
        before_state: &Board,
        game_move: GameMove,
    ) -> Result<(), String> {
        // call post-move effect on the piece that moved
        let piece = {
            let square = self
                .get_square_at(&game_move.to)
                .ok_or_else(|| format!("No square at {:?}", game_move.to))?;

            square
                .piece
                .clone()
                .ok_or_else(|| format!("No piece at {:?}", game_move.to))?
        };
        piece.post_move_effects(before_state, self, &game_move.from, &game_move.to);
        Ok(())
    }
}
