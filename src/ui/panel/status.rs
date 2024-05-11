use std::collections::{BTreeMap, HashMap};

use bevy::prelude::*;
use bevy_egui::egui;
use once_cell::sync::Lazy;

use crate::state::EditorState;

#[derive(Resource, Default)]
pub struct Status(pub egui::RichText);
