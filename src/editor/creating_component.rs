use crate::pla::{ComponentBundle, CreatedComponent, EditorComponent, SelectedComponent};
use crate::{ComponentType, EditorState, PlaComponent, Skin};
use bevy::prelude::*;
use iyes_loopless::prelude::*;

pub fn create_component(
    mut commands: Commands,
    mut created_query: Query<&mut EditorComponent, With<CreatedComponent>>,
    buttons: Res<Input<MouseButton>>,
    skin: Res<Skin>,
    state: Res<CurrentState<EditorState>>,
) {
    if let EditorState::CreatingComponent(type_) = &state.0 {
        if buttons.just_released(MouseButton::Left) {
            if *type_ == ComponentType::Point {
                commands
                    .spawn_bundle(ComponentBundle {
                        data: EditorComponent::new(type_.to_owned()),
                    })
                    .insert(SelectedComponent);
                return;
            }
            if created_query.is_empty() {
                commands
                    .spawn_bundle(ComponentBundle {
                        data: EditorComponent::new(type_.to_owned()),
                    })
                    .insert(CreatedComponent);
            } else {
                let p = created_query.single_mut();
                match p.get_type(&skin).unwrap() {
                    ComponentType::Line => {},
                    ComponentType::Area => {},
                    ComponentType::Point => unreachable!()
                }
            }
        } else if buttons.just_released(MouseButton::Middle) { // or double left-click?
            // TODO complete line/area
        } // TODO hover
    }
}
