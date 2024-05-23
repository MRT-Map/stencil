use bevy::{prelude::*, window::WindowMode};
use bevy_egui::egui;
use surf::Url;

#[cfg(target_os = "linux")]
use crate::window::settings::LinuxWindow;
use crate::{
    action::Action,
    dirs_paths::{cache_path, data_path},
    misc_config::settings::MiscSettings,
    ui::panel::dock::{DockWindow, PanelDockState, PanelParams, TabViewer},
};

pub struct OpenMiscSettingsAct;

#[derive(Clone, Copy)]
pub struct MiscSettingsEditor;

impl DockWindow for MiscSettingsEditor {
    fn title(self) -> String {
        "Misc Settings".into()
    }
    fn ui(self, tab_viewer: &mut TabViewer, ui: &mut egui::Ui) {
        let PanelParams { misc_settings, .. } = tab_viewer.params;
        let mut invalid = false;
        let old_settings = misc_settings.to_owned();

        ui.add(egui::TextEdit::singleline(&mut misc_settings.skin_url).hint_text("Skin URL"));
        if let Err(e) = Url::try_from(&*misc_settings.skin_url) {
            ui.colored_label(egui::Color32::RED, format!("Invalid URL: {e:?}"));
            invalid = true;
        }
        ui.label("The URL for the skin");
        if ui
            .add_enabled(
                cache_path("skin.msgpack").exists(),
                egui::Button::new("Clear skin cache"),
            )
            .clicked
            && cache_path("skin.msgpack").exists()
        {
            std::fs::remove_file(cache_path("skin.msgpack")).unwrap();
        }
        ui.separator();

        if !invalid && old_settings != **misc_settings {
            misc_settings.save().unwrap();
            if old_settings.skin_url != misc_settings.skin_url
                && cache_path("skin.msgpack").exists()
            {
                std::fs::remove_file(cache_path("skin.msgpack")).unwrap();
            }
        }
    }
}

pub fn misc_settings_asy(mut actions: EventReader<Action>, mut state: ResMut<PanelDockState>) {
    for event in actions.read() {
        if matches!(event.downcast_ref(), Some(OpenMiscSettingsAct))
            && !state
                .state
                .iter_all_tabs()
                .any(|(_, a)| a.title() == MiscSettingsEditor.title())
        {
            state.state.add_window(vec![MiscSettingsEditor.into()]);
        }
    }
}
