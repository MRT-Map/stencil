# Stencil 2.0.6 Manual
Correct as of 29/11/22

## Editing Components
### Creating Components
#### Points
- Click `Point` on the toolbar or press `4` on the keyboard.
- Left-click on the map where you want your point to be.

#### Lines and Areas
- Click `Line` on the toolbar or press `5` on the keyboard for lines; or
- Click `Area` on the toolbar or press `6` on the keyboard for areas.
- Left-click on the map where you want the line/area to start. This is your first node.
- Continue left-clicking to place more nodes and continue the line/area.
  - Hold down the left `alt` key for angle-snapping.
- If you had made a mistake with the previous node, left-click on the previous node to remove it.
- Right-click to complete the line/area at the last node placed.
  - For areas, left-clicking on the first node completes it too.

### Selecting Components
- Click `Select` on the toolbar or `1` on the keyboard, then left-click the component you want to edit.

#### Editing Component Information
- A panel of text boxes and sliders should appear on the left of the screen.
  - `ns.` is the namespace that the component belongs to. It should be a three-letter code that your project organisers have assigned, or that you have registered.
    - If you are in the MRT Mapping Services, see #namespace-list for an up-to-date list of namespaces.
    - This field should be automatically prefilled the next time you edit a component's information.
  - `id` should be prefilled automatically with a randomly-generated ID.
  - `Displayed as` is the text that would appear on the final map.
  - `Description` is a brief description of the component.
  - `Component type` is what the component would show up as on the final map.
  - `Tags` is a comma-separated list of tags on the component.
    - e.g. `oneWay` makes a line one-directional.
  - `Layer` controls whether the component appears below or above other components.
  - In lines, `Reverse direction` reverses the order of the nodes. This is useful for one-directional components.
  - A list of coordinates is shown as well below the information fields.

#### Moving Components
- Right-click and drag the component to move it.

#### Editing Individual Nodes
- Click `Edit Nodes` on the toolbar or `2` on the keyboard.
- For the large circles:
  - right-click and drag to move the node
  - right-click without dragging to delete the node
- For the small circles:
  - right-click to create a node

### Deleting Components
- Click `Delete` on the toolbar or `3` on the keyboard.
- Left-click a component to delete it.

### Undo & Redo
- Click `Edit > Undo` on the menu or `u` on the keyboard to undo.
- Click `Edit > Redo` on the menu or `y` on the keyboard to redo.

## Importing and Exporting
### Loading Namespaces
- Click `File > Load namespace` on the menu or `l` on the keyboard.
- Check the PLA files that you want loaded and click `Select`.
- The namespaces should now be loaded if the namespace is not already loaded and there is no problems with the file.

### Saving Namespaces
- Click `File > Save namespaces` on the menu or `s` on the keyboard.
- Navigate to the directory that you want to save the files in and click `Select`.
- The namespaces should now be saved in the folder if there are no components with no namespaces.

## Editing the Tilemap
- Click `Settings > Tilemap` on the menu or `t` on the keyboard.
- More information is documented under each setting.

## Quitting
- Click `Stencil v(version) > Quit` on the menu or `esc` on the keyboard to quit.
