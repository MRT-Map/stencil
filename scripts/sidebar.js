document.querySelectorAll("[contenteditable]").forEach((element) => {
  element.parentElement.onclick = (e) => {
    e.target.focus();
  }
})

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
  pane: document.getElementById("componentInfo").innerHTML,
  title: 'Component Info'
});

/*sidebar.addPanel({
  id: 'discordLogin',
  tab: '<i class="fab fa-discord"></i>',
  pane: document.getElementById("discordLogin").innerHTML,
  title: 'Discord Login',
  position: 'bottom'
});*/

sidebar.open('pane_welcome');
sidebar.disablePanel('pane_componentInfo');