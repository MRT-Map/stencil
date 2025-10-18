use std::{
    collections::HashMap,
    fmt::{Display, Formatter},
    sync::Arc,
};

use eyre::Result;
use itertools::Either;
use serde::{Deserialize, Serialize};

use crate::project::skin::{Skin, SkinComponent};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PlaNodes {
    Area(geo::Polygon<f32>),
    Line(geo::LineString<f32>),
    Point(geo::Point<f32>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlaComponent {
    pub namespace: String,
    pub id: String,
    pub skin_component: Arc<SkinComponent>,
    pub display_name: String,
    pub layer: f32,
    pub nodes: PlaNodes,
    pub attributes: HashMap<String, String>,
}

impl Display for PlaComponent {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}-{}", self.namespace, self.id)?;
        if !self.display_name.is_empty() {
            write!(f, " ({})", self.display_name)?;
        }
        Ok(())
    }
}

impl PlaComponent {
    pub fn load_from_toml(toml: toml::Table) -> Result<Self> {
        todo!()
    }
}
