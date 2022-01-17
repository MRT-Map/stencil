/// <reference path="references.ts" />
var setName = "city";
qs(document, "#nameField").value = setName;
function genId() {
    function b10_b64(n) {
        const BASE64 = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        let q = n;
        let o = "";
        while (true) {
            o += BASE64[q % 64];
            q = Math.floor(q / 64);
            if (q == 0)
                break;
        }
        return o.split("").reverse().join("");
    }
    let decimalId = Math.round(new Date().getTime() * 10000000);
    return b10_b64(decimalId) + "-" + b10_b64(Math.floor(Math.random() * Math.pow(64, 5) + 1));
}
function exportData() {
    qs(document, "#pane_export #err").innerHTML = "";
    try {
        checkLayerIds(layers.getLayers());
    }
    catch ([err, where]) {
        qs(document, "#pane_export #err").innerHTML = err;
        if (where) {
            try {
                map.setView(where.getCenter(), map.getZoom());
            }
            catch (_a) {
                map.setView(where.getLatLng(), map.getZoom());
            }
            where.fire('click');
        }
        return;
    }
    let [comps, nodes] = layersToPla(layers.getLayers());
    let dataStr = "data:text/json;charset=utf-8," + encodeURIComponent(JSON.stringify(comps, null, 2));
    let dlAnchorElem = document.querySelector('#downloader');
    dlAnchorElem.href = dataStr;
    dlAnchorElem.download = setName + ".comps.pla";
    dlAnchorElem.click();
    dataStr = "data:text/json;charset=utf-8," + encodeURIComponent(JSON.stringify(nodes, null, 2));
    dlAnchorElem.href = dataStr;
    dlAnchorElem.download = setName + ".nodes.pla";
    dlAnchorElem.click();
}
function checkLayerIds(layers) {
    let ids = [];
    layers.forEach(layer => {
        if (ids.includes(layer.mapInfo.id))
            throw ["Duplicate ID: '" + layer.mapInfo.id + "'", layer];
        else if (layer.mapInfo.id.trim() == "")
            throw ["Empty ID", layer];
        ids.push(layer.mapInfo.id);
    });
}
function layersToPla(layers) {
    let comps = {};
    let nodes = {};
    let unused_nodes = [];
    layers.forEach(layer => {
        let newComps = JSON.parse(JSON.stringify(layer.mapInfo));
        delete newComps.id;
        newComps.type = (newComps.type + " " + newComps.tags.trim()).trim();
        delete newComps.tags;
        //(layer.getLatLngs()[0] as L.LatLng[]).forEach(latlng => {
        function resolve_nodes(latlng, hollowIndex) {
            let id;
            let possibleNodes = Object.entries(nodes)
                .filter(([id, node]) => node.x == Math.round(latlng.lng * 64) && node.y == Math.round(latlng.lat * 64));
            if (possibleNodes.length > 0) {
                id = possibleNodes[0][0];
                var index = unused_nodes.indexOf(id);
                if (index !== -1)
                    unused_nodes.splice(index, 1);
            }
            else {
                id = genId();
                nodes[id] = { x: Math.round(latlng.lng * 64), y: Math.round(latlng.lat * 64), connections: [] };
            }
            if (hollowIndex) {
                if (newComps.hollows === undefined)
                    newComps.hollows = [];
                if (newComps.hollows.length <= hollowIndex)
                    newComps.hollows.push([]);
                newComps.hollows[hollowIndex].push(id);
            }
            else
                newComps.nodes.push(id);
        }
        if (layer instanceof L.Polygon) {
            layer.getLatLngs()[0].forEach(layer => resolve_nodes(layer));
            if (layer.getLatLngs().length > 1)
                layer.getLatLngs().slice(1)
                    .forEach((layer, i) => resolve_nodes(layer, i - 1));
        }
        else if (layer instanceof L.Marker)
            resolve_nodes(layer.getLatLng());
        else
            layer.getLatLngs().forEach(layer => resolve_nodes(layer));
        comps[layer.mapInfo.id] = newComps;
    });
    unused_nodes.forEach(node => {
        if (nodes[node].connections.length > 1)
            return;
        delete nodes[node];
    });
    return [comps, nodes];
}
