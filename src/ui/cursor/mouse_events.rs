use bevy::prelude::*;
use bevy_egui::EguiContexts;
use bevy_mod_picking::prelude::*;

use crate::{
    misc_config::settings::MiscSettings,
    ui::{
        cursor::mouse_pos::{MousePos, MousePosWorld},
        panel::dock::{within_tilemap, PanelDockState},
    },
};

#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct HoveredComponent;

#[derive(Debug, Event)]
pub enum MouseEvent {
    HoverOver(Entity),
    HoverLeave(Entity),
    LeftPress(Option<Entity>, MousePosWorld),
    LeftRelease(Option<Entity>, MousePosWorld),
    LeftClick(Option<Entity>, MousePosWorld),
    RightPress(MousePosWorld),
    RightRelease(MousePosWorld),
    RightClick(MousePosWorld),
}

#[tracing::instrument(skip_all)]
pub fn hover_handler_sy(
    mut commands: Commands,
    mut hovered_entity: Local<Option<Entity>>,
    mut event_reader_over: EventReader<Pointer<Over>>,
    mut event_reader_out: EventReader<Pointer<Out>>,
    mut event_writer: EventWriter<MouseEvent>,
) {
    for _ in event_reader_out.read() {
        let Some(target) = hovered_entity.take() else {
            break;
        };
        trace!(?target, "HoverLeave detected");
        event_writer.send(MouseEvent::HoverLeave(target));
        if let Some(mut commands) = commands.get_entity(target) {
            commands.remove::<HoveredComponent>();
        }
    }
    for e in event_reader_over.read() {
        if let Some(mut commands) = commands.get_entity(e.target) {
            trace!(?e.target, "HoverOver detected");
            *hovered_entity = Some(e.target);
            event_writer.send(MouseEvent::HoverOver(e.target));
            commands.insert(HoveredComponent);
        }
    }
}

#[tracing::instrument(skip_all)]
pub fn right_click_handler_sy(
    mut event_writer: EventWriter<MouseEvent>,
    mut ctx: EguiContexts,
    buttons: Res<ButtonInput<MouseButton>>,
    mut prev_mouse_pos: Local<Option<MousePos>>,
    mouse_pos: Res<MousePos>,
    mouse_pos_world: Res<MousePosWorld>,
    panel: Res<PanelDockState>,
    misc_settings: Res<MiscSettings>,
) {
    if buttons.just_pressed(MouseButton::Right) && within_tilemap(&mut ctx, &panel) {
        debug!("RightPress detected");
        *prev_mouse_pos = Some(*mouse_pos);
        event_writer.send(MouseEvent::RightPress(*mouse_pos_world));
    }
    if buttons.just_released(MouseButton::Right) && within_tilemap(&mut ctx, &panel) {
        debug!("RightRelease detected");
        event_writer.send(MouseEvent::RightRelease(*mouse_pos_world));
        if let Some(prev) = *prev_mouse_pos {
            if (*prev - **mouse_pos).length_squared() <= misc_settings.click_max_offset
                && within_tilemap(&mut ctx, &panel)
            {
                debug!("RightClick detected");
                event_writer.send(MouseEvent::RightClick(*mouse_pos_world));
            }
        }
    }
}

#[tracing::instrument(skip_all)]
pub fn left_click_handler_sy(
    mut event_reader_down: EventReader<Pointer<Down>>,
    mut event_writer: EventWriter<MouseEvent>,
    mut ctx: EguiContexts,
    buttons: Res<ButtonInput<MouseButton>>,
    mut selected_entity: Local<Option<Entity>>,
    mut prev_mouse_pos: Local<Option<MousePos>>,
    mouse_pos: Res<MousePos>,
    mouse_pos_world: Res<MousePosWorld>,
    panel: Res<PanelDockState>,
    misc_settings: Res<MiscSettings>,
) {
    let mut pressed_on_comp = false;
    if within_tilemap(&mut ctx, &panel) {
        for e in event_reader_down.read() {
            if e.button != PointerButton::Primary {
                continue;
            }
            debug!(?e.target, "LeftPress detected");
            *selected_entity = Some(e.target);
            *prev_mouse_pos = Some(*mouse_pos);
            event_writer.send(MouseEvent::LeftPress(Some(e.target), *mouse_pos_world));
            pressed_on_comp = true;
        }
    }

    if buttons.just_pressed(MouseButton::Left) && !pressed_on_comp {
        debug!(e = ?Option::<Entity>::None, "LeftPress detected");
        *prev_mouse_pos = Some(*mouse_pos);
        *selected_entity = None;
        event_writer.send(MouseEvent::LeftPress(None, *mouse_pos_world));
    }

    if !buttons.just_released(MouseButton::Left) {
        return;
    }
    let prev = if let Some(prev) = *prev_mouse_pos {
        *prev_mouse_pos = None;
        prev
    } else {
        return;
    };
    let curr = *mouse_pos;
    debug!(e = ?selected_entity, "LeftRelease detected");
    event_writer.send(MouseEvent::LeftRelease(*selected_entity, *mouse_pos_world));
    if (*prev - *curr).length_squared() <= misc_settings.click_max_offset
        && within_tilemap(&mut ctx, &panel)
    {
        debug!(e = ?selected_entity, "LeftClick detected");
        event_writer.send(MouseEvent::LeftClick(*selected_entity, *mouse_pos_world));
    }
    *selected_entity = None;
}
