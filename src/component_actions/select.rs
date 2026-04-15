use tracing::info;

use crate::{
    App, coord_conversion::CoordConversionExt, map::MapWindow, pointer::ResponsePointerExt,
    project::pla3::FullId,
};

impl MapWindow {
    pub fn select_hovered_component(
        app: &mut App,
        ctx: &egui::Context,
        response: &egui::Response,
        painter: &egui::Painter,
    ) {
        if app.mode.is_editing() {
            app.ui.map.selected.clear();
            return;
        }

        let id = "marquee select".into();
        if let Some(cursor_world_pos) = app.ui.map.cursor_world_pos {
            if response.drag_started_by2(egui::PointerButton::Primary)
                && response
                    .ctx
                    .input(|i| !i.modifiers.command && !i.modifiers.alt)
            {
                info!("Marquee start");
                ctx.data_mut(|d| d.insert_temp(id, cursor_world_pos));
                return;
            }
            if let Some(start_world_pos) = ctx.data(|d| d.get_temp::<geo::Coord<f32>>(id)) {
                if response.dragged_by2(egui::PointerButton::Primary) {
                    painter.add(Self::white_dash(
                        &[
                            start_world_pos,
                            geo::coord! { x: start_world_pos.x, y: cursor_world_pos.y },
                            cursor_world_pos,
                            geo::coord! { x: cursor_world_pos.x, y: start_world_pos.y },
                            start_world_pos,
                        ]
                        .map(|c| app.map_world_to_screen(response.rect.center(), c)),
                        false,
                    ));
                    return;
                }
                if response.drag_stopped_by2(egui::PointerButton::Primary) {
                    info!("Marquee end");
                    let bounding_box = egui::Rect::from_two_pos(
                        start_world_pos.to_egui_pos2(),
                        cursor_world_pos.to_egui_pos2(),
                    );
                    let components_to_add = app
                        .project
                        .components
                        .iter()
                        .filter(|a| {
                            a.nodes
                                .bounding_box()
                                .is_some_and(|rect| bounding_box.contains_rect(rect))
                        })
                        .map(|a| (a.full_id.clone(), Vec::new()));
                    if ctx.input(|a| a.modifiers.shift) {
                        app.ui.map.selected.extend(components_to_add);
                    } else {
                        app.ui.map.selected = components_to_add.collect();
                    }
                    ctx.data_mut(|d| d.remove_temp::<geo::Coord<f32>>(id));
                    return;
                }
            }
        } else if response.drag_stopped_by2(egui::PointerButton::Primary) {
            info!("Marquee cancelled");
            ctx.data_mut(|d| d.remove_temp::<geo::Coord<f32>>(id));
            return;
        }

        if !response.clicked_by2(egui::PointerButton::Primary)
            && !response.clicked_by2(egui::PointerButton::Secondary)
        {
            return;
        }

        let Some(hovered_component) = &app.ui.map.hovered_component else {
            info!(ids=?app.ui.map.selected, "Deselected all");
            app.ui.map.selected.clear();
            app.status_default(ctx);
            return;
        };
        app.select_component(ctx, hovered_component.to_owned());
    }
}
impl App {
    pub fn select_component(&mut self, ctx: &egui::Context, id: FullId) {
        if ctx.input(|a| a.modifiers.shift) {
            #[expect(clippy::map_entry)]
            if self.ui.map.selected.contains_key(&id) {
                info!(%id, "Deselected");
                self.ui.map.selected.remove(&id);
            } else {
                info!(%id, "Selected");
                self.ui.map.selected.insert(id, Vec::new());
            }
        } else {
            info!(%id, "Deselected all and selected one");
            self.ui.map.selected.retain(|k, _| *k != id);
            self.ui.map.selected.entry(id).or_default();
        }
        self.status_select(ctx);
    }
}
