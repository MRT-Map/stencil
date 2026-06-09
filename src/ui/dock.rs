use std::sync::Arc;

use declarative_enum_dispatch::enum_dispatch;
use egui::mutex::Mutex;
use etcetera::AppStrategy;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use tracing::{error, info};

use crate::{
    App, impl_load_save,
    map::MapWindow,
    project::{
        component_editor::ComponentEditorWindow, history_viewer::HistoryViewerWindow,
        project_editor::ProjectEditorWindow,
    },
    settings::SettingsWindow,
    ui::notif::NotifLogWindow,
    utils::file::FOLDERS,
};

enum_dispatch! {
    pub trait DockWindow: Copy {
        fn title(self) -> String;
        fn allowed_in_windows(self) -> bool {
            true
        }
        fn is_closeable(self) -> bool {
            true
        }
        fn ui(&mut self, app: &mut App, ui: &mut egui::Ui);
    }

    #[derive(Clone, Copy, Serialize, Deserialize, Debug)]
    #[serde(tag = "ty")]
    pub enum DockWindows {
        Map(MapWindow),
        ComponentEditor(ComponentEditorWindow),
        ProjectEditor(ProjectEditorWindow),
        Settings(SettingsWindow),
        NotifLog(NotifLogWindow),
        HistoryViewer(HistoryViewerWindow),
    }
}

#[derive(Clone)]
pub struct DockLayout(Arc<Mutex<egui_dock::DockState<DockWindows>>>);

impl DockLayout {
    #[inline]
    pub fn get(&self) -> Arc<Mutex<egui_dock::DockState<DockWindows>>> {
        Arc::clone(&self.0)
    }
}

impl Serialize for DockLayout {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0.lock().serialize(serializer)
    }
}
impl<'de> Deserialize<'de> for DockLayout {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(Self(Arc::new(Mutex::new(Deserialize::deserialize(
            deserializer,
        )?))))
    }
    fn deserialize_in_place<D>(deserializer: D, place: &mut Self) -> Result<(), D::Error>
    where
        D: Deserializer<'de>,
    {
        *place.0.lock() = Deserialize::deserialize(deserializer)?;
        Ok(())
    }
}

impl_load_save!(json DockLayout, FOLDERS.in_data_dir("dock.json"));

impl Default for DockLayout {
    fn default() -> Self {
        let mut state = egui_dock::DockState::new(vec![MapWindow.into()]);
        let tree = state.main_surface_mut();
        let [_, _] = tree.split_left(
            egui_dock::NodeIndex::root(),
            0.2,
            vec![ComponentEditorWindow.into()],
        );
        let [_, _] = tree.split_right(
            egui_dock::NodeIndex::root(),
            0.8,
            vec![ProjectEditorWindow.into(), HistoryViewerWindow.into()],
        );
        Self(Arc::new(Mutex::new(state)))
    }
}
impl DockLayout {
    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

impl egui_dock::TabViewer for App {
    type Tab = DockWindows;

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        tab.title().into()
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        tab.ui(self, ui);
    }

    fn is_closeable(&self, tab: &Self::Tab) -> bool {
        tab.is_closeable()
    }

    fn allowed_in_windows(&self, tab: &mut Self::Tab) -> bool {
        tab.allowed_in_windows()
    }

    fn scroll_bars(&self, _tab: &Self::Tab) -> [bool; 2] {
        [false, false]
    }
}

impl App {
    pub fn dock(&mut self, ui: &mut egui::Ui) {
        let dock_state = self.ui.dock_layout.get();
        egui_dock::DockArea::new(&mut dock_state.lock())
            .style(egui_dock::Style::from_egui(ui.style().as_ref()))
            .show_inside(ui, self);
    }
}
impl DockLayout {
    pub fn open_window<W: Into<DockWindows>>(&self, window: W) {
        let mut dock_state = self.0.lock();
        let window = window.into();
        let tab_path = dock_state.find_tab_from(|a| a.title() == window.title());
        if let Some(tab_path) = tab_path {
            info!("Focusing on {}", window.title());
            let _ = dock_state
                .set_active_tab(tab_path)
                .inspect_err(|e| error!("{e:#}"));
        } else {
            info!("Creating new window {}", window.title());
            dock_state.add_window(vec![window]);
        }
    }
}
