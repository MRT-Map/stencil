# v2.0.5
* URLs in tile cache are now escaped to fit Windows file-naming restrictions (especially `:`)

# v2.0.4
* Fix textboxes losing focus (again)
* ~~Changed backend of `surf` (http request lib) from `curl` to `async-h1` (it may or may not help render tiles in Windows?)~~
  * note from v2.0.5: this change was apparently not done; the actual change screwed up the tilemap
* Spantrace in panic files

# v2.0.3
* Update bevy
* Fix tilemap also moving when window is moving
* Fix tiles not loading fast enough... hopefully
* Option to show checkerboard instead of downloading tiles through environment variable

# v2.0.2
* Fix skin not loading properly due to change in format of `font` field

# v2.0.1
* Fix file where tile settings are stored being created as a folder instead of a file

# v2.0.0
* Added hotkeys
* Fix errors loading tilemap
* Added manual

# v2.0.0-beta.4
* Added license page
* Fixed text boxes not being editable and immediately losing focus
* Undo / redo functionality
  * native-dialog is no longer a dependency
* Asset folder is now bundled together with the executable and unloaded before bevy starts
* Tile settings, initial zoom and tilemap is now editable
* Better panic handling
* Log output to file

# v2.0.0-beta.3
* For lines and areas, clicking on the last node placed will remove the node
* Move popups (except one temporary one) to a new internal popup system
* New internal file explorer for importing and exporting PLA 2 files
  * mostly because file importing broke on macOS Ventura

# v2.0.0-beta.2
* Stencil 2 is now distributed as .app in .dmg in macOS
* Fix crosshair not showing up on Mac builds (maybe other builds too)
* Shows a confirmation popup when deleting a component of more than 5 nodes

# v2.0.0-beta.1
* Added popup handling system
* Handles in node editing of components with 50+ nodes will only show if they are nearest
* Added changelog window
* Added angle-snapping for lines and areas
* Fix quitting the app crashing on Mac builds (maybe other builds too)
* Fix a majority of z-fighting in components
* Add mouse X and Z-coordinate display
* Increase transparency of areas

# v2.0.0-beta.0
* Added tile map
* Added component modes for creation, deletion, node editing
* Added top menu, toolbar, component panel
* Added component actions for selecting and moving
* Added cursor changing and crosshair
* Added importing and exporting
* Added quitting
* Added info screen
