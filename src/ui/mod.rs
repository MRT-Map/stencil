use bevy::prelude::*;
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

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
#[system_set(base)]
pub struct UiBaseSet;

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
            .configure_set(UiBaseSet.before(CoreSet::Update))
            .configure_set(UiSet::Init.run_if_not_loading().in_base_set(UiBaseSet))
            .configure_set(
                UiSet::Popups
                    .run_if_not_loading()
                    .in_base_set(UiBaseSet)
                    .after(UiSet::Init),
            )
            .configure_set(
                UiSet::Panels
                    .run_if_not_loading()
                    .in_base_set(UiBaseSet)
                    .after(UiSet::Popups),
            )
            .configure_set(
                UiSet::Tiles
                    .run_if_not_loading()
                    .in_base_set(UiBaseSet)
                    .after(UiSet::Panels),
            )
            .configure_set(
                UiSet::Mouse
                    .run_if_not_loading()
                    .in_base_set(UiBaseSet)
                    .after(UiSet::Tiles),
            )
            .configure_set(UiSet::Reset.in_base_set(UiBaseSet).after(UiSet::Mouse))
            .add_plugin(popup::PopupPlugin)
            .add_plugin(panel::PanelPlugin)
            .add_plugin(cursor::CursorPlugin)
            .add_system(reset_hovering_over_gui_sy.in_set(UiSet::Reset))
            .add_system(init_focus.in_set(UiSet::Init))
            .add_system(save_focus.in_set(UiSet::Reset));
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
