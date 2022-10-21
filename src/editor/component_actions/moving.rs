use bevy::{math::Vec3Swizzles, prelude::*};
use bevy_mouse_tracking_plugin::MousePosWorld;
use iyes_loopless::{condition::ConditionSet, prelude::CurrentState};

use crate::{
    editor::{
        bundles::component::SelectedComponent,
        cursor::mouse_events::{HoveredComponent, MouseEvent},
    },
    types::{
        pla::{EditorCoords, PlaComponent},
        EditorState,
    },
};

#[tracing::instrument(skip_all)]
pub fn move_component_sy(
    mut selected: Query<
        (
            &mut Transform,
            &mut PlaComponent<EditorCoords>,
            Option<&HoveredComponent>,
        ),
        With<SelectedComponent>,
    >,
    mut orig: Local<Option<(MousePosWorld, Vec3)>>,
    mut events: EventReader<MouseEvent>,
    mouse_pos_world: Res<MousePosWorld>,
    state: Res<CurrentState<EditorState>>,
) {
    if matches!(
        &state.0,
        EditorState::CreatingComponent(_) | EditorState::DeletingComponent
    ) {
        return;
    }
    let (mut transform, mut pla, hovered): (
        Mut<Transform>,
        Mut<PlaComponent<EditorCoords>>,
        Option<&HoveredComponent>,
    ) = if let Ok(query_data) = selected.get_single_mut() {
        query_data
    } else {
        return;
    };
    if let Some((orig_mouse_pos_world, orig_select_translation)) = *orig {
        transform.translation.x =
            (mouse_pos_world.x - orig_mouse_pos_world.x + orig_select_translation.x).round();
        transform.translation.y =
            (mouse_pos_world.y - orig_mouse_pos_world.y + orig_select_translation.y).round();
    }
    for event in events.iter() {
        if let MouseEvent::RightPress(mouse_pos_world) = event {
            if hovered.is_some() {
                *orig = Some((*mouse_pos_world, transform.translation));
            }
        } else if let MouseEvent::RightRelease(_) = event {
            if let Some((_, orig_select_translation)) = *orig {
                for node in pla.nodes.iter_mut() {
                    node.0 += (transform.translation.xy() - orig_select_translation.xy())
                        .round()
                        .as_ivec2()
                }
            }
            *orig = None;
        }
    }
}

pub struct MoveComponentPlugin;
impl Plugin for MoveComponentPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            ConditionSet::new()
                .run_not_in_state(EditorState::Loading)
                .with_system(move_component_sy)
                .into(),
        );
    }
}
