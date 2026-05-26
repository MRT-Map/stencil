use std::num::ParseIntError;

use ordered_float::FloatIsNan;
use thiserror::Error;

use crate::FullId;

#[derive(Error, Debug)]
pub enum InvalidLabelError {
    #[error("Does not start with #")]
    MissingPrefix,
    #[error("Invalid number")]
    InvalidNumber(#[from] ParseIntError),
}

#[derive(Error, Debug)]
pub enum InvalidLayerError {
    #[error("Neither integer nor float")]
    NeitherIntegerNorFloat,
    #[error("Is NaN")]
    IsNaN(#[from] FloatIsNan),
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("Invalid label `{0}`")]
    InvalidLabel(String, #[source] InvalidLabelError),
    #[error("`---` not found in: {0:?}")]
    MissingSeparator(String),
    #[error("`{0}` has invalid split length {1}")]
    InvalidSplitLength(String, usize),
    #[error("Invalid coordinate {0}")]
    InvalidCoordinate(
        String,
        #[source] Box<dyn std::error::Error + Send + Sync + 'static>,
    ),
    #[error("First node must exist and not be a curve (got {0})")]
    FirstNodeIsCurve(String),
    #[error("No type `{0}`")]
    MissingType(String),
    #[error("Invalid display name, must be string (got {0})")]
    InvalidDisplayName(toml::Value),
    #[error("Invalid layer, must be non-NaN number (got {0})")]
    InvalidLayer(toml::Value, #[source] InvalidLayerError),
    #[error("Invalid skin type, must be string (got {0})")]
    InvalidSkinType(toml::Value),
    #[error("Unknown skin type for component {0}: {1}")]
    UnknownType(FullId, String),
    #[error("TOML serialisation error")]
    TOMLSerialisation(#[from] toml::ser::Error),
    #[error("TOML deserialisation error")]
    TOMLDeserialisation(#[from] toml::de::Error),
    #[error("Writing error")]
    Writing(#[from] std::fmt::Error),
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
