import type { Coord, GameMove, MoveType } from "./types";


export function isMoveTo(m: GameMove): m is GameMove & { move_type: { kind: "MoveTo"; target: Coord } } {
    return m.move_type.kind === "MoveTo";
}

export function isMoveIntoCarrier(m: GameMove): m is GameMove & { move_type: { kind: "MoveIntoCarrier"; target: Coord } } {
    return m.move_type.kind === "MoveIntoCarrier";
}

export function isPieceInCarrier(m: GameMove): m is GameMove & { move_type: { kind: "PieceInCarrier"; target: { piece_index: number; move_type: MoveType } } } {
    return m.move_type.kind === "PieceInCarrier";
}

/// "Special" = needs the side-actions panel (PhaseShift, ThrowSwitch,
/// PlaceTornado). Board moves — MoveTo, MoveIntoCarrier, PieceInCarrier,
/// and the core chess moves Castle / Promotion / EnPassant — are reached
/// by clicking the board, not the side panel.
export function isSpecialMove(m: GameMove): boolean {
    const k = m.move_type.kind;
    return k !== "MoveTo"
        && k !== "MoveIntoCarrier"
        && k !== "PieceInCarrier"
        && k !== "Castle"
        && k !== "Promotion"
        && k !== "EnPassant";
}

/// The king's landing square for a Castle move — the board square a user
/// clicks to castle. Kingside → g-file (6), queenside → c-file (2), on the
/// king's own rank (`GameMove.from.rank`). `null` for non-Castle moves.
export function castleKingDest(m: GameMove): Coord | null {
    if (m.move_type.kind !== "Castle") return null;
    const file = m.move_type.target.side === "Kingside" ? 6 : 2;
    return { file, rank: m.from.rank };
}

/// A target the board should highlight, with the visual style it deserves.
/// - "move"   → normal destination (Bus driving, regular piece moving)
/// - "board"  → board a friendly carrier (MoveIntoCarrier)
/// - "deploy" → a passenger deploying out of the carrier
export type BoardTarget = { target: Coord; kind: "move" | "board" | "deploy" };

/// Decide which targets are currently visible on the board given the
/// passenger-pick state. When no passenger is selected we show the
/// carrier's own moves (drive + board); when one is, we show only that
/// passenger's deploy destinations.
export function visibleMoveTargets(moves: GameMove[], passengerIdx: number | null): BoardTarget[] {
    const out: BoardTarget[] = [];
    if (passengerIdx === null) {
        for (const m of moves) {
            if (isMoveTo(m)) out.push({ target: m.move_type.target, kind: "move" });
            else if (isMoveIntoCarrier(m)) out.push({ target: m.move_type.target, kind: "board" });
            // Core chess moves are clicked on the board like any move. The
            // four Promotion entries (one per piece) collapse to a single
            // highlighted square; the picker disambiguates on click.
            else if (m.move_type.kind === "Promotion") out.push({ target: m.move_type.target.target, kind: "move" });
            else if (m.move_type.kind === "EnPassant") out.push({ target: m.move_type.target.target, kind: "move" });
            else if (m.move_type.kind === "Castle") {
                const dest = castleKingDest(m);
                if (dest) out.push({ target: dest, kind: "move" });
            }
        }
    } else {
        for (const m of moves) {
            if (!isPieceInCarrier(m)) continue;
            if (m.move_type.target.piece_index !== passengerIdx) continue;
            const inner = m.move_type.target.move_type;
            // For now we only render passengers whose nested move is a MoveTo.
            // Other nested kinds (Promotion etc.) would need their own affordance.
            if (inner.kind === "MoveTo") {
                out.push({ target: inner.target, kind: "deploy" });
            }
        }
    }
    return out;
}

const sameCoord = (a: Coord, b: Coord) => a.file === b.file && a.rank === b.rank;

export function isAllowedSquare(c: Coord, moves: GameMove[], passengerIdx: number | null): boolean {
    return visibleMoveTargets(moves, passengerIdx).some(t => sameCoord(t.target, c));
}

/// The `Promotion` moves landing on `c` (one per promotion piece). Non-empty
/// means the click needs a piece choice before a move can be submitted.
export function promotionsAt(c: Coord, moves: GameMove[]): GameMove[] {
    return moves.filter(
        m => m.move_type.kind === "Promotion" && sameCoord(m.move_type.target.target, c),
    );
}

/// Given a clicked target square, find the corresponding GameMove from
/// the allowed list, respecting whether we're showing the carrier's own
/// moves or a specific passenger's deploys.
export function findMoveForTarget(clicked: Coord, moves: GameMove[], passengerIdx: number | null): GameMove | null {
    for (const m of moves) {
        if (passengerIdx === null) {
            if (m.move_type.kind === "MoveTo" && sameCoord(m.move_type.target, clicked)) return m;
            if (m.move_type.kind === "MoveIntoCarrier" && sameCoord(m.move_type.target, clicked)) return m;
            if (m.move_type.kind === "EnPassant" && sameCoord(m.move_type.target.target, clicked)) return m;
            if (m.move_type.kind === "Castle") {
                const dest = castleKingDest(m);
                if (dest && sameCoord(dest, clicked)) return m;
            }
        } else if (isPieceInCarrier(m)
            && m.move_type.target.piece_index === passengerIdx
            && m.move_type.target.move_type.kind === "MoveTo"
            && sameCoord(m.move_type.target.move_type.target, clicked)) {
            return m;
        }
    }
    return null;
}
