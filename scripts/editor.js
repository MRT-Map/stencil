var selected = null

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
    console.log("Created")

    e.layer.on("click", e => {
      // generates a row of attrs
      function genTr(timestamp, name, value) {
        let element = document.createElement('tr');
        element.innerHTML = document.getElementById("c_attr-row").innerHTML;
        element.setAttribute("name", timestamp ? timestamp : new Date().toUTCString());
        element.querySelector(".c_attr-name").innerHTML = name ? name : "";
        element.querySelector(".c_attr-value").innerHTML = value ? value : "";
        element.querySelector(".c_attr-delete").onclick = e => {
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
              }
            }
          }
        })
        return element
      }

      selected = e.target;
      document.getElementById("pane_componentInfo").querySelector("div").innerHTML = ""; // clear the pane
      document.getElementById("c_id").innerHTML = selected.mapInfo.id; // sets id, displayname, description, layer
      document.getElementById("c_displayname").innerHTML = selected.mapInfo.displayname;
      document.getElementById("c_description").innerHTML = selected.mapInfo.description;
      document.getElementById("c_layer").innerHTML = selected.mapInfo.layer;

      document.getElementById("c_attr").innerHTML = ""; //sets attrs
      let attrs = selected.mapInfo.attrs
      Object.keys(attrs).map(key => [key, attrs[key]]).forEach(([timestamp, info]) => {
        element = genTr(timestamp, info.name, info.value);
        document.getElementById("c_attr").appendChild(element);
      })

      // adds content to pane
      document.getElementById("pane_componentInfo").querySelector("div").innerHTML = document.getElementById("componentInfo").querySelector("div").innerHTML;

      document.getElementById("c_add-attr").onclick = () => { //adds add attribute button functionality
        element = genTr()
        document.getElementById("c_attr").appendChild(element)
      };
      
      // TODO c_type

      ["id", "displayname", "description"].forEach(property => { //adds saving and filter for id, displayname, description
        document.getElementById("c_"+property).onblur = () => {
          document.getElementById("c_"+property).innerHTML = document.getElementById("c_"+property).innerHTML.replaceAll(/<br>/gm, "");
          selected.mapInfo[property] = document.getElementById("c_"+property).innerHTML;
        }
      })

      document.getElementById("c_layer").onblur = () => { //adds saving and filter for layer
        document.getElementById("c_layer").innerHTML = parseFloat(document.getElementById("c_layer").innerHTML.replaceAll(/(?:(?<=^.+)-|[^\d-\.])/gm, "")).toString();
        selected.mapInfo.layer = parseFloat(document.getElementById("c_layer").innerHTML);
      }

      sidebar.enablePanel('pane_componentInfo');
      sidebar.open('pane_componentInfo'); // opens the pane
    });

    e.layer.off("click", e => {
      selected = null;
      sidebar.disablePanel('pane_componentInfo');
    })
});