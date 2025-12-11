

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

export type MoveType =
    | { kind: "MoveTo"; target: Coord }
    | { kind: "PhaseShift" };


export type GameMove = { from: Coord; move_type: MoveType };

export let selectedSquare: Coord | null = null;

export function setSelectedSquare(s: Coord | null) {
    selectedSquare = s;
}

export let allowedMoves: GameMove[] = []; // returned from API

export function setAllowedMoves(a: GameMove[]) {
    allowedMoves = a;
}