use bevy::prelude::*;
use bevy_mod_picking::PickableBundle;
use std::collections::HashMap;
use bevy_prototype_lyon::entity::ShapeBundle;
use bevy_prototype_lyon::prelude::*;
use crate::types::ComponentType;
use crate::types::pla::ComponentCoords;
use crate::types::skin::Skin;

#[derive(Debug, Default, Component)]
pub struct EditorComponent {
    pub namespace: String,
    pub id: String,
    pub display_name: String,
    pub description: String,
    pub tags: String,
    pub layer: f64,
    pub type_: String,
    pub attributes: HashMap<String, String>,
}

impl EditorComponent {
    pub fn new(type_: ComponentType) -> Self {
        Self {
            type_: format!(
                "simple{}",
                match type_ {
                    ComponentType::Point => "Point",
                    ComponentType::Line => "Line",
                    ComponentType::Area => "Area",
                }
            ),
            ..default()
        }
    }
    pub fn get_type(&self, skin: &Skin) -> Option<ComponentType> {
        Some(skin.types.get(self.type_.as_str())?.get_type())
    }
    pub fn get_shape(&self, coords: ComponentCoords, skin: &Skin, selected: bool) -> ShapeBundle {
        if self.get_type(skin) == Some(ComponentType::Point) {
            GeometryBuilder::build_as(
                &shapes::Rectangle {
                    extents: Vec2::new(10.0, 10.0),
                    origin: RectangleOrigin::Center,
                },
                DrawMode::Fill(FillMode::color(if selected {
                    Color::YELLOW
                } else {
                    Color::CYAN
                })),
                Transform::from_xyz(coords.0[0].x as f32, coords.0[0].y as f32, 10.0),
            )
        } else {
            GeometryBuilder::build_as(
                &{
                    let mut pb = PathBuilder::new();
                    for coord in coords.0 {
                        pb.line_to(coord.as_vec2());
                    }
                    pb.build()
                },
                DrawMode::Stroke(StrokeMode::new(
                    if selected { Color::YELLOW } else { Color::CYAN },
                    8.0,
                )),
                Transform::from_xyz(0.0, 0.0, 10.0),
            )
        }
    }
}

#[derive(Bundle)]
pub struct ComponentBundle {
    pub data: EditorComponent,
    pub coords: ComponentCoords,

    #[bundle]
    pub shape: ShapeBundle,
    #[bundle]
    pub pickable: PickableBundle
}

impl ComponentBundle {
    pub fn new(data: EditorComponent, orig_coords: IVec2) -> Self {
        Self {
            data,
            coords: ComponentCoords(vec![orig_coords]),
            shape: ShapeBundle::default(),
            pickable: PickableBundle::default()
        }
    }
    pub fn update_shape(&mut self, skin: &Skin) {
        self.shape = self.data.get_shape(self.coords.to_owned(), skin, false);
    }
}

#[derive(Component)]
pub struct CreatedComponent;

#[derive(Component)]
pub struct SelectedComponent;
