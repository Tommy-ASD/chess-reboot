use crate::board::{Board, Coord, GameMove};

impl Board {
    /// Attempts to execute a move on the board.
    /// Returns Ok(()) if successful, Err(...) if illegal.
    pub fn make_move(&mut self, game_move: GameMove) -> Result<(), String> {
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

        // 5. Special movement hooks (stub)
        self.handle_post_move_effects(from, to)?;

        Ok(())
    }

    /// Hook for future rules that trigger after a move is made.
    /// Currently a stub that always returns Ok.
    fn handle_post_move_effects(&mut self, _from: &Coord, _to: &Coord) -> Result<(), String> {
        // No rules implemented yet
        Ok(())
    }
}
