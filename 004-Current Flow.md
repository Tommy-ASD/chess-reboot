Okay current flow for development
We have the typescript frontend.
It has a FEN input, and renders the provided FEN
Each square has an onclick listener. Onclick, make request to the chess API, `http://localhost:8080/board/moves`
Takes in a board FEN, a coordinate
Returns a list of allowed coordinates to move to
These coordinates are highlighted in the frontend.
Upon pressing one of these allowed squares, a request is sent to `http://localhost:8080/board/new_state`
This endpoint takes in an initial board FEN, where a piece moves from, where it moves to, and returns the new FEN.
The frontend renders the new FEN.
This endpoint doesn't currently check whether the move is valid. If it isn't, the endpoint returns the old FEN, unaltered.
The API is currently *completely* stateless.

How allowed moves are fetched;
The handler turns the provided FEN into a board
We call board.get_moves, which returns Vec<GameMove>
Currently, GameMove is a type that simply contains a `from` and a `to` coordinate.
We will add more fields later
This function currently *only* calls piece.get_moves for the piece present (if any) and returns it.
More complex logic will be necessary later, for things like turrets, global pieces, whatever I feel like adding
The handler takes these provided moves and iters through them, retaining the `to` field in the GameMove objects, returning it as the API response

How moves are made;
The handler turns the provided FEN into a board, turns the provided `to` and `from` coordinates into a GameMove
Calls `board.make_move(game_move)`
Returns `board_to_fen(&board)`, which has the move applied to it
the `board.make_move` method first makes a clone of initial state
It checks whether the move is  using `board.is_valid_move`, returning early if it isn't
`board.is_valid_move` is simply a method which calls `board.get_moves` and returns `true` if the provided GameMove is present
Remove moving piece from original square
Replace piece at target square with moving piece
Run `board.handle_post_move_effects`, passing in a mutable reference to self, immutable reference to the old board, and the game move
This is a hook which can affect the board after a move is made
The `Piece` trait also has `Piece::post_move_effects`. Currently, this is only used by `Goblin`, which replaces itself with the kidnapped piece upon returning to home square

Okay we gotta go bedtime now
but for certain pieces (currently thinking of the Skibidi), the current GameMove struct won't be sufficient
```
Skibidi
    Stuns all pieces (enemy and ally) in a given circle radius. They can no longer move due to extreme brainrot.
    No effect on others (phase 1) -> the 4 non-diagonal neighboring cells (circle of radius with 1, phase 2) -> 
        a circle with a radius of 2 (phase 3) -> a circle with a radius of 3 (phase 4)
    This effect is removed once the Skibidi is captured, or another neutralizing Skibidi enters the Brainrot radius.
    Any Skibidi (enemy or ally) entering the Brainrot radius acts as a neutralizing Skibidi.
    After being neutralized, the Skibidi is set back to phase 1.
    The Skibidi can move, but it is set back to phase 1 each time it moves.
    Increasing the radius of brainrot uses a move.
    If there is no opposing Skibidi, the maximum phase your Skibidi can reach is 3.
    It moves like a king (to any directly neighboring cells), but cannot take other pieces. 
    It can take other Skibidis
    If your Skibidi your enemy cannot make a move due to your Brainrot, 
        you win by Brainrot instead of stalemate being declared.
    If your Skibidi is captured while your opponent's Skibidi is in phase 4, there is nothing you can do.
```
So, I'm thinking, maybe we add a `MoveType` enum?
That has variants like `MoveTo`, `CaptureWithoutMove`
And `PhaseShift`
And each of these variants could have relevant fields in them
Something like 
```
enum MoveType {
    MoveTo(Coord),
    CaptureWithoutMove(Coord),
    PhaseShift, // no necessary fields, yet
}
```

And GameMove could be changed to be 
```
struct GameMove {
    from: Coord,
    move_type: MoveType
}
```

This pattern can also work with the bus, but we'll deal with that when we get there