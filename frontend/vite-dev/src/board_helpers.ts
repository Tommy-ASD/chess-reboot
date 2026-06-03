import { allowedMoves, selectedPassengerIndex, setAllowedMoves, setSelectedPassengerIndex, setSelectedSquare, type Coord, type GameMove, type MoveType } from "./variables";


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

export function isAllowedSquare(c: Coord): boolean {
    const targets = visibleMoveTargets(allowedMoves, selectedPassengerIndex);
    return targets.some(t => t.target.file === c.file && t.target.rank === c.rank);
}

/// Highlight all currently visible move targets on the board, applying
/// the right CSS class for each kind so they read distinctly.
export function highlightMoves(moves: GameMove[]) {
    const squares = document.querySelectorAll(".square");
    squares.forEach(sq => sq.classList.remove("highlight", "highlight-board", "highlight-deploy"));

    // The squares NodeList is in row-major order — same as the rendered
    // grid. Read the column count off the `--cols` CSS variable so this
    // works for any board width, not just 8.
    const colsRaw = getComputedStyle(document.documentElement)
        .getPropertyValue("--cols")
        .trim();
    const cols = Number(colsRaw) || 8;

    const targets = visibleMoveTargets(moves, selectedPassengerIndex);
    for (const t of targets) {
        const idx = t.target.rank * cols + t.target.file;
        const sq = squares[idx];
        if (!sq) continue;
        sq.classList.add("highlight");
        if (t.kind === "board") sq.classList.add("highlight-board");
        if (t.kind === "deploy") sq.classList.add("highlight-deploy");
    }
}

/// Clears any selected square, carrier state, and highlighted moves
export function clearSelection() {
    setSelectedSquare(null);
    setAllowedMoves([]);
    setSelectedPassengerIndex(null);

    const squares = document.querySelectorAll(".square");
    squares.forEach(s => s.classList.remove("selected", "highlight", "highlight-board", "highlight-deploy"));

    const list = document.getElementById("special-actions")!;
    list.innerHTML = "";
    const carrier = document.getElementById("carrier-moves")!;
    carrier.innerHTML = "";
}
