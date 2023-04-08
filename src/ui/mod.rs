use bevy::prelude::*;

use crate::{
    misc::{CustomSet, EditorState},
    ui::component_panel::PrevNamespaceUsed,
};

pub mod component_panel;
pub mod file_explorer;
pub mod menu;
pub mod popup;
pub mod toolbar;

#[derive(Default, Resource, PartialEq, Eq, Copy, Clone)]
pub struct HoveringOverGui(pub bool);

pub struct UiPlugin;

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
struct UiSet;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<HoveringOverGui>()
            .init_resource::<PrevNamespaceUsed>()
            .configure_set(CustomSet::Ui.before(CoreSet::Update))
            .configure_set(
                UiSet
                    .run_if(not(in_state(EditorState::Loading)))
                    .in_base_set(CustomSet::Ui),
            )
            .add_systems(
                (menu::ui_sy, component_panel::ui_sy, toolbar::ui_sy)
                    .chain()
                    .in_set(UiSet),
            )
            .add_system(
                (|mut hovering_over_gui: ResMut<HoveringOverGui>| hovering_over_gui.0 = false)
                    .in_base_set(CoreSet::Last),
            )
            .add_plugin(popup::PopupPlugin);
    }
}
