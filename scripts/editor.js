var selected = null;
var drawingType = {
  line: null,
  point: null,
  area: null
};
var selectShadowGroup = L.layerGroup([]);
map.addLayer(selectShadowGroup);
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

map.on("pm:drag pm:edit pm:cut pm:rotate", e => {
  if (e.shape == selected) {
    selectShadowGroup.clearLayers();
    if (selected instanceof L.Polygon) {
      L.polygon(selected.getLatLngs(), {color: "yellow", weight: 5, pmIgnore: true, interactive: false}).addTo(selectShadowGroup);
    }
    else if (selected instanceof L.Marker) {
      L.circleMarker(selected.getLatLng(), {color: "yellow", weight: 5, pmIgnore: true, interactive: false}).addTo(selectShadowGroup);
    }
    else {
      L.polyline(selected.getLatLngs(), {color: "yellow", weight: 5, pmIgnore: true, interactive: false}).addTo(selectShadowGroup);
    }
  }
})

map.on("pm:create", e => {
    e.layer.mapInfo = {
      id: "",
      type: drawingType[e.shape == "Line" ? "line" : e.shape == "Marker" ? "point" : "area"],
      displayname: "",
      description: "",
      layer: 0,
      nodes: [],
      attrs: {}
    }

    e.layer.on("click", e => {
      setTimeout(e => {
        // generates a row of attrs
        function genTr(timestamp, name, value) {
          let element = document.createElement('tr');
          element.innerHTML = document.getElementById("c_attr-row").innerHTML;
          element.setAttribute("name", timestamp ? timestamp : new Date().getTime());
          element.querySelector(".c_attr-name").innerHTML = name ? name : "";
          element.querySelector(".c_attr-value").innerHTML = value ? value : "";
          element.querySelector(".c_attr-delete").onclick = e => { // adds deleting button functionality
            delete selected.mapInfo.attrs[e.target.parentElement.getAttribute("name")]
            e.target.parentElement.remove()
            e.target.parentElement.innerHTML = "";
          }
          element.querySelector(".c_attr-delete").querySelector("i").onclick = e => {
            delete selected.mapInfo.attrs[e.target.parentElement.parentElement.getAttribute("name")]
            e.target.parentElement.parentElement.remove()
            e.target.parentElement.parentElement.innerHTML = "";
          }
          element.querySelectorAll(".c_attr-name, .c_attr-value").forEach(element => { //adds saving for attrs
            element.onblur = () => {
              rowElements = document.getElementById("c_attr").querySelectorAll("tr");
              nameElements = document.getElementById("c_attr").querySelectorAll(".c_attr-name");
              valueElements = document.getElementById("c_attr").querySelectorAll(".c_attr-value");

              for (i=0; i < rowElements.length; i++) {
                selected.mapInfo.attrs[rowElements[i].getAttribute("name")] = {
                  name: nameElements[i].value,
                  value: valueElements[i].value
                };
              }
            };
          });
          return element
        }

        selected = e.target;
        document.getElementById("pane_componentInfo").querySelector("div").innerHTML = ""; // clear the pane
        document.getElementById("c_id").innerHTML = selected.mapInfo.id; // sets id, displayname, description, layer
        document.getElementById("c_displayname").innerHTML = selected.mapInfo.displayname;
        document.getElementById("c_description").innerHTML = selected.mapInfo.description;
        document.getElementById("c_layer").innerHTML = selected.mapInfo.layer;

        // add type dropdown
        document.getElementById("c_type").innerHTML = "";
        ComponentTypes[e.shape == "Line" ? "line" : e.shape == "Marker" ? "point" : "area"].forEach(type => {
          let option = document.createElement("option");
          option.setAttribute("value", type);
          option.innerHTML = type;
          document.getElementById("c_type").appendChild(option);
        })
        document.querySelector(`#c_type [value=${selected.mapInfo.type}]`).selected = true;

        // creates selector shadow
        selectShadowGroup.clearLayers();
        if (selected instanceof L.Polygon) {
          L.polygon(selected.getLatLngs(), {color: "yellow", weight: 5, pmIgnore: true, interactive: false}).addTo(selectShadowGroup);
        }
        else if (selected instanceof L.Marker) {
          L.circleMarker(selected.getLatLng(), {color: "yellow", weight: 5, pmIgnore: true, interactive: false}).addTo(selectShadowGroup);
        }
        else {
          L.polyline(selected.getLatLngs(), {color: "yellow", weight: 5, pmIgnore: true, interactive: false}).addTo(selectShadowGroup);
        }

        // adds content to pane
        document.getElementById("pane_componentInfo").querySelector("div").innerHTML = document.getElementById("componentInfo").querySelector("div").innerHTML;

        // check for same IDs
        let ids = layers.getLayers().map(layer => layer.mapInfo.id)
        var hasError = false;
        if (selected.mapInfo.id.trim() == "") {
          document.getElementById("c_emptyIdMsg").hidden = false;
          hasError = true;
        } else document.getElementById("c_emptyIdMsg").hidden = true;
        if (ids.filter(id => id == selected.mapInfo.id).length > 1) {
          document.getElementById("c_duplicateIdMsg").hidden = false;
          document.getElementById("c_duplicateIdMsg").onclick = () => {
            let otherLayer = layers.getLayers().filter(layer => layer.mapInfo.id == selected.mapInfo.id && layer != selected)[0];
            try {map.setView(otherLayer.getCenter(), map.getZoom())}
            catch {map.setView(otherLayer.getLatLng(), map.getZoom())}
            otherLayer.fire('click');
          };
          hasError = true;
        } else document.getElementById("c_duplicateIdMsg").hidden = true;
        if (hasError) {
          document.getElementById("c_id").classList.add("warning");
        } else {
          document.getElementById("c_id").classList.remove("warning");
        }

        document.getElementById("c_add-attr").onclick = () => { //adds add attribute button functionality
          element = genTr()
          document.getElementById("c_attr").appendChild(element)
        };

        document.getElementById("c_attr").innerHTML = ""; //sets attrs
        let attrs = selected.mapInfo.attrs
        Object.keys(attrs).map(key => [key, attrs[key]]).forEach(([timestamp, info]) => {
          element = genTr(timestamp, info.name, info.value);
          document.getElementById("c_attr").appendChild(element);
        })

        // TODO c_type

        Array.from(["id", "displayname", "description"]).forEach(property => { //adds saving and filter for id, displayname, description
          document.getElementById("c_"+property).onblur = () => {
            document.getElementById("c_"+property).innerHTML = document.getElementById("c_"+property).innerHTML.replace(/<br>/gm, "").trim();
            selected.mapInfo[property] = document.getElementById("c_"+property).innerHTML;
          };
        });

        document.getElementById("c_id").onkeyup = () => { // warns if ids have the same name
          let ids = layers.getLayers().map(layer => layer.mapInfo.id)
          let filteredId = document.getElementById("c_id").innerHTML.replace(/<br>/gm, "").trim()
          var hasError = false;
          if (filteredId == "") {
            document.getElementById("c_emptyIdMsg").hidden = false;
            hasError = true;
          } else document.getElementById("c_emptyIdMsg").hidden = true;
          if (ids.filter(id => id == filteredId).length > (filteredId == selected.mapInfo.id ? 1 : 0)) {
            document.getElementById("c_duplicateIdMsg").hidden = false;
            document.getElementById("c_duplicateIdMsg").onclick = () => {
              let otherLayer = layers.getLayers().filter(layer => layer.mapInfo.id == filteredId && layer != selected)[0];
              map.setView(otherLayer.getCenter(), map.getZoom())
              otherLayer.fire('click');
            };
            hasError = true;
          } else document.getElementById("c_duplicateIdMsg").hidden = true;
          if (hasError) document.getElementById("c_id").classList.add("warning")
          else document.getElementById("c_id").classList.remove("warning")
        }

        document.getElementById("c_layer").onblur = () => { //adds saving and filter for layer
          document.getElementById("c_layer").innerHTML = parseFloat(document.getElementById("c_layer").innerHTML.replace(/(?:(?<=^.+)-|[^\d-\.])/gm, "")).toString();
          selected.mapInfo.layer = parseFloat(document.getElementById("c_layer").innerHTML);
        }

        sidebar.enablePanel('pane_componentInfo');
        sidebar.open('pane_componentInfo'); // opens the pane
      }, 10, e);
    });

    e.layer.fire("click");
});

map.on("click", e => {
      selectShadowGroup.clearLayers();
      selected = null;
      document.getElementById('pane_componentInfo').querySelector('div').innerHTML = '<h1>Select a component...</h1>'
    })

map.on("pm:drawstart", e => {
  var types;
  var shape;
  switch (e.shape) {
    case "Line":
      types = ComponentTypes.line;
      shape = "line";
      break;
    case "Marker":
      types = ComponentTypes.point;
      shape = "point";
      break;
    case "Polygon":
    case "Rectangle":
      types = ComponentTypes.area;
      shape = "area";
      break;
    default:
      break;
  }
  if (types) {
    document.getElementById("pane_typePicker").querySelector("div").innerHTML = document.getElementById("typePicker").querySelector("div").innerHTML;
    document.getElementById("tp_table").innerHTML = "";
    document.getElementById('tp_shape').innerHTML = shape;

    types.forEach(type => {
      let element = document.createElement('tr');
      element.innerHTML = document.getElementById("tp_template").innerHTML;
      
      element.classList.add("tp_typeOption")
      element.querySelector(".tp_typeColor").style.background = getFrontColor(type);
      element.querySelector(".tp_typeColor").style.border = "2px solid "+getBackColor(type);
      element.querySelector(".tp_typeName").innerHTML = type;

      element.onclick = e => {
        drawingType[shape] = e.target.parentElement.querySelector(".tp_typeName").innerHTML;
        document.getElementById("tp_table").querySelectorAll("tr").forEach(tr => tr.classList.remove("tp_selected"));
        e.target.parentElement.classList.add("tp_selected")
      }

      document.getElementById("tp_table").appendChild(element);
    })
  }
  if (drawingType[shape] != null) {
    Array.from(document.getElementById("tp_table").querySelectorAll("tr")).filter(tr => tr.querySelector(".tp_typeName").innerHTML == drawingType[shape])[0].classList.add("tp_selected")
  }

  sidebar.open('pane_typePicker');
})

map.on("pm:drawend", e => {
  document.getElementById("pane_typePicker").querySelector("div").innerHTML = document.getElementById("typePicker").querySelector("div").innerHTML;
})