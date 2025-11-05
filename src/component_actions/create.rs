use std::{collections::HashMap, sync::Arc};

use tracing::info;

use crate::{
    App,
    map::MapWindow,
    project::{
        pla3::{PlaComponent, PlaNode},
        skin::SkinType,
    },
};

impl MapWindow {
    pub fn create_point(
        &self,
        app: &mut App,
        ui: &egui::Ui,
        response: &egui::Response,
        painter: &egui::Painter,
    ) {
        if app.project.new_component_ns.is_empty() {
            return;
        }
        let (Some(cursor_world_pos), Some(skin)) = (self.cursor_world_pos, app.project.skin())
        else {
            return;
        };
        let Some(ty) = self
            .created_point_type
            .as_ref()
            .or_else(|| skin.get_type("simplePoint"))
        else {
            return;
        };
        let Some(style) = ty.point_style_in_zoom_level(self.zoom_level(app)) else {
            return;
        };

        let world_coord = geo::coord! {
            x: cursor_world_pos.x.round() as i32,
            y: cursor_world_pos.y.round() as i32,
        };
        let screen_coord = self.world_to_screen(
            app,
            response.rect.center(),
            geo::coord! { x: world_coord.x as f32, y: world_coord.y as f32 },
        );
        Self::paint_point(ui, response, painter, false, screen_coord, ty.name(), style);

        if !response.clicked_by(egui::PointerButton::Primary) {
            return;
        }
        let component = PlaComponent {
            namespace: app.project.new_component_ns.clone(),
            id: app
                .project
                .components
                .get_new_id(&app.project.new_component_ns),
            ty: Arc::clone(ty),
            display_name: String::new(),
            layer: 0.0,
            nodes: vec![PlaNode::Line {
                coord: world_coord,
                label: None,
            }],
            misc: HashMap::default(),
        };
        info!(?world_coord, %component, "Created new point");

        app.project.components.insert(skin, component);
    }
}
