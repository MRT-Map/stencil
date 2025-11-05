use std::{collections::HashMap, sync::Arc};

use base64::engine::general_purpose::STANDARD;
use base64_serde::base64_serde_type;
use duplicate::duplicate_item;
use itertools::Itertools;
use serde::{
    Deserialize, Deserializer, Serialize, Serializer,
    de::{DeserializeOwned, Unexpected},
};

base64_serde_type!(Base64Standard, STANDARD);

#[expect(clippy::ref_option, clippy::trivially_copy_pass_by_ref)]
fn serialise_option_color32<S: Serializer>(
    c: &Option<egui::Color32>,
    ser: S,
) -> Result<S::Ok, S::Error> {
    c.map(|c| c.to_hex()).serialize(ser)
}
#[expect(clippy::trivially_copy_pass_by_ref)]
fn serialise_color32<S: Serializer>(c: &egui::Color32, ser: S) -> Result<S::Ok, S::Error> {
    c.to_hex().serialize(ser)
}
fn deserialise_option_color32<'de, D: Deserializer<'de>>(
    de: D,
) -> Result<Option<egui::Color32>, D::Error> {
    let Some(s) = Option::<&str>::deserialize(de)? else {
        return Ok(None);
    };
    egui::Color32::from_hex(s).map_or_else(
        |_| {
            Err(serde::de::Error::invalid_value(
                Unexpected::Str(s),
                &"valid hex",
            ))
        },
        |c| Ok(Some(c)),
    )
}
fn deserialise_color32<'de, D: Deserializer<'de>>(de: D) -> Result<egui::Color32, D::Error> {
    let s = <&str>::deserialize(de)?;

    egui::Color32::from_hex(s).map_or_else(
        |_| {
            Err(serde::de::Error::invalid_value(
                Unexpected::Str(s),
                &"valid hex",
            ))
        },
        Ok,
    )
}
fn serialise_styles<S: Serializer, T: Serialize>(
    map: &HashMap<(u8, u8), T>,
    ser: S,
) -> Result<S::Ok, S::Error> {
    map.iter()
        .map(|((k1, k2), v)| (format!("{k1}-{k2}"), v))
        .collect::<HashMap<_, _>>()
        .serialize(ser)
}
fn deserialise_styles<'de, D: Deserializer<'de>, T: DeserializeOwned>(
    de: D,
) -> Result<HashMap<(u8, u8), T>, D::Error> {
    HashMap::<String, T>::deserialize(de)?
        .into_iter()
        .map(|(k, v)| {
            Ok((
                {
                    let (k1, k2) = k.split('-').collect_tuple().ok_or_else(|| {
                        serde::de::Error::invalid_type(Unexpected::Str(&k), &"zoom level range")
                    })?;
                    (
                        k1.parse::<u8>().map_err(serde::de::Error::custom)?,
                        if k2.is_empty() {
                            u8::MAX
                        } else {
                            k2.parse::<u8>().map_err(serde::de::Error::custom)?
                        },
                    )
                },
                v,
            ))
        })
        .collect()
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
pub enum SkinType {
    #[serde(rename = "point")]
    Point {
        name: String,
        tags: Vec<String>,
        #[serde(
            serialize_with = "serialise_styles",
            deserialize_with = "deserialise_styles"
        )]
        styles: HashMap<(u8, u8), Vec<PointStyle>>,
    },
    #[serde(rename = "line")]
    Line {
        name: String,
        tags: Vec<String>,
        #[serde(
            serialize_with = "serialise_styles",
            deserialize_with = "deserialise_styles"
        )]
        styles: HashMap<(u8, u8), Vec<LineStyle>>,
    },
    #[serde(rename = "area")]
    Area {
        name: String,
        tags: Vec<String>,
        #[serde(
            serialize_with = "serialise_styles",
            deserialize_with = "deserialise_styles"
        )]
        styles: HashMap<(u8, u8), Vec<AreaStyle>>,
    },
}
impl SkinType {
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
    #[duplicate_item(
        fn_name          StyleType      TypeName ;
        [ point_styles ] [ PointStyle ] [ Point ];
        [ line_styles  ] [ LineStyle  ] [ Line  ];
        [ area_styles  ] [ AreaStyle  ] [ Area  ];
    )]
    #[must_use]
    pub const fn fn_name(&self) -> Option<&HashMap<(u8, u8), Vec<StyleType>>> {
        if let Self::TypeName { styles, .. } = self {
            Some(styles)
        } else {
            None
        }
    }
    pub fn style_in_zoom_level<T>(
        styles: &HashMap<(u8, u8), Vec<T>>,
        zoom_level: u8,
    ) -> Option<&Vec<T>> {
        styles
            .iter()
            .find(|((min, max), _)| (*min..=*max).contains(&zoom_level))
            .map(|(_, v)| v)
    }
    #[duplicate_item(
        fn_name                       style_fn_name    StyleType     ;
        [ point_style_in_zoom_level ] [ point_styles ] [ PointStyle ];
        [ line_style_in_zoom_level  ] [ line_styles  ] [ LineStyle  ];
        [ area_style_in_zoom_level  ] [ area_styles  ] [ AreaStyle  ];
    )]
    #[must_use]
    pub fn fn_name(&self, zoom_level: u8) -> Option<&Vec<StyleType>> {
        Self::style_in_zoom_level(self.style_fn_name()?, zoom_level)
    }
    fn style_in_max_zoom<T>(style: &HashMap<(u8, u8), Vec<T>>) -> Option<&Vec<T>> {
        style.iter().find(|((min, _), _)| *min == 0).map(|(_, v)| v)
    }
    #[duplicate_item(
        fn_name                     style_fn_name    StyleType     ;
        [ point_style_in_max_zoom ] [ point_styles ] [ PointStyle ];
        [ line_style_in_max_zoom  ] [ line_styles  ] [ LineStyle  ];
        [ area_style_in_max_zoom  ] [ area_styles  ] [ AreaStyle  ];
    )]
    #[must_use]
    pub fn fn_name(&self) -> Option<&Vec<StyleType>> {
        Self::style_in_max_zoom(self.style_fn_name()?)
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
    pub fn width(&self) -> Option<f32> {
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
        ui: &egui::Ui,
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

#[derive(Serialize, Deserialize, Debug, Default, Clone, PartialEq)]
pub struct Skin {
    pub version: u8,
    pub name: String,
    pub types: Vec<Arc<SkinType>>,
    pub font_files: Vec<(String, String)>,
    pub font_string: String,
    #[serde(
        serialize_with = "serialise_color32",
        deserialize_with = "deserialise_color32"
    )]
    pub background: egui::Color32,
    pub prune_small_text: Option<f32>,
    pub licence: String,

    #[serde(default)]
    pub order: HashMap<String, usize>,
}

impl Skin {
    #[must_use]
    pub fn get_type(&self, ty: &str) -> Option<&Arc<SkinType>> {
        self.types.iter().find(|a| a.name() == ty)
    }
    #[must_use]
    pub fn show_type(
        &self,
        ty: &str,
        ui: &egui::Ui,
        text_style: &egui::TextStyle,
    ) -> impl Into<egui::WidgetText> + use<> {
        self.get_type(ty).map_or_else(
            || egui::WidgetText::from(ty),
            |a| a.widget_text(ui, text_style).into(),
        )
    }
    pub fn setup_order_cache(&mut self) {
        self.order = self
            .types
            .iter()
            .enumerate()
            .map(|(i, a)| (a.name().into(), i))
            .collect();
    }
}
