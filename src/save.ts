/// <reference path="references.ts" />

function genId(): string {
    function b10_b64(n: number) {
        const BASE64 = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        let q = n;
        let o = "";
        while (true) {
            o += BASE64[q % 64];
            q = Math.floor(q / 64);
            if (q == 0) break;
        }
        return o.split("").reverse().join("");
    }
    let decimalId = Math.round(new Date().getTime() * 10000000)
    return b10_b64(decimalId) + "-" + b10_b64(Math.floor(Math.random() * 64**5 + 1));
}

function layersToPla(layers: Selected[]): [{ [id: string] : PLAComponent; }, { [id: string] : PLANode; }] {
  let comps:  { [id: string] : PLAComponent; } = {};
  let nodes: { [id: string] : PLANode; } = {};
  let unused_nodes: string[] = [];
  layers.forEach(layer => {
    let newComps = JSON.parse(JSON.stringify(layer.mapInfo));
    delete newComps.id;
    
    //(layer.getLatLngs()[0] as L.LatLng[]).forEach(latlng => {
    function resolve_nodes(latlng: L.LatLng, hollowIndex?: number) {
      let id;
      let possibleNodes = Object.entries(nodes)
        .filter(([id, node]) =>
          node.x == Math.round(latlng.lng*64) && node.y == Math.round(latlng.lat*64));
      if (possibleNodes.length > 0) {
        id = possibleNodes[0][0];
        var index = unused_nodes.indexOf(id);
        if (index !== -1) unused_nodes.splice(index, 1);
      }
      else {
        id = genId()
        nodes[id] = {x: Math.round(latlng.lng*64), y: Math.round(latlng.lat*64), connections: []};
      }
      if (hollowIndex) {
        if (newComps.hollows === undefined) newComps.hollows = [];
        if (newComps.hollows.length <= hollowIndex) newComps.hollows.push([]);
        newComps.hollows[hollowIndex].push(id);
      } else newComps.nodes.push(id);
    }
    if (layer instanceof L.Polygon) {
      (layer.getLatLngs()[0] as L.LatLng[]).forEach(layer => resolve_nodes(layer));
      if (layer.getLatLngs().length > 1) (layer.getLatLngs().slice(1) as L.LatLng[])
        .forEach((layer, i) => resolve_nodes(layer, i-1))
    }
    else if (layer instanceof L.Marker) resolve_nodes(layer.getLatLng());
    else (layer.getLatLngs() as L.LatLng[]).forEach(layer => resolve_nodes(layer));
    comps[layer.mapInfo.id] = newComps;
  });
  unused_nodes.forEach(node => {
    if (nodes[node].connections.length > 1) return;
    delete nodes[node];
  });
  return [comps, nodes];
}