/// <reference path="references.ts" />

var allNodes = {};

function triggerImportData() {
    qs(document, "#importerNodes").click();
    qs(document, "#importerComps").click();
}

function importData(id: string) {
    var importedFile = (qs(document, '#'+id) as HTMLInputElement).files[0];

    var reader = new FileReader();
    reader.onload = function() {
        var fileContent = JSON.parse(reader.result as string);
        (document.getElementById('importer') as HTMLInputElement).value = "";
        //console.log(fileContent);
        if (id == "importerComps") {
            if (allNodes === undefined) {qs(document, "#pane_import #err").innerHTML = `No nodes`; return;}
            Object.entries(fileContent).forEach(([id, value]: [string, PLAComponent]) => {
                if (value.nodes === undefined) {qs(document, "#pane_import #err").innerHTML = `Component ${id} has no 'nodes' field`; return;}
                if (value.type === undefined) {qs(document, "#pane_import #err").innerHTML = `Component ${id} has no 'type' field`; return;}
                let mapInfo: PLAComponent = {
                    attrs: value.attrs ?? {},
                    description: value.description ?? "",
                    displayname: value.displayname ?? "",
                    layer: value.layer ?? 0,
                    nodes: value.nodes,
                    type: value.type
                }
                let comp: L.Layer;
                let coords: L.LatLngExpression[] = value.nodes.map(node => {
                    if (Object.keys(allNodes).includes(node)) {qs(document, "#pane_import #err").innerHTML = `Node ${node} of component ${id} not found in node list`; return;}
                    return allNodes[node].x, allNodes[node].y;
                })
                if (ComponentTypes.area.contains(value.type)) {
                    comp = L.polygon(coords, {color: getFrontColor(value.type), weight: getWeight(value.type)});
                }
                else if (ComponentTypes.point.contains(value.type)) {
                    comp = L.circleMarker(coords[0], {color: getFrontColor(value.type), weight: getWeight(value.type)});
                }
                else if (ComponentTypes.line.contains(value.type)) {
                    comp = L.polyline(coords, {color: getFrontColor(value.type), weight: getWeight(value.type)});
                } else {
                    qs(document, "#pane_import #err").innerHTML = `Component ${id} has no 'type' field`;
                    return;
                }
                comp.addTo(layers);
            })
        } else if (id == "importerNodes") {

        }
    };
    reader.readAsText(importedFile); 
}