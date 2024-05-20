use bevy::{prelude::*, window::WindowMode};
use bevy_egui::egui;

#[cfg(target_os = "linux")]
use crate::window::settings::LinuxWindow;
use crate::{
    action::Action,
    dirs_paths::data_path,
    ui::panel::dock::{DockWindow, PanelDockState, PanelParams, TabViewer},
    window::settings::WindowSettings,
};

pub struct OpenWindowSettingsAct;

#[derive(Clone, Copy)]
pub struct WindowSettingsEditor;

impl DockWindow for WindowSettingsEditor {
    fn title(self) -> String {
        "Window Settings".into()
    }
    fn ui(self, tab_viewer: &mut TabViewer, ui: &mut egui::Ui) {
        let PanelParams {
            window_settings, ..
        } = tab_viewer.params;
        let mut invalid = false;
        let old_settings = window_settings.to_owned();

        if ui
            .add_enabled(
                **window_settings != WindowSettings::default(),
                egui::Button::new("Reset"),
            )
            .clicked()
        {
            **window_settings = WindowSettings::default();
        }
        ui.colored_label(
            egui::Color32::YELLOW,
            format!(
                "Window settings can also be edited at: {}",
                data_path("window_settings.toml").to_string_lossy()
            ),
        );
        ui.label("Changes will come into affect on the next launch of Stencil2");
        ui.label("If Stencil2 crashes the next time after changing anything here, you will have to edit the TOML file manually, good luck :)");
        ui.separator();
        ui.label("Enabled backends: Stencil2 will use one of these to render the window");
        ui.checkbox(&mut window_settings.backends.vulkan, "Vulkan");
        ui.checkbox(&mut window_settings.backends.metal, "Metal");
        ui.checkbox(&mut window_settings.backends.dx12, "DX12");
        ui.checkbox(&mut window_settings.backends.dx11, "DX11");
        if window_settings.backends.is_none() {
            ui.colored_label(
                egui::Color32::RED,
                "Select at least one backend!".to_owned(),
            );
            invalid = true;
        }
        ui.separator();
        ui.label("Window mode: not all will work");
        ui.radio_value(
            &mut window_settings.window_mode,
            WindowMode::Windowed,
            "Windowed",
        );
        ui.radio_value(
            &mut window_settings.window_mode,
            WindowMode::BorderlessFullscreen,
            "Borderless Fullscreen",
        );
        ui.radio_value(
            &mut window_settings.window_mode,
            WindowMode::SizedFullscreen,
            "Sized Fullscreen",
        );
        ui.radio_value(
            &mut window_settings.window_mode,
            WindowMode::Fullscreen,
            "Fullscreen",
        );

        #[cfg(target_os = "linux")]
        {
            ui.separator();
            ui.label("Display server protocol");
            ui.radio_value(
                &mut window_settings.display_server_protocol,
                LinuxWindow::Auto,
                "Automatic",
            );
            ui.radio_value(
                &mut window_settings.display_server_protocol,
                LinuxWindow::Xorg,
                "Xorg",
            );
            ui.radio_value(
                &mut window_settings.display_server_protocol,
                LinuxWindow::Wayland,
                "Wayland",
            );
        }

        if !invalid && old_settings != **window_settings {
            window_settings.save().unwrap();
        }
    }
}

pub fn window_settings_asy(mut actions: EventReader<Action>, mut state: ResMut<PanelDockState>) {
    for event in actions.read() {
        if matches!(event.downcast_ref(), Some(OpenWindowSettingsAct))
            && !state
                .state
                .iter_all_tabs()
                .any(|(_, a)| a.title() == WindowSettingsEditor.title())
        {
            state.state.add_window(vec![WindowSettingsEditor.into()]);
        }
    }
}
