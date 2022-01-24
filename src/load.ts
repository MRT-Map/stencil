/// <reference path="references.ts" />

var allNodes = {};

function triggerImportData() {
    try {
        if (layers.getLayers() && !confirm("Importing a new set will erase the current set. Are you sure you want to import?")) return;
        qs(document, "#importerNodes").click();
        qs(document, "#importerComps").click();
    } catch (err) {
        qs(document, "#pane_import #err").innerHTML = err;
    }
}

function importData(id: string) { try {
    qs(document, "#pane_export #err").innerHTML = "";
    var importedFile = (qs(document, '#'+id) as HTMLInputElement).files[0];

    var reader = new FileReader();
    reader.onload = function() {
        var fileContent = JSON.parse(reader.result as string);
        (document.getElementById(id) as HTMLInputElement).value = "";
        //console.log(fileContent);
        if (id == "importerComps") {
            if (allNodes === undefined) {throw `No nodes`;}
            layers.clearLayers();
            selectShadowGroup.clearLayers();
            Object.entries(fileContent).forEach(([id, value]: [string, PLAComponent]) => {
                if (value.nodes === undefined) throw `Component '${id}' has no 'nodes' field`;
                if (value.type === undefined) throw `Component '${id}' has no 'type' field`;
                let mapInfo: PLAComponent = {
                    id: id,
                    attrs: value.attrs ?? {},
                    description: value.description ?? "",
                    displayname: value.displayname ?? "",
                    layer: value.layer ?? 0,
                    nodes: value.nodes,
                    type: value.type.split(" ")[0],
                    tags: value.type.split(" ").length > 1 ? value.type.split(" ").slice(1).join(" ") : ""
                }
                value.type = value.type.split(" ")[0];
                let comp: L.Layer;
                let coords: L.LatLngExpression[] = value.nodes.map(node => {
                    if (!Object.keys(allNodes).includes(node)) throw `Node '${node}' of component '${id}' not found in node list`;
                    return mapcoord([allNodes[node].x, allNodes[node].y]) as L.LatLngExpression;
                });
                //console.log(coords);
                if (ComponentTypes.area.includes(value.type)) {
                    comp = L.polygon(coords, {color: getFrontColor(value.type), weight: getWeight(value.type), pmIgnore: false});
                }
                else if (ComponentTypes.point.includes(value.type)) {
                    comp = L.marker(coords[0], {pmIgnore: false});
                }
                else if (ComponentTypes.line.includes(value.type)) {
                    comp = L.polyline(coords, {color: getFrontColor(value.type), weight: getWeight(value.type), pmIgnore: false});
                } else {
                    throw `Component ${id} has no 'type' field`;
                }
                (comp as Selected).mapInfo = mapInfo;
                // @ts-ignore
                comp._drawnByGeoman = true;
                var a = (e: L.LeafletEvent) => {
                    if (e.layer == selected) select();
                  }
                comp.on("pm:drag", a);
                comp.on("pm:markerdrag", a);
                comp.on("pm:vertexadded", a);
                comp.on("pm:vertexremoved", a);
                comp.on("pm:rotate", a);
                comp.on("click", layerClickEvent);
                //console.log(comp);
                comp.addTo(layers);
            })
        } else if (id == "importerNodes") {
            allNodes = {};
            Object.entries(fileContent).forEach(([id, value]: [string, PLANode]) => {
                if (value.x === undefined) throw `Node ${id} has no field 'x'`;
                if (value.y === undefined) throw `Node ${id} has no field 'y'`;
                allNodes[id] = {
                    x: value.x,
                    y: value.y,
                    connections: value.connections ?? []
                };
            })
        }
    };
    reader.readAsText(importedFile);
    } catch (err) {
        qs(document, "#pane_import #err").innerHTML = `<span style="color: red;">${err}</span>`;
    }
}