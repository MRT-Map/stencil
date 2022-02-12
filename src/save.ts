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
  localStorage.stencil = LZString.compress(JSON.stringify(getComponentState()));
}, 1000);

setTimeout(() => {
  if (!("stencil" in localStorage)) localStorage.stencil = LZString.compress("[]");
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
    layer.addTo(layers);
  });
}, 100);