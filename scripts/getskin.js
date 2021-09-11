Skin = null;
ComponentTypes = null;
$.ajax({
    async: false,
    url: "https://raw.githubusercontent.com/MRT-Map/tile-renderer/main/renderer/skins/default.json",
    success: data => {
      json = JSON.parse(data)
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
  let typeLayers = Object.values(Skin.types[type].style)[0];
  let filteredLayers;
  switch (Skin.types[type].type) {
    case "point":
      filteredLayers = typeLayers.filter(typeLayer => ['circle', 'square'].indexOf(typeLayer.layer) != -1);
      if (filteredLayers.length == 0) return "#808080"
      else return filteredLayers[filteredLayers.length-1].colour;
      break;
    case "line":
      filteredLayers = typeLayers.filter(typeLayer => typeLayer.layer == "fore");
      return filteredLayers[filteredLayers.length-1].colour;
      break;
    case "area":
      filteredLayers = typeLayers.filter(typeLayer => typeLayer.layer == "fill");
      return filteredLayers[filteredLayers.length-1].colour;
      break;
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
      break;
    case "line":
      filteredLayers = typeLayers.filter(typeLayer => typeLayer.layer == "back");
      if (filteredLayers.length != 0) return filteredLayers[filteredLayers.length-1].colour;
      else return null;
      break;
    case "area":
      filteredLayers = typeLayers.filter(typeLayer => typeLayer.layer == "fill");
      return filteredLayers[filteredLayers.length-1].outline;
      break;
    default:
      return undefined;
  }
}