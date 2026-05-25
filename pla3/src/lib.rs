use std::{
    collections::BTreeMap,
    error::Error,
    fmt::{Debug, Display, Formatter, Write},
    ops::{Add, AddAssign, Deref, DerefMut},
    path::{Path, PathBuf},
    str::FromStr,
    sync::Arc,
};

use eyre::{ContextCompat, Report, Result, eyre};
use itertools::Itertools;
use ordered_float::NotNan;
use serde::{Deserialize, Serialize, de::DeserializeOwned};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaNodeIndex {
    Coord(usize),
    Ctrl1(usize),
    Ctrl2(usize),
}

pub trait PlaNodeType: Debug + Clone + Copy + PartialEq + Eq + Add<Self, Output = Self> {}
impl<T: Debug + Clone + Copy + PartialEq + Eq + Add<Self, Output = Self>> PlaNodeType for T {}
pub trait PlaNodeTypeNew: PlaNodeType {
    type C: FromStr<Err: Error + Send + Sync + 'static>;
    fn new(x: Self::C, y: Self::C) -> Self;
}
pub trait PlaNodeTypeGet: PlaNodeType {
    type C: Display;
    fn x(self) -> Self::C;
    fn y(self) -> Self::C;
}
pub trait PlaNodeTypeRect: PlaNodeType {
    type Rect;
    fn combine_rect(a: Self::Rect, b: Self::Rect) -> Self::Rect;
    fn rect_from_point(self) -> Self::Rect;
    fn rect_from_line(a: Self, b: Self) -> Self::Rect;
    fn rect_centre(rect: Self::Rect) -> Self;
}
pub trait PlaNodeTypeBezier: PlaNodeType {
    fn flatten_quadratic(a: Self, b: Self, c: Self) -> Vec<Self>;
    fn flatten_cubic(a: Self, b: Self, c: Self, d: Self) -> Vec<Self>;
}
pub trait PlaNodeTypeBezierRect: PlaNodeTypeRect + PlaNodeTypeBezier {
    fn rect_from_quadratic(a: Self, b: Self, c: Self) -> Self::Rect;
    fn rect_from_cubic(a: Self, b: Self, c: Self, d: Self) -> Self::Rect;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(bound = "T: Serialize + DeserializeOwned")]
pub enum PlaNode<T: PlaNodeType> {
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
impl<T: PlaNodeType> PlaNode<T> {
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

impl<T: PlaNodeType> Add<T> for PlaNode<T> {
    type Output = Self;

    fn add(mut self, rhs: T) -> Self::Output {
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
impl<T: PlaNodeType> AddAssign<T> for PlaNode<T> {
    fn add_assign(&mut self, rhs: T) {
        *self = *self + rhs;
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(bound = "T: Serialize + DeserializeOwned")]
pub struct PlaNodeVec<T: PlaNodeType>(Vec<PlaNode<T>>);

impl<T: PlaNodeType> PlaNodeVec<T> {
    #[must_use]
    pub const fn new() -> Self {
        Self(Vec::new())
    }
    #[must_use]
    pub fn second_last(&self) -> Option<&PlaNode<T>> {
        self.get(self.len() - 2)
    }
    #[must_use]
    pub fn rev(&self) -> Self {
        let mut s = self.iter().rev().peekable();
        let Some(last) = s.peek() else {
            return Self::new();
        };
        std::iter::once(PlaNode::Line {
            coord: last.coord(),
            label: last.label(),
        })
        .chain(s.tuple_windows().map(|(b, f)| match *b {
            PlaNode::Line { .. } => PlaNode::Line {
                coord: f.coord(),
                label: f.label(),
            },
            PlaNode::QuadraticBezier { ctrl, .. } => PlaNode::QuadraticBezier {
                ctrl,
                coord: f.coord(),
                label: f.label(),
            },
            PlaNode::CubicBezier { ctrl1, ctrl2, .. } => PlaNode::CubicBezier {
                ctrl1: ctrl2,
                ctrl2: ctrl1,
                coord: f.coord(),
                label: f.label(),
            },
        }))
        .collect()
    }
}
impl<T: PlaNodeTypeBezierRect> PlaNodeVec<T> {
    pub fn bounding_box(&self) -> Option<T::Rect> {
        let mut s = self.iter().peekable();
        let mut bb = s.peek()?.coord().rect_from_point();
        if let Some(bb2) = s
            .tuple_windows()
            .map(|(n1, n2)| match n2 {
                PlaNode::Line { coord, .. } => T::rect_from_line(n1.coord(), *coord),
                PlaNode::QuadraticBezier { ctrl, coord, .. } => {
                    T::rect_from_quadratic(n1.coord(), *ctrl, *coord)
                }
                PlaNode::CubicBezier {
                    ctrl1,
                    ctrl2,
                    coord,
                    ..
                } => T::rect_from_cubic(n1.coord(), *ctrl1, *ctrl2, *coord),
            })
            .reduce(T::combine_rect)
        {
            bb = T::combine_rect(bb, bb2);
        }
        Some(bb)
    }
    pub fn centre(&self) -> Option<T> {
        self.bounding_box().map(T::rect_centre)
    }
}
impl<T: PlaNodeTypeBezier> PlaNodeVec<T> {
    #[must_use]
    pub fn outline(&self) -> Vec<T> {
        let mut previous_coord = Option::<T>::None;
        let mut out = Vec::new();
        for n in self {
            out.extend(&match (*n, previous_coord) {
                (PlaNode::Line { coord, .. }, _) => vec![coord],
                (PlaNode::QuadraticBezier { ctrl, coord, .. }, Some(previous_coord)) => {
                    T::flatten_quadratic(previous_coord, ctrl, coord)
                }
                (
                    PlaNode::CubicBezier {
                        ctrl1,
                        ctrl2,
                        coord,
                        ..
                    },
                    Some(previous_coord),
                ) => T::flatten_cubic(previous_coord, ctrl1, ctrl2, coord),
                _ => unreachable!(),
            });
            previous_coord = Some(n.coord());
        }
        out.dedup();
        out
    }
}
impl<T: PlaNodeType> From<Vec<PlaNode<T>>> for PlaNodeVec<T> {
    fn from(value: Vec<PlaNode<T>>) -> Self {
        Self(value)
    }
}
impl<T: PlaNodeType> Add<T> for PlaNodeVec<T> {
    type Output = Self;

    fn add(self, rhs: T) -> Self::Output {
        self.iter().map(|a| *a + rhs).collect()
    }
}
impl<T: PlaNodeType> AddAssign<T> for PlaNodeVec<T> {
    fn add_assign(&mut self, rhs: T) {
        for a in self {
            *a += rhs;
        }
    }
}

impl<T: PlaNodeType> Deref for PlaNodeVec<T> {
    type Target = Vec<PlaNode<T>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<T: PlaNodeType> DerefMut for PlaNodeVec<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T: PlaNodeType> IntoIterator for PlaNodeVec<T> {
    type Item = PlaNode<T>;
    type IntoIter = std::vec::IntoIter<PlaNode<T>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}
impl<'a, T: PlaNodeType> IntoIterator for &'a PlaNodeVec<T> {
    type Item = &'a PlaNode<T>;
    type IntoIter = std::slice::Iter<'a, PlaNode<T>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}
impl<'a, T: PlaNodeType> IntoIterator for &'a mut PlaNodeVec<T> {
    type Item = &'a mut PlaNode<T>;
    type IntoIter = std::slice::IterMut<'a, PlaNode<T>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter_mut()
    }
}
impl<T: PlaNodeType> FromIterator<PlaNode<T>> for PlaNodeVec<T> {
    fn from_iter<I: IntoIterator<Item = PlaNode<T>>>(iter: I) -> Self {
        Self(iter.into_iter().collect())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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
    #[must_use]
    pub const fn new(namespace: String, id: String) -> Self {
        Self { namespace, id }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlaComponent<S, T: PlaNodeType> {
    pub full_id: FullId,
    pub ty: Arc<S>,
    pub display_name: String,
    pub layer: NotNan<f32>,
    pub nodes: PlaNodeVec<T>,
    pub misc: BTreeMap<String, toml::Value>,
}

impl<S, T: PlaNodeType> Display for PlaComponent<S, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.full_id)?;
        if !self.display_name.is_empty() {
            write!(f, " ({})", self.display_name)?;
        }
        Ok(())
    }
}

impl<S, T: PlaNodeType> PlaComponent<S, T> {
    #[must_use]
    pub fn path(&self, root: &Path) -> PathBuf {
        root.join(&self.full_id.namespace)
            .join(format!("{}.pla3", self.full_id.id))
    }
}
impl<S, T: PlaNodeTypeNew> PlaComponent<S, T>
where
    <T::C as FromStr>::Err: 'static,
{
    fn get_coord(split: &[&str], i: usize) -> Result<T, <T::C as FromStr>::Err> {
        let (x, y) = (split[i], split[i + 1]);
        Ok(PlaNodeTypeNew::new(x.parse()?, y.parse()?))
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
    #[tracing::instrument(skip_all, fields(full_id))]
    pub fn load_from_string(
        s: &str,
        full_id: FullId,
        get_type: impl Fn(&str) -> Option<Arc<S>>,
    ) -> Result<(Self, Option<Report>)> {
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
                        coord: Self::get_coord(&split, 0)?,
                        label: Self::get_label(&split, 2)?,
                    })),
                    4 | 5 => Ok(Some(PlaNode::QuadraticBezier {
                        ctrl: Self::get_coord(&split, 0)?,
                        coord: Self::get_coord(&split, 2)?,
                        label: Self::get_label(&split, 4)?,
                    })),
                    6 | 7 => Ok(Some(PlaNode::CubicBezier {
                        ctrl1: Self::get_coord(&split, 0)?,
                        ctrl2: Self::get_coord(&split, 2)?,
                        coord: Self::get_coord(&split, 4)?,
                        label: Self::get_label(&split, 6)?,
                    })),
                    len => Err(eyre!("`{node_str}` has invalid split length {len}")),
                }
            })
            .filter_map(Result::transpose)
            .collect::<Result<PlaNodeVec<T>>>()?;

        if !matches!(nodes.first(), Some(PlaNode::Line { .. })) {
            return Err(eyre!(
                "First node must exist and not be a curve (Got {:?})",
                nodes.first()
            ));
        }

        let mut display_name = String::new();
        let mut layer = NotNan::<f32>::default();
        let mut skin_component = if nodes.len() == 1 {
            get_type("simplePoint").wrap_err("No type `simplePoint`")?
        } else {
            get_type("simpleLine").wrap_err("No type `simpleLine`")?
        };
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
                    if let Some(s) = get_type(v.as_str().wrap_err(format!("`{v}` not string"))?) {
                        skin_component = s;
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
}
impl<S: Display, T: PlaNodeTypeGet> PlaComponent<S, T> {
    #[tracing::instrument(skip_all, fields(self.full_id))]
    pub fn save_to_string(&self) -> Result<String> {
        let mut out = String::new();

        for node in &self.nodes {
            match node {
                PlaNode::Line { coord, .. } => write!(out, "{} {}", coord.x(), coord.y())?,
                PlaNode::QuadraticBezier { ctrl, coord, .. } => {
                    write!(out, "{} {} {} {}", ctrl.x(), ctrl.y(), coord.x(), coord.y())?;
                }
                PlaNode::CubicBezier {
                    ctrl1,
                    ctrl2,
                    coord,
                    ..
                } => write!(
                    out,
                    "{} {} {} {} {} {}",
                    ctrl1.x(),
                    ctrl1.y(),
                    ctrl2.x(),
                    ctrl2.y(),
                    coord.x(),
                    coord.y()
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
                ("type".into(), self.ty.to_string().into()),
            ])
            .collect::<toml::Table>();
        out += &toml::to_string_pretty(&attrs)?;
        Ok(out)
    }
}
