/// <reference path="references.ts" />
let Skin = null;
let ComponentTypes = null;
$.ajax({
    async: false,
    url: "https://raw.githubusercontent.com/MRT-Map/tile-renderer/beta/renderer/skins/default.json",
    success: data => {
        let json = JSON.parse(data);
        Skin = json;
        ComponentTypes = {
            all: Object.keys(json.types),
            point: Object.keys(json.types).filter(type => json.types[type].type == "point"),
            line: Object.keys(json.types).filter(type => json.types[type].type == "line"),
            area: Object.keys(json.types).filter(type => json.types[type].type == "area")
        };
    }
});
function getFrontColor(type) {
    let typeLayers = Object.values(Skin.types[type].style)[0]; // TODO type for PLA thingy
    let filteredLayers;
    switch (Skin.types[type].type) {
        case "point":
            filteredLayers = typeLayers.filter(typeLayer => ['circle', 'square'].indexOf(typeLayer.layer) != -1);
            if (filteredLayers.length == 0)
                return "#808080";
            else
                return filteredLayers[filteredLayers.length - 1].colour;
        case "line":
            filteredLayers = typeLayers.filter(typeLayer => typeLayer.layer == "fore");
            return filteredLayers[filteredLayers.length - 1].colour;
        case "area":
            filteredLayers = typeLayers.filter(typeLayer => typeLayer.layer == "fill");
            return filteredLayers[filteredLayers.length - 1].colour;
        default:
            return undefined;
    }
}
function getBackColor(type) {
    let typeLayers = Object.values(Skin.types[type].style)[0];
    let filteredLayers;
    switch (Skin.types[type].type) {
        case "point":
            return null;
        case "line":
            filteredLayers = typeLayers.filter(typeLayer => typeLayer.layer == "back");
            if (filteredLayers.length != 0)
                return filteredLayers[filteredLayers.length - 1].colour;
            else
                return null;
        case "area":
            filteredLayers = typeLayers.filter(typeLayer => typeLayer.layer == "fill");
            return filteredLayers[filteredLayers.length - 1].outline;
        default:
            return undefined;
    }
}
function getWeight(type) {
    let typeLayers = Object.values(Skin.types[type].style)[0];
    let filteredLayers;
    switch (Skin.types[type].type) {
        case "point":
            return null;
        case "line":
            filteredLayers = typeLayers.filter(typeLayer => typeLayer.layer == "fore");
            if (filteredLayers.length != 0)
                return filteredLayers[filteredLayers.length - 1].width / 2;
            else
                return null;
        case "area":
            /*filteredLayers = typeLayers.filter(typeLayer => typeLayer.layer == "fill");
            return filteredLayers[filteredLayers.length-1].outline;*/
            return 4;
        default:
            return undefined;
    }
}
// *2**(8-map.getZoom())
