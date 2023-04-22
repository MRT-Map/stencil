pub mod component_panel;
pub mod menu;
pub mod toolbar;
use bevy::prelude::*;

use crate::ui::{panel::component_panel::PrevNamespaceUsed, UiSet};

pub struct PanelPlugin;

impl Plugin for PanelPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PrevNamespaceUsed>().add_systems(
            (menu::ui_sy, component_panel::ui_sy, toolbar::ui_sy)
                .chain()
                .in_set(UiSet::Panels),
        );
    }
}
