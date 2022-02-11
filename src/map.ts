/// <reference path="references.ts" />
const VERSION = "v1.1b3";

var map = L.map('map', {
  crs: L.CRS.Simple
}).setView([0, 0], 8);

//override the default
//@ts-ignore
L.TileLayer.customTileLayer = L.TileLayer.extend({
  getTileUrl: function(coords) {


    let Zcoord = 2 ** (8 - coords.z);
    let Xcoord = (coords.x * 1);
    let Ycoord = coords.y * -1;

    let group = {
      x: Math.floor(Xcoord * Zcoord / 32),
      y: Math.floor(Ycoord * Zcoord / 32),
    };

    let numberInGroup = {
      x: Math.floor(Xcoord * Zcoord),
      y: Math.floor(Ycoord * Zcoord)
    };

    /* console.log(coords);
     console.log(group);
     console.log(numberInGroup);*/

    let zzz = "";

    for (var i = 8; i > coords.z; i--) {
      zzz += "z";
    }

    if (zzz.length != 0) zzz += "_";
    let url = `https://dynmap.minecartrapidtransit.net/tiles/new/flat/${group.x}_${group.y}/${zzz}${numberInGroup.x}_${numberInGroup.y}.png`;
    //console.log(url)
    return url;

    // return L.TileLayer.prototype.getTileUrl.call(this, coords);
  }
});

// static factory as recommended by http://leafletjs.com/reference-1.0.3.html#class
// @ts-ignore
L.tileLayer.customTileLayer = function(templateUrl, options) {
  // @ts-ignore
  return new L.TileLayer.customTileLayer(templateUrl, options);
};

function f(t, n) {
  return t.replace(n, function(t, i) {
    var e = n[i];
    if (void 0 === e)
      throw new Error("No value provided for variable " + t);
    return "function" == typeof e && (e = e(n)),
      e;
  });
}
// @ts-ignore
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
console.log("%cme when the when the the", 'color: orange; font-size: 25px');
function mapcoord([x, y]) {
  let NewX = (y / -64) - 0.5;
  let NewY = x / 64;
  return [NewX, NewY];
}
function worldcoord([x, y]) {
    let NewX = Math.round(y * 64);
    let NewY = Math.round((x + 0.5) * -64);
    return [NewX, NewY];
}

function roundLatLng({lat, lng}: {lat: number, lng: number}) {
  const c = 64;
  return {lat: Math.round(lat*c)/c, lng: Math.round(lng*c)/c};
}
function floorLatLng({lat, lng}: {lat: number, lng: number}) {
  const c = 64;
  return {lat: Math.floor(lat*c)/c, lng: Math.floor(lng*c)/c};
}


var MyControl = L.Control.extend({
  options: {position: 'bottomright'},
  onAdd: function (map) {
    let container = L.DomUtil.create('div');
    container.innerHTML = "<img src='media/stencilicon_lighttext.png' style='height: 50px;' title='Logo by Cortesi'>";
    return container;
  },
  onRemove: function(map) {}
});

var logo = new MyControl();
map.addControl(logo);