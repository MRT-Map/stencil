use bevy::color::palettes::basic::{BLACK, LIME, RED};
use bevy::prelude::*;
use bevy::render::primitives::Aabb;
use bevy_prototype_lyon::draw::{Fill, Stroke};
use itertools::Itertools;
use crate::component::actions::hovering::HoveredComponent;
use crate::component::actions::selecting::SelectedComponent;
use crate::component::circle::circle;
use crate::component::pla2::{ComponentType, EditorCoords, PlaComponent, HighlightExt};
use crate::component::skin::Skin;
use crate::misc_config::settings::MiscSettings;
use crate::state::EditorState;
use crate::tile::zoom::Zoom;
use crate::ui::cursor::mouse_pos::MousePosWorld;
use crate::ui::UiSchedule;

#[tracing::instrument(skip_all)]
pub fn on_render(
    trigger: Trigger<RenderEv>,
    mut commands: Commands,
    skin: Res<Skin>,
    query: Query<(&mut PlaComponent<EditorCoords>, Option<&HoveredComponent>, Option<&SelectedComponent>)>,
    zoom: Res<Zoom>,
    misc_settings: Res<MiscSettings>,
    state: Res<State<EditorState>>,
    mouse_pos_world: Res<MousePosWorld>
) {
    let e = trigger.entity();
    let Ok((data, hovered, selected)) = query.get(e) else {
        return;
    };
    let data = trigger.0.as_ref().unwrap_or(data);
    let ty = data.get_type(&skin);
    commands.entity(e).insert(data.get_shape(&skin));

    let (fill, stroke) = if selected.is_some() {
        (data.get_fill(&skin).select(ty).to_owned(), data.get_stroke(&skin).select(ty).to_owned())
    } else if hovered.is_some() {
        (data.get_fill(&skin).hover(ty).to_owned(), data.get_stroke(&skin).hover(ty).to_owned())
    } else {
        (data.get_fill(&skin), data.get_stroke(&skin))
    };
    if fill.color == Color::NONE {
        commands.entity(e).remove::<Fill>();
    } else {
        commands.entity(e).insert(fill);
    }
    if stroke.color == Color::NONE {
        commands.entity(e).remove::<Stroke>();
    } else {
        commands.entity(e).insert(stroke);
    }

    commands.entity(e).despawn_descendants();
    if *state == EditorState::EditingNodes && selected.is_some() {
        trace!("Updating handles");
        let children = data
            .nodes
            .iter()
            .map(|coord| &coord.0)
            .filter(|coord| {
                if data.nodes.len() > misc_settings.hide_far_handles_threshold {
                    (coord.as_vec2() - **mouse_pos_world).length_squared()
                        < misc_settings.hide_far_handles_distance
                } else {
                    true
                }
            })
            .map(|coord| {
                circle(
                    &zoom,
                    if ty == ComponentType::Point {
                        Vec2::ZERO
                    } else {
                        coord.as_vec2()
                    },
                    misc_settings.big_handle_size,
                    BLACK.into(),
                )
            })
            .map(|bundle| commands.spawn(bundle).id())
            .collect::<Vec<_>>();
        trace!("Pushing first set of children");
        commands.entity(e).add_children(&children);

        let more_children = if data.get_type(&skin) == ComponentType::Area {
            data.nodes
                .iter()
                .circular_tuple_windows::<(_, _)>()
                .collect::<Vec<_>>()
                .into_iter()
        } else {
            data.nodes
                .iter()
                .tuple_windows::<(_, _)>()
                .collect::<Vec<_>>()
                .into_iter()
        }
            .map(|(c1, c2)| (c1.0 + c2.0) / 2)
            .filter(|coord| {
                if data.nodes.len() > misc_settings.hide_far_handles_threshold {
                    (coord.as_vec2() - **mouse_pos_world).length_squared()
                        < misc_settings.hide_far_handles_distance
                } else {
                    true
                }
            })
            .map(|coord| {
                circle(
                    &zoom,
                    coord.as_vec2(),
                    misc_settings.small_handle_size,
                    BLACK.into(),
                )
            })
            .map(|bundle| commands.spawn(bundle).id())
            .collect::<Vec<_>>();
        trace!("Pushing second set of children");
        commands.entity(e).add_children(&more_children);

    } else if ty == ComponentType::Line && !data.nodes.is_empty() && selected.is_some() {
        let start = commands
            .spawn(circle(
                &zoom,
                data.nodes.first().unwrap().0.as_vec2(),
                misc_settings.big_handle_size,
                LIME.into(),
            ))
            .id();
        let end = commands
            .spawn(circle(
                &zoom,
                data.nodes.last().unwrap().0.as_vec2(),
                misc_settings.big_handle_size,
                RED.into(),
            ))
            .id();
        commands.entity(e).add_child(start).add_child(end);
    }
}

#[expect(clippy::needless_pass_by_value)]
pub fn rerender_selected_sy(selected: Query<Entity, With<SelectedComponent>>, mut commands: Commands) {
    for e in selected.iter() {
        commands.trigger_targets(RenderEv::default(), e);
    }
}

pub struct RenderComponentPlugin;
impl Plugin for RenderComponentPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_observer(on_render)
            .add_systems(UiSchedule, rerender_selected_sy.run_if(resource_changed::<Zoom>.or(state_changed::<EditorState>)));
    }
}

#[derive(Clone, PartialEq, Default, Event)]
pub struct RenderEv(pub Option<PlaComponent<EditorCoords>>);