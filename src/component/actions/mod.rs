use bevy::{app::PluginGroupBuilder, prelude::*};

pub mod hovering;
pub mod moving;
pub mod rendering;
pub mod selecting;

pub struct ComponentActionPlugins;

impl PluginGroup for ComponentActionPlugins {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(moving::MoveComponentPlugin)
            .add(selecting::SelectComponentPlugin)
            .add(hovering::HoverComponentPlugin)
            .add(rendering::RenderComponentPlugin)
    }
}
