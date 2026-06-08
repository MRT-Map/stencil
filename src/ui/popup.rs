use std::borrow::Cow;

use declarative_enum_dispatch::enum_dispatch;
use tracing::info;

use crate::{
    App,
    info_windows::{
        changelog::ChangelogPopup, info::InfoPopup, licenses::LicensesPopup, manual::ManualPopup,
        quit::QuitPopup,
    },
    project::load_save::ChooseNamespacesPopup,
};

enum_dispatch! {
    pub trait Popup {
        fn id(&self) -> Cow<'static, str>;
        fn title(&self) -> egui::WidgetText;
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
    }

    #[derive(Clone)]
    pub enum Popups {
        Changelog(ChangelogPopup),
        Info(InfoPopup),
        Licenses(LicensesPopup),
        Manual(ManualPopup),
        Quit(QuitPopup),
        ChooseNamespaces(ChooseNamespacesPopup),
    }
}

impl Popups {
    pub fn alert_ui<T: Into<egui::WidgetText>, F: FnOnce(&egui::Context, &mut App)>(
        app: &mut App,
        ui: &mut egui::Ui,
        text: T,
        close_fn: Option<F>,
    ) -> bool {
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.label(text);
        });
        if ui.button("Close").clicked() {
            if let Some(close_fn) = close_fn {
                close_fn(ui, app);
            }
            false
        } else {
            true
        }
    }
    pub fn confirm_ui<
        T: Into<egui::WidgetText>,
        F1: FnOnce(&egui::Context, &mut App),
        F2: FnOnce(&egui::Context, &mut App),
    >(
        app: &mut App,
        ui: &mut egui::Ui,
        text: T,
        yes_fn: Option<F1>,
        no_fn: Option<F2>,
    ) -> bool {
        Self::choice_ui(app, ui, text, "Yes", yes_fn, "No", no_fn)
    }
    pub fn choice_ui<
        'a,
        T: Into<egui::WidgetText>,
        T1: egui::IntoAtoms<'a>,
        F1: FnOnce(&egui::Context, &mut App),
        T2: egui::IntoAtoms<'a>,
        F2: FnOnce(&egui::Context, &mut App),
    >(
        app: &mut App,
        ui: &mut egui::Ui,
        text: T,
        text1: T1,
        fn1: Option<F1>,
        text2: T2,
        fn2: Option<F2>,
    ) -> bool {
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.label(text);
        });
        ui.horizontal(|ui| {
            if ui.button(text1).clicked() {
                if let Some(fn1) = fn1 {
                    fn1(ui, app);
                }
                false
            } else if ui.button(text2).clicked() {
                if let Some(fn2) = fn2 {
                    fn2(ui, app);
                }
                false
            } else {
                true
            }
        })
        .inner
    }
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
