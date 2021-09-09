var layers = L.layerGroup([]);
map.addLayer(layers);
map.pm.setGlobalOptions({
  layerGroup: layers
});

map.pm.addControls({  
  position: 'bottomleft',
  drawCircleMarker: false,
  drawCircle: false
}); 

map.on("pm:vertexadded pm:centerplaced", e => {
    e.lat = Math.round(e.lat);
    e.lng = Math.round(e.lng);
});

map.on("pm:create", e => {
    e.layer.mapInfo = {
        id: "",
        type: e.shape == "Line" ? "simpleLine" : e.shape == "Marker" ? "simplePoint" : "simpleArea",
        displayname: "",
        description: "",
        layer: 0,
        nodes: [],
        attrs: {}
    }

    e.layer.on("click", e => {
        document.getElementById("componentInfo").innerHTML = "asdfasdfasdf";

        sidebar.removePanel('pane_componentInfo');
        sidebar.addPanel({
            id: 'pane_componentInfo',
            tab: '<i class="fas fa-draw-polygon"></i>',
            pane: document.getElementById("componentInfo").innerHTML,
            title: 'Component Info'
          });
          sidebar.open('pane_componentInfo');
    });
});