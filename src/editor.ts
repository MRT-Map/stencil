/// <reference path="references.ts" />

interface Selected extends L.Polyline {
  mapInfo: any
}

var selected: Selected = null;
var drawingType = {
  line: null,
  point: null,
  area: null
};
var selectShadowGroup = L.layerGroup([]);
map.addLayer(selectShadowGroup);
var layers = L.layerGroup([]);
map.addLayer(layers);

map.pm.addControls({  
  position: 'bottomleft',
  drawCircleMarker: false,
  drawCircle: false
}); 
/*
map.on("pm:drawstart", ({workingLayer}) => {
  workingLayer.on("pm:vertexadded pm:centerplaced", e => {
    let newLatlng = {lat: Math.round(e.latlng.lat*64)/64, lng: Math.round(e.latlng.lng*64)/64}
    e.layer._latlngs[e.layer._latlngs.length-1] = newLatlng;
    e.layer._latlngInfo[e.layer._latlngInfo.length-1].latlng = newLatlng;
    e.latlng = newLatlng
  });
});
*/


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
      L.polyline(selected.getLatLngs() as L.LatLngExpression[], {color: "yellow", weight: 5, pmIgnore: true, interactive: false}).addTo(selectShadowGroup);
    }
  }
});

// changes the component type of a component
function typeChange(type?) {
  if (Skin.types[selected.mapInfo.type].type == "point") return;
  selected.mapInfo.type = type ?? (document.getElementById("c_type") as HTMLInputElement).value;
  console.log(selected.mapInfo.type);
  selected.setStyle({weight: getWeight(selected.mapInfo.type), color: getFrontColor(selected.mapInfo.type)});
  if (selectShadowGroup.getLayers().length != 0) (selectShadowGroup.getLayers()[0] as L.Polyline).setStyle({weight: getWeight(selected.mapInfo.type)});
  displayText();
}
map.on("zoomend", e => {if (selected) typeChange();});

// adds text to components
function displayText() {
  if (ComponentTypes.line.includes(selected.mapInfo.type)) {
    (selected as L.Polyline).setText(null);
    (selected as L.Polyline).setText("     "+selected.mapInfo.id+"     ", {
      repeat: true,
      offset: getWeight(selected.mapInfo.type)/2,
      attributes: {
        fill: 'black',
        style: `font-size: ${Math.max(12, getWeight(selected.mapInfo.type))}px; font-weight: bold;  text-shadow: -1px 0 #fff, 0 1px #fff, 1px 0 #fff, 0 -1px #fff;`
      }
    });
  } /*else if (ComponentTypes.area.includes(selected.mapInfo.type)) {
    
  }*/
}

map.on("pm:create", e => {
    (e.layer as Selected).mapInfo = {
      id: "",
      type: drawingType[e.shape == "Line" ? "line" : e.shape == "Marker" ? "point" : "area"],
      displayname: "",
      description: "",
      layer: 0,
      nodes: [],
      attrs: {}
    };

    e.layer.on("click", e => {
      setTimeout(() => {
        // generates a row of attrs
        function genTr(timestamp?, name?, value?) {
          let element = document.createElement('tr');
          element.innerHTML = document.getElementById("c_attr-row").innerHTML;
          element.setAttribute("name", timestamp ?? new Date().getTime());
          element.querySelector(".c_attr-name").innerHTML = name ?? "";
          element.querySelector(".c_attr-value").innerHTML = value ?? "";
          element.querySelector(".c_attr-delete").addEventListener("click", e => { // adds deleting button functionality
            delete selected.mapInfo.attrs[(e.target as HTMLElement).parentElement.getAttribute("name")];
            (e.target as HTMLElement).parentElement.remove();
            (e.target as HTMLElement).parentElement.innerHTML = "";
          });
          element.querySelector(".c_attr-delete").querySelector("i").addEventListener("click", e => {
            delete selected.mapInfo.attrs[(e.target as HTMLElement).parentElement.parentElement.getAttribute("name")];
            (e.target as HTMLElement).parentElement.parentElement.remove();
            (e.target as HTMLElement).parentElement.parentElement.innerHTML = "";
          });
          element.querySelectorAll(".c_attr-name, .c_attr-value").forEach(element => { //adds saving for attrs
            element.addEventListener("blur", () => {
              let rowElements = document.getElementById("c_attr").querySelectorAll("tr");
              let nameElements: NodeListOf<HTMLInputElement> = document.getElementById("c_attr").querySelectorAll(".c_attr-name");
              let valueElements: NodeListOf<HTMLInputElement> = document.getElementById("c_attr").querySelectorAll(".c_attr-value");

              for (let i=0; i < rowElements.length; i++) {
                selected.mapInfo.attrs[rowElements[i].getAttribute("name")] = {
                  name: nameElements[i].value,
                  value: valueElements[i].value
                };
              }
            });
          });
          return element;
        }

        selected = e.target;
        document.getElementById("pane_componentInfo").querySelector("div").innerHTML = ""; // clear the pane
        document.getElementById("c_id").innerHTML = selected.mapInfo.id; // sets id, displayname, description, layer
        document.getElementById("c_displayname").innerHTML = selected.mapInfo.displayname;
        document.getElementById("c_description").innerHTML = selected.mapInfo.description;
        document.getElementById("c_layer").innerHTML = selected.mapInfo.layer;

        // adds content to pane
        document.getElementById("pane_componentInfo").querySelector("div").innerHTML = document.getElementById("componentInfo").querySelector("div").innerHTML;
        
        // add type dropdown
        document.getElementById("c_type").innerHTML = "";
        ComponentTypes[Skin.types[selected.mapInfo.type].type].forEach(type => {
          let option = document.createElement("option");
          option.setAttribute("value", type);
          option.innerHTML = type;
          document.getElementById("c_type").appendChild(option);
        });
        let selectedOption: HTMLOptionElement = document.querySelector(`#c_type [value=${selected.mapInfo.type}]`);
        selectedOption.selected = true;
        (document.getElementById("c_type") as HTMLSelectElement).value = selected.mapInfo.type;
        (document.getElementById("c_type") as HTMLSelectElement).selectedIndex = ComponentTypes[Skin.types[selected.mapInfo.type].type].indexOf(selected.mapInfo.type);
        typeChange(selected.mapInfo.type);

        // creates selector shadow
        selectShadowGroup.clearLayers();
        if (selected instanceof L.Polygon) {
          L.polygon(selected.getLatLngs(), {color: "yellow", weight: getWeight(selected.mapInfo.type), pmIgnore: true, interactive: false}).addTo(selectShadowGroup);
        }
        else if (selected instanceof L.Marker) {
          L.circleMarker(selected.getLatLng(), {color: "yellow", weight: getWeight(selected.mapInfo.type), pmIgnore: true, interactive: false}).addTo(selectShadowGroup);
        }
        else if (selected instanceof L.Polyline) {
          L.polyline(selected.getLatLngs() as L.LatLng[], {color: "yellow", weight: getWeight(selected.mapInfo.type), pmIgnore: true, interactive: false}).addTo(selectShadowGroup);
        }

        
        // check for same IDs
        let ids = layers.getLayers().map(layer => (layer as Selected).mapInfo.id);
        var hasError = false;
        if (selected.mapInfo.id.trim() == "") {
          document.getElementById("c_emptyIdMsg").hidden = false;
          hasError = true;
        } else document.getElementById("c_emptyIdMsg").hidden = true;
        if (ids.filter(id => id == selected.mapInfo.id).length > 1) {
          document.getElementById("c_duplicateIdMsg").hidden = false;
          document.getElementById("c_duplicateIdMsg").addEventListener("click", () => {
            let otherLayer = layers.getLayers().filter(layer => (layer as Selected).mapInfo.id == selected.mapInfo.id && layer != selected)[0];
            try {map.setView((otherLayer as L.Polyline).getCenter(), map.getZoom());}
            catch {map.setView((otherLayer as L.Marker).getLatLng(), map.getZoom());}
            otherLayer.fire('click');
          });
          hasError = true;
        } else document.getElementById("c_duplicateIdMsg").hidden = true;
        if (hasError) {
          document.getElementById("c_id").classList.add("warning");
        } else {
          document.getElementById("c_id").classList.remove("warning");
        }

        document.getElementById("c_add-attr").addEventListener("click", () => { //adds add attribute button functionality
          let element = genTr();
          document.getElementById("c_attr").appendChild(element);
        });

        document.getElementById("c_attr").innerHTML = ""; //sets attrs
        let attrs = selected.mapInfo.attrs;
        Object.keys(attrs).map(key => [key, attrs[key]]).forEach(([timestamp, info]) => {
          let element = genTr(timestamp, info.name, info.value);
          document.getElementById("c_attr").appendChild(element);
        });

        Array.from(["id", "displayname", "description"]).forEach(property => { //adds saving and filter for id, displayname, description
          document.getElementById("c_"+property).addEventListener("blur", () => {
            document.getElementById("c_"+property).innerHTML = document.getElementById("c_"+property).innerHTML.replace(/<br>/gm, "").trim();
            selected.mapInfo[property] = document.getElementById("c_"+property).innerHTML;
            displayText();
          });
        });

        document.getElementById("c_id").addEventListener("keyup", () => { // warns if ids have the same name
          let ids = layers.getLayers().map(layer => (layer as Selected).mapInfo.id);
          let filteredId = document.getElementById("c_id").innerHTML.replace(/<br>/gm, "").trim();
          var hasError = false;
          if (filteredId == "") {
            document.getElementById("c_emptyIdMsg").hidden = false;
            hasError = true;
          } else document.getElementById("c_emptyIdMsg").hidden = true;
          if (ids.filter(id => id == filteredId).length > (filteredId == selected.mapInfo.id ? 1 : 0)) {
            document.getElementById("c_duplicateIdMsg").hidden = false;
            document.getElementById("c_duplicateIdMsg").addEventListener("click", () => {
              let otherLayer = layers.getLayers().filter(layer => (layer as Selected).mapInfo.id == filteredId && layer != selected)[0];
              map.setView((otherLayer as L.Polyline).getCenter(), map.getZoom());
              otherLayer.fire('click');
            });
            hasError = true;
          } else document.getElementById("c_duplicateIdMsg").hidden = true;
          if (hasError) document.getElementById("c_id").classList.add("warning");
          else document.getElementById("c_id").classList.remove("warning");
        });

        document.getElementById("c_layer").addEventListener('blur', () => { //adds saving and filter for layer
          document.getElementById("c_layer").innerHTML = parseFloat(document.getElementById("c_layer").innerHTML.replace(/(?:(?<=^.+)-|[^\d-\.])/gm, "")).toString();
          selected.mapInfo.layer = parseFloat(document.getElementById("c_layer").innerHTML);
        });

        sidebar.enablePanel('pane_componentInfo');
        sidebar.open('pane_componentInfo'); // opens the pane
      }, 10);
    });

    e.layer.fire("click");
});

map.on("click", e => {
      selectShadowGroup.clearLayers();
      selected = null;
      document.getElementById('pane_componentInfo').querySelector('div').innerHTML = '<h1>Select a component...</h1>';
});

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
      
      element.classList.add("tp_typeOption");
      (<HTMLElement>element.querySelector(".tp_typeColor")).style.background = getFrontColor(type);
      (<HTMLElement>element.querySelector(".tp_typeColor")).style.border = "2px solid "+getBackColor(type);
      element.querySelector(".tp_typeName").innerHTML = type;

      element.addEventListener("click", e => {
        drawingType[shape] = (e.target as HTMLElement).parentElement.querySelector(".tp_typeName").innerHTML;
        document.getElementById("tp_table").querySelectorAll("tr").forEach(tr => tr.classList.remove("tp_selected"));
        (e.target as HTMLElement).parentElement.classList.add("tp_selected");
      });

      if (type == "simple"+shape.charAt(0).toUpperCase() + shape.slice(1)) {
        element.classList.add("tp_selected");
      }

      document.getElementById("tp_table").appendChild(element);
    });
  }
  if (drawingType[shape] != null) {
    Array.from(document.getElementById("tp_table").querySelectorAll("tr")).filter(tr => tr.querySelector(".tp_typeName").innerHTML == drawingType[shape])[0].classList.add("tp_selected");
  }
  else {
    drawingType[shape] = "simple"+shape.charAt(0).toUpperCase() + shape.slice(1);
  }

  sidebar.open('pane_typePicker');
});

map.on("pm:drawend", e => {
  document.getElementById("pane_typePicker").querySelector("div").innerHTML = document.getElementById("typePicker").querySelector("div").innerHTML;
});