use std::sync::{Arc, Mutex};

use bevy::{prelude::*, window::WindowMode};
use bevy_egui::{egui, egui::Color32};

#[cfg(target_os = "linux")]
use crate::window_settings::settings::LinuxWindow;
use crate::{
    misc::{data_path, Action},
    ui::popup::Popup,
    window_settings::settings::WindowSettings,
};

#[allow(dead_code)]
pub enum WindowSettingsAct {
    Open,
    Update(WindowSettings),
}

pub fn window_settings_msy(
    mut actions: EventReader<Action>,
    mut popup: EventWriter<Arc<Popup>>,
    mut window_settings: ResMut<WindowSettings>,
) {
    for event in actions.iter() {
        if matches!(event.downcast_ref(), Some(WindowSettingsAct::Open)) {
            popup.send(Popup::new(
                "window_settings_win",
                || {
                    egui::Window::new("Window Settings")
                        .resizable(true)
                        .collapsible(true)
                },
                |state, ui, ew, shown| {
                    let mut state = state.lock().unwrap();
                    let mut invalid = false;
                    let window_settings: &mut WindowSettings = state.downcast_mut().unwrap();
                    if ui.add_enabled(*window_settings != WindowSettings::default(), egui::Button::new("Reset")).clicked() {
                        *window_settings = WindowSettings::default();
                    }
                    ui.colored_label(Color32::YELLOW, format!("Window settings can also be edited at: {}", data_path("window_settings.toml").to_string_lossy()));
                    ui.label("Changes will come into affect on the next launch of Stencil2");
                    ui.label("If Stencil2 crashes the next time after changing anything here, you will have to edit the TOML file manually, good luck :)");
                    ui.separator();
                    ui.label("Enabled backends: Stencil2 will use one of these to render the window");
                    ui.checkbox(&mut window_settings.backends.vulkan, "Vulkan");
                    ui.checkbox(&mut window_settings.backends.metal, "Metal");
                    ui.checkbox(&mut window_settings.backends.dx12, "DX12");
                    ui.checkbox(&mut window_settings.backends.dx11, "DX11");
                    if window_settings.backends.is_none() {
                        ui.colored_label(Color32::RED, format!("Select at least one backend!"));
                        invalid = true;
                    }
                    ui.separator();
                    ui.label("Window mode: not all will work");
                    ui.radio_value(&mut window_settings.window_mode, WindowMode::Windowed, "Windowed");
                    ui.radio_value(&mut window_settings.window_mode, WindowMode::BorderlessFullscreen, "Borderless Fullscreen");
                    ui.radio_value(&mut window_settings.window_mode, WindowMode::SizedFullscreen, "Sized Fullscreen");
                    ui.radio_value(&mut window_settings.window_mode, WindowMode::Fullscreen, "Fullscreen");
                    ui.separator();
                    #[cfg(target_os = "linux")]
                    {
                        ui.label("Display server protocol");
                        ui.radio_value(&mut window_settings.display_server_protocol, LinuxWindow::Auto, "Automatic");
                        ui.radio_value(&mut window_settings.display_server_protocol, LinuxWindow::Xorg, "Xorg");
                        ui.radio_value(&mut window_settings.display_server_protocol, LinuxWindow::Wayland, "Wayland");
                        ui.separator();
                    }

                    if ui.add_enabled(!invalid, egui::Button::new("Save")).clicked() {
                        ew.send(Box::new(WindowSettingsAct::Update(window_settings.to_owned())));
                        *shown = false;
                    }
                    if ui.button("Cancel").clicked() {
                        *shown = false;
                    }
                },
                Mutex::new(Box::new(window_settings.to_owned())),
            ));
        } else if let Some(WindowSettingsAct::Update(new_settings)) = event.downcast_ref() {
            *window_settings = new_settings.to_owned();
            new_settings.save().unwrap();
        }
    }
}
