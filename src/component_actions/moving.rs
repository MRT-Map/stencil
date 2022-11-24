use bevy::{math::Vec3Swizzles, prelude::*};
use bevy_mouse_tracking_plugin::MousePosWorld;
use iyes_loopless::{condition::ConditionSet, prelude::CurrentState};

use crate::{
    cursor::mouse_events::{HoveredComponent, MouseEvent},
    misc::EditorState,
    pla2::{
        bundle::SelectedComponent,
        component::{EditorCoords, PlaComponent},
    },
};

#[allow(clippy::type_complexity)]
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
    mut mouse: EventReader<MouseEvent>,
    mouse_pos_world: Res<MousePosWorld>,
    state: Res<CurrentState<EditorState>>,
) {
    if matches!(
        &state.0,
        EditorState::CreatingComponent(_)
            | EditorState::DeletingComponent
            | EditorState::EditingNodes
    ) {
        return;
    }
    let (mut transform, mut pla, hovered) = if let Ok(query_data) = selected.get_single_mut() {
        query_data
    } else {
        return;
    };
    if let Some((orig_mouse_pos_world, orig_select_translation)) = &*orig {
        transform.translation.x =
            (mouse_pos_world.x - orig_mouse_pos_world.x + orig_select_translation.x).round();
        transform.translation.y =
            (mouse_pos_world.y - orig_mouse_pos_world.y + orig_select_translation.y).round();
    }
    for event in mouse.iter() {
        if let MouseEvent::RightPress(mouse_pos_world) = event {
            if hovered.is_some() {
                *orig = Some((*mouse_pos_world, transform.translation));
            }
            info!("Started move");
        } else if let MouseEvent::RightRelease(_) = event {
            if let Some((orig_mouse_pos_world, _)) = *orig {
                for node in pla.nodes.iter_mut() {
                    node.0 += (mouse_pos_world.xy() - orig_mouse_pos_world.xy())
                        .round()
                        .as_ivec2()
                }
            }
            info!("Ended move");
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
