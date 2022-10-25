use std::{
    any::Any,
    collections::HashMap,
    hash::{Hash, Hasher},
    sync::Arc,
};

use bevy::prelude::*;
use bevy_egui::{egui, egui::WidgetText, EguiContext};

use crate::{
    misc::Action,
    ui::{HoveringOverGui, UiStage},
};

#[allow(clippy::type_complexity)]
pub struct Popup {
    pub id: String,
    pub window: Box<dyn Fn() -> egui::Window<'static> + Sync + Send>,
    pub ui: Box<dyn Fn(&mut egui::Ui, &mut EventWriter<Action>, &mut bool) + Sync + Send>,
}

impl Hash for Popup {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state)
    }
}

impl PartialEq<Self> for Popup {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Popup {}

impl Popup {
    pub fn base_alert(
        id: impl std::fmt::Display,
        title: impl Into<WidgetText> + Clone + Sync + Send + 'static,
        text: impl Into<WidgetText> + Clone + Sync + Send + 'static,
    ) -> Self {
        Self {
            id: id.to_string(),
            window: Box::new(move || {
                egui::Window::new(title.to_owned())
                    .collapsible(false)
                    .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            }),
            ui: Box::new(move |ui, _, show| {
                ui.label(text.to_owned());
                if ui.button("Close").clicked() {
                    *show = false;
                }
            }),
        }
    }
    pub fn base_confirm(
        id: impl std::fmt::Display + Send + Sync + 'static,
        title: impl Into<WidgetText> + Clone + Sync + Send + 'static,
        text: impl Into<WidgetText> + Clone + Sync + Send + 'static,
        payload: impl Any + Sync + Send + Clone,
    ) -> Self {
        Self {
            id: id.to_string(),
            window: Box::new(move || {
                egui::Window::new(title.to_owned())
                    .collapsible(false)
                    .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            }),
            ui: Box::new(move |ui, ew, show| {
                ui.label(text.to_owned());
                if ui.button("Yes").clicked() {
                    ew.send(Action {
                        id: id.to_string(),
                        payload: Box::new(payload.to_owned()),
                    });
                    *show = false;
                }
                if ui.button("No").clicked() {
                    *show = false;
                }
            }),
        }
    }
}

pub fn popup_handler(
    mut ctx: ResMut<EguiContext>,
    mut event_reader: EventReader<Arc<Popup>>,
    mut event_writer: EventWriter<Action>,
    mut show: Local<HashMap<Arc<Popup>, bool>>,
    mut hovering_over_gui: ResMut<HoveringOverGui>,
) {
    for popup in event_reader.iter() {
        show.insert(popup.to_owned(), true);
    }
    let ctx = ctx.ctx_mut();
    for (popup, showed) in show.iter_mut() {
        let response: egui::InnerResponse<Option<()>> = (popup.window)()
            .show(ctx, |ui| (popup.ui)(ui, &mut event_writer, showed))
            .unwrap();
        if response.response.hovered() {
            hovering_over_gui.0 = true;
        }
    }
    show.retain(|_, a| *a);
}

pub struct PopupPlugin;

impl Plugin for PopupPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<Arc<Popup>>()
            .add_system_to_stage(UiStage, popup_handler);
    }
}
