use bevy::{ecs::schedule::ScheduleLabel, prelude::*, window::PrimaryWindow};
use bevy_egui::{EguiContextSettings, EguiContexts, EguiPrimaryContextPass, egui};

use crate::state::IntoSystemConfigExt;

pub mod cursor;
pub mod file_dialogs;
pub mod map;
pub mod notif;
pub mod panel;
pub mod popup;

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
            .init_schedule(EguiPrimaryContextPass)
            .configure_sets(
                EguiPrimaryContextPass,
                UiSet::Panels.run_if_not_loading().after(UiSet::Popups),
            )
            .configure_sets(
                EguiPrimaryContextPass,
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
                    let Ok(ctx) = ctx.ctx_mut() else {
                        return Ok(());
                    };
                    egui_extras::install_image_loaders(ctx);

                    settings.single_mut()?.capture_pointer_input = false;
                    Ok(())
                },
            );
    }
}
