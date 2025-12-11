import { clearSelection } from "./board_helpers";
import { pieceToSymbol } from "./fen";
import type { Square, SquareType } from "./variables";

let editorMode = false;        // toggled by checkbox
let selectedEditorPiece: string | null = null;
let selectedEditorType: SquareType | null = null;
let selectedEditorCondition: string | null = null;

let currentBoard: Square[][] = []; // the board in memory

document.getElementById("editor-toggle")!.addEventListener("change", (ev) => {
    editorMode = (ev.target as HTMLInputElement).checked;
    clearSelection();
});


const PIECES = [
    "K", "Q", "R", "B", "N", "P",
    "k", "q", "r", "b", "n", "p",
    "G", "g", "S", "s"
];

function buildPiecePalette() {
    const div = document.getElementById("piece-palette")!;
    div.innerHTML = "";

    for (const p of PIECES) {
        const btn = document.createElement("button");
        btn.textContent = pieceToSymbol(p);
        btn.onclick = () => {
            selectedEditorPiece = p;
            selectedEditorType = null;
            selectedEditorCondition = null;
        };
        div.appendChild(btn);
    }

    // eraser
    const erase = document.createElement("button");
    erase.textContent = "Erase Piece";
    erase.onclick = () => selectedEditorPiece = null;
    div.appendChild(erase);
}
