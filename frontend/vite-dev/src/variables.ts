

// ----------------------------------------------
// Type Definitions
// ----------------------------------------------

export type SquareType = "STANDARD" | "TURRET" | "VENT";

export type Square = {
    piece: string | null;        // "P", "q", "G(W=...)" etc
    squareType: SquareType;
    conditions: string[];        // ["FROZEN", ...]
};

export type Coord = { file: number; rank: number };

/// PieceInCarrier
/// Rendering carrier move: 
/// Object { from: {…}, move_type: {…} }
/// from: Object { file: 5, rank: 3 }
/// move_type: Object { kind: "PieceInCarrier", target: {…} }
/// kind: "PieceInCarrier"
/// target: Object { piece_index: 0, move_type: {…} }
/// move_type: Object { kind: "MoveTo", target: {…} }
/// kind: "MoveTo"
/// target: Object { file: 7, rank: 2 }
/// piece_index: 0

// Nested MoveType 
/// { kind: "PieceInCarrier", target: { piece_index: number; move_type: MoveType } }


export type MoveType =
    | { kind: "MoveTo"; target: Coord }
    | { kind: "PhaseShift" }
    | { kind: "PieceInCarrier"; target: { piece_index: number; move_type: MoveType } };


export type GameMove = { from: Coord; move_type: MoveType };

export let selectedSquare: Coord | null = null;

export function setSelectedSquare(s: Coord | null) {
    selectedSquare = s;
}

export let allowedMoves: GameMove[] = []; // returned from API

export function setAllowedMoves(a: GameMove[]) {
    allowedMoves = a;
}

export let currentBoard: Square[][] = []; // the board in memory

export function setCurrentBoard(board: Square[][]) {
    currentBoard = board
}