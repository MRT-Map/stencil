Skin = null;
ComponentTypes = null;
$.ajax({
    async: true,
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