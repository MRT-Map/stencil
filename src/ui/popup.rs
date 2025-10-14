use std::{any::Any, fmt::Display, sync::Mutex};

use egui::Widget;
use enum_dispatch::enum_dispatch;
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::{
    App,
    event::{Event, Events},
    info_windows::changelog::ChangelogPopup,
};

#[enum_dispatch(Popups)]
pub trait Popup {
    fn id(&self) -> String;
    fn title(&self) -> String;
    fn window(&self) -> egui::Window<'static> {
        egui::Window::new(self.title())
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .id(egui::Id::new(self.id()))
    }
    fn ui(&mut self, app: &mut App, ui: &mut egui::Ui) -> bool;
}

pub trait AlertPopup: Popup {
    fn text(&self) -> impl Into<egui::WidgetText>;
    fn close_event(&self) -> Option<impl Into<Events>> {
        Option::<Events>::None
    }
    fn _ui(&mut self, app: &mut App, ui: &mut egui::Ui) -> bool {
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.label(self.text());
        });
        if ui.button("Close").clicked() {
            if let Some(close_event) = self.close_event() {
                app.events.push_back(close_event.into())
            }
            false
        } else {
            true
        }
    }
}

pub trait ConfirmPopup: Popup {
    fn text(&self) -> impl Into<egui::WidgetText>;
    fn yes_event(&self) -> Option<impl Into<Events>> {
        Option::<Events>::None
    }
    fn no_event(&self) -> Option<impl Into<Events>> {
        Option::<Events>::None
    }
    fn _ui(&mut self, app: &mut App, ui: &mut egui::Ui) -> bool {
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.label(self.text());
        });
        ui.horizontal(|ui| {
            if ui.button("Yes").clicked() {
                if let Some(yes_event) = self.yes_event() {
                    app.events.push_back(yes_event.into())
                }
                false
            } else if ui.button("No").clicked() {
                if let Some(no_event) = self.no_event() {
                    app.events.push_back(no_event.into())
                }
                false
            } else {
                true
            }
        })
        .inner
    }
}

pub trait ChoicePopup: Popup {
    fn text(&self) -> impl Into<egui::WidgetText>;
    fn action1<'a>(&self) -> (impl egui::IntoAtoms<'a>, Option<impl Into<Events>>);
    fn action2<'a>(&self) -> (impl egui::IntoAtoms<'a>, Option<impl Into<Events>>);
    fn _ui(&mut self, app: &mut App, ui: &mut egui::Ui) -> bool {
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.label(self.text());
        });
        ui.horizontal(|ui| {
            let (text1, event1) = self.action1();
            let (text2, event2) = self.action2();

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

#[enum_dispatch]
#[derive(Clone, Serialize, Deserialize)]
#[serde(tag = "ty")]
pub enum Popups {
    ChangelogPopup,
    // Info,
    // Licenses,
    // Manual,
    // Quit,
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
