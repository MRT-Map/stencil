use std::{
    fmt::Debug,
    ops::{Add, AddAssign},
};

use crate::{PlaNodeTypeAdd, node_type::PlaNodeType};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(
    feature = "serde",
    serde(bound = "T: serde::Serialize + serde::de::DeserializeOwned")
)]
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
    #[must_use]
    pub const fn label(self) -> Option<u8> {
        match self {
            Self::Line { label, .. }
            | Self::QuadraticBezier { label, .. }
            | Self::CubicBezier { label, .. } => label,
        }
    }
    #[must_use]
    pub const fn label_mut(&mut self) -> &mut Option<u8> {
        match self {
            Self::Line { label, .. }
            | Self::QuadraticBezier { label, .. }
            | Self::CubicBezier { label, .. } => label,
        }
    }
    #[must_use]
    pub const fn coord(self) -> T {
        match self {
            Self::Line { coord, .. }
            | Self::QuadraticBezier { coord, .. }
            | Self::CubicBezier { coord, .. } => coord,
        }
    }
    #[must_use]
    pub const fn coord_mut(&mut self) -> &mut T {
        match self {
            Self::Line { coord, .. }
            | Self::QuadraticBezier { coord, .. }
            | Self::CubicBezier { coord, .. } => coord,
        }
    }
    #[must_use]
    pub fn map<U: PlaNodeType, F: Fn(T) -> U>(self, f: F) -> PlaNode<U> {
        match self {
            Self::Line { coord, label } => PlaNode::Line {
                coord: f(coord),
                label,
            },
            Self::QuadraticBezier { ctrl, coord, label } => PlaNode::QuadraticBezier {
                ctrl: f(ctrl),
                coord: f(coord),
                label,
            },
            Self::CubicBezier {
                ctrl1,
                ctrl2,
                coord,
                label,
            } => PlaNode::CubicBezier {
                ctrl1: f(ctrl1),
                ctrl2: f(ctrl2),
                coord: f(coord),
                label,
            },
        }
    }
    pub fn try_map<U: PlaNodeType, F: Fn(T) -> Result<U, E>, E>(
        self,
        f: F,
    ) -> Result<PlaNode<U>, E> {
        Ok(match self {
            Self::Line { coord, label } => PlaNode::Line {
                coord: f(coord)?,
                label,
            },
            Self::QuadraticBezier { ctrl, coord, label } => PlaNode::QuadraticBezier {
                ctrl: f(ctrl)?,
                coord: f(coord)?,
                label,
            },
            Self::CubicBezier {
                ctrl1,
                ctrl2,
                coord,
                label,
            } => PlaNode::CubicBezier {
                ctrl1: f(ctrl1)?,
                ctrl2: f(ctrl2)?,
                coord: f(coord)?,
                label,
            },
        })
    }
    #[must_use]
    pub fn map_into<U: PlaNodeType + From<T>>(self) -> PlaNode<U> {
        self.map(Into::into)
    }
    pub fn try_map_into<U: PlaNodeType + TryFrom<T>>(self) -> Result<PlaNode<U>, U::Error> {
        self.try_map(TryInto::try_into)
    }
    #[must_use]
    pub fn map_from<U: PlaNodeType>(value: PlaNode<U>) -> Self
    where
        T: From<U>,
    {
        value.map_into()
    }
    pub fn try_map_from<U: PlaNodeType>(value: PlaNode<U>) -> Result<Self, T::Error>
    where
        T: TryFrom<U>,
    {
        value.try_map_into()
    }
}

impl<Delta: Debug + Copy + Eq, T: PlaNodeTypeAdd<Delta>> Add<Delta> for PlaNode<T> {
    type Output = Self;

    fn add(mut self, rhs: Delta) -> Self::Output {
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

impl<Delta: Debug + Copy + Eq, T: PlaNodeTypeAdd<Delta>> AddAssign<Delta> for PlaNode<T> {
    fn add_assign(&mut self, rhs: Delta) {
        *self = *self + rhs;
    }
}
