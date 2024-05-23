use bevy::prelude::*;

pub mod component_editor;
pub mod component_list;

pub struct ComponentPanelsPlugin;
impl Plugin for ComponentPanelsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                component_editor::component_editor_asy,
                component_list::component_list_asy,
            ),
        );
    }
}
