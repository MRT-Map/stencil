use bevy::{math::Vec3Swizzles, prelude::*};
use bevy_mouse_tracking::MousePosWorld;

use crate::{
    component_actions::undo_redo::{History, UndoRedoAct},
    misc::Action,
    pla2::{
        bundle::SelectedComponent,
        component::{EditorCoords, PlaComponent},
    },
    state::{EditorState, IntoSystemConfigExt},
    ui::cursor::mouse_events::{HoveredComponent, MouseEvent},
};

#[tracing::instrument(skip_all)]
pub fn move_component_sy(
    mut selected: Query<
        (
            Entity,
            &mut Transform,
            &mut PlaComponent<EditorCoords>,
            Option<&HoveredComponent>,
        ),
        With<SelectedComponent>,
    >,
    mut orig: Local<Option<(MousePosWorld, Vec3)>>,
    mut mouse: EventReader<MouseEvent>,
    mut actions: EventWriter<Action>,
    mouse_pos_world: Res<MousePosWorld>,
    state: Res<State<EditorState>>,
) {
    if state.0.component_type().is_some() {
        mouse.clear();
        return;
    }
    let Ok((entity, mut transform, mut pla, hovered)) = selected.get_single_mut() else {
            mouse.clear();
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
                let old_pla = pla.to_owned();
                for node in &mut pla.nodes {
                    node.0 += (mouse_pos_world.xy() - orig_mouse_pos_world.xy())
                        .round()
                        .as_ivec2();
                }
                actions.send(Box::new(UndoRedoAct::one_history(History {
                    component_id: entity,
                    before: Some(old_pla),
                    after: Some(pla.to_owned()),
                })));
            }
            info!("Ended move");
            *orig = None;
        }
    }
}

pub struct MoveComponentPlugin;
impl Plugin for MoveComponentPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(move_component_sy.run_if_not_loading());
    }
}
