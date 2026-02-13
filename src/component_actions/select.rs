use tracing::info;

use crate::{
    App,
    coord_conversion::CoordConversionExt,
    map::MapWindow,
    pointer::ResponsePointerExt,
    project::pla3::{FullId, PlaNode},
};

impl MapWindow {
    pub fn select_hovered_component(
        app: &mut App,
        ui: &egui::Ui,
        response: &egui::Response,
        painter: &egui::Painter,
    ) {
        if app.mode.is_editing() {
            app.ui.map.selected_components.clear();
            return;
        }

        if let Some(cursor_world_pos) = app.ui.map.cursor_world_pos {
            let id = "marquee select".into();
            if response.drag_started_by2(egui::PointerButton::Primary)
                && response
                    .ctx
                    .input(|i| !i.modifiers.command && !i.modifiers.alt && !i.modifiers.shift)
            {
                info!("Drag start");
                ui.data_mut(|d| d.insert_temp(id, cursor_world_pos));
                return;
            }
            if let Some(start_world_pos) = ui.data(|d| d.get_temp::<geo::Coord<f32>>(id)) {
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
                    info!("Drag end");
                    let bounding_box = egui::Rect::from_two_pos(
                        start_world_pos.to_egui_pos2(),
                        cursor_world_pos.to_egui_pos2(),
                    );
                    let components_to_add = app
                        .project
                        .components
                        .iter()
                        .filter(|a| {
                            PlaNode::bounding_box(a.nodes.iter().copied())
                                .is_some_and(|rect| bounding_box.contains_rect(rect))
                        })
                        .map(|a| a.full_id.clone());
                    if ui.ctx().input(|a| a.modifiers.shift) {
                        app.ui.map.selected_components.extend(components_to_add);
                    } else {
                        app.ui.map.selected_components = components_to_add.collect();
                    }
                    ui.data_mut(|d| d.remove_temp::<geo::Coord<f32>>(id));
                    return;
                }
            }
        }

        if !response.clicked_by2(egui::PointerButton::Primary)
            && !response.clicked_by2(egui::PointerButton::Secondary)
        {
            return;
        }

        let Some(hovered_component) = &app.ui.map.hovered_component else {
            info!(ids=?app.ui.map.selected_components, "Deselected all");
            app.ui.map.selected_components.clear();
            app.status_default(ui.ctx());
            return;
        };
        app.select_component(ui, hovered_component.to_owned());
    }
}
impl App {
    pub fn select_component(&mut self, ui: &egui::Ui, id: FullId) {
        if ui.ctx().input(|a| a.modifiers.shift) {
            if let Some(pos) = self
                .ui
                .map
                .selected_components
                .iter()
                .position(|a| *a == id)
            {
                info!(%id, "Deselected");
                self.ui.map.selected_components.remove(pos);
            } else {
                info!(%id, "Selected");
                self.ui.map.selected_components.push(id);
            }
        } else {
            info!(%id, "Deselected all and selected one");
            self.ui.map.selected_components.clear();
            self.ui.map.selected_components.push(id);
        }
        self.status_select(ui.ctx());
    }
}
