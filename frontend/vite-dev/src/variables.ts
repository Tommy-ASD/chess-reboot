

// ----------------------------------------------
// Type Definitions
// ----------------------------------------------

export type SquareType =
    | "STANDARD"
    | "TURRET"
    | "VENT"
    /// Plan 12: impassable terrain. No piece may stand on or slide
    /// through a BLOCK square. Payload-free; the simplest non-walkable
    /// type.
    | "BLOCK"
    | "SWITCH"
    | "JUNCTION"
    | "GATE"
    | "PLATE"
    /// Plan 09: a track tile. Trains follow the chain of TRACK tiles along
    /// each tile's stored direction (or its opposite for Reverse-heading
    /// trains). Walkable for non-train pieces too.
    | "TRACK";

export type Square = {
    piece: string | null;        // "P", "q", "G(W=...)" etc
    squareType: SquareType;
    conditions: string[];        // ["FROZEN", ...]
    /// Variant-specific payload fields preserved verbatim from the FEN
    /// extended block, keyed by tag (e.g. `TARGETS`, `ID`, `STATE`,
    /// `BRANCHES`, `OPEN`, `FIRES`). The editor doesn't yet expose UI for
    /// these — they're round-tripped through unchanged so we don't lose
    /// data on paste-edit-copy. Once plan 09 step 9 lands the dedicated
    /// editor surfaces, replace this with typed fields per variant.
    extraFields?: Record<string, string>;
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
    | { kind: "MoveIntoCarrier"; target: Coord }
    | { kind: "PhaseShift" }
    | { kind: "PieceInCarrier"; target: { piece_index: number; move_type: MoveType } }
    /// Plan 08: throw the Switch tile the piece is standing on. The
    /// `switch` coord is currently always equal to `GameMove.from`, but
    /// the engine carries it explicitly so a future "throw an adjacent
    /// switch" mechanic doesn't break the wire format.
    | { kind: "ThrowSwitch"; target: { switch: Coord } };


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

/// When a carrier (e.g. Bus) is selected, which passenger is the user
/// currently "piloting" through deploy moves? `null` means we're showing
/// the carrier's own moves (drive the bus). A number is an index into the
/// carrier's passenger list, matching `piece_index` on PieceInCarrier moves.
export let selectedPassengerIndex: number | null = null;

export function setSelectedPassengerIndex(i: number | null) {
    selectedPassengerIndex = i;
}