use bevy::prelude::*;

use crate::{
    component::{
        actions::selecting::{highlight_selected_sy, on_select, on_select_left_click},
        bundle::SelectedComponent,
        pla2::{EditorCoords, PlaComponent},
        skin::Skin,
    },
    misc_config::settings::MiscSettings,
    state::EditorState,
    tile::zoom::Zoom,
    ui::UiSet,
};

#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct HoveredComponent;

#[tracing::instrument(skip_all)]
pub fn on_hover_over(
    trigger: Trigger<Pointer<Over>>,
    pickables: Query<(), With<RayCastPickable>>,
    mut commands: Commands,
) {
    let entity = trigger.entity();
    if !pickables.contains(entity) {
        return;
    }
    debug!(?entity, "Entered hover over component");
    commands.entity(entity).insert(HoveredComponent);
}

#[tracing::instrument(skip_all)]
pub fn on_hover_out(
    trigger: Trigger<Pointer<Out>>,
    pickables: Query<(), With<RayCastPickable>>,
    mut commands: Commands,
) {
    let entity = trigger.entity();
    if !pickables.contains(entity) {
        return;
    }
    debug!(?entity, "Entered hover out of component");
    commands.entity(entity).remove::<HoveredComponent>();
}

pub struct HoverComponentPlugin;
impl Plugin for HoverComponentPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_hover_over).add_observer(on_hover_out);
    }
}
