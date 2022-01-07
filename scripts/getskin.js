/// <reference path="references.ts" />
var Skin = null;
var ComponentTypes = null;
var $;
$.ajax({
    async: false,
    url: "https://raw.githubusercontent.com/MRT-Map/tile-renderer/main/renderer/skins/default.json",
    success: function (data) {
        var json = JSON.parse(data);
        Skin = json;
        ComponentTypes = {
            all: Object.keys(json.types),
            point: Object.keys(json.types).filter(function (type) { return json.types[type].type == "point"; }),
            line: Object.keys(json.types).filter(function (type) { return json.types[type].type == "line"; }),
            area: Object.keys(json.types).filter(function (type) { return json.types[type].type == "area"; })
        };
    }
});
function getFrontColor(type) {
    var typeLayers = Object.values(Skin.types[type].style)[0]; // TODO type for PLA thingy
    var filteredLayers;
    switch (Skin.types[type].type) {
        case "point":
            filteredLayers = typeLayers.filter(function (typeLayer) { return ['circle', 'square'].indexOf(typeLayer.layer) != -1; });
            if (filteredLayers.length == 0)
                return "#808080";
            else
                return filteredLayers[filteredLayers.length - 1].colour;
            break;
        case "line":
            filteredLayers = typeLayers.filter(function (typeLayer) { return typeLayer.layer == "fore"; });
            return filteredLayers[filteredLayers.length - 1].colour;
        case "area":
            filteredLayers = typeLayers.filter(function (typeLayer) { return typeLayer.layer == "fill"; });
            return filteredLayers[filteredLayers.length - 1].colour;
        default:
            return undefined;
    }
}
function getBackColor(type) {
    var typeLayers = Object.values(Skin.types[type].style)[0];
    var filteredLayers;
    switch (Skin.types[type].type) {
        case "point":
            return null;
        case "line":
            filteredLayers = typeLayers.filter(function (typeLayer) { return typeLayer.layer == "back"; });
            if (filteredLayers.length != 0)
                return filteredLayers[filteredLayers.length - 1].colour;
            else
                return null;
            break;
        case "area":
            filteredLayers = typeLayers.filter(function (typeLayer) { return typeLayer.layer == "fill"; });
            return filteredLayers[filteredLayers.length - 1].outline;
        default:
            return undefined;
    }
}
function getWeight(type) {
    var typeLayers = Object.values(Skin.types[type].style)[0];
    var filteredLayers;
    switch (Skin.types[type].type) {
        case "point":
            return null;
        case "line":
            filteredLayers = typeLayers.filter(function (typeLayer) { return typeLayer.layer == "fore"; });
            if (filteredLayers.length != 0)
                return filteredLayers[filteredLayers.length - 1].width / Math.pow(2, (8 - map.getZoom() + 1));
            else
                return null;
            break;
        case "area":
            /*filteredLayers = typeLayers.filter(typeLayer => typeLayer.layer == "fill");
            return filteredLayers[filteredLayers.length-1].outline;*/
            return 2 / Math.pow(2, (8 - map.getZoom() + 1));
        default:
            return undefined;
    }
}
