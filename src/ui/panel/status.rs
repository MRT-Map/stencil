use bevy::prelude::*;
use bevy_egui::egui;

#[derive(Resource, Default)]
pub struct Status(pub egui::RichText);

impl Status {
    pub fn set<T: Into<egui::RichText>>(&mut self, text: T) {
        self.0 = text.into();
    }
}
