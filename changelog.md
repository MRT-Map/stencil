***Coming soon***
* Directly download namespace files from GitHub repositories
* Show colours of component types in component type editor
* Selecting multiple components
* Copying and pasting
* Font configuration

# v2.2.2
* Update skin format for `tile-renderer` v5
* Sort components by ID before saving

# v2.2.1
* Upgrade `bevy` to v0.14
* Fix crash that happens sometimes (hopefully)

# v2.2.0
* Revamped UI
  * New tabs and windows / docking format from `egui_dock`
  * New notifications from `egui_notify`
  * Menu bar now has
    * status messages
    * zoom level
    * number of pending tiles
    * time per frame in milliseconds
* Added more settings
  * Revamp tilemap settings: multiple registered basemaps, one is selected for rendering
  * Keymap settings
  * Miscellaneous settings
* Added welcome message when opening the application for the first time
* Added project panel
  * Open, save and reload multiple namespace files in the same folder
  * Hide and show namespaces
  * Create and delete namespaces
  * Default project is a scratchpad
* Namespace in component editor is now a dropdown and only accepts visible namespaces
  * `_misc` is the default and cannot be hidden or deleted
* Added autosaving (default: every 60s)
* Added component list panel
* Added history panel
* Added notification log window
* Added green and red circles on start and end of line components respectively to show line direction
  * First and last coordinates in the component editor are also green and red respectively for line components
* If the skin type of a loaded component is invalid, now tries to guess whether it is a point, line or area depending on the nature of the nodes, instead of crashing
* Size of handles in node editing are now independent of the width of the component
* Added safe deleting, moving and writing of all files: now backs old versions up to a cache directory
* Added caching of skin JSON file
* Added Apple silicon support in `dmg` file

# v2.1.3
* Upgrade `bevy` to v0.13
* Fix mouse icon not changing
* Fix application not existing in `dmg` file

# v2.1.2
* Upgrade `bevy` to v0.12 (painfully)
* No longer able to move the component in node-editing mode, go to select mode to do that
* No longer able to select a line from its center like an area
* Update default tile URL for Minecart Rapid Transit

# v2.1.1
* Added back limit to max number of HTTP requests
* Added an option to clear tile cache on startup
* All fields in `tile_settings.toml` are now optional too

# v2.1.0
* Added window settings, GPU backend, Linux display server protocol configuration
* Stencil2 now starts in windowed mode instead of fullscreen
* Settings files are saved as toml instead of msgpack
* Moved manual online
* Now recognises `RUST_LOG` environment variable
* Now shows more licenses from dependencies
* Tile cache is moved from data folder to cache folder

# v2.0.5
* URLs in tile cache are now escaped to fit Windows file-naming restrictions (especially `:`)

# v2.0.4
* Fix textboxes losing focus (again)
* ~~Changed backend of `surf` (http request lib) from `curl` to `async-h1` (it may or may not help render tiles in Windows?)~~
  * note from v2.0.5: this change was apparently not done; the actual change screwed up the tilemap
* Spantrace in panic files

# v2.0.3
* Update bevy
* Fix tilemap also moving when the window is moving
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
* Tile settings, initial zoom, and tilemap are now editable
* Better panic handling
* Log output to file

# v2.0.0-beta.3
* For lines and areas, clicking on the last node placed will remove the node
* Move popups (except one temporary one) to a new internal popup system
* New internal file explorer for importing and exporting PLA 2 files
  * mostly because file importing broke on macOS Ventura

# v2.0.0-beta.2
* Stencil 2 is now distributed as .app in .dmg on macOS
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
