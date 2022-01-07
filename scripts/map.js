/// <reference path="references.ts" />
var VERSION = "v0.0";
console.log(Skin);
var map = L.map('map', {
    crs: L.CRS.Simple
}).setView([0, 0], 8);
//override the default
L.TileLayer.customTileLayer = L.TileLayer.extend({
    getTileUrl: function (coords) {
        var Zcoord = Math.pow(2, (8 - coords.z));
        var Xcoord = (coords.x * 1);
        var Ycoord = coords.y * -1;
        var group = {
            x: Math.floor(Xcoord * Zcoord / 32),
            y: Math.floor(Ycoord * Zcoord / 32)
        };
        var numberInGroup = {
            x: Math.floor(Xcoord * Zcoord),
            y: Math.floor(Ycoord * Zcoord)
        };
        /* console.log(coords);
         console.log(group);
         console.log(numberInGroup);*/
        var zzz = "";
        for (var i = 8; i > coords.z; i--) {
            zzz += "z";
        }
        if (zzz.length != 0)
            zzz += "_";
        var url = "https://dynmap.minecartrapidtransit.net/tiles/new/flat/".concat(group.x, "_").concat(group.y, "/").concat(zzz).concat(numberInGroup.x, "_").concat(numberInGroup.y, ".png");
        //console.log(url)
        return url;
        // return L.TileLayer.prototype.getTileUrl.call(this, coords);
    }
});
// static factory as recommended by http://leafletjs.com/reference-1.0.3.html#class
L.tileLayer.customTileLayer = function (templateUrl, options) {
    return new L.TileLayer.customTileLayer(templateUrl, options);
};
function f(t, n) {
    return t.replace(n, function (t, i) {
        var e = n[i];
        if (void 0 === e)
            throw new Error("No value provided for variable " + t);
        return "function" == typeof e && (e = e(n)),
            e;
    });
}
L.tileLayer.customTileLayer("unused url; check custom function", {
    maxZoom: 8,
    zoomControl: false,
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
function mapcoord(_a) {
    var x = _a[0], y = _a[1];
    var NewX = (y / -64) - 0.5;
    var NewY = x / 64;
    return [NewX, NewY];
}
function worldcoord(_a) {
    var x = _a[0], y = _a[1];
    var NewX = y * 64;
    var NewY = (x + 0.5) * -64;
    return [NewX, NewY];
}
var MyControl = L.Control.extend({
    options: { position: 'bottomright' },
    onAdd: function (map) {
        var container = L.DomUtil.create('div');
        container.innerHTML = "<img src='media/stencilicon_lighttext.png' style='height: 50px;' title='Logo by Cortesi'>";
        return container;
    },
    onRemove: function (map) { }
});
var logo = new MyControl();
map.addControl(logo);
