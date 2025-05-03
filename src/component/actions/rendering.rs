use bevy::{
    color::palettes::basic::{BLACK, LIME, RED},
    prelude::*,
    render::primitives::Aabb,
};
use itertools::Itertools;

use crate::{
    component::{
        actions::{hovering::HoveredComponent, selecting::SelectedComponent},
        circle::make_circle,
        pla2::{ComponentType, EditorCoords, HighlightExt, PlaComponent},
        skin::Skin,
        tools::creating::CreatedComponent,
    },
    misc_config::settings::MiscSettings,
    state::EditorState,
    ui::{cursor::mouse_pos::MousePosWorld, map::zoom::Zoom},
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
        Option<&CreatedComponent>,
    )>,
    zoom: Res<Zoom>,
    misc_settings: Res<MiscSettings>,
    state: Res<State<EditorState>>,
    mouse_pos_world: Res<MousePosWorld>,
) {
    let e = trigger.target();
    let Ok((pla, hovered, selected, created)) = query.get(e) else {
        return;
    };
    let pla = trigger.0.as_ref().unwrap_or(pla);
    let ty = pla.get_skin_type(&skin);

    let (mut shape, _) = pla.get_shape(&skin);
    let (mut fill, mut stroke) = (pla.get_fill(&skin), pla.get_stroke(&skin));
    if selected.is_some() {
        shape.fill = (fill.color != Color::NONE).then(|| fill.select(ty).to_owned());
        shape.stroke = (stroke.color != Color::NONE).then(|| stroke.select(ty).to_owned());
    } else if hovered.is_some() && created.is_none() {
        shape.fill = (fill.color != Color::NONE).then(|| fill.hover(ty).to_owned());
        shape.stroke = (stroke.color != Color::NONE).then(|| stroke.hover(ty).to_owned());
    }
    commands.entity(e).remove::<Aabb>().insert(shape);

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
                make_circle(
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

        let more_children = if pla.get_skin_type(&skin) == ComponentType::Area {
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
            make_circle(
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
            .spawn(make_circle(
                &zoom,
                pla.nodes.first().unwrap().0.as_vec2(),
                misc_settings.big_handle_size,
                LIME.into(),
            ))
            .id();
        let end = commands
            .spawn(make_circle(
                &zoom,
                pla.nodes.last().unwrap().0.as_vec2(),
                misc_settings.big_handle_size,
                RED.into(),
            ))
            .id();
        commands.entity(e).add_child(start).add_child(end);
    }
}

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
            Update,
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
