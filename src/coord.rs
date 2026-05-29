use ordered_float::NotNan;

pub type Nnf32 = NotNan<f32>;

#[inline]
pub fn nn(value: f32) -> Nnf32 {
    Nnf32::new(value).unwrap()
}

pub trait Nnf32UpdateExt: Copy {
    fn update(&mut self, f: impl FnOnce(f32) -> f32);
}
impl Nnf32UpdateExt for Nnf32 {
    fn update(&mut self, f: impl FnOnce(f32) -> f32) {
        *self = nn(f(**self));
    }
}

pub trait CoordFrom<F: ?Sized>: Copy {
    fn coord_from(value: F) -> Self;
}
impl<F: Copy, T: CoordFrom<F>> CoordFrom<&F> for T {
    fn coord_from(value: &F) -> Self {
        Self::coord_from(*value)
    }
}
pub trait CoordInto<T>: Copy
where
    T: CoordFrom<Self>,
{
    fn coord_into(self) -> T {
        T::coord_from(self)
    }
}
impl<T: CoordFrom<F>, F: Copy> CoordInto<T> for F {}

impl CoordFrom<geo::Coord<f32>> for geo::Coord<Nnf32> {
    fn coord_from(value: geo::Coord<f32>) -> Self {
        geo::coord! { x: nn(value.x), y: nn(value.y) }
    }
}
impl CoordFrom<geo::Coord<Nnf32>> for geo::Coord<f32> {
    fn coord_from(value: geo::Coord<Nnf32>) -> Self {
        geo::coord! { x: *value.x, y: *value.y }
    }
}

impl CoordFrom<geo::Coord<i32>> for geo::Coord<f32> {
    fn coord_from(value: geo::Coord<i32>) -> Self {
        geo::coord! { x: value.x as f32, y: value.y as f32 }
    }
}
impl CoordFrom<geo::Coord<i32>> for geo::Coord<Nnf32> {
    fn coord_from(value: geo::Coord<i32>) -> Self {
        geo::coord! { x: nn(value.x as f32), y: nn(value.y as f32) }
    }
}
impl CoordFrom<geo::Coord<i32>> for egui::Pos2 {
    fn coord_from(value: geo::Coord<i32>) -> Self {
        egui::pos2(value.x as f32, value.y as f32)
    }
}

impl CoordFrom<geo::Coord<f32>> for geo::Coord<i32> {
    fn coord_from(value: geo::Coord<f32>) -> Self {
        geo::coord! { x: value.x.round() as i32, y: value.y.round() as i32 }
    }
}
impl CoordFrom<geo::Coord<Nnf32>> for geo::Coord<i32> {
    fn coord_from(value: geo::Coord<Nnf32>) -> Self {
        geo::coord! { x: value.x.round() as i32, y: value.y.round() as i32 }
    }
}
impl CoordFrom<geo::Coord<f32>> for egui::Pos2 {
    fn coord_from(value: geo::Coord<f32>) -> Self {
        egui::pos2(value.x, value.y)
    }
}
impl CoordFrom<geo::Coord<Nnf32>> for egui::Pos2 {
    fn coord_from(value: geo::Coord<Nnf32>) -> Self {
        egui::pos2(*value.x, *value.y)
    }
}
impl CoordFrom<geo::Coord<f32>> for egui::Vec2 {
    fn coord_from(value: geo::Coord<f32>) -> Self {
        egui::vec2(value.x, value.y)
    }
}
impl CoordFrom<geo::Coord<Nnf32>> for egui::Vec2 {
    fn coord_from(value: geo::Coord<Nnf32>) -> Self {
        egui::vec2(*value.x, *value.y)
    }
}

impl CoordFrom<egui::Pos2> for geo::Coord<i32> {
    fn coord_from(value: egui::Pos2) -> Self {
        geo::coord! { x: value.x.round() as i32, y: value.y.round() as i32 }
    }
}
impl CoordFrom<egui::Pos2> for geo::Coord<f32> {
    fn coord_from(value: egui::Pos2) -> Self {
        geo::coord! { x: value.x, y: value.y }
    }
}
impl CoordFrom<egui::Pos2> for geo::Coord<Nnf32> {
    fn coord_from(value: egui::Pos2) -> Self {
        geo::coord! { x: nn(value.x), y: nn(value.y) }
    }
}

impl CoordFrom<egui::Vec2> for geo::Coord<Nnf32> {
    fn coord_from(value: egui::Vec2) -> Self {
        geo::coord! { x: nn(value.x), y: nn(value.y) }
    }
}
impl CoordFrom<egui::Vec2> for geo::Coord<f32> {
    fn coord_from(value: egui::Vec2) -> Self {
        geo::coord! { x: value.x, y: value.y }
    }
}

impl CoordFrom<egui::Rect> for geo::Rect<f32> {
    fn coord_from(value: egui::Rect) -> Self {
        Self::new::<geo::Coord<f32>>(value.min.coord_into(), value.max.coord_into())
    }
}
impl CoordFrom<egui::Rect> for geo::Rect<Nnf32> {
    fn coord_from(value: egui::Rect) -> Self {
        Self::new::<geo::Coord<Nnf32>>(value.min.coord_into(), value.max.coord_into())
    }
}
impl CoordFrom<egui::Rect> for geo::Rect<i32> {
    fn coord_from(value: egui::Rect) -> Self {
        Self::new::<geo::Coord<i32>>(value.min.coord_into(), value.max.coord_into())
    }
}

impl CoordFrom<geo::Rect<i32>> for egui::Rect {
    fn coord_from(value: geo::Rect<i32>) -> Self {
        Self::from_two_pos(value.min().coord_into(), value.max().coord_into())
    }
}

impl CoordFrom<geo::Rect<f32>> for egui::Rect {
    fn coord_from(value: geo::Rect<f32>) -> Self {
        Self::from_two_pos(value.min().coord_into(), value.max().coord_into())
    }
}
impl CoordFrom<geo::Rect<Nnf32>> for egui::Rect {
    fn coord_from(value: geo::Rect<Nnf32>) -> Self {
        Self::from_two_pos(value.min().coord_into(), value.max().coord_into())
    }
}
