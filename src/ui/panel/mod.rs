pub mod component_editor;
pub mod dock;
pub mod menu;
mod tilemap;
pub mod toolbar;

use bevy::prelude::*;

use crate::ui::{
    panel::{component_editor::PrevNamespaceUsed, dock::PanelDockState},
    UiSchedule, UiSet,
};

pub struct PanelPlugin;

impl Plugin for PanelPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PrevNamespaceUsed>()
            .init_resource::<PanelDockState>()
            .add_systems(
                UiSchedule,
                menu::ui_sy.in_set(UiSet::Panels).before(dock::panel_sy),
            )
            .add_systems(UiSchedule, dock::panel_sy.in_set(UiSet::Panels));
    }
}
