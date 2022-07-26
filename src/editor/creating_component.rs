use crate::pla::{ComponentBundle, CreatedComponent, EditorComponent, SelectedComponent};
use crate::{ComponentType, EditorState, PlaComponent, Skin};
use bevy::math::Vec3Swizzles;
use bevy::prelude::*;
use bevy_mouse_tracking_plugin::{MousePos, MousePosWorld};
use iyes_loopless::prelude::*;

pub fn create_component(
    mut commands: Commands,
    mut created_query: Query<&mut EditorComponent, With<CreatedComponent>>,
    buttons: Res<Input<MouseButton>>,
    skin: Res<Skin>,
    state: Res<CurrentState<EditorState>>,
    mouse_pos: Res<MousePosWorld>,
) {
    if let EditorState::CreatingComponent(type_) = &state.0 {
        if buttons.just_released(MouseButton::Left) {
            if *type_ == ComponentType::Point {
                commands
                    .spawn_bundle(ComponentBundle::new(
                        EditorComponent::new(type_.to_owned()),
                        mouse_pos.xy().as_ivec2(),
                    ))
                    .insert(SelectedComponent);
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
                let p = created_query.single_mut();
                match p.get_type(&skin).unwrap() {
                    ComponentType::Line => {}
                    ComponentType::Area => {}
                    ComponentType::Point => unreachable!(),
                }
            }
        } else if buttons.just_released(MouseButton::Middle) { // or double left-click?
             // TODO complete line/area
        } // TODO hover
    }
}
