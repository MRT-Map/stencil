use bevy::{prelude::*, window::PrimaryWindow};
use bevy_egui::{egui, EguiContextPass, EguiContextSettings, EguiContexts};

use crate::state::IntoSystemConfigExt;

pub mod cursor;
pub mod file_dialogs;
pub mod notif;
pub mod panel;
pub mod popup;
pub mod tilemap;

#[derive(Default, Resource, PartialEq, Eq, Copy, Clone)]
pub struct Focus(pub Option<egui::Id>);

pub struct UiPlugin;

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
pub enum UiSet {
    Popups,
    Panels,
    Tiles,
}

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Focus>()
            .init_schedule(EguiContextPass)
            .configure_sets(
                EguiContextPass,
                UiSet::Panels.run_if_not_loading().after(UiSet::Popups),
            )
            .configure_sets(
                EguiContextPass,
                UiSet::Tiles.run_if_not_loading().after(UiSet::Panels),
            )
            .add_plugins(popup::PopupPlugin)
            .add_plugins(panel::PanelPlugin)
            .add_plugins(cursor::CursorPlugin)
            .add_systems(
                Startup,
                |mut ctx: EguiContexts,
                 mut settings: Query<&mut EguiContextSettings, With<PrimaryWindow>>|
                 -> Result {
                    let Some(ctx) = ctx.try_ctx_mut() else {
                        return Ok(());
                    };
                    egui_extras::install_image_loaders(ctx);

                    settings.single_mut()?.capture_pointer_input = false;
                    Ok(())
                },
            );
    }
}
