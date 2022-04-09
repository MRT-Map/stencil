/// <reference path="references.ts" />

map.on("pm:keyevent", ({event, eventType}) => {
  if (eventType == "keydown") return;
  if (document.activeElement.hasAttribute("contenteditable") || document.activeElement.tagName == "INPUT") return;
  switch (event.key) {
    case "Escape":
      map.pm.disableDraw();
      map.pm.disableGlobalCutMode();
      map.pm.disableGlobalDragMode();
      map.pm.disableGlobalEditMode();
      map.pm.disableGlobalRemovalMode();
      map.pm.disableGlobalRotateMode();
      map.fire("click");
      break;
    case "1":
      map.pm.enableGlobalEditMode();
      break;
    case "2":
      map.pm.enableGlobalDragMode();
      break;
    case "3":
      map.pm.enableGlobalRemovalMode();
      break;
    case "4":
      map.pm.enableGlobalRotateMode();
      break;
    case "5":
      map.pm.enableDraw("Marker");
      break;
    case "6":
      map.pm.enableDraw("Line");
      break;
    case "7":
      map.pm.enableDraw("Rectangle");
      break;
    case "8":
      map.pm.enableDraw("Polygon");
      break;
  }
})