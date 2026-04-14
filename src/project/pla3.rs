use std::{
    collections::BTreeMap,
    fmt::{Debug, Display, Formatter, Write},
    ops::{Add, AddAssign, Deref, DerefMut},
    path::{Path, PathBuf},
    sync::Arc,
};

use eyre::{ContextCompat, Report, Result, eyre};
use itertools::Itertools;
use ordered_float::NotNan;
use serde::{Deserialize, Serialize, de::DeserializeOwned};

use crate::{
    App,
    coord_conversion::CoordConversionExt,
    project::{Project, skin::SkinType},
};

pub trait PlaNodeType: Debug + Clone + Copy + PartialEq + Eq {}
impl<T: Debug + Clone + Copy + PartialEq + Eq> PlaNodeType for T {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(bound = "T: Serialize + DeserializeOwned")]
pub enum PlaNodeBase<T: PlaNodeType> {
    Line {
        label: Option<u8>,
        coord: T,
    },
    QuadraticBezier {
        label: Option<u8>,
        ctrl: T,
        coord: T,
    },
    CubicBezier {
        label: Option<u8>,
        ctrl1: T,
        ctrl2: T,
        coord: T,
    },
}
pub type PlaNode = PlaNodeBase<geo::Coord<i32>>;
pub type PlaNodeScreen = PlaNodeBase<egui::Pos2>;
impl<T: PlaNodeType> PlaNodeBase<T> {
    pub const fn label(self) -> Option<u8> {
        match self {
            Self::Line { label, .. }
            | Self::QuadraticBezier { label, .. }
            | Self::CubicBezier { label, .. } => label,
        }
    }
    pub const fn label_mut(&mut self) -> &mut Option<u8> {
        match self {
            Self::Line { label, .. }
            | Self::QuadraticBezier { label, .. }
            | Self::CubicBezier { label, .. } => label,
        }
    }
    pub const fn coord(self) -> T {
        match self {
            Self::Line { coord, .. }
            | Self::QuadraticBezier { coord, .. }
            | Self::CubicBezier { coord, .. } => coord,
        }
    }
    pub const fn coord_mut(&mut self) -> &mut T {
        match self {
            Self::Line { coord, .. }
            | Self::QuadraticBezier { coord, .. }
            | Self::CubicBezier { coord, .. } => coord,
        }
    }
}
impl PlaNode {
    pub fn to_screen(self, app: &App, map_centre: egui::Pos2) -> PlaNodeScreen {
        let world_to_screen =
            |coord: geo::Coord<i32>| app.map_world_to_screen(map_centre, coord.to_geo_coord_f32());
        match self {
            Self::Line { coord, label } => PlaNodeScreen::Line {
                coord: world_to_screen(coord),
                label,
            },
            Self::QuadraticBezier { ctrl, coord, label } => PlaNodeBase::QuadraticBezier {
                ctrl: world_to_screen(ctrl),
                coord: world_to_screen(coord),
                label,
            },
            Self::CubicBezier {
                ctrl1,
                ctrl2,
                coord,
                label,
            } => PlaNodeBase::CubicBezier {
                ctrl1: world_to_screen(ctrl1),
                ctrl2: world_to_screen(ctrl2),
                coord: world_to_screen(coord),
                label,
            },
        }
    }
}

impl Add<geo::Coord<i32>> for PlaNode {
    type Output = Self;

    fn add(mut self, rhs: geo::Coord<i32>) -> Self::Output {
        match &mut self {
            Self::Line { coord, .. } => {
                *coord = *coord + rhs;
            }
            Self::QuadraticBezier { ctrl, coord, .. } => {
                *ctrl = *ctrl + rhs;
                *coord = *coord + rhs;
            }
            Self::CubicBezier {
                ctrl1,
                ctrl2,
                coord,
                ..
            } => {
                *ctrl1 = *ctrl1 + rhs;
                *ctrl2 = *ctrl2 + rhs;
                *coord = *coord + rhs;
            }
        }
        self
    }
}
impl AddAssign<geo::Coord<i32>> for PlaNode {
    fn add_assign(&mut self, rhs: geo::Coord<i32>) {
        *self = *self + rhs;
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(bound = "T: Serialize + DeserializeOwned")]
pub struct PlaNodeListBase<T: PlaNodeType>(Vec<PlaNodeBase<T>>);

pub type PlaNodeList = PlaNodeListBase<geo::Coord<i32>>;
pub type PlaNodeListScreen = PlaNodeListBase<egui::Pos2>;
impl<T: PlaNodeType> PlaNodeListBase<T> {
    pub const fn new() -> Self {
        Self(Vec::new())
    }
    pub fn second_last(&self) -> Option<&PlaNodeBase<T>> {
        self.0.get(self.0.len() - 2)
    }
    pub fn rev(&self) -> Self {
        let mut s = self.0.iter().rev().peekable();
        let Some(last) = s.peek() else {
            return Self::new();
        };
        std::iter::once(PlaNodeBase::Line {
            coord: last.coord(),
            label: last.label(),
        })
        .chain(s.tuple_windows().map(|(b, f)| match *b {
            PlaNodeBase::Line { .. } => PlaNodeBase::Line {
                coord: f.coord(),
                label: f.label(),
            },
            PlaNodeBase::QuadraticBezier { ctrl, .. } => PlaNodeBase::QuadraticBezier {
                ctrl,
                coord: f.coord(),
                label: f.label(),
            },
            PlaNodeBase::CubicBezier { ctrl1, ctrl2, .. } => PlaNodeBase::CubicBezier {
                ctrl1: ctrl2,
                ctrl2: ctrl1,
                coord: f.coord(),
                label: f.label(),
            },
        }))
        .collect()
    }
}
impl PlaNodeList {
    pub fn bounding_box(&self) -> Option<egui::Rect> {
        let mut s = self.0.iter().peekable();
        let mut bb = egui::Rect::from_pos(s.peek()?.coord().to_egui_pos2());
        if let Some(bb2) = s
            .tuple_windows()
            .map(|(n1, n2)| match n2 {
                PlaNode::Line { coord, .. } => {
                    egui::Rect::from_two_pos(n1.coord().to_egui_pos2(), coord.to_egui_pos2())
                }
                PlaNode::QuadraticBezier { ctrl, coord, .. } => {
                    egui::epaint::QuadraticBezierShape::from_points_stroke(
                        [
                            n1.coord().to_egui_pos2(),
                            ctrl.to_egui_pos2(),
                            coord.to_egui_pos2(),
                        ],
                        false,
                        egui::Color32::TRANSPARENT,
                        egui::epaint::PathStroke::default(),
                    )
                    .logical_bounding_rect()
                }
                PlaNode::CubicBezier {
                    ctrl1,
                    ctrl2,
                    coord,
                    ..
                } => egui::epaint::CubicBezierShape::from_points_stroke(
                    [
                        n1.coord().to_egui_pos2(),
                        ctrl1.to_egui_pos2(),
                        ctrl2.to_egui_pos2(),
                        coord.to_egui_pos2(),
                    ],
                    false,
                    egui::Color32::TRANSPARENT,
                    egui::epaint::PathStroke::default(),
                )
                .logical_bounding_rect(),
            })
            .reduce(|n1, n2| n1 | n2)
        {
            bb |= bb2;
        }
        Some(bb)
    }
    pub fn centre(&self) -> Option<geo::Coord<i32>> {
        self.bounding_box().map(|a| a.center().to_geo_coord_i32())
    }
    pub fn to_screen(&self, app: &App, map_centre: egui::Pos2) -> PlaNodeListScreen {
        self.0
            .iter()
            .map(|a| a.to_screen(app, map_centre))
            .collect()
    }
}
impl<T: PlaNodeType> From<Vec<PlaNodeBase<T>>> for PlaNodeListBase<T> {
    fn from(value: Vec<PlaNodeBase<T>>) -> Self {
        Self(value)
    }
}
impl Add<geo::Coord<i32>> for PlaNodeList {
    type Output = Self;

    fn add(self, rhs: geo::Coord<i32>) -> Self::Output {
        self.0.iter().map(|a| *a + rhs).collect()
    }
}
impl AddAssign<geo::Coord<i32>> for PlaNodeList {
    fn add_assign(&mut self, rhs: geo::Coord<i32>) {
        for a in &mut self.0 {
            *a += rhs;
        }
    }
}

impl<T: PlaNodeType> Deref for PlaNodeListBase<T> {
    type Target = Vec<PlaNodeBase<T>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<T: PlaNodeType> DerefMut for PlaNodeListBase<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T: PlaNodeType> IntoIterator for PlaNodeListBase<T> {
    type Item = PlaNodeBase<T>;
    type IntoIter = std::vec::IntoIter<PlaNodeBase<T>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}
impl<'a, T: PlaNodeType> IntoIterator for &'a PlaNodeListBase<T> {
    type Item = &'a PlaNodeBase<T>;
    type IntoIter = std::slice::Iter<'a, PlaNodeBase<T>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}
impl<'a, T: PlaNodeType> IntoIterator for &'a mut PlaNodeListBase<T> {
    type Item = &'a mut PlaNodeBase<T>;
    type IntoIter = std::slice::IterMut<'a, PlaNodeBase<T>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter_mut()
    }
}
impl<T: PlaNodeType> FromIterator<PlaNodeBase<T>> for PlaNodeListBase<T> {
    fn from_iter<I: IntoIterator<Item = PlaNodeBase<T>>>(iter: I) -> Self {
        Self(iter.into_iter().collect())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FullId {
    pub namespace: String,
    pub id: String,
}
impl Display for FullId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}-{}", self.namespace, self.id)?;
        Ok(())
    }
}
impl FullId {
    pub const fn new(namespace: String, id: String) -> Self {
        Self { namespace, id }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlaComponent {
    pub full_id: FullId,
    pub ty: Arc<SkinType>,
    pub display_name: String,
    pub layer: NotNan<f32>,
    pub nodes: PlaNodeList,
    pub misc: BTreeMap<String, toml::Value>,
}

impl Display for PlaComponent {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.full_id)?;
        if !self.display_name.is_empty() {
            write!(f, " ({})", self.display_name)?;
        }
        Ok(())
    }
}

impl PlaComponent {
    pub fn path(&self, root: &Path) -> PathBuf {
        root.join(&self.full_id.namespace)
            .join(format!("{}.pla3", self.full_id.id))
    }
    pub fn load_from_string(
        s: &str,
        full_id: FullId,
        project: &Project,
    ) -> Result<(Self, Option<Report>)> {
        fn get_coord(split: &[&str], i: usize) -> Result<geo::Coord<i32>> {
            let (x, y) = (split[i], split[i + 1]);
            Ok(geo::coord! { x: x.parse()?, y: y.parse()? })
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

        let mut unknown_type_error = None;
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
            .filter_map(Result::transpose)
            .collect::<Result<PlaNodeList>>()?;

        if !matches!(nodes.first(), Some(PlaNode::Line { .. })) {
            return Err(eyre!(
                "First node must exist and not be a curve (Got {:?})",
                nodes.first()
            ));
        }

        let mut display_name = String::new();
        let mut layer = NotNan::<f32>::default();
        let mut skin_component = Arc::clone(if nodes.len() == 1 {
            project.skin().unwrap().get_type("simplePoint").unwrap()
        } else {
            project.skin().unwrap().get_type("simpleLine").unwrap()
        });
        let mut misc = BTreeMap::<String, toml::Value>::new();
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
                        .and_then(|a| NotNan::new(a as f32).ok())
                        .or_else(|| v.as_integer().and_then(|a| NotNan::new(a as f32).ok()))
                        .wrap_err(format!("`{v}` not number"))?;
                }
                "type" => {
                    if let Some(s) = project
                        .skin()
                        .unwrap()
                        .get_type(v.as_str().wrap_err(format!("`{v}` not string"))?)
                    {
                        skin_component = Arc::clone(s);
                    } else {
                        unknown_type_error =
                            Some(eyre!("Unknown skin type for component {full_id}: {v}"));
                    }
                }
                _ => {
                    misc.insert(k, v);
                }
            }
        }

        Ok((
            Self {
                full_id,
                ty: skin_component,
                display_name,
                layer,
                nodes,
                misc,
            },
            unknown_type_error,
        ))
    }
    pub fn save_to_string(&self) -> Result<String> {
        let mut out = String::new();

        for node in &self.nodes {
            match node {
                PlaNode::Line { coord, .. } => write!(out, "{} {}", coord.x, coord.y)?,
                PlaNode::QuadraticBezier { ctrl, coord, .. } => {
                    write!(out, "{} {} {} {}", ctrl.x, ctrl.y, coord.x, coord.y)?;
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
                ("layer".into(), (*self.layer).into()),
                ("type".into(), self.ty.name().as_str().into()),
            ])
            .collect::<toml::Table>();
        out += &toml::to_string_pretty(&attrs)?;
        Ok(out)
    }
    pub fn bounding_rect(&self) -> geo::Rect<f32> {
        let mut x_min = f32::MAX;
        let mut x_max = f32::MIN;
        let mut y_min = f32::MAX;
        let mut y_max = f32::MIN;

        let mut cmp = |coord: geo::Coord<i32>| {
            if (coord.x as f32) < x_min {
                x_min = coord.x as f32;
            }
            if (coord.x as f32) > x_max {
                x_max = coord.x as f32;
            }
            if (coord.y as f32) < y_min {
                y_min = coord.y as f32;
            }
            if (coord.y as f32) > y_max {
                y_max = coord.y as f32;
            }
        };

        for node in &self.nodes {
            match node {
                PlaNode::Line { coord, .. } => cmp(*coord),
                PlaNode::QuadraticBezier { ctrl, coord, .. } => {
                    cmp(*ctrl);
                    cmp(*coord);
                }
                PlaNode::CubicBezier {
                    ctrl1,
                    ctrl2,
                    coord,
                    ..
                } => {
                    cmp(*ctrl1);
                    cmp(*ctrl2);
                    cmp(*coord);
                }
            }
        }

        geo::Rect::new(
            geo::coord! {x: x_min, y: y_min},
            geo::coord! {x: x_max, y: y_max},
        )
    }
}
