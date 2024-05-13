use bevy::prelude::*;
use bevy_egui::egui;

#[derive(Resource, Default)]
pub struct Status(pub egui::RichText);
