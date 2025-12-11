import { clearSelection } from "./board_helpers";
import { pieceToSymbol } from "./fen";
import type { SquareType } from "./variables";

export let editorMode = false;        // toggled by checkbox
export let selectedEditorPiece: string | null = null;
export let selectedEditorType: SquareType | null = null;
export let selectedEditorCondition: string | null = null;

document.getElementById("editor-toggle")!.addEventListener("change", (ev) => {
    editorMode = (ev.target as HTMLInputElement).checked;
    clearSelection();
});


const PIECES = [
    "K", "Q", "R", "B", "N", "P",
    "k", "q", "r", "b", "n", "p",
    "G", "g", "S", "s", "bus", "BUS"
];

function buildPiecePalette() {
    console.log("Building piece palette");
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
    erase.onclick = () => {
        selectedEditorPiece = null;
        selectedEditorType = null;
        selectedEditorCondition = null;
    };
    div.appendChild(erase);
}

const TYPES: SquareType[] = ["STANDARD", "VENT", "TURRET"];

function buildTypePalette() {
    console.log("Building type palette");
    const div = document.getElementById("type-palette")!;
    div.innerHTML = "";

    for (const t of TYPES) {
        const btn = document.createElement("button");
        btn.textContent = t;
        btn.onclick = () => {
            selectedEditorType = t;
            selectedEditorPiece = null;
            selectedEditorCondition = null;
        };
        div.appendChild(btn);
    }
}

const CONDITIONS = ["FROZEN", "BRAINROT"]; // extend later

function buildConditionPalette() {
    console.log("Building condition palette");
    const div = document.getElementById("condition-palette")!;
    div.innerHTML = "";

    for (const c of CONDITIONS) {
        const btn = document.createElement("button");
        btn.textContent = c;
        btn.onclick = () => {
            selectedEditorCondition = c;
            selectedEditorPiece = null;
            selectedEditorType = null;
        };
        div.appendChild(btn);
    }
}

export function buildPalettes() {
    buildPiecePalette();
    buildTypePalette();
    buildConditionPalette();
}