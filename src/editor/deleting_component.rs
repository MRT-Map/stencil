use bevy::prelude::*;
use bevy_mod_picking::{HoverEvent, PickingEvent};
use iyes_loopless::prelude::*;

use crate::editor::selecting_component::HoveringOverComponent;
use crate::editor::ui::HoveringOverGui;
use crate::types::{DetectMouseMoveOnClick, DetectMouseMoveOnClickExt, EditorState};

#[tracing::instrument(skip_all)]
pub fn delete_component(
    mut events: EventReader<PickingEvent>,
    mut commands: Commands,
    buttons: Res<Input<MouseButton>>,
    hovering_over_gui: Res<HoveringOverGui>,
    mut hovering_over_comp: ResMut<HoveringOverComponent>,
    mut selected_entity: Local<Option<Entity>>,
    mut mm_detector: DetectMouseMoveOnClick,
) {
    mm_detector.handle_press(&buttons);
    for event in events.iter() {
        if let PickingEvent::Clicked(e) = event {
            if !hovering_over_gui.0 {
                info!(?e, "Select detected");
                *selected_entity = Some(*e);
                *mm_detector.0 = Some(*mm_detector.1);
            }
        } else if let PickingEvent::Hover(e) = event {
            trace!("Hover detected");
            hovering_over_comp.0 = match e {
                HoverEvent::JustLeft(_) => false,
                HoverEvent::JustEntered(_) => true,
            };
        }
    }
    if buttons.just_released(MouseButton::Left)
        && !mm_detector.handle_release()
        && !hovering_over_gui.0
    {
        if let Some(selected_entity) = *selected_entity {
            info!(?selected_entity, "Deleting entity");
            commands.entity(selected_entity).despawn();
        }
        *selected_entity = None;
    }
}

pub struct DeleteComponentPlugin;

impl Plugin for DeleteComponentPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            ConditionSet::new()
                .run_in_state(EditorState::DeletingComponent)
                .with_system(delete_component)
                .into(),
        );
    }
}
