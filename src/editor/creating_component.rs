use crate::pla::{
    ComponentBundle, ComponentCoords, CreatedComponent, EditorComponent, SelectedComponent,
};
use crate::{get_cursor_world_pos, ComponentType, EditorState, HoveringOverGui, Skin};
use bevy::prelude::*;
use bevy_mouse_tracking_plugin::MainCamera;
use iyes_loopless::prelude::*;

#[allow(clippy::too_many_arguments)]
pub fn create_component(
    mut commands: Commands,
    mut created_query: Query<
        (&EditorComponent, &mut ComponentCoords, Entity),
        With<CreatedComponent>,
    >,
    selected_query: Query<Entity, With<SelectedComponent>>,
    buttons: Res<Input<MouseButton>>,
    skin: Res<Skin>,
    state: Res<CurrentState<EditorState>>,
    windows: Res<Windows>,
    mut camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    hovering: Res<HoveringOverGui>,
) {
    // TODO check if screen is moved
    if let EditorState::CreatingComponent(type_) = &state.0 {
        let (camera, transform): (&Camera, &GlobalTransform) = camera.single_mut();
        let mouse_pos = if let Some(mp) = get_cursor_world_pos(&windows, camera, transform) {
            mp
        } else {
            return;
        };
        if buttons.just_released(MouseButton::Left) && !hovering.0 {
            if *type_ == ComponentType::Point {
                for entity in selected_query.iter() {
                    commands.entity(entity).remove::<SelectedComponent>();
                }
                let mut new_point = ComponentBundle::new(
                    EditorComponent::new(type_.to_owned()),
                    mouse_pos.as_ivec2(),
                );
                new_point.update_shape(&skin);
                commands.spawn_bundle(new_point).insert(SelectedComponent);
                return;
            }
            if created_query.is_empty() {
                let mut new_comp = ComponentBundle::new(
                    EditorComponent::new(type_.to_owned()),
                    mouse_pos.as_ivec2(),
                );
                new_comp.update_shape(&skin);
                commands.spawn_bundle(new_comp).insert(CreatedComponent);
            } else {
                let (data, mut coords, entity): (&EditorComponent, Mut<ComponentCoords>, Entity) =
                    created_query.single_mut();
                match data.get_type(&skin).unwrap() {
                    ComponentType::Line | ComponentType::Area => {
                        coords.0.push(mouse_pos.as_ivec2());
                        commands
                            .entity(entity)
                            .insert_bundle(data.get_shape((*coords).to_owned(), &skin));
                    }
                    ComponentType::Point => unreachable!(),
                }
            }
        } else if buttons.just_released(MouseButton::Right) { // or double left-click?
             // TODO complete line/area
        } else if *type_ != ComponentType::Point && !created_query.is_empty() {
            let (data, coords, entity): (&EditorComponent, Mut<ComponentCoords>, Entity) =
                created_query.single_mut();
            let mut coords = coords.to_owned();
            coords.0.push(mouse_pos.as_ivec2());
            commands
                .entity(entity)
                .insert_bundle(data.get_shape(coords, &skin));
        }
    }
}

pub fn clear_created_component(
    mut commands: Commands,
    mut skin: Res<Skin>,
    created_query: Query<
        (&EditorComponent, &mut ComponentCoords, Entity),
        With<CreatedComponent>,
    >,
    state: Res<CurrentState<EditorState>>,
) {
    if let EditorState::CreatingComponent(_) = &state.0 {
        return
    }
    for (data, coords, entity) in created_query.iter() {
        commands
            .entity(entity)
            .insert_bundle(data.get_shape(coords.to_owned(), &skin));
        commands.entity(entity).remove::<CreatedComponent>();
    }
}