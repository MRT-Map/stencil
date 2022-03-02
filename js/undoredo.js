/// <reference path="references.ts" />
let editHistory = [];
let undoStack = [];
function undo() {
    let prevLog = editHistory.pop();
    if (prevLog === undefined)
        return;
    undoStack.push(prevLog);
    if (prevLog.action == "create") {
        layers.removeLayer(prevLog.layer);
    }
    else if (prevLog.action == "delete") {
        layers.addLayer(prevLog.layer);
    }
    else if (prevLog.action == "modifyLatLng") {
        if (prevLog.oldLatLngs !== undefined)
            prevLog.layer.setLatLngs(prevLog.oldLatLngs);
        // @ts-ignore
        else
            prevLog.layer.setLatLng(prevLog.oldLatLng);
        ;
        prevLog.layer.fire("click");
    }
    else if (prevLog.action == "modifyMapInfo") {
        prevLog.layer.mapInfo = prevLog.oldMapInfo;
        ;
        prevLog.layer.fire("click");
    }
}
function redo() {
    let prevLog = undoStack.pop();
    if (prevLog === undefined)
        return;
    editHistory.push(prevLog);
    if (prevLog.action == "create") {
        layers.addLayer(prevLog.layer);
    }
    else if (prevLog.action == "delete") {
        layers.removeLayer(prevLog.layer);
    }
    else if (prevLog.action == "modifyLatLng") {
        if (prevLog.newLatLngs !== undefined)
            prevLog.layer.setLatLngs(prevLog.newLatLngs);
        // @ts-ignore
        else
            prevLog.layer.setLatLng(prevLog.newLatLng);
        ;
        prevLog.layer.fire("click");
    }
    else if (prevLog.action == "modifyMapInfo") {
        prevLog.layer.mapInfo = prevLog.newMapInfo;
        prevLog.layer.fire("click");
    }
}
