use std::{
    error::Error,
    fmt::{Debug, Display},
    ops::Add,
    str::FromStr,
};

pub trait PlaNodeType: Debug + Copy + PartialEq {}

impl<T: Debug + Copy + PartialEq> PlaNodeType for T {}

pub trait PlaNodeTypeNew: PlaNodeType {
    type C: FromStr<Err: Error + Send + Sync + 'static>;
    fn new(x: Self::C, y: Self::C) -> Self;
}

pub trait PlaNodeTypeGet: PlaNodeType {
    type C: Display;
    fn x(self) -> Self::C;
    fn y(self) -> Self::C;
}

pub trait PlaNodeTypeAdd<Delta: Debug + Copy + Eq>:
    PlaNodeType + Add<Delta, Output = Self>
{
}
impl<Delta: Debug + Copy + Eq, T: PlaNodeType + Add<Delta, Output = Self>> PlaNodeTypeAdd<Delta>
    for T
{
}

pub trait PlaNodeTypeRect: PlaNodeType {
    type Rect;
    fn combine_rect(a: Self::Rect, b: Self::Rect) -> Self::Rect;
    fn rect_from_point(self) -> Self::Rect;
    fn rect_from_line(a: Self, b: Self) -> Self::Rect;
    fn rect_centre(rect: Self::Rect) -> Self;
}

pub trait PlaNodeTypeBezier: PlaNodeType {
    fn flatten_quadratic(a: Self, b: Self, c: Self, tolerance: impl Into<Option<f32>>)
    -> Vec<Self>;
    fn flatten_cubic(
        a: Self,
        b: Self,
        c: Self,
        d: Self,
        tolerance: impl Into<Option<f32>>,
    ) -> Vec<Self>;
}

pub trait PlaNodeTypeBezierRect: PlaNodeTypeRect + PlaNodeTypeBezier {
    fn rect_from_quadratic(a: Self, b: Self, c: Self) -> Self::Rect;
    fn rect_from_cubic(a: Self, b: Self, c: Self, d: Self) -> Self::Rect;
}

#[cfg(feature = "mint")]
#[duplicate::duplicate_item(
    Type; [mint::Point2]; [mint::Vector2]
)]
impl<CC: Debug + Copy + Eq + FromStr<Err: Error + Send + Sync + 'static>> PlaNodeTypeNew
    for Type<CC>
{
    type C = CC;
    fn new(x: Self::C, y: Self::C) -> Self {
        Self::from([x, y])
    }
}

#[cfg(feature = "mint")]
#[duplicate::duplicate_item(
    Type; [mint::Point2]; [mint::Vector2]
)]
impl<CC: Debug + Copy + Eq + Display> PlaNodeTypeGet for Type<CC> {
    type C = CC;
    fn x(self) -> Self::C {
        self.x
    }
    fn y(self) -> Self::C {
        self.y
    }
}

#[cfg(feature = "egui")]
#[duplicate::duplicate_item(
    Type; [egui::Pos2]; [egui::Vec2]
)]
impl PlaNodeTypeNew for Type {
    type C = f32;
    fn new(x: Self::C, y: Self::C) -> Self {
        Self::from([x, y])
    }
}

#[cfg(feature = "egui")]
#[duplicate::duplicate_item(
    Type; [egui::Pos2]; [egui::Vec2]
)]
impl PlaNodeTypeGet for Type {
    type C = f32;
    fn x(self) -> Self::C {
        self.x
    }
    fn y(self) -> Self::C {
        self.y
    }
}

#[cfg(feature = "egui")]
impl PlaNodeTypeRect for egui::Vec2 {
    type Rect = egui::Rect;
    fn combine_rect(a: Self::Rect, b: Self::Rect) -> Self::Rect {
        a | b
    }
    fn rect_from_point(self) -> Self::Rect {
        Self::Rect::from_pos(self.to_pos2())
    }
    fn rect_from_line(a: Self, b: Self) -> Self::Rect {
        Self::Rect::from_two_pos(a.to_pos2(), b.to_pos2())
    }
    fn rect_centre(rect: Self::Rect) -> Self {
        rect.center().to_vec2()
    }
}
#[cfg(feature = "egui")]
impl PlaNodeTypeRect for egui::Pos2 {
    type Rect = egui::Rect;
    fn combine_rect(a: Self::Rect, b: Self::Rect) -> Self::Rect {
        a | b
    }
    fn rect_from_point(self) -> Self::Rect {
        Self::Rect::from_pos(self)
    }
    fn rect_from_line(a: Self, b: Self) -> Self::Rect {
        Self::Rect::from_two_pos(a, b)
    }
    fn rect_centre(rect: Self::Rect) -> Self {
        rect.center()
    }
}

#[cfg(feature = "egui")]
impl PlaNodeTypeBezier for egui::Vec2 {
    fn flatten_quadratic(
        a: Self,
        b: Self,
        c: Self,
        tolerance: impl Into<Option<f32>>,
    ) -> Vec<Self> {
        egui::epaint::QuadraticBezierShape::from_points_stroke(
            [a.to_pos2(), b.to_pos2(), c.to_pos2()],
            false,
            egui::Color32::default(),
            egui::Stroke::default(),
        )
        .flatten(tolerance.into())
        .into_iter()
        .map(egui::Pos2::to_vec2)
        .collect()
    }

    fn flatten_cubic(
        a: Self,
        b: Self,
        c: Self,
        d: Self,
        tolerance: impl Into<Option<f32>>,
    ) -> Vec<Self> {
        egui::epaint::CubicBezierShape::from_points_stroke(
            [a.to_pos2(), b.to_pos2(), c.to_pos2(), d.to_pos2()],
            false,
            egui::Color32::default(),
            egui::Stroke::default(),
        )
        .flatten(tolerance.into())
        .into_iter()
        .map(egui::Pos2::to_vec2)
        .collect()
    }
}

#[cfg(feature = "egui")]
impl PlaNodeTypeBezier for egui::Pos2 {
    fn flatten_quadratic(
        a: Self,
        b: Self,
        c: Self,
        tolerance: impl Into<Option<f32>>,
    ) -> Vec<Self> {
        egui::epaint::QuadraticBezierShape::from_points_stroke(
            [a, b, c],
            false,
            egui::Color32::default(),
            egui::Stroke::default(),
        )
        .flatten(tolerance.into())
    }

    fn flatten_cubic(
        a: Self,
        b: Self,
        c: Self,
        d: Self,
        tolerance: impl Into<Option<f32>>,
    ) -> Vec<Self> {
        egui::epaint::CubicBezierShape::from_points_stroke(
            [a, b, c, d],
            false,
            egui::Color32::default(),
            egui::Stroke::default(),
        )
        .flatten(tolerance.into())
    }
}

#[cfg(feature = "egui")]
impl PlaNodeTypeBezierRect for egui::Vec2 {
    fn rect_from_quadratic(a: Self, b: Self, c: Self) -> Self::Rect {
        egui::epaint::QuadraticBezierShape::from_points_stroke(
            [a.to_pos2(), b.to_pos2(), c.to_pos2()],
            false,
            egui::Color32::default(),
            egui::Stroke::default(),
        )
        .logical_bounding_rect()
    }
    fn rect_from_cubic(a: Self, b: Self, c: Self, d: Self) -> Self::Rect {
        egui::epaint::CubicBezierShape::from_points_stroke(
            [a.to_pos2(), b.to_pos2(), c.to_pos2(), d.to_pos2()],
            false,
            egui::Color32::default(),
            egui::Stroke::default(),
        )
        .logical_bounding_rect()
    }
}

#[cfg(feature = "egui")]
impl PlaNodeTypeBezierRect for egui::Pos2 {
    fn rect_from_quadratic(a: Self, b: Self, c: Self) -> Self::Rect {
        egui::epaint::QuadraticBezierShape::from_points_stroke(
            [a, b, c],
            false,
            egui::Color32::default(),
            egui::Stroke::default(),
        )
        .logical_bounding_rect()
    }
    fn rect_from_cubic(a: Self, b: Self, c: Self, d: Self) -> Self::Rect {
        egui::epaint::CubicBezierShape::from_points_stroke(
            [a, b, c, d],
            false,
            egui::Color32::default(),
            egui::Stroke::default(),
        )
        .logical_bounding_rect()
    }
}

#[cfg(feature = "geo")]
#[duplicate::duplicate_item(
    Type; [geo::Coord]; [geo::Point]
)]
impl<T: geo::CoordNum + Eq + FromStr<Err: Error + Send + Sync + 'static>> PlaNodeTypeNew
    for Type<T>
{
    type C = T;
    fn new(x: Self::C, y: Self::C) -> Self {
        Self::from([x, y])
    }
}

#[cfg(feature = "geo")]
#[duplicate::duplicate_item(
    Type; [geo::Coord]; [geo::Point]
)]
impl<T: geo::CoordNum + Eq + Display> PlaNodeTypeGet for Type<T> {
    type C = T;
    fn x(self) -> Self::C {
        self.x_y().0
    }
    fn y(self) -> Self::C {
        self.x_y().1
    }
}

#[cfg(feature = "geo")]
impl<T: geo::CoordNum + Eq + geo::GeoFloat> PlaNodeTypeRect for geo::Coord<T> {
    type Rect = geo::Rect<T>;
    fn combine_rect(a: Self::Rect, b: Self::Rect) -> Self::Rect {
        geo::Rect::new(
            geo::coord! {x: a.min().x.min(b.min().x), y: a.min().y.min(b.min().y)},
            geo::coord! {x: a.max().x.max(b.max().x), y: a.max().y.max(b.max().y)},
        )
    }
    fn rect_from_point(self) -> Self::Rect {
        geo::Rect::new(self, self)
    }
    fn rect_from_line(a: Self, b: Self) -> Self::Rect {
        geo::Rect::new(a, b)
    }
    fn rect_centre(rect: Self::Rect) -> Self {
        rect.center()
    }
}
#[cfg(feature = "geo")]
impl<T: geo::CoordNum + Eq + geo::GeoFloat> PlaNodeTypeRect for geo::Point<T> {
    type Rect = geo::Rect<T>;
    fn combine_rect(a: Self::Rect, b: Self::Rect) -> Self::Rect {
        geo::Rect::new(
            geo::coord! {x: a.min().x.min(b.min().x), y: a.min().y.min(b.min().y)},
            geo::coord! {x: a.max().x.max(b.max().x), y: a.max().y.max(b.max().y)},
        )
    }
    fn rect_from_point(self) -> Self::Rect {
        geo::Rect::new(self, self)
    }
    fn rect_from_line(a: Self, b: Self) -> Self::Rect {
        geo::Rect::new(a, b)
    }
    fn rect_centre(rect: Self::Rect) -> Self {
        geo::Centroid::centroid(&rect)
    }
}

#[cfg(feature = "glam")]
#[duplicate::duplicate_item(
    Type CC;
    [glam::f32::Vec2] [f32];
    [glam::f64::DVec2] [f64];
    [glam::i8::I8Vec2] [i8];
    [glam::i16::I16Vec2] [i16];
    [glam::i32::IVec2] [i32];
    [glam::i64::I64Vec2] [i64];
    [glam::isize::ISizeVec2] [isize];
    [glam::u8::U8Vec2] [u8];
    [glam::u16::U16Vec2] [u16];
    [glam::u32::UVec2] [u32];
    [glam::u64::U64Vec2] [u64];
    [glam::usize::USizeVec2] [usize];
)]
impl PlaNodeTypeNew for Type {
    type C = CC;
    fn new(x: Self::C, y: Self::C) -> Self {
        Self::from([x, y])
    }
}

#[cfg(feature = "glam")]
#[duplicate::duplicate_item(
    Type CC;
    [glam::f32::Vec2] [f32];
    [glam::f64::DVec2] [f64];
    [glam::i8::I8Vec2] [i8];
    [glam::i16::I16Vec2] [i16];
    [glam::i32::IVec2] [i32];
    [glam::i64::I64Vec2] [i64];
    [glam::isize::ISizeVec2] [isize];
    [glam::u8::U8Vec2] [u8];
    [glam::u16::U16Vec2] [u16];
    [glam::u32::UVec2] [u32];
    [glam::u64::U64Vec2] [u64];
    [glam::usize::USizeVec2] [usize];
)]
impl PlaNodeTypeGet for Type {
    type C = CC;
    fn x(self) -> Self::C {
        self.x
    }
    fn y(self) -> Self::C {
        self.y
    }
}
