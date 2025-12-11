import { allowedMoves, setAllowedMoves, setSelectedSquare, type Coord, type GameMove } from "./variables";


export function isAllowedSquare(c: Coord): boolean {
    // filter allowedMoves for MoveTo moves only
    let allowedCoords = allowedMoves
        .filter(m => isMoveTo(m))
        .map(m => m.move_type.target);

    return allowedCoords.some(ac => ac.file === c.file && ac.rank === c.rank);
}

/// Simple helper to highlight squares given a list of coordinates
export function highlightMoves(moves: GameMove[]) {
    const squares = document.querySelectorAll(".square");
    squares.forEach(sq => sq.classList.remove("highlight"));

    // filter moves for MoveTo only
    let moveCoords = moves
        .filter(m => isMoveTo(m))
        .map(m => m.move_type.target);

    for (const m of moveCoords) {
        const index = m.rank * 8 + m.file;
        squares[index].classList.add("highlight");
    }
}

export function isMoveTo(m: GameMove): m is GameMove & { move_type: { kind: "MoveTo"; target: Coord } } {
    return m.move_type.kind === "MoveTo";
}

export function isSpecialMove(m: GameMove): boolean {
    return m.move_type.kind !== "MoveTo";
}

/// Clears any selected square and highlighted moves
export function clearSelection() {
    setSelectedSquare(null);
    setAllowedMoves([]);

    const squares = document.querySelectorAll(".square");
    squares.forEach(s => s.classList.remove("selected", "highlight"));

    const list = document.getElementById("special-actions")!;
    list.innerHTML = "";
}