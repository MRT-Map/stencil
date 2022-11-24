use bevy::{app::PluginGroupBuilder, prelude::*};

pub mod moving;
pub mod selecting;
pub mod undo_redo;

pub struct ComponentActionPlugins;

impl PluginGroup for ComponentActionPlugins {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(moving::MoveComponentPlugin)
            .add(selecting::SelectComponentPlugin)
            .add(undo_redo::UndoRedoPlugin)
    }
}
