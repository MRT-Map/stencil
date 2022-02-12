/// <reference path="references.ts" />
interface HistoryLog {
    action: string;
    layer: Selected;
    oldLatLngs?: L.LatLngExpression | L.LatLngExpression[];
    newLatLngs?: L.LatLngExpression | L.LatLngExpression[];
    oldMapInfo?: PLAComponent;
    newMapInfo?: PLAComponent;
}

let history: HistoryLog[] = [];
let undoStack: HistoryLog[] = [];

function undo() {
    
}

function redo() {
    
}
