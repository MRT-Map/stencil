use crate::pla::{
    ComponentBundle, ComponentCoords, CreatedComponent, EditorComponent, SelectedComponent,
};
use crate::{ComponentType, EditorState, HoveringOverGui, Skin};
use bevy::math::Vec3Swizzles;
use bevy::prelude::*;
use bevy_mouse_tracking_plugin::MousePosWorld;
use iyes_loopless::prelude::*;

#[allow(clippy::too_many_arguments)]
pub fn create_component(
    mut commands: Commands,
    mut created_query: Query<(&EditorComponent, &mut ComponentCoords, Entity), With<CreatedComponent>>,
    selected_query: Query<Entity, With<SelectedComponent>>,
    buttons: Res<Input<MouseButton>>,
    skin: Res<Skin>,
    state: Res<CurrentState<EditorState>>,
    mouse_pos: Res<MousePosWorld>,
    hovering: Res<HoveringOverGui>,
) { // TODO check if screen is moved
    if let EditorState::CreatingComponent(type_) = &state.0 {
        if buttons.just_released(MouseButton::Left) && !hovering.0 {
            if *type_ == ComponentType::Point {
                for entity in selected_query.iter() {
                    commands.entity(entity).remove::<SelectedComponent>();
                }
                let mut new_point = ComponentBundle::new(
                    EditorComponent::new(type_.to_owned()),
                    mouse_pos.xy().as_ivec2(),
                );
                new_point.update_shape(&skin);
                commands.spawn_bundle(new_point).insert(SelectedComponent);
                return;
            }
            if created_query.is_empty() {
                commands
                    .spawn_bundle(ComponentBundle::new(
                        EditorComponent::new(type_.to_owned()),
                        mouse_pos.xy().as_ivec2(),
                    ))
                    .insert(CreatedComponent);
            } else {
                let (data, mut coords, entity): (&EditorComponent, Mut<ComponentCoords>, Entity)
                    = created_query.single_mut();
                match data.get_type(&skin).unwrap() {
                    ComponentType::Line | ComponentType::Area => {
                        coords.0.push(mouse_pos.xy().as_ivec2());
                        commands.entity(entity).insert_bundle(data.get_shape((*coords).to_owned(), &skin));
                    }
                    ComponentType::Point => unreachable!(),
                }
            }
        } else if buttons.just_released(MouseButton::Right) { // or double left-click?
             // TODO complete line/area
        } // TODO hover
    }
}
