use enum_dispatch::enum_dispatch;
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::{
    App,
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
        close_fn: Option<impl FnOnce(&egui::Context, &mut App)>,
    ) -> bool {
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.label(text);
        });
        if ui.button("Close").clicked() {
            if let Some(close_fn) = close_fn {
                close_fn(ui.ctx(), app);
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
        yes_fn: Option<impl FnOnce(&egui::Context, &mut App)>,
        no_fn: Option<impl FnOnce(&egui::Context, &mut App)>,
    ) -> bool {
        self.choice_ui(app, ui, text, "Yes", yes_fn, "No", no_fn)
    }
    fn choice_ui<'a>(
        &mut self,
        app: &mut App,
        ui: &mut egui::Ui,
        text: impl Into<egui::WidgetText>,
        text1: impl egui::IntoAtoms<'a>,
        fn1: Option<impl FnOnce(&egui::Context, &mut App)>,
        text2: impl egui::IntoAtoms<'a>,
        fn2: Option<impl FnOnce(&egui::Context, &mut App)>,
    ) -> bool {
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.label(text);
        });
        ui.horizontal(|ui| {
            if ui.button(text1).clicked() {
                if let Some(fn1) = fn1 {
                    fn1(ui.ctx(), app);
                }
                false
            } else if ui.button(text2).clicked() {
                if let Some(fn2) = fn2 {
                    fn2(ui.ctx(), app);
                }
                false
            } else {
                true
            }
        })
        .inner
    }
}

#[expect(clippy::enum_variant_names)]
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
    pub fn add_popup<P: Into<Popups>>(&mut self, popup: P) {
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
