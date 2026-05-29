use std::{
    fmt::Debug,
    ops::{Add, AddAssign, Deref, DerefMut},
};

use itertools::Itertools;

use crate::{
    PlaNodeTypeAdd,
    node::PlaNode,
    node_type::{PlaNodeType, PlaNodeTypeBezier, PlaNodeTypeBezierRect},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaNodeIndex {
    Coord(usize),
    Ctrl1(usize),
    Ctrl2(usize),
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(
    feature = "serde",
    serde(bound = "T: serde::Serialize + serde::de::DeserializeOwned")
)]
pub struct PlaNodeVec<T: PlaNodeType>(Vec<PlaNode<T>>);

impl<T: PlaNodeType> PlaNodeVec<T> {
    #[must_use]
    pub const fn new() -> Self {
        Self(Vec::new())
    }
    #[must_use]
    pub fn second_last(&self) -> Option<&PlaNode<T>> {
        if self.len() < 2 {
            return None;
        }
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
    pub fn map<U: PlaNodeType, F: Fn(T) -> U>(self, f: F) -> PlaNodeVec<U> {
        self.iter().map(|a| a.map(&f)).collect()
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
    pub fn outline<Tolerance: Into<Option<f32>> + Copy>(&self, tolerance: Tolerance) -> Vec<T> {
        let mut previous_coord = Option::<T>::None;
        let mut out = Vec::new();
        for n in self {
            out.extend(&match (*n, previous_coord) {
                (PlaNode::Line { coord, .. }, _) => vec![coord],
                (PlaNode::QuadraticBezier { ctrl, coord, .. }, Some(previous_coord)) => {
                    T::flatten_quadratic(previous_coord, ctrl, coord, tolerance)
                }
                (
                    PlaNode::CubicBezier {
                        ctrl1,
                        ctrl2,
                        coord,
                        ..
                    },
                    Some(previous_coord),
                ) => T::flatten_cubic(previous_coord, ctrl1, ctrl2, coord, tolerance),
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

impl<Delta: PlaNodeType, T: PlaNodeTypeAdd<Delta>> Add<Delta> for PlaNodeVec<T> {
    type Output = Self;

    fn add(self, rhs: Delta) -> Self::Output {
        self.iter().map(|a| *a + rhs).collect()
    }
}

impl<Delta: PlaNodeType, T: PlaNodeTypeAdd<Delta>> AddAssign<Delta> for PlaNodeVec<T> {
    fn add_assign(&mut self, rhs: Delta) {
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

#[cfg(test)]
mod test {
    use proptest::prelude::*;

    use crate::{PlaNode, PlaNodeVec};

    prop_compose! {
        fn vec2()(a in any::<f32>(), b in any::<f32>()) -> (f32, f32) {
            (a, b)
        }
    }

    proptest! {
        #[test]
        fn test_rev(a in vec2(), b in vec2(), c in vec2(), d in vec2(), e in vec2(), f in vec2(), g in vec2()) {
            let vec = [
                PlaNode::Line {coord: a, label: None},
                PlaNode::Line {coord: b, label: None},
                PlaNode::QuadraticBezier {ctrl: c, coord: d, label: None},
                PlaNode::CubicBezier {ctrl1: e, ctrl2: f, coord: g, label: None}
            ].into_iter().collect::<PlaNodeVec<(f32, f32)>>();

            let actual = vec.rev();
            let expected = [
                PlaNode::Line {coord: g, label: None},
                PlaNode::CubicBezier {ctrl1: f, ctrl2: e, coord: d, label: None},
                PlaNode::QuadraticBezier {ctrl: c, coord: b, label: None},
                PlaNode::Line {coord: a, label: None}
            ].into_iter().collect::<PlaNodeVec<(f32, f32)>>();
            prop_assert_eq!(actual, expected);
        }
    }
}
