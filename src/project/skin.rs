use std::{collections::HashMap, sync::Arc};

use base64::engine::general_purpose::STANDARD;
use base64_serde::base64_serde_type;
use serde::{Deserialize, Deserializer, Serialize, Serializer, de::Unexpected};

base64_serde_type!(Base64Standard, STANDARD);

fn serialise_option_color32<S: Serializer>(
    c: &Option<egui::Color32>,
    ser: S,
) -> Result<S::Ok, S::Error> {
    c.map(|c| c.to_hex()).serialize(ser)
}
fn serialise_color32<S: Serializer>(c: &egui::Color32, ser: S) -> Result<S::Ok, S::Error> {
    c.to_hex().serialize(ser)
}
fn deserialise_option_color32<'de, D: Deserializer<'de>>(
    de: D,
) -> Result<Option<egui::Color32>, D::Error> {
    let Some(s) = Option::<&str>::deserialize(de)? else {
        return Ok(None);
    };
    match egui::Color32::from_hex(s) {
        Ok(c) => Ok(Some(c)),
        Err(_) => Err(<D::Error as serde::de::Error>::invalid_value(
            Unexpected::Str(s),
            &"valid hex",
        )),
    }
}
fn deserialise_color32<'de, D: Deserializer<'de>>(de: D) -> Result<egui::Color32, D::Error> {
    let s = <&str>::deserialize(de)?;

    match egui::Color32::from_hex(s) {
        Ok(c) => Ok(c),
        Err(_) => Err(<D::Error as serde::de::Error>::invalid_value(
            Unexpected::Str(s),
            &"valid hex",
        )),
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(tag = "ty")]
pub enum AreaStyle {
    #[serde(rename = "areaFill")]
    Fill {
        zoom_multiplier: f32,
        #[serde(
            serialize_with = "serialise_option_color32",
            deserialize_with = "deserialise_option_color32"
        )]
        colour: Option<egui::Color32>,
        #[serde(
            serialize_with = "serialise_option_color32",
            deserialize_with = "deserialise_option_color32"
        )]
        outline: Option<egui::Color32>,
        outline_width: f32,
    },
    #[serde(rename = "areaCentreText")]
    CenterText {
        zoom_multiplier: f32,
        #[serde(
            serialize_with = "serialise_option_color32",
            deserialize_with = "deserialise_option_color32"
        )]
        colour: Option<egui::Color32>,
        offset: egui::Vec2,
        size: f32,
    },
    #[serde(rename = "areaBorderText")]
    BorderText {
        zoom_multiplier: f32,
        #[serde(
            serialize_with = "serialise_option_color32",
            deserialize_with = "deserialise_option_color32"
        )]
        colour: Option<egui::Color32>,
        offset: f32,
        size: f32,
    },
    #[serde(rename = "areaCentreImage")]
    CentreImage {
        zoom_multiplier: f32,
        #[serde(with = "Base64Standard")]
        image: Vec<u8>,
        extension: String,
        size: egui::Vec2,
        offset: egui::Vec2,
    },
}
#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(tag = "ty")]
pub enum LineStyle {
    #[serde(rename = "lineFore")]
    Fore {
        zoom_multiplier: f32,
        #[serde(
            serialize_with = "serialise_option_color32",
            deserialize_with = "deserialise_option_color32"
        )]
        colour: Option<egui::Color32>,
        width: f32,
        dash: Option<Vec<f32>>,
        unrounded: bool,
    },
    #[serde(rename = "lineBack")]
    Back {
        zoom_multiplier: f32,
        #[serde(
            serialize_with = "serialise_option_color32",
            deserialize_with = "deserialise_option_color32"
        )]
        colour: Option<egui::Color32>,
        width: f32,
        dash: Option<Vec<f32>>,
        unrounded: bool,
    },
    #[serde(rename = "lineText")]
    Text {
        zoom_multiplier: f32,
        #[serde(
            serialize_with = "serialise_option_color32",
            deserialize_with = "deserialise_option_color32"
        )]
        colour: Option<egui::Color32>,
        #[serde(
            serialize_with = "serialise_option_color32",
            deserialize_with = "deserialise_option_color32"
        )]
        arrow_colour: Option<egui::Color32>,
        size: f32,
        offset: f32,
    },
}
#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(tag = "ty")]
pub enum PointStyle {
    #[serde(rename = "pointImage")]
    Image {
        zoom_multiplier: f32,
        #[serde(with = "Base64Standard")]
        image: Vec<u8>,
        extension: String,
        size: egui::Vec2,
        offset: egui::Vec2,
    },
    #[serde(rename = "pointSquare")]
    Square {
        zoom_multiplier: f32,
        #[serde(
            serialize_with = "serialise_option_color32",
            deserialize_with = "deserialise_option_color32"
        )]
        colour: Option<egui::Color32>,
        border_radius: f32,
        size: f32,
        width: f32,
    },
    #[serde(rename = "pointText")]
    Text {
        zoom_multiplier: f32,
        #[serde(
            serialize_with = "serialise_option_color32",
            deserialize_with = "deserialise_option_color32"
        )]
        colour: Option<egui::Color32>,
        size: f32,
        offset: egui::Vec2,
        anchor: String,
    },
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(tag = "shape")]
pub enum SkinComponent {
    #[serde(rename = "point")]
    Point {
        name: String,
        tags: Vec<String>,
        styles: HashMap<String, Vec<PointStyle>>,
    },
    #[serde(rename = "line")]
    Line {
        name: String,
        tags: Vec<String>,
        styles: HashMap<String, Vec<LineStyle>>,
    },
    #[serde(rename = "area")]
    Area {
        name: String,
        tags: Vec<String>,
        styles: HashMap<String, Vec<AreaStyle>>,
    },
}
impl SkinComponent {
    #[must_use]
    pub const fn name(&self) -> &String {
        match self {
            Self::Point { name, .. } | Self::Line { name, .. } | Self::Area { name, .. } => name,
        }
    }
    #[must_use]
    pub const fn tags(&self) -> &Vec<String> {
        match self {
            Self::Point { tags, .. } | Self::Line { tags, .. } | Self::Area { tags, .. } => tags,
        }
    }
    fn style_in_max_zoom<T>(style: &HashMap<String, Vec<T>>) -> Option<&Vec<T>> {
        Some(
            style
                .iter()
                .map(|(zl, v)| (zl.split('-').next().unwrap().parse::<u8>().unwrap(), v))
                .find(|(min, _)| *min == 0)?
                .1,
        )
    }

    #[must_use]
    pub fn front_colour(&self) -> Option<egui::Color32> {
        match self {
            Self::Point { styles, .. } => Self::style_in_max_zoom(styles)?
                .iter()
                .filter_map(|style| match style {
                    PointStyle::Square { colour, .. } => *colour,
                    _ => None,
                })
                .next_back(),
            Self::Line { styles, .. } => Self::style_in_max_zoom(styles)?
                .iter()
                .filter_map(|style| match style {
                    LineStyle::Fore { colour, .. } => *colour,
                    _ => None,
                })
                .next_back(),
            Self::Area { styles, .. } => Self::style_in_max_zoom(styles)?
                .iter()
                .filter_map(|style| match style {
                    AreaStyle::Fill { colour, .. } => *colour,
                    _ => None,
                })
                .next_back(),
        }
    }
    #[must_use]
    pub fn back_colour(&self) -> Option<egui::Color32> {
        match self {
            Self::Point { .. } => None,
            Self::Line { styles, .. } => Self::style_in_max_zoom(styles)?
                .iter()
                .filter_map(|style| match style {
                    LineStyle::Back { colour, .. } => *colour,
                    _ => None,
                })
                .next_back(),
            Self::Area { styles, .. } => Self::style_in_max_zoom(styles)?
                .iter()
                .filter_map(|style| match style {
                    AreaStyle::Fill { outline, .. } => *outline,
                    _ => None,
                })
                .next_back(),
        }
    }
    #[must_use]
    pub fn text_colour(&self) -> Option<egui::Color32> {
        match self {
            Self::Point { styles, .. } => Self::style_in_max_zoom(styles)?
                .iter()
                .filter_map(|style| match style {
                    PointStyle::Text { colour, .. } => *colour,
                    _ => None,
                })
                .next_back(),
            Self::Line { styles, .. } => Self::style_in_max_zoom(styles)?
                .iter()
                .filter_map(|style| match style {
                    LineStyle::Text { colour, .. } => *colour,
                    _ => None,
                })
                .next_back(),
            Self::Area { styles, .. } => Self::style_in_max_zoom(styles)?
                .iter()
                .filter_map(|style| match style {
                    AreaStyle::CenterText { colour, .. } => *colour,
                    _ => None,
                })
                .next_back(),
        }
    }
    #[must_use]
    pub fn weight(&self) -> Option<f32> {
        match self {
            Self::Point { .. } => None,
            Self::Line { styles, .. } => Self::style_in_max_zoom(styles)?
                .iter()
                .filter_map(|style| match style {
                    LineStyle::Fore { width, .. } => Some(*width),
                    _ => None,
                })
                .next_back(),
            Self::Area { styles, .. } => Self::style_in_max_zoom(styles)?
                .iter()
                .filter_map(|style| match style {
                    AreaStyle::Fill { outline_width, .. } => Some(outline_width * 5.0),
                    _ => None,
                })
                .next_back(),
        }
    }

    #[must_use]
    pub fn widget_text(
        &self,
        ui: &mut egui::Ui,
        text_style: &egui::TextStyle,
    ) -> impl Into<egui::WidgetText> + use<> {
        let font_id = &ui.style().text_styles[text_style];
        let mut label = egui::text::LayoutJob::default();
        let space = if let Some(color) = self.front_colour() {
            label.append(
                "◼",
                0.0,
                egui::TextFormat {
                    font_id: font_id.to_owned(),
                    color,
                    ..Default::default()
                },
            );
            font_id.size / 4.0
        } else if let Some(color) = self.back_colour() {
            label.append(
                "□",
                0.0,
                egui::TextFormat {
                    font_id: font_id.to_owned(),
                    color,
                    ..Default::default()
                },
            );
            font_id.size / 4.0
        } else {
            0.0
        };
        label.append(
            self.name(),
            space,
            egui::TextFormat {
                font_id: font_id.to_owned(),
                ..Default::default()
            },
        );
        label
    }
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Skin {
    pub version: u8,
    pub name: String,
    pub types: Vec<Arc<SkinComponent>>,
    pub font_files: Vec<(String, String)>,
    pub font_string: String,
    #[serde(
        serialize_with = "serialise_color32",
        deserialize_with = "deserialise_color32"
    )]
    pub background: egui::Color32,
    pub prune_small_text: Option<f32>,
    pub licence: String,
}

impl Skin {
    #[must_use]
    pub fn get_type(&self, ty: &str) -> Option<&Arc<SkinComponent>> {
        self.types.iter().find(|a| a.name() == ty)
    }
    #[must_use]
    pub fn show_type(
        &self,
        ty: &str,
        ui: &mut egui::Ui,
        text_style: &egui::TextStyle,
    ) -> impl Into<egui::WidgetText> + use<> {
        self.get_type(ty).map_or_else(
            || egui::WidgetText::from(ty),
            |a| a.widget_text(ui, text_style).into(),
        )
    }
    #[must_use]
    pub fn get_order(&self, ty: &str) -> Option<usize> {
        self.types.iter().position(|a| a.name() == ty)
    }
}
