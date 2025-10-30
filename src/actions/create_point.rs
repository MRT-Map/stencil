use std::{collections::HashMap, sync::Arc};

use rand::distr::{Alphanumeric, SampleString};
use tracing::info;

use crate::{
    App,
    map::MapWindow,
    project::pla3::{PlaComponent, PlaNode},
};

impl MapWindow {
    pub fn create_point(
        &self,
        app: &mut App,
        response: &egui::Response,
        cursor_world_pos: geo::Coord<f32>,
    ) {
        if app.project.skin().is_none() || app.project.new_component_ns.is_empty() {
            return;
        }
        let Some(ty) = self
            .created_point_type
            .as_ref()
            .or_else(|| app.project.skin().and_then(|a| a.get_type("simplePoint")))
        else {
            return;
        };

        if !response.clicked_by(egui::PointerButton::Primary) {
            return;
        }
        let coord = geo::Coord::from((
            cursor_world_pos.x.round() as i32,
            cursor_world_pos.y.round() as i32,
        ));
        let component = PlaComponent {
            namespace: app.project.new_component_ns.clone(),
            id: Alphanumeric.sample_string(&mut rand::rng(), 16),
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
