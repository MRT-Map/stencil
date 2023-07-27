use bevy::{app::MainScheduleOrder, ecs::schedule::ScheduleLabel, prelude::*};
use bevy_egui::{
    egui::{Id, Pos2, Response},
    EguiContexts,
};
use bevy_mouse_tracking::MousePos;

use crate::state::{EditorState, IntoSystemSetConfigExt};

pub mod cursor;
pub mod file_explorer;
pub mod panel;
pub mod popup;
pub mod tilemap;

#[derive(Default, Resource, PartialEq, Eq, Copy, Clone)]
pub struct HoveringOverGui(pub bool);

impl HoveringOverGui {
    pub fn egui(&mut self, response: &Response, mouse_pos: MousePos) {
        if response.hovered() || response.rect.contains(Pos2::from(mouse_pos.to_array())) {
            self.0 = true;
        }
    }
}

#[derive(Default, Resource, PartialEq, Eq, Copy, Clone)]
pub struct Focus(pub Option<Id>);

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
        app.init_resource::<HoveringOverGui>()
            .init_resource::<Focus>()
            .init_schedule(UiSchedule)
            .configure_set(UiSchedule, UiSet::Init.run_if_not_loading())
            .configure_set(
                UiSchedule,
                UiSet::Popups.run_if_not_loading().after(UiSet::Init),
            )
            .configure_set(
                UiSchedule,
                UiSet::Panels.run_if_not_loading().after(UiSet::Popups),
            )
            .configure_set(
                UiSchedule,
                UiSet::Tiles.run_if_not_loading().after(UiSet::Panels),
            )
            .configure_set(
                UiSchedule,
                UiSet::Mouse.run_if_not_loading().after(UiSet::Tiles),
            )
            .configure_set(UiSchedule, UiSet::Reset.after(UiSet::Mouse))
            .add_plugins(popup::PopupPlugin)
            .add_plugins(panel::PanelPlugin)
            .add_plugins(cursor::CursorPlugin)
            .add_systems(Update, reset_hovering_over_gui_sy.in_set(UiSet::Reset))
            .add_systems(Update, init_focus.in_set(UiSet::Init))
            .add_systems(Update, save_focus.in_set(UiSet::Reset));
        let mut order = app.world.resource_mut::<MainScheduleOrder>();
        order.insert_after(PreUpdate, UiSchedule);
    }
}

#[allow(clippy::needless_pass_by_value)]
pub fn init_focus(mut ctx: EguiContexts, focus: Res<Focus>) {
    let ctx = ctx.ctx_mut();
    if let Some(f) = focus.0 {
        ctx.memory_mut(|a| a.request_focus(f));
    }
}

#[allow(clippy::needless_pass_by_value)]
pub fn save_focus(mut ctx: EguiContexts, mut focus: ResMut<Focus>) {
    let ctx = ctx.ctx_mut();
    focus.0 = ctx.memory(bevy_egui::egui::Memory::focus);
}

#[allow(clippy::needless_pass_by_value)]
pub fn reset_hovering_over_gui_sy(
    mut hovering_over_gui: ResMut<HoveringOverGui>,
    buttons: Res<Input<MouseButton>>,
) {
    if !buttons.any_pressed([MouseButton::Left, MouseButton::Middle, MouseButton::Right]) {
        hovering_over_gui.0 = false;
    }
}
