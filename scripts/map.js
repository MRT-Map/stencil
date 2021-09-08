VERSION = "v0.0"

var map = L.map('map', {
  crs: L.CRS.Simple
}).setView([0, 0], 8);

//override the default
L.TileLayer.customTileLayer = L.TileLayer.extend({
  getTileUrl: function(coords) {


    let Zcoord = 2 ** (8 - coords.z);
    let Xcoord = (coords.x * 1);
    let Ycoord = coords.y * -1;

    let group = {
      x: Math.floor(Xcoord * Zcoord / 32),
      y: Math.floor(Ycoord * Zcoord / 32),
    }

    let numberInGroup = {
      x: Math.floor(Xcoord * Zcoord),
      y: Math.floor(Ycoord * Zcoord)
    }

    /* console.log(coords);
     console.log(group);
     console.log(numberInGroup);*/

    let zzz = ""

    for (var i = 8; i > coords.z; i--) {
      zzz += "z";
    }

    if (zzz.length != "") zzz += "_";

    let url = `https://dynmap.minecartrapidtransit.net/tiles/new/flat/${group.x}_${group.y}/${zzz}${numberInGroup.x}_${numberInGroup.y}.png`
    //console.log(url)
    return url;

    // return L.TileLayer.prototype.getTileUrl.call(this, coords);
  }
});

// static factory as recommended by http://leafletjs.com/reference-1.0.3.html#class
L.tileLayer.customTileLayer = function(templateUrl, options) {
  return new L.TileLayer.customTileLayer(templateUrl, options);
}

function f(t, n) {
  return t.replace(d, function(t, i) {
    var e = n[i];
    if (void 0 === e)
      throw new Error("No value provided for variable " + t);
    return "function" == typeof e && (e = e(n)),
      e
  })
}

L.tileLayer.customTileLayer("unused url; check custom function", {
  maxZoom: 8,
  zoomControl: false, //there's also css to do this bc this line doesn't work
  id: 'map',
  tileSize: 128,
  zoomOffset: 0,
  noWrap: true,
  bounds: [
    [-900, -900],
    [900, 900]
  ],
  attribution: "Minecart Rapid Transit"
}).addTo(map);

/*L.control.zoom({
  position: 'topright'
}).addTo(map);*/

function mapcoord([x, y]) {
  NewX = (y / -64) - 0.5;
  NewY = x / 64;
  return [NewX, NewY];
}
function worldcoord([x, y]) {
    NewX = y * 64;
    NewY = (x + 0.5) * -64
    return [NewX, NewY];
}

map.pm.addControls({  
  position: 'bottomleft',
  drawCircleMarker: false,
  drawCircle: false
}); 

map.on("pm:vertexadded pm:centerplaced", e => {
    e.lat = Math.round(e.lat);
    e.lng = Math.round(e.lng);
});

var sidebar = L.control.sidebar({
  autopan: false, 
  closeButton: true, 
  container: 'sidebar',
  position: 'left',
}).addTo(map);

sidebar.addPanel({
    id: 'welcome',
    tab: '<i class="fas fa-door-open"></i>',
    pane: document.getElementById("welcome").innerHTML,
    title: 'Welcome to Stencil ' + VERSION
});

sidebar.addPanel({
    id: 'componentInfo',
    tab: '<i class="fas fa-draw-polygon"></i>',
    pane: document.getElementById("componentInfo").innerHTML,
    title: 'Component Info'
});
sidebar.open('welcome');