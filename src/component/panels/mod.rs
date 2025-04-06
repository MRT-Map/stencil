use bevy::prelude::*;

pub mod component_editor;
pub mod component_list;

pub struct ComponentPanelsPlugin;
impl Plugin for ComponentPanelsPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(component_editor::on_component_editor)
            .add_observer(component_list::on_component_list);
    }
}
