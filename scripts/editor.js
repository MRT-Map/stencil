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
        document.getElementById("pane_componentInfo").querySelector("div").innerHTML = "";
        document.getElementById("c_id").innerHTML = e.target.mapInfo.id;
        document.getElementById("c_displayname").innerHTML = e.target.mapInfo.displayname;
        document.getElementById("c_description").innerHTML = e.target.mapInfo.description;

        document.getElementById("pane_componentInfo").querySelector("div").innerHTML = document.getElementById("componentInfo").querySelector("div").innerHTML;

        document.getElementById("c_add-attr").onclick = () => {
          let element = document.createElement('tr');
          element.innerHTML = document.getElementById("c_attr-row").innerHTML;
          element.querySelector(".c_attr-delete").onclick = e => {
            e.target.parentElement.remove()
            e.target.parentElement.innerHTML = "";
          }
          element.querySelector(".c_attr-delete").querySelector("i").onclick = e => {
            e.target.parentElement.parentElement.remove()
            e.target.parentElement.parentElement.innerHTML = "";
          }
          document.getElementById("c_attr").appendChild(element)
        };

        /*sidebar.removePanel('pane_componentInfo');
        sidebar.addPanel({
          id: 'pane_componentInfo',
          tab: '<i class="fas fa-draw-polygon"></i>',
          pane: document.getElementById("componentInfo").innerHTML,
          title: 'Component Info'
        });*/
        sidebar.open('pane_componentInfo');
    });
});