use bevy::prelude::*;

use crate::misc::EditorState;

pub mod cursor;
pub mod file_explorer;
pub mod panel;
pub mod popup;
pub mod tilemap;

#[derive(Default, Resource, PartialEq, Eq, Copy, Clone)]
pub struct HoveringOverGui(pub bool);

pub struct UiPlugin;

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
#[system_set(base)]
pub struct UiBaseSet;

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
pub enum UiSet {
    Popups,
    Panels,
    Tiles,
    Mouse,
    Reset,
}

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<HoveringOverGui>()
            .configure_set(UiBaseSet.before(CoreSet::Update))
            .configure_set(
                UiSet::Popups
                    .run_if(not(in_state(EditorState::Loading)))
                    .in_base_set(UiBaseSet),
            )
            .configure_set(
                UiSet::Panels
                    .run_if(not(in_state(EditorState::Loading)))
                    .in_base_set(UiBaseSet)
                    .after(UiSet::Popups),
            )
            .configure_set(
                UiSet::Tiles
                    .run_if(not(in_state(EditorState::Loading)))
                    .in_base_set(UiBaseSet)
                    .after(UiSet::Panels),
            )
            .configure_set(
                UiSet::Mouse
                    .run_if(not(in_state(EditorState::Loading)))
                    .in_base_set(UiBaseSet)
                    .after(UiSet::Tiles),
            )
            .configure_set(UiSet::Reset.in_base_set(UiBaseSet).after(UiSet::Mouse))
            .add_plugin(popup::PopupPlugin)
            .add_plugin(panel::PanelPlugin)
            .add_plugin(cursor::CursorPlugin)
            .add_system(
                (|mut hovering_over_gui: ResMut<HoveringOverGui>| hovering_over_gui.0 = false)
                    .in_set(UiSet::Reset),
            );
    }
}
