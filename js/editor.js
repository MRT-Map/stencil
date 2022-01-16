/// <reference path="references.ts" />
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
map.pm.addControls({
    position: 'bottomleft',
    drawCircleMarker: false,
    drawCircle: false
});
// @ts-ignore
map.pm.setGlobalOptions({
    layerGroup: layers
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
const qs = (ele, query) => ele.querySelector(query);
const qsa = (ele, query) => ele.querySelectorAll(query);
map.on("pm:drag pm:edit pm:cut pm:rotate", e => {
    // @ts-ignore
    if (e.shape == selected) {
        selectShadowGroup.clearLayers();
        if (selected instanceof L.Polygon) {
            L.polygon(selected.getLatLngs(), { color: "yellow", weight: 5, pmIgnore: true, interactive: false }).addTo(selectShadowGroup);
        }
        else if (selected instanceof L.Marker) {
            L.circleMarker(selected.getLatLng(), { color: "yellow", weight: 5, pmIgnore: true, interactive: false }).addTo(selectShadowGroup);
        }
        else {
            L.polyline(selected.getLatLngs(), { color: "yellow", weight: 5, pmIgnore: true, interactive: false }).addTo(selectShadowGroup);
        }
    }
});
// changes the component type of a component
function typeChange(type) {
    if (Skin.types[selected.mapInfo.type].type == "point")
        return;
    selected.mapInfo.type = type !== null && type !== void 0 ? type : qs(document, "#c_type").value;
    console.log(selected.mapInfo.type);
    selected.setStyle({ weight: getWeight(selected.mapInfo.type), color: getFrontColor(selected.mapInfo.type) });
    if (selectShadowGroup.getLayers().length != 0)
        selectShadowGroup.getLayers()[0].setStyle({ weight: getWeight(selected.mapInfo.type) });
    displayText();
}
map.on("zoomend", e => { if (selected)
    typeChange(); });
// adds text to components
function displayText() {
    return;
    if (ComponentTypes.line.includes(selected.mapInfo.type)) {
        selected.setText(null);
        selected.setText("     " + selected.mapInfo.id + "     ", {
            repeat: true,
            offset: getWeight(selected.mapInfo.type) / 2,
            attributes: {
                fill: 'black',
                style: `font-size: ${Math.max(12, getWeight(selected.mapInfo.type))}px; font-weight: bold;  text-shadow: -1px 0 #fff, 0 1px #fff, 1px 0 #fff, 0 -1px #fff;`
            }
        });
    } /*else if (ComponentTypes.area.includes(selected.mapInfo.type)) {
      
    }*/
}
map.on("pm:create", e => {
    e.layer.mapInfo = {
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
            function genTr(timestamp, name, value) {
                let element = document.createElement('tr');
                element.innerHTML = qs(document, "#c_attr-row").innerHTML;
                element.setAttribute("name", timestamp !== null && timestamp !== void 0 ? timestamp : new Date().getTime());
                qs(element, ".c_attr-name").innerHTML = name !== null && name !== void 0 ? name : "";
                qs(element, ".c_attr-value").innerHTML = value !== null && value !== void 0 ? value : "";
                qs(element, ".c_attr-delete").addEventListener("click", e => {
                    delete selected.mapInfo.attrs[e.target.parentElement.getAttribute("name")];
                    e.target.parentElement.remove();
                    e.target.parentElement.innerHTML = "";
                });
                qs(element, ".c_attr-delete i").addEventListener("click", e => {
                    delete selected.mapInfo.attrs[e.target.parentElement.parentElement.getAttribute("name")];
                    e.target.parentElement.parentElement.remove();
                    e.target.parentElement.parentElement.innerHTML = "";
                });
                qsa(element, ".c_attr-name, .c_attr-value").forEach(element => {
                    element.addEventListener("blur", () => {
                        let rowElements = qsa(document, "#c_attr tr");
                        let nameElements = document.querySelectorAll("#c_attr .c_attr-name");
                        let valueElements = document.querySelectorAll("#c_attr .c_attr-value");
                        for (let i = 0; i < rowElements.length; i++) {
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
            qs(document, "#pane_componentInfo div").innerHTML = ""; // clear the pane
            qs(document, "#c_id").innerHTML = selected.mapInfo.id; // sets id, displayname, description, layer
            qs(document, "#c_displayname").innerHTML = selected.mapInfo.displayname;
            qs(document, "#c_description").innerHTML = selected.mapInfo.description;
            qs(document, "#c_layer").innerHTML = selected.mapInfo.layer.toString();
            // adds content to pane
            qs(document, "#pane_componentInfo div").innerHTML = qs(document, "#componentInfo div").innerHTML;
            // add type dropdown
            qs(document, "#c_type").innerHTML = "";
            ComponentTypes[Skin.types[selected.mapInfo.type].type].forEach(type => {
                let option = document.createElement("option");
                option.setAttribute("value", type);
                option.innerHTML = type;
                qs(document, "#c_type").appendChild(option);
            });
            let selectedOption = document.querySelector(`#c_type [value=${selected.mapInfo.type}]`);
            selectedOption.selected = true;
            qs(document, "#c_type").value = selected.mapInfo.type;
            qs(document, "#c_type").selectedIndex = ComponentTypes[Skin.types[selected.mapInfo.type].type].indexOf(selected.mapInfo.type);
            typeChange(selected.mapInfo.type);
            // creates selector shadow
            selectShadowGroup.clearLayers();
            if (selected instanceof L.Polygon) {
                L.polygon(selected.getLatLngs(), { color: "yellow", weight: getWeight(selected.mapInfo.type), pmIgnore: true, interactive: false }).addTo(selectShadowGroup);
            }
            else if (selected instanceof L.Marker) {
                L.circleMarker(selected.getLatLng(), { color: "yellow", weight: getWeight(selected.mapInfo.type), pmIgnore: true, interactive: false }).addTo(selectShadowGroup);
            }
            else if (selected instanceof L.Polyline) {
                L.polyline(selected.getLatLngs(), { color: "yellow", weight: getWeight(selected.mapInfo.type), pmIgnore: true, interactive: false }).addTo(selectShadowGroup);
            }
            // check for same IDs
            let ids = layers.getLayers().map(layer => layer.mapInfo.id);
            var hasError = false;
            if (selected.mapInfo.id.trim() == "") {
                qs(document, "#c_emptyIdMsg").hidden = false;
                hasError = true;
            }
            else
                qs(document, "#c_emptyIdMsg").hidden = true;
            if (ids.filter(id => id == selected.mapInfo.id).length > 1) {
                qs(document, "#c_duplicateIdMsg").hidden = false;
                qs(document, "#c_duplicateIdMsg").addEventListener("click", () => {
                    let otherLayer = layers.getLayers().filter(layer => layer.mapInfo.id == selected.mapInfo.id && layer != selected)[0];
                    try {
                        map.setView(otherLayer.getCenter(), map.getZoom());
                    }
                    catch (_a) {
                        map.setView(otherLayer.getLatLng(), map.getZoom());
                    }
                    otherLayer.fire('click');
                    // -3824 -29096
                });
                hasError = true;
            }
            else
                qs(document, "#c_duplicateIdMsg").hidden = true;
            if (hasError) {
                qs(document, "#c_id").classList.add("warning");
            }
            else {
                qs(document, "#c_id").classList.remove("warning");
            }
            qs(document, "#c_add-attr").addEventListener("click", () => {
                let element = genTr();
                qs(document, "#c_attr").appendChild(element);
            });
            qs(document, "#c_attr").innerHTML = ""; //sets attrs
            let attrs = selected.mapInfo.attrs;
            Object.keys(attrs).map(key => [key, attrs[key]]).forEach(([timestamp, info]) => {
                let element = genTr(timestamp, info.name, info.value);
                qs(document, "#c_attr").appendChild(element);
            });
            Array.from(["id", "displayname", "description"]).forEach(property => {
                qs(document, "#c_" + property).addEventListener("blur", () => {
                    qs(document, "#c_" + property).innerHTML = qs(document, "#c_" + property).innerHTML.replace(/<br>/gm, "").trim();
                    selected.mapInfo[property] = qs(document, "#c_" + property).innerHTML;
                    displayText();
                });
            });
            qs(document, "#c_id").addEventListener("keyup", () => {
                let ids = layers.getLayers().map(layer => layer.mapInfo.id);
                let filteredId = qs(document, "#c_id").innerHTML.replace(/<br>/gm, "").trim();
                var hasError = false;
                if (filteredId == "") {
                    qs(document, "#c_emptyIdMsg").hidden = false;
                    hasError = true;
                }
                else
                    qs(document, "#c_emptyIdMsg").hidden = true;
                if (ids.filter(id => id == filteredId).length > (filteredId == selected.mapInfo.id ? 1 : 0)) {
                    qs(document, "#c_duplicateIdMsg").hidden = false;
                    qs(document, "#c_duplicateIdMsg").addEventListener("click", () => {
                        let otherLayer = layers.getLayers().filter(layer => layer.mapInfo.id == filteredId && layer != selected)[0];
                        map.setView(otherLayer.getCenter(), map.getZoom());
                        otherLayer.fire('click');
                    });
                    hasError = true;
                }
                else
                    qs(document, "#c_duplicateIdMsg").hidden = true;
                if (hasError)
                    qs(document, "#c_id").classList.add("warning");
                else
                    qs(document, "#c_id").classList.remove("warning");
            });
            qs(document, "#c_layer").addEventListener('blur', () => {
                qs(document, "#c_layer").innerHTML = parseFloat(qs(document, "#c_layer").innerHTML.replace(/(?:(?<=^.+)-|[^\d-\.])/gm, "")).toString();
                selected.mapInfo.layer = parseFloat(qs(document, "#c_layer").innerHTML);
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
    qs(document, '#pane_componentInfo div').innerHTML = '<h1>Select a component...</h1>';
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
        qs(document, "#pane_typePicker div").innerHTML = qs(document, "#typePicker div").innerHTML;
        qs(document, "#tp_table").innerHTML = "";
        qs(document, '#tp_shape').innerHTML = shape;
        types.forEach(type => {
            let element = document.createElement('tr');
            element.innerHTML = qs(document, "#tp_template").innerHTML;
            element.classList.add("tp_typeOption");
            qs(element, ".tp_typeColor").style.background = getFrontColor(type);
            qs(element, ".tp_typeColor").style.border = "2px solid " + getBackColor(type);
            qs(element, ".tp_typeName").innerHTML = type;
            element.addEventListener("click", e => {
                drawingType[shape] = qs(e.target.parentElement, ".tp_typeName").innerHTML;
                qsa(document, "#tp_table tr").forEach(tr => tr.classList.remove("tp_selected"));
                e.target.parentElement.classList.add("tp_selected");
            });
            if (type == "simple" + shape.charAt(0).toUpperCase() + shape.slice(1)) {
                element.classList.add("tp_selected");
            }
            qs(document, "#tp_table").appendChild(element);
        });
    }
    if (drawingType[shape] != null) {
        Array.from(qsa(document, "#tp_table tr")).filter(tr => qs(tr, ".tp_typeName").innerHTML == drawingType[shape])[0].classList.add("tp_selected");
    }
    else {
        drawingType[shape] = "simple" + shape.charAt(0).toUpperCase() + shape.slice(1);
    }
    sidebar.open('pane_typePicker');
});
map.on("pm:drawend", e => {
    qs(document, "#pane_typePicker div").innerHTML = qs(document, "#typePicker div").innerHTML;
});
