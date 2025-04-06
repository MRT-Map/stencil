use bevy::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;

#[derive(Clone, Copy, Debug, Default, Resource)]
pub struct ShowInspector;

pub struct InspectorPlugin;

impl Plugin for InspectorPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(WorldInspectorPlugin::new().run_if(resource_exists::<ShowInspector>));
    }
}
