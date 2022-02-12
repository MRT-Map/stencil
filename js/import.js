/// <reference path="references.ts" />
var allNodes = {};
function triggerImportData() {
    try {
        if (layers.getLayers() && !confirm("Importing a new set will erase the current set. Are you sure you want to import?"))
            return;
        qs(document, "#importerNodes").click();
        qs(document, "#importerComps").click();
    }
    catch (err) {
        qs(document, "#pane_import #err").innerHTML = err;
    }
}
function importData(id) {
    try {
        qs(document, "#pane_export #err").innerHTML = "";
        var importedFile = qs(document, '#' + id).files[0];
        var reader = new FileReader();
        reader.onload = function () {
            var fileContent = JSON.parse(reader.result);
            document.getElementById(id).value = "";
            //console.log(fileContent);
            if (id == "importerComps") {
                if (allNodes === undefined) {
                    throw `No nodes`;
                }
                layers.clearLayers();
                selectShadowGroup.clearLayers();
                Object.entries(fileContent).forEach(([id, value]) => {
                    var _a, _b, _c, _d;
                    if (value.nodes === undefined)
                        throw `Component '${id}' has no 'nodes' field`;
                    if (value.type === undefined)
                        throw `Component '${id}' has no 'type' field`;
                    let mapInfo = {
                        id: id,
                        attrs: (_a = value.attrs) !== null && _a !== void 0 ? _a : {},
                        description: (_b = value.description) !== null && _b !== void 0 ? _b : "",
                        displayname: (_c = value.displayname) !== null && _c !== void 0 ? _c : "",
                        layer: (_d = value.layer) !== null && _d !== void 0 ? _d : 0,
                        nodes: value.nodes,
                        type: value.type.split(" ")[0],
                        tags: value.type.split(" ").length > 1 ? value.type.split(" ").slice(1).join(" ") : ""
                    };
                    value.type = value.type.split(" ")[0];
                    let comp;
                    let coords = value.nodes.map(node => {
                        if (!Object.keys(allNodes).includes(node))
                            throw `Node '${node}' of component '${id}' not found in node list`;
                        return mapcoord([allNodes[node].x, allNodes[node].y]);
                    });
                    //console.log(coords);
                    if (ComponentTypes.area.includes(value.type)) {
                        comp = L.polygon(coords, { color: getFrontColor(value.type), weight: getWeight(value.type), pmIgnore: false });
                    }
                    else if (ComponentTypes.point.includes(value.type)) {
                        comp = L.marker(coords[0], { pmIgnore: false });
                    }
                    else if (ComponentTypes.line.includes(value.type)) {
                        comp = L.polyline(coords, { color: getFrontColor(value.type), weight: getWeight(value.type), pmIgnore: false });
                    }
                    else {
                        throw `Component ${id} has no 'type' field`;
                    }
                    comp.mapInfo = mapInfo;
                    // @ts-ignore
                    comp._drawnByGeoman = true;
                    var a = (e) => {
                        if (e.layer == selected)
                            select();
                    };
                    comp.on("pm:drag", a);
                    comp.on("pm:markerdrag", a);
                    comp.on("pm:vertexadded", a);
                    comp.on("pm:vertexremoved", a);
                    comp.on("pm:rotate", a);
                    comp.on("click", layerClickEvent);
                    //console.log(comp);
                    comp.addTo(layers);
                });
            }
            else if (id == "importerNodes") {
                allNodes = {};
                Object.entries(fileContent).forEach(([id, value]) => {
                    var _a;
                    if (value.x === undefined)
                        throw `Node ${id} has no field 'x'`;
                    if (value.y === undefined)
                        throw `Node ${id} has no field 'y'`;
                    allNodes[id] = {
                        x: value.x,
                        y: value.y,
                        connections: (_a = value.connections) !== null && _a !== void 0 ? _a : []
                    };
                });
            }
        };
        reader.readAsText(importedFile);
    }
    catch (err) {
        qs(document, "#pane_import #err").innerHTML = `<span style="color: red;">${err}</span>`;
    }
}
