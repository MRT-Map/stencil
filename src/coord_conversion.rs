pub trait CoordConversionExt: Copy {
    fn to_geo_coord_i32(self) -> geo::Coord<i32>;
    fn to_geo_coord_f32(self) -> geo::Coord<f32>;
    #[expect(dead_code)]
    fn to_egui_vec2(self) -> egui::Vec2;
    fn to_egui_pos2(self) -> egui::Pos2;
}

impl CoordConversionExt for geo::Coord<i32> {
    fn to_geo_coord_i32(self) -> geo::Coord<i32> {
        self
    }

    fn to_geo_coord_f32(self) -> geo::Coord<f32> {
        geo::coord! { x: self.x as f32, y: self.y as f32 }
    }

    fn to_egui_vec2(self) -> egui::Vec2 {
        egui::vec2(self.x as f32, self.y as f32)
    }

    fn to_egui_pos2(self) -> egui::Pos2 {
        egui::pos2(self.x as f32, self.y as f32)
    }
}

impl CoordConversionExt for geo::Coord<f32> {
    fn to_geo_coord_i32(self) -> geo::Coord<i32> {
        geo::coord! { x: self.x.round() as i32, y: self.y.round() as i32 }
    }

    fn to_geo_coord_f32(self) -> geo::Coord<f32> {
        self
    }

    fn to_egui_vec2(self) -> egui::Vec2 {
        egui::vec2(self.x, self.y)
    }

    fn to_egui_pos2(self) -> egui::Pos2 {
        egui::pos2(self.x, self.y)
    }
}

impl CoordConversionExt for egui::Vec2 {
    fn to_geo_coord_i32(self) -> geo::Coord<i32> {
        geo::coord! { x: self.x.round() as i32, y: self.y.round() as i32 }
    }

    fn to_geo_coord_f32(self) -> geo::Coord<f32> {
        geo::coord! { x: self.x , y: self.y }
    }

    fn to_egui_vec2(self) -> egui::Vec2 {
        self
    }

    fn to_egui_pos2(self) -> egui::Pos2 {
        egui::pos2(self.x, self.y)
    }
}

impl CoordConversionExt for egui::Pos2 {
    fn to_geo_coord_i32(self) -> geo::Coord<i32> {
        geo::coord! { x: self.x.round() as i32, y: self.y.round() as i32 }
    }

    fn to_geo_coord_f32(self) -> geo::Coord<f32> {
        geo::coord! { x: self.x, y: self.y }
    }

    fn to_egui_vec2(self) -> egui::Vec2 {
        egui::vec2(self.x, self.y)
    }

    fn to_egui_pos2(self) -> egui::Pos2 {
        self
    }
}
