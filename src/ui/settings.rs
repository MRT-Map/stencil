use std::{any::Any, borrow::Cow, path::PathBuf, sync::Arc};

use eframe::{egui_glow, egui_wgpu};
use etcetera::AppStrategy;
use eyre::eyre;
use itertools::Itertools;
use rfd::FileDialog;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_with::{DeserializeAs, SerializeAs};

use crate::{
    impl_load_save, settings,
    settings::{Settings, settings_ui_field, settings_ui_field_no_display},
    utils::{file::FOLDERS, warnings::ResultExt},
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
#[serde(remote = "egui_glow::ShaderVersion")]
pub enum ShaderVersion {
    Gl120,
    Gl140,
    Es100,
    Es300,
}
impl SerializeAs<egui_glow::ShaderVersion> for ShaderVersion {
    fn serialize_as<S>(source: &egui_glow::ShaderVersion, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        Self::serialize(source, serializer)
    }
}
impl<'de> DeserializeAs<'de, egui_glow::ShaderVersion> for ShaderVersion {
    fn deserialize_as<D>(deserializer: D) -> Result<egui_glow::ShaderVersion, D::Error>
    where
        D: Deserializer<'de>,
    {
        Self::deserialize(deserializer)
    }
}
serde_with::serde_conv!(
    InstanceFlags,
    wgpu_types::InstanceFlags,
    |flags: &wgpu_types::InstanceFlags| flags.bits(),
    |bits: u32| -> eyre::Result<_> {
        wgpu_types::InstanceFlags::from_bits(bits).ok_or_else(|| eyre!("Unknown bits in {bits:b}"))
    }
);

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AdditionalFontPriority {
    Highest,
    Lowest,
}
const fn afp_string(value: Option<AdditionalFontPriority>) -> &'static str {
    match value {
        None => "Off",
        Some(AdditionalFontPriority::Highest) => "Highest",
        Some(AdditionalFontPriority::Lowest) => "Lowest",
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdditionalFont {
    path: PathBuf,
    proportional: Option<AdditionalFontPriority>,
    monospace: Option<AdditionalFontPriority>,
}
impl AdditionalFont {
    pub fn name(&self) -> Cow<'_, str> {
        self.path
            .file_stem()
            .or_else(|| self.path.file_name())
            .unwrap_or_default()
            .to_string_lossy()
    }
}

settings! {
    #[serde_with::serde_as] #[derive(Eq)] UiSettings {
        additional_fonts: Vec<AdditionalFont> = Vec::new(),

        multisampling: u16 = eframe::NativeOptions::default().multisampling,
        #[serde(with = "HardwareAcceleration")] hardware_acceleration: eframe::HardwareAcceleration = eframe::NativeOptions::default().hardware_acceleration,
        #[serde(with = "Renderer")] renderer: eframe::Renderer = eframe::NativeOptions::default().renderer,
        centered: bool = eframe::NativeOptions::default().centered,
        persist_window: bool = eframe::NativeOptions::default().persist_window,
        dithering: bool = eframe::NativeOptions::default().dithering,

        glow_vsync: bool = eframe::NativeOptions::default().vsync,
        #[serde_as(as = "Option<ShaderVersion>")] glow_shader_version: Option<egui_glow::ShaderVersion> = eframe::NativeOptions::default().shader_version,

        wgpu_present_mode: wgpu_types::PresentMode = egui_wgpu::WgpuConfiguration::default().present_mode,
        wgpu_desired_maximum_frame_latency: Option<u32> = egui_wgpu::WgpuConfiguration::default().desired_maximum_frame_latency,
        wgpu_backends: wgpu_types::Backends = egui_wgpu::WgpuSetupCreateNew::without_display_handle().instance_descriptor.backends,
        #[serde_as(as = "InstanceFlags")] wgpu_flags: wgpu_types::InstanceFlags = egui_wgpu::WgpuSetupCreateNew::without_display_handle().instance_descriptor.flags,
        wgpu_power_preference: wgpu_types::PowerPreference = egui_wgpu::WgpuSetupCreateNew::without_display_handle().power_preference,
    }
}
impl_load_save!(toml UiSettings, FOLDERS.in_config_dir("ui.toml"), "# Documentation is at https://mrt-map.github.io/stencil3/doc/UI-Settings.html");

impl Settings for UiSettings {
    fn ui_inner(&mut self, ui: &mut egui::Ui, _tab_state: &mut dyn Any) {
        ui.heading("Egui Settings");
        ui.ctx().clone().settings_ui(ui);

        ui.separator();
        ui.label("Custom Fonts");

        self.custom_fonts_ui(ui);

        ui.separator();

        ui.heading("Eframe Settings");
        ui.colored_label(egui::Color32::YELLOW, "Restart to see changes. May cause crashes, please be careful! If Stencil3 is unable to startup due to any of the below settings, you will have to edit the configuration manually at the path above.");

        self.eframe_settings_ui(ui);
    }
}

impl UiSettings {
    #[must_use]
    pub fn get_native_options(&self) -> eframe::NativeOptions {
        eframe::NativeOptions {
            vsync: self.glow_vsync,
            multisampling: self.multisampling,
            hardware_acceleration: self.hardware_acceleration,
            renderer: self.renderer,
            shader_version: self.glow_shader_version,
            centered: self.centered,
            persist_window: self.persist_window,
            dithering: self.dithering,
            wgpu_options: egui_wgpu::WgpuConfiguration {
                present_mode: self.wgpu_present_mode,
                desired_maximum_frame_latency: self.wgpu_desired_maximum_frame_latency,
                wgpu_setup: egui_wgpu::WgpuSetup::CreateNew(egui_wgpu::WgpuSetupCreateNew {
                    instance_descriptor: wgpu_types::InstanceDescriptor {
                        backends: self.wgpu_backends,
                        flags: self.wgpu_flags,
                        ..egui_wgpu::WgpuSetupCreateNew::without_display_handle()
                            .instance_descriptor
                    },
                    power_preference: self.wgpu_power_preference,
                    ..egui_wgpu::WgpuSetupCreateNew::without_display_handle()
                }),
                ..Default::default()
            },
            ..Default::default()
        }
    }
    pub fn reload_fonts(&self, ctx: &egui::Context) {
        let mut fd = egui::FontDefinitions::default();
        for font in &self.additional_fonts {
            let Ok(data) = std::fs::read(&font.path)
                .notify(format!("Unable to read font file {}", font.path.display()))
            else {
                continue;
            };
            fd.font_data.insert(
                font.name().into_owned(),
                Arc::new(egui::FontData::from_owned(data)),
            );
            for (family, priority) in [
                (egui::FontFamily::Proportional, font.proportional),
                (egui::FontFamily::Monospace, font.monospace),
            ] {
                match priority {
                    Some(AdditionalFontPriority::Highest) => fd
                        .families
                        .entry(family)
                        .or_default()
                        .insert(0, font.name().into_owned()),
                    Some(AdditionalFontPriority::Lowest) => fd
                        .families
                        .entry(family)
                        .or_default()
                        .push(font.name().into_owned()),
                    None => (),
                }
            }
        }
        ctx.set_fonts(fd);
    }

    fn custom_fonts_ui(&mut self, ui: &mut egui::Ui) {
        let mut to_remove = None;
        let mut to_swap = None;
        let length = self.additional_fonts.len();

        for (i, font) in self.additional_fonts.iter_mut().enumerate() {
            ui.label(font.name());
            ui.small(font.path.to_string_lossy());
            ui.horizontal(|ui| {
                egui::ComboBox::new(format!("prop-{i}"), "Proportional")
                    .selected_text(afp_string(font.proportional))
                    .show_ui(ui, |ui| {
                        for option in [
                            None,
                            Some(AdditionalFontPriority::Highest),
                            Some(AdditionalFontPriority::Lowest),
                        ] {
                            ui.selectable_value(&mut font.proportional, option, afp_string(option));
                        }
                    });
                egui::ComboBox::new(format!("mono-{i}"), "Monospace")
                    .selected_text(afp_string(font.monospace))
                    .show_ui(ui, |ui| {
                        for option in [
                            None,
                            Some(AdditionalFontPriority::Highest),
                            Some(AdditionalFontPriority::Lowest),
                        ] {
                            ui.selectable_value(&mut font.monospace, option, afp_string(option));
                        }
                    });

                if ui.add_enabled(i != 0, egui::Button::new("⬆")).clicked() {
                    to_swap = Some((i, i - 1));
                }
                if ui
                    .add_enabled(i != length - 1, egui::Button::new("⬇"))
                    .clicked()
                {
                    to_swap = Some((i, i + 1));
                }
                if ui
                    .add(egui::Button::new("❌").fill(egui::Color32::DARK_RED))
                    .clicked()
                {
                    to_remove = Some(i);
                }
            });
        }

        if let Some(to_remove) = to_remove {
            self.additional_fonts.remove(to_remove);
        }
        if let Some((a, b)) = to_swap {
            self.additional_fonts.swap(a, b);
        }

        ui.horizontal(|ui| {
            if ui.button("➕ Custom Font").clicked() {
                let Some(files) = FileDialog::new()
                    .set_title("Import Custom Font")
                    .add_filter("Font File", &["ttf", "otf"])
                    .pick_files()
                else {
                    return;
                };
                for file in files {
                    self.additional_fonts.push(AdditionalFont {
                        path: file,
                        proportional: Some(AdditionalFontPriority::Highest),
                        monospace: None,
                    });
                }
            }

            if ui.button("⟳ Reload Fonts").clicked() {
                self.reload_fonts(ui);
            }
        });

        ui.label("Settings for each font located above under \"Font Tweaks\"");
    }

    #[expect(clippy::too_many_lines)]
    fn eframe_settings_ui(&mut self, ui: &mut egui::Ui) {
        let default = Self::default();

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

        ui.separator();

        if self.renderer == eframe::Renderer::Glow {
            ui.label("Glow Settings");
            settings_ui_field(
                ui,
                &mut self.glow_vsync,
                default.glow_vsync,
                Some("Limit the FPS to the display refresh rate"),
                |ui, value| {
                    ui.checkbox(value, "Vsync");
                },
            );

            #[expect(clippy::items_after_statements)]
            const fn shader_string(v: Option<egui_glow::ShaderVersion>) -> &'static str {
                match v {
                    None => "Default",
                    Some(egui_glow::ShaderVersion::Gl120) => "GL120",
                    Some(egui_glow::ShaderVersion::Gl140) => "GL140",
                    Some(egui_glow::ShaderVersion::Es100) => "ES100",
                    Some(egui_glow::ShaderVersion::Es300) => "ES300",
                }
            }

            settings_ui_field_no_display(
                ui,
                &mut self.glow_shader_version,
                default.glow_shader_version,
                shader_string(default.glow_shader_version),
                Option::<egui::WidgetText>::None,
                |ui, value| {
                    egui::ComboBox::from_label("Shader Version")
                        .selected_text(shader_string(*value))
                        .show_ui(ui, |ui| {
                            for option in [
                                None,
                                Some(egui_glow::ShaderVersion::Gl120),
                                Some(egui_glow::ShaderVersion::Gl140),
                                Some(egui_glow::ShaderVersion::Es100),
                                Some(egui_glow::ShaderVersion::Es300),
                            ] {
                                ui.selectable_value(value, option, shader_string(option));
                            }
                        });
                },
            );
        } else {
            ui.label("Wgpu Settings");
            #[expect(clippy::items_after_statements)]
            const fn present_mode_string(v: wgpu_types::PresentMode) -> &'static str {
                match v {
                    wgpu_types::PresentMode::AutoVsync => "Auto Vsync",
                    wgpu_types::PresentMode::AutoNoVsync => "Auto No Vsync",
                    wgpu_types::PresentMode::Fifo => "Vsync On (FIFO)",
                    wgpu_types::PresentMode::FifoRelaxed => "Adaptive Vsync (FIFO Relaxed)",
                    wgpu_types::PresentMode::Immediate => "Vsync Off (Immediate)",
                    wgpu_types::PresentMode::Mailbox => "Fast Vsync (Mailbox)",
                }
            }

            settings_ui_field_no_display(
                ui,
                &mut self.wgpu_present_mode,
                default.wgpu_present_mode,
                present_mode_string(default.wgpu_present_mode),
                Option::<egui::WidgetText>::None,
                |ui, value| {
                    egui::ComboBox::from_label("Present Mode")
                        .selected_text(present_mode_string(*value))
                        .show_ui(ui, |ui| {
                            for option in [
                                wgpu_types::PresentMode::AutoVsync,
                                wgpu_types::PresentMode::AutoNoVsync,
                                wgpu_types::PresentMode::Fifo,
                                wgpu_types::PresentMode::FifoRelaxed,
                                wgpu_types::PresentMode::Immediate,
                                wgpu_types::PresentMode::Mailbox,
                            ] {
                                ui.selectable_value(value, option, present_mode_string(option));
                            }
                        });
                },
            );

            #[expect(clippy::items_after_statements)]
            fn dmfl_string(v: Option<u32>) -> Cow<'static, str> {
                match v {
                    None => "Default".into(),
                    Some(1) => "Low Latency".into(),
                    Some(2) => "High Throughput".into(),
                    Some(a) => a.to_string().into(),
                }
            }

            settings_ui_field_no_display(
                ui,
                &mut self.wgpu_desired_maximum_frame_latency,
                default.wgpu_desired_maximum_frame_latency,
                dmfl_string(default.wgpu_desired_maximum_frame_latency),
                Option::<egui::WidgetText>::None,
                |ui, value| {
                    egui::ComboBox::from_label("Desired Maximum Frame Latency")
                        .selected_text(dmfl_string(*value))
                        .show_ui(ui, |ui| {
                            for option in [None, Some(1), Some(2)] {
                                ui.selectable_value(value, option, dmfl_string(option));
                            }
                        });
                },
            );

            settings_ui_field_no_display(
                ui,
                &mut self.wgpu_backends,
                default.wgpu_backends,
                default
                    .wgpu_backends
                    .iter_names()
                    .map(|(a, _)| a)
                    .join(", "),
                Option::<egui::WidgetText>::None,
                |ui, value| {
                    ui.style_mut().override_text_style = Some(egui::TextStyle::Small);
                    for (option, string) in [
                        (wgpu_types::Backends::VULKAN, "Vulkan"),
                        (wgpu_types::Backends::GL, "GL"),
                        (wgpu_types::Backends::METAL, "Metal"),
                        (wgpu_types::Backends::DX12, "DX12"),
                        (wgpu_types::Backends::BROWSER_WEBGPU, "Browser WebGPU"),
                    ] {
                        if ui
                            .selectable_label(value.contains(option), string)
                            .clicked()
                        {
                            value.toggle(option);
                        }
                    }
                    ui.label("Backends");
                },
            );

            settings_ui_field_no_display(
                ui,
                &mut self.wgpu_flags,
                default.wgpu_flags,
                default.wgpu_flags.iter_names().map(|(a, _)| a).join(", "),
                Option::<egui::WidgetText>::None,
                |ui, value| {
                    ui.style_mut().override_text_style = Some(egui::TextStyle::Small);
                    for (option, string) in [
                        (wgpu_types::InstanceFlags::DEBUG, "Debug"),
                        (wgpu_types::InstanceFlags::VALIDATION, "Validation"),
                        (
                            wgpu_types::InstanceFlags::DISCARD_HAL_LABELS,
                            "Discard HAL Labels",
                        ),
                        (
                            wgpu_types::InstanceFlags::ALLOW_UNDERLYING_NONCOMPLIANT_ADAPTER,
                            "Allow Underlying Noncompliant Adapter",
                        ),
                        (
                            wgpu_types::InstanceFlags::GPU_BASED_VALIDATION,
                            "GPU-Based Validation",
                        ),
                        (
                            wgpu_types::InstanceFlags::VALIDATION_INDIRECT_CALL,
                            "Validation Indirect Call",
                        ),
                        (
                            wgpu_types::InstanceFlags::AUTOMATIC_TIMESTAMP_NORMALIZATION,
                            "Automatic Timestamp Normalisation",
                        ),
                    ] {
                        if ui.checkbox(&mut value.contains(option), string).clicked() {
                            value.toggle(option);
                        }
                    }
                    ui.label("Flags");
                },
            );

            #[expect(clippy::items_after_statements)]
            const fn pp_string(v: wgpu_types::PowerPreference) -> &'static str {
                match v {
                    wgpu_types::PowerPreference::None => "None",
                    wgpu_types::PowerPreference::LowPower => "Low Power",
                    wgpu_types::PowerPreference::HighPerformance => "High Performance",
                }
            }

            settings_ui_field_no_display(
                ui,
                &mut self.wgpu_power_preference,
                default.wgpu_power_preference,
                pp_string(default.wgpu_power_preference),
                Option::<egui::WidgetText>::None,
                |ui, value| {
                    for option in [
                        wgpu_types::PowerPreference::None,
                        wgpu_types::PowerPreference::LowPower,
                        wgpu_types::PowerPreference::HighPerformance,
                    ] {
                        ui.selectable_value(value, option, pp_string(option));
                    }
                    ui.label("Power Preference");
                },
            );
        }
    }
}
