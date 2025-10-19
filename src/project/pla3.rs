use std::{
    collections::HashMap,
    fmt::{Display, Formatter, Write},
    sync::Arc,
};

use egui_notify::ToastLevel;
use eyre::{ContextCompat, Result, eyre};
use serde::{Deserialize, Serialize};

use crate::{App, project::skin::SkinComponent};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlaNode {
    Line {
        label: Option<u8>,
        coord: geo::Coord<i32>,
    },
    QuadraticBezier {
        label: Option<u8>,
        ctrl: geo::Coord<i32>,
        coord: geo::Coord<i32>,
    },
    CubicBezier {
        label: Option<u8>,
        ctrl1: geo::Coord<i32>,
        ctrl2: geo::Coord<i32>,
        coord: geo::Coord<i32>,
    },
}
impl PlaNode {
    pub const fn label(self) -> Option<u8> {
        match self {
            Self::Line { label, .. }
            | Self::QuadraticBezier { label, .. }
            | Self::CubicBezier { label, .. } => label,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlaComponent {
    pub namespace: String,
    pub id: String,
    pub skin_component: Arc<SkinComponent>,
    pub display_name: String,
    pub layer: f32,
    pub nodes: Vec<PlaNode>,
    pub misc: HashMap<String, toml::Value>,
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
    pub fn load_from_string(s: &str, namespace: String, id: String, app: &mut App) -> Result<Self> {
        fn get_coord(split: &[&str], i: usize) -> Result<geo::Coord<i32>> {
            let (x, y) = (split[i], split[i + 1]);
            Ok(geo::Coord::from((x.parse()?, y.parse()?)))
        }
        fn get_label(split: &[&str], i: usize) -> Result<Option<u8>> {
            let Some(label) = split.get(i) else {
                return Ok(None);
            };
            let Some(label) = label.strip_suffix("#") else {
                return Err(eyre!("`{label}` does not start with #"));
            };
            label.parse::<u8>().map(Some).map_err(Into::into)
        }

        let (nodes_str, attrs_str) = s
            .split_once("\n---\n")
            .wrap_err(format!("`---` not found in: {s}"))?;

        let nodes = nodes_str
            .split('\n')
            .map(|node_str| {
                let split = node_str.split(' ').collect::<Vec<_>>();
                match split.len() {
                    2 | 3 => Ok(Some(PlaNode::Line {
                        coord: get_coord(&split, 0)?,
                        label: get_label(&split, 2)?,
                    })),
                    4 | 5 => Ok(Some(PlaNode::QuadraticBezier {
                        ctrl: get_coord(&split, 0)?,
                        coord: get_coord(&split, 2)?,
                        label: get_label(&split, 4)?,
                    })),
                    6 | 7 => Ok(Some(PlaNode::CubicBezier {
                        ctrl1: get_coord(&split, 0)?,
                        ctrl2: get_coord(&split, 2)?,
                        coord: get_coord(&split, 4)?,
                        label: get_label(&split, 6)?,
                    })),
                    len => Err(eyre!("`{node_str}` has invalid split length {len}")),
                }
            })
            .filter_map(std::result::Result::transpose)
            .collect::<Result<Vec<_>>>()?;

        let mut display_name = String::new();
        let mut layer = 0.0f32;
        let mut skin_component = Arc::clone(if nodes.len() == 1 {
            app.project.skin.get_type("simplePoint").unwrap()
        } else {
            app.project.skin.get_type("simpleLine").unwrap()
        });
        let mut misc = HashMap::<String, toml::Value>::new();
        for (k, v) in toml::from_str::<toml::Table>(attrs_str)? {
            match &*k {
                "display_name" => {
                    v.as_str()
                        .wrap_err(format!("`{v}` not string"))?
                        .clone_into(&mut display_name);
                }
                "layer" => {
                    layer = v
                        .as_float()
                        .map(|a| a as f32)
                        .or_else(|| v.as_integer().map(|a| a as f32))
                        .wrap_err(format!("`{v}` not number"))?;
                }
                "type" => {
                    if let Some(s) = app
                        .project
                        .skin
                        .get_type(v.as_str().wrap_err(format!("`{v}` not string"))?)
                    {
                        skin_component = Arc::clone(s);
                    } else {
                        app.ui.notifs.push(
                            format!("Unknown skin type for component {namespace}-{id}: {v}"),
                            ToastLevel::Warning,
                            &app.misc_settings,
                        );
                    }
                }
                _ => {
                    misc.insert(k, v);
                }
            }
        }

        Ok(Self {
            namespace,
            id,
            skin_component,
            display_name,
            layer,
            nodes,
            misc,
        })
    }
    pub fn save_to_string(&self) -> Result<String> {
        let mut out = String::new();

        for node in &self.nodes {
            match node {
                PlaNode::Line { coord, .. } => write!(out, "{} {}", coord.x, coord.y)?,
                PlaNode::QuadraticBezier { ctrl, coord, .. } => {
                    write!(out, "{} {} {} {}", ctrl.x, ctrl.y, coord.x, coord.y)?
                }
                PlaNode::CubicBezier {
                    ctrl1,
                    ctrl2,
                    coord,
                    ..
                } => write!(
                    out,
                    "{} {} {} {} {} {}",
                    ctrl1.x, ctrl1.y, ctrl2.x, ctrl2.y, coord.x, coord.y
                )?,
            }
            if let Some(label) = node.label() {
                writeln!(out, " #{label}")?;
            } else {
                out += "\n";
            }
        }
        out += "---\n";

        let attrs = self
            .misc
            .clone()
            .into_iter()
            .chain([
                ("display_name".into(), self.display_name.clone().into()),
                ("layer".into(), self.layer.into()),
                ("type".into(), self.skin_component.name().as_str().into()),
            ])
            .collect::<toml::Table>();
        out += &toml::to_string_pretty(&attrs)?;
        Ok(out)
    }
}
