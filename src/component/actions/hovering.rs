use bevy::prelude::*;

use crate::{component::actions::rendering::RenderEv, ui::tilemap::window::PointerWithinTilemap};

#[tracing::instrument(skip_all)]
pub fn on_hover_over(
    trigger: Trigger<Pointer<Over>>,
    mut commands: Commands,
    pointer_within_tilemap: Option<Res<PointerWithinTilemap>>,
) {
    let e = trigger.target();
    if pointer_within_tilemap.is_none() {
        return;
    }
    debug!(?e, "Hovering over component");
    commands
        .entity(e)
        .insert(HoveredComponent)
        .trigger(RenderEv::default());
}

#[tracing::instrument(skip_all)]
pub fn on_hover_out(
    trigger: Trigger<Pointer<Out>>,
    mut commands: Commands,
    pointer_within_tilemap: Option<Res<PointerWithinTilemap>>,
) {
    let e = trigger.target();
    if pointer_within_tilemap.is_none() {
        return;
    }
    debug!(?e, "Hovering out of component");
    commands
        .entity(e)
        .remove::<HoveredComponent>()
        .trigger(RenderEv::default());
}

pub struct HoverComponentPlugin;
impl Plugin for HoverComponentPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_hover_over).add_observer(on_hover_out);
    }
}

#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct HoveredComponent;
