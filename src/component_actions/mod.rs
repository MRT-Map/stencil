use bevy::{app::PluginGroupBuilder, prelude::*};

pub mod moving;
pub mod selecting;

pub struct ComponentActionPlugins;

impl PluginGroup for ComponentActionPlugins {
    fn build(&mut self, group: &mut PluginGroupBuilder) {
        group
            .add(moving::MoveComponentPlugin)
            .add(selecting::SelectComponentPlugin);
    }
}
