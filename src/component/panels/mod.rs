use bevy::prelude::*;

pub mod component_editor;
pub mod component_list;

pub struct ComponentPanelsPlugin;
impl Plugin for ComponentPanelsPlugin {
    fn build(&self, app: &mut App) {
        app.observe(component_editor::on_component_editor)
            .observe(component_list::on_component_list);
    }
}
