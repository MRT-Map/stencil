use bevy::prelude::*;
use std::fmt::{Display, Formatter};

use crate::types::zoom::Zoom;

#[derive(Component, Default, PartialEq, Eq, Copy, Clone, Debug)]
pub struct TileCoord {
    pub x: i32,
    pub y: i32,
    pub z: i8,
}
impl Display for TileCoord {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{self.z}}, {{self.y}}, {{self.x}}")
    }
}

impl TileCoord {
    pub fn from_world_coords(x: f64, y: f64, z: i8) -> Self {
        Self {
            x: (x / Zoom(z as f32).world_size() as f64) as i32,
            y: (y / Zoom(z as f32).world_size() as f64) as i32,
            z,
        }
    }

    pub fn get_edges(&self) -> (f32, f32, f32, f32) {
        (
            self.x as f32 * Zoom(self.z as f32).world_size() as f32,
            self.y as f32 * Zoom(self.z as f32).world_size() as f32,
            (self.x + 1) as f32 * Zoom(self.z as f32).world_size() as f32,
            (self.y + 1) as f32 * Zoom(self.z as f32).world_size() as f32,
        )
    }

    #[allow(unreachable_code)]
    pub fn url(&self) -> String {
        return "".into();
        let z = 2.0f64.powi((8 - self.z) as i32);
        let x = self.x as f64;
        let y = self.y as f64;

        let group_x = ((x * z) as f64 / 32.0).floor() as i32;
        let group_y = ((y * z) as f64 / 32.0).floor() as i32;

        let num_in_group_x = x * z;
        let num_in_group_y = y * z;

        let mut zzz = "".to_string();
        let mut i = 8;
        while i > self.z {
            zzz += "z";
            i -= 1;
        }

        if !zzz.is_empty() {
            zzz += "_"
        };
        format!("http://api.allorigins.win/raw?url=https%3A//dynmap.minecartrapidtransit.net/tiles/new/flat/{}_{}/{}{}_{}.png",
            group_x, group_y, zzz, num_in_group_x, num_in_group_y)
    }
}
