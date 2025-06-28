pub mod dock;
pub mod menu;
pub mod status;
pub mod toolbar;

use bevy::prelude::*;

use crate::ui::{
    EguiContextPass, UiSet,
    file_dialogs::FileDialogs,
    panel::{dock::DockLayout, status::Status},
};

pub struct PanelPlugin;

impl Plugin for PanelPlugin {
    fn build(&self, app: &mut App) {
        app.world_mut().init_resource::<FileDialogs>();
        app.insert_resource(DockLayout::load())
            .init_resource::<Status>()
            .add_systems(
                EguiContextPass,
                menu::ui_sy.in_set(UiSet::Panels).before(dock::panel_sy),
            )
            .add_observer(menu::on_all_settings)
            .add_systems(EguiContextPass, dock::panel_sy.in_set(UiSet::Panels))
            .add_observer(dock::on_reset_panel);
    }
}
