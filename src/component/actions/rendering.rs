use bevy::{
    color::palettes::basic::{BLACK, LIME, RED},
    prelude::*,
};
use bevy_egui::EguiContextPass;
use bevy_prototype_lyon::prelude::*;
use itertools::Itertools;

use crate::{
    component::{
        actions::{hovering::HoveredComponent, selecting::SelectedComponent},
        circle::circle,
        pla2::{ComponentType, EditorCoords, HighlightExt, PlaComponent},
        skin::Skin,
    },
    misc_config::settings::MiscSettings,
    state::EditorState,
    tile::zoom::Zoom,
    ui::cursor::mouse_pos::MousePosWorld,
};

#[tracing::instrument(skip_all)]
pub fn on_render(
    trigger: Trigger<RenderEv>,
    mut commands: Commands,
    skin: Res<Skin>,
    query: Query<(
        &mut PlaComponent<EditorCoords>,
        Option<&HoveredComponent>,
        Option<&SelectedComponent>,
    )>,
    zoom: Res<Zoom>,
    misc_settings: Res<MiscSettings>,
    state: Res<State<EditorState>>,
    mouse_pos_world: Res<MousePosWorld>,
) {
    let e = trigger.target();
    let Ok((pla, hovered, selected)) = query.get(e) else {
        return;
    };
    let pla = trigger.0.as_ref().unwrap_or(pla);
    let ty = pla.get_type(&skin);

    let (mut shape, _) = pla.get_shape(&skin);
    if selected.is_some() {
        shape.fill = Some(pla.get_fill(&skin).select(ty).to_owned());
        shape.stroke = Some(pla.get_stroke(&skin).select(ty).to_owned());
    }
    if hovered.is_some() {
        shape.fill = Some(pla.get_fill(&skin).hover(ty).to_owned());
        shape.stroke = Some(pla.get_stroke(&skin).hover(ty).to_owned());
    }
    commands.entity(e).insert(shape);

    commands.entity(e).despawn_related::<Children>();
    if *state == EditorState::EditingNodes && selected.is_some() {
        let filter_by_distance = |coord: &IVec2| -> bool {
            if pla.nodes.len() > misc_settings.hide_far_handles_threshold {
                (coord.as_vec2() - **mouse_pos_world).length_squared()
                    < misc_settings.hide_far_handles_distance
            } else {
                true
            }
        };

        trace!("Updating handles");
        let children = pla
            .nodes
            .iter()
            .map(|coord| coord.0)
            .filter(filter_by_distance)
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

        let more_children = if pla.get_type(&skin) == ComponentType::Area {
            pla.nodes
                .iter()
                .circular_tuple_windows::<(_, _)>()
                .collect::<Vec<_>>()
                .into_iter()
        } else {
            pla.nodes
                .iter()
                .tuple_windows::<(_, _)>()
                .collect::<Vec<_>>()
                .into_iter()
        }
        .map(|(c1, c2)| (c1.0 + c2.0) / 2)
        .filter(filter_by_distance)
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
    } else if ty == ComponentType::Line && !pla.nodes.is_empty() && selected.is_some() {
        let start = commands
            .spawn(circle(
                &zoom,
                pla.nodes.first().unwrap().0.as_vec2(),
                misc_settings.big_handle_size,
                LIME.into(),
            ))
            .id();
        let end = commands
            .spawn(circle(
                &zoom,
                pla.nodes.last().unwrap().0.as_vec2(),
                misc_settings.big_handle_size,
                RED.into(),
            ))
            .id();
        commands.entity(e).add_child(start).add_child(end);
    }
}

#[expect(clippy::needless_pass_by_value)]
pub fn rerender_selected_sy(
    selected: Query<Entity, With<SelectedComponent>>,
    mut commands: Commands,
) {
    for e in selected.iter() {
        commands.trigger_targets(RenderEv::default(), e);
    }
}

pub struct RenderComponentPlugin;
impl Plugin for RenderComponentPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_render).add_systems(
            EguiContextPass,
            rerender_selected_sy.run_if(
                resource_changed::<Zoom>
                    .or(in_state(EditorState::EditingNodes).and(resource_changed::<MousePosWorld>))
                    .or(state_changed::<EditorState>),
            ),
        );
    }
}

#[derive(Clone, PartialEq, Default, Event)]
pub struct RenderEv(pub Option<PlaComponent<EditorCoords>>);
