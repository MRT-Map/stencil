use std::any::Any;

use egui::Ui;
use etcetera::AppStrategy;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_with::{DeserializeAs, SerializeAs};

use crate::{
    impl_load_save, settings,
    settings::{Settings, settings_ui_field, settings_ui_field_no_display},
    utils::file::FOLDERS,
};

#[derive(Deserialize, Serialize)]
#[serde(remote = "eframe::HardwareAcceleration")]
enum HardwareAcceleration {
    Required,
    Preferred,
    Off,
}
#[derive(Deserialize, Serialize)]
#[serde(remote = "eframe::Renderer")]
enum Renderer {
    Glow,
    Wgpu,
}
#[derive(Deserialize, Serialize)]
#[serde(remote = "eframe::egui_glow::ShaderVersion")]
pub enum ShaderVersion {
    Gl120,
    Gl140,
    Es100,
    Es300,
}
impl SerializeAs<eframe::egui_glow::ShaderVersion> for ShaderVersion {
    fn serialize_as<S>(
        source: &eframe::egui_glow::ShaderVersion,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        Self::serialize(source, serializer)
    }
}
impl<'de> DeserializeAs<'de, eframe::egui_glow::ShaderVersion> for ShaderVersion {
    fn deserialize_as<D>(deserializer: D) -> Result<eframe::egui_glow::ShaderVersion, D::Error>
    where
        D: Deserializer<'de>,
    {
        Self::deserialize(deserializer)
    }
}

settings! {
    #[serde_with::serde_as] #[derive(Eq)] UiSettings {
        vsync: bool = eframe::NativeOptions::default().vsync,
        multisampling: u16 = eframe::NativeOptions::default().multisampling,
        #[serde(with = "HardwareAcceleration")] hardware_acceleration: eframe::HardwareAcceleration = eframe::NativeOptions::default().hardware_acceleration,
        #[serde(with = "Renderer")] renderer: eframe::Renderer = eframe::NativeOptions::default().renderer,
        #[serde_as(as = "Option<ShaderVersion>")] shader_version: Option<eframe::egui_glow::ShaderVersion> = eframe::NativeOptions::default().shader_version,
        centered: bool = eframe::NativeOptions::default().centered,
        persist_window: bool = eframe::NativeOptions::default().persist_window,
        dithering: bool = eframe::NativeOptions::default().dithering,
    }
}
impl_load_save!(toml UiSettings, FOLDERS.in_config_dir("ui.toml"), "# Documentation is at https://mrt-map.github.io/stencil3/doc/UI-Settings.html");

impl Settings for UiSettings {
    #[expect(clippy::too_many_lines)]
    fn ui_inner(&mut self, ui: &mut Ui, _tab_state: &mut dyn Any) {
        ui.heading("Egui Settings");
        ui.ctx().clone().settings_ui(ui);

        ui.separator();
        let default = Self::default();
        ui.heading("Eframe Settings");
        ui.colored_label(egui::Color32::YELLOW, "Restart to see changes");

        settings_ui_field_no_display(
            ui,
            &mut self.renderer,
            default.renderer,
            r_string(default.renderer),
            Option::<egui::WidgetText>::None,
            |ui, value| {
                for option in [eframe::Renderer::Wgpu, eframe::Renderer::Glow] {
                    ui.selectable_value(value, option, r_string(option));
                }
                ui.label("Rendering Backend");
            },
        );

        settings_ui_field(
            ui,
            &mut self.multisampling,
            default.multisampling,
            Some("Level of multisampling anti-aliasing (MSAA)"),
            |ui, value| {
                egui::ComboBox::from_label("Multisampling")
                    .selected_text(if *value == 0 {
                        "Off".into()
                    } else {
                        value.to_string()
                    })
                    .show_ui(ui, |ui| {
                        ui.selectable_value(value, 0, "Off");
                        for option in (0..16).map(|n| 2u16.pow(n)) {
                            ui.selectable_value(value, option, option.to_string());
                        }
                    });
            },
        );

        #[expect(clippy::items_after_statements)]
        const fn hw_string(v: eframe::HardwareAcceleration) -> &'static str {
            match v {
                eframe::HardwareAcceleration::Off => "Off",
                eframe::HardwareAcceleration::Required => "Required",
                eframe::HardwareAcceleration::Preferred => "Preferred",
            }
        }

        settings_ui_field_no_display(
            ui,
            &mut self.hardware_acceleration,
            default.hardware_acceleration,
            hw_string(default.hardware_acceleration),
            Option::<egui::WidgetText>::None,
            |ui, value| {
                for option in [
                    eframe::HardwareAcceleration::Preferred,
                    eframe::HardwareAcceleration::Required,
                    eframe::HardwareAcceleration::Off,
                ] {
                    ui.selectable_value(value, option, hw_string(option));
                }
                ui.label("Hardware Acceleration");
            },
        );

        #[expect(clippy::items_after_statements)]
        const fn r_string(v: eframe::Renderer) -> &'static str {
            match v {
                eframe::Renderer::Wgpu => "Wgpu",
                eframe::Renderer::Glow => "Glow",
            }
        }

        #[expect(clippy::items_after_statements)]
        const fn shader_string(v: Option<eframe::egui_glow::ShaderVersion>) -> &'static str {
            match v {
                None => "Default",
                Some(eframe::egui_glow::ShaderVersion::Gl120) => "GL120",
                Some(eframe::egui_glow::ShaderVersion::Gl140) => "GL140",
                Some(eframe::egui_glow::ShaderVersion::Es100) => "ES100",
                Some(eframe::egui_glow::ShaderVersion::Es300) => "ES300",
            }
        }

        settings_ui_field(
            ui,
            &mut self.centered,
            default.centered,
            Some("Does not work on Wayland"),
            |ui, value| {
                ui.checkbox(value, "Centre window on initialisation");
            },
        );

        settings_ui_field(
            ui,
            &mut self.persist_window,
            default.persist_window,
            Option::<egui::WidgetText>::None,
            |ui, value| {
                ui.checkbox(value, "Persist window position and size");
            },
        );

        settings_ui_field(
            ui,
            &mut self.dithering,
            default.dithering,
            Option::<egui::WidgetText>::None,
            |ui, value| {
                ui.checkbox(value, "Dithering");
            },
        );

        if self.renderer == eframe::Renderer::Glow {
            settings_ui_field_no_display(
                ui,
                &mut self.shader_version,
                default.shader_version,
                shader_string(default.shader_version),
                Option::<egui::WidgetText>::None,
                |ui, value| {
                    egui::ComboBox::from_label("Shader Version. Glow only")
                        .selected_text(shader_string(*value))
                        .show_ui(ui, |ui| {
                            for option in [
                                None,
                                Some(eframe::egui_glow::ShaderVersion::Gl120),
                                Some(eframe::egui_glow::ShaderVersion::Gl140),
                                Some(eframe::egui_glow::ShaderVersion::Es100),
                                Some(eframe::egui_glow::ShaderVersion::Es300),
                            ] {
                                ui.selectable_value(value, option, shader_string(option));
                            }
                        });
                },
            );

            settings_ui_field(
                ui,
                &mut self.vsync,
                default.vsync,
                Some("Limit the FPS to the display refresh rate. Glow only"),
                |ui, value| {
                    ui.checkbox(value, "Vsync");
                },
            );
        }
    }
}

impl UiSettings {
    #[must_use]
    pub fn get_native_options(&self) -> eframe::NativeOptions {
        eframe::NativeOptions {
            vsync: self.vsync,
            multisampling: self.multisampling,
            hardware_acceleration: self.hardware_acceleration,
            renderer: self.renderer,
            shader_version: self.shader_version,
            centered: self.centered,
            persist_window: self.persist_window,
            dithering: self.dithering,
            ..Default::default()
        }
    }
}
