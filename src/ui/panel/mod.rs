pub mod dock;
pub mod menu;
pub mod status;
pub mod toolbar;

use bevy::prelude::*;

use crate::ui::{
    panel::{
        dock::{FileDialogs, PanelDockState},
        status::Status,
    },
    UiSchedule, UiSet,
};

pub struct PanelPlugin;

impl Plugin for PanelPlugin {
    fn build(&self, app: &mut App) {
        app.world.insert_non_send_resource(FileDialogs::default());
        app.init_resource::<PanelDockState>()
            .init_resource::<Status>()
            .add_systems(
                UiSchedule,
                (menu::ui_sy, menu::all_settings_asy)
                    .in_set(UiSet::Panels)
                    .before(dock::panel_sy),
            )
            .add_systems(
                UiSchedule,
                (dock::panel_sy, dock::reset_panel_asy).in_set(UiSet::Panels),
            );
    }
}
