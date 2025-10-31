use std::{collections::HashMap, sync::Arc};

use rand::distr::{Alphanumeric, SampleString};
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
        if app.project.skin().is_none() || app.project.new_component_ns.is_empty() {
            return;
        }
        let Some(hover_pos) = response.hover_pos() else {
            return;
        };
        let Some(ty) = self
            .created_point_type
            .as_ref()
            .or_else(|| app.project.skin().and_then(|a| a.get_type("simplePoint")))
        else {
            return;
        };
        let SkinType::Point { styles, .. } = &**ty else {
            return;
        };

        let Some(style) = SkinType::style_in_zoom_level(styles, self.zoom_level(app)) else {
            return;
        };

        self.paint_point(ui, response, painter, false, hover_pos, style);

        if !response.clicked_by(egui::PointerButton::Primary) {
            return;
        }
        let coord = geo::coord! {
            x: self.cursor_world_pos.unwrap().x.round() as i32,
            y: self.cursor_world_pos.unwrap().y.round() as i32,
        };
        let component = PlaComponent {
            namespace: app.project.new_component_ns.clone(),
            id: app
                .project
                .components
                .get_new_id(&app.project.new_component_ns),
            ty: Arc::clone(ty),
            display_name: String::new(),
            layer: 0.0,
            nodes: vec![PlaNode::Line { coord, label: None }],
            misc: HashMap::default(),
        };
        info!(?coord, %component, "Created new point");

        app.project
            .components
            .insert(app.project.skin().unwrap(), component);
    }
}
