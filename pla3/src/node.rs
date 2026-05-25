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
