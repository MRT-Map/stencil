pub mod component_editor;
pub mod dock;
pub mod menu;
pub mod status;
pub mod tilemap;
pub mod toolbar;

use std::sync::Arc;

use bevy::prelude::*;

use crate::ui::{
    panel::{
        component_editor::PrevNamespaceUsed,
        dock::{FileDialogs, PanelDockState},
        status::Status,
    },
    UiSchedule, UiSet,
};

pub struct PanelPlugin;

impl Plugin for PanelPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PrevNamespaceUsed>()
            .init_resource::<PanelDockState>()
            .init_resource::<Status>()
            .init_resource::<FileDialogs>()
            .add_systems(
                UiSchedule,
                menu::ui_sy.in_set(UiSet::Panels).before(dock::panel_sy),
            )
            .add_systems(UiSchedule, dock::panel_sy.exclusive().in_set(UiSet::Panels));
    }
}
