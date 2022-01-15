/// <reference path="references.ts" />
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
function layersToPla(layers) {
    let comps = {};
    let nodes = {};
    let unused_nodes = [];
    layers.forEach(layer => {
        let newComps = JSON.parse(JSON.stringify(layer.mapInfo));
        delete newComps.id;
        layer.getLatLngs()[0].forEach(latlng => {
            let possibleNodes = Object.entries(nodes)
                .filter(([id, node]) => node.x == Math.round(latlng.lng * 64) && node.y == Math.round(latlng.lat * 64));
            if (possibleNodes.length > 0) {
                newComps.nodes.push(possibleNodes[0][0]);
                var index = unused_nodes.indexOf(possibleNodes[0][0]);
                if (index !== -1)
                    unused_nodes.splice(index, 1);
            }
            else {
                let id = genId();
                nodes[id] = { x: Math.round(latlng.lng * 64), y: Math.round(latlng.lat * 64), connections: [] };
                newComps.nodes.push(id);
            }
        });
        comps[layer.mapInfo.id] = newComps;
    });
    unused_nodes.forEach(node => {
        if (nodes[node].connections.length > 1)
            return;
        delete nodes[node];
    });
    return [comps, nodes];
}
