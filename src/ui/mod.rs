use bevy::{app::MainScheduleOrder, ecs::schedule::ScheduleLabel, prelude::*};
use bevy_egui::{egui, EguiContexts};

use crate::state::IntoSystemSetConfigExt;

pub mod cursor;
pub mod notif;
pub mod panel;
pub mod popup;
pub mod tilemap;

#[derive(Default, Resource, PartialEq, Eq, Copy, Clone)]
pub struct Focus(pub Option<egui::Id>);

pub struct UiPlugin;

#[derive(Debug, Hash, PartialEq, Eq, Clone, ScheduleLabel)]
pub struct UiSchedule;

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
pub enum UiSet {
    Init,
    Popups,
    Panels,
    Tiles,
    Mouse,
    Reset,
}

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Focus>()
            .init_schedule(UiSchedule)
            .configure_sets(UiSchedule, UiSet::Init.run_if_not_loading())
            .configure_sets(
                UiSchedule,
                UiSet::Popups.run_if_not_loading().after(UiSet::Init),
            )
            .configure_sets(
                UiSchedule,
                UiSet::Panels.run_if_not_loading().after(UiSet::Popups),
            )
            .configure_sets(
                UiSchedule,
                UiSet::Tiles.run_if_not_loading().after(UiSet::Panels),
            )
            .configure_sets(
                UiSchedule,
                UiSet::Mouse.run_if_not_loading().after(UiSet::Tiles),
            )
            .configure_sets(UiSchedule, UiSet::Reset.after(UiSet::Mouse))
            .add_plugins(popup::PopupPlugin)
            .add_plugins(panel::PanelPlugin)
            .add_plugins(cursor::CursorPlugin)
            .add_systems(UiSchedule, init_focus.in_set(UiSet::Init))
            .add_systems(UiSchedule, save_focus.in_set(UiSet::Reset))
            .add_systems(Startup, |mut ctx: EguiContexts| {
                let Some(ctx) = ctx.try_ctx_mut() else {
                    return;
                };
                egui_extras::install_image_loaders(ctx);
            });
        let mut order = app.world_mut().resource_mut::<MainScheduleOrder>();
        order.insert_after(PostUpdate, UiSchedule);
    }
}

#[expect(clippy::needless_pass_by_value)]
pub fn init_focus(mut ctx: EguiContexts, focus: Res<Focus>) {
    let Some(ctx) = ctx.try_ctx_mut() else { return };
    if let Some(f) = focus.0 {
        ctx.memory_mut(|a| a.request_focus(f));
    }
}

pub fn save_focus(mut ctx: EguiContexts, mut focus: ResMut<Focus>) {
    let Some(ctx) = ctx.try_ctx_mut() else { return };
    focus.0 = ctx.memory(egui::Memory::focused);
}
