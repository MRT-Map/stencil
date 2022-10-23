#[derive(Copy, Clone, Debug)]
pub struct Zoom(pub f32);

impl Zoom {
    pub fn map_size(&self) -> f64 {
        2f64.powf((8f32 - self.0) as f64) * 32f64
    }
    pub fn world_size(&self) -> i32 {
        (2f64.powf((8f32 - self.0) as f64) * 32f64) as i32
    }
}
