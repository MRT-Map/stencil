/// <reference path="references.ts" />
console.log(Skin);
var sidebar = L.control.sidebar({
    autopan: false,
    closeButton: true,
    container: 'sidebar',
    position: 'left',
}).addTo(map);
sidebar.addPanel({
    id: 'pane_welcome',
    tab: '<i class="fas fa-door-open"></i>',
    pane: document.getElementById("welcome").innerHTML,
    title: 'Welcome to Stencil ' + VERSION
});
sidebar.addPanel({
    id: 'pane_componentInfo',
    tab: '<i class="fas fa-draw-polygon"></i>',
    pane: '<div><h1>Select a component...</h1></div>',
    title: 'Component Info'
});
sidebar.addPanel({
    id: 'pane_typePicker',
    tab: '<i class="fas fa-palette"></i>',
    pane: document.getElementById("typePicker").innerHTML,
    title: 'Pick Type'
});
/*sidebar.addPanel({
  id: 'discordLogin',
  tab: '<i class="fab fa-discord"></i>',
  pane: document.getElementById("discordLogin").innerHTML,
  title: 'Discord Login',
  position: 'bottom'
});*/
sidebar.open('pane_welcome');
