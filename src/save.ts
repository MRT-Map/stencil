/// <reference path="references.ts" />

function getComponentState() {
  return layers.getLayers().map((layer: Selected) => {
    return {
      mapInfo: layer.mapInfo,
      shape: layer instanceof L.Marker ? "point" : layer instanceof L.Polygon ? "area" : "line",
      latlngs: layer instanceof L.Marker ? layer.getLatLng() : layer.getLatLngs()
    };  
  });
}

setInterval(() => {
  if (!("stencil" in localStorage)) localStorage.stencil = LZString.compress("[]");
  if (!("stencilview" in localStorage)) localStorage.stencilview = LZString.compress(JSON.stringify({lat: 0, lng: 0, zoom: 8}));
  localStorage.stencil = LZString.compress(JSON.stringify(getComponentState()));
  localStorage.stencilview = LZString.compress(JSON.stringify({...map.getBounds().getCenter(), zoom: map.getZoom()}))
  if (!("stencilallnodes" in localStorage)) localStorage.stencilallnodes = LZString.compress("{}");
  localStorage.stencilallnodes = LZString.compress(JSON.stringify(allNodes));
}, 1000);

setTimeout(() => {
  if (!("stencil" in localStorage)) localStorage.stencil = LZString.compress("[]");
  if (!("stencilview" in localStorage)) localStorage.stencilview = LZString.compress(JSON.stringify({lat: 0, lng: 0, zoom: 8}));
  if (!("stencilallnodes" in localStorage)) localStorage.stencilallnodes = LZString.compress("{}");
  let {lat, lng, zoom} = JSON.parse(LZString.decompress(localStorage.stencilview));
  map.setView({lat: lat, lng: lng}, zoom);
  allNodes = JSON.parse(LZString.decompress(localStorage.stencilallnodes))
  JSON.parse(LZString.decompress(localStorage.stencil)).forEach(({mapInfo, shape, latlngs}) => {
    let layer = shape == "point" ? L.marker(latlngs, {pmIgnore: false})
              : shape == "line" ? L.polyline(latlngs, {
                  color: getFrontColor(mapInfo.type),
                  weight: getWeight(mapInfo.type),
                  pmIgnore: false
                })
              : L.polygon(latlngs, {
                color: getFrontColor(mapInfo.type),
                weight: getWeight(mapInfo.type),
                pmIgnore: false
              });
    (layer as Selected).mapInfo = mapInfo;
    // @ts-ignore
    layer._drawnByGeoman = true;
    var a = (e: L.LeafletEvent) => {
        if (e.layer == selected) select();
      }
    layer.on("pm:drag", a);
    layer.on("pm:markerdrag", a);
    layer.on("pm:vertexadded", a);
    layer.on("pm:vertexremoved", a);
    layer.on("pm:rotate", a);
    layer.on("click", layerClickEvent);
    layer.addTo(layers);
  });
}, 100);
