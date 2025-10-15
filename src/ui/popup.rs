use std::{any::Any, fmt::Display, sync::Mutex};

use enum_dispatch::enum_dispatch;
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::{
    App,
    event::{Event, Events},
    info_windows::{
        changelog::ChangelogPopup, info::InfoPopup, licenses::LicensesPopup, manual::ManualPopup,
        quit::QuitPopup,
    },
};

#[enum_dispatch]
pub trait Popup {
    fn id(&self) -> String;
    fn title(&self) -> String;
    fn window(&self) -> egui::Window<'static> {
        self.default_window()
    }
    fn default_window(&self) -> egui::Window<'static> {
        egui::Window::new(self.title())
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .id(egui::Id::new(self.id()))
    }
    fn ui(&mut self, app: &mut App, ui: &mut egui::Ui) -> bool;
    fn alert_ui(
        &mut self,
        app: &mut App,
        ui: &mut egui::Ui,
        text: impl Into<egui::WidgetText>,
        close_event: Option<impl Into<Events>>,
    ) -> bool {
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.label(text);
        });
        if ui.button("Close").clicked() {
            if let Some(close_event) = close_event {
                app.events.push_back(close_event.into())
            }
            false
        } else {
            true
        }
    }
    fn confirm_ui(
        &mut self,
        app: &mut App,
        ui: &mut egui::Ui,
        text: impl Into<egui::WidgetText>,
        yes_event: Option<impl Into<Events>>,
        no_event: Option<impl Into<Events>>,
    ) -> bool {
        self.choice_ui(app, ui, text, "Yes", yes_event, "No", no_event)
    }
    fn choice_ui<'a>(
        &mut self,
        app: &mut App,
        ui: &mut egui::Ui,
        text: impl Into<egui::WidgetText>,
        text1: impl egui::IntoAtoms<'a>,
        event1: Option<impl Into<Events>>,
        text2: impl egui::IntoAtoms<'a>,
        event2: Option<impl Into<Events>>,
    ) -> bool {
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.label(text);
        });
        ui.horizontal(|ui| {
            if ui.button(text1).clicked() {
                if let Some(event1) = event1 {
                    app.events.push_back(event1.into())
                }
                false
            } else if ui.button(text2).clicked() {
                if let Some(event2) = event2 {
                    app.events.push_back(event2.into())
                }
                false
            } else {
                true
            }
        })
        .inner
    }
}

#[enum_dispatch(Popup)]
#[derive(Clone, Serialize, Deserialize)]
#[serde(tag = "ty")]
pub enum Popups {
    ChangelogPopup,
    InfoPopup,
    LicensesPopup,
    ManualPopup,
    QuitPopup,
}

impl App {
    pub fn add_popup(&mut self, popup: impl Into<Popups>) {
        let popup = popup.into();
        if self.ui.popups.contains_key(&popup.id()) {
            return;
        }
        info!(id=?popup.id(), "Opening popup");
        self.ui.popups.insert(popup.id(), popup);
    }
    pub fn popups(&mut self, ctx: &egui::Context) {
        let mut popups = self.ui.popups.clone();
        popups.retain(|id, popup| {
            let shown = popup
                .window()
                .show(ctx, |ui| popup.ui(self, ui))
                .unwrap()
                .inner
                .unwrap();
            if !shown {
                info!(?id, "Closing popup");
            }
            shown
        });
        self.ui.popups = popups;
    }
}
