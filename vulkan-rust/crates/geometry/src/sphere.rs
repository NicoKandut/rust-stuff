use std::f32::consts::PI;

pub struct Sphere {
    radius: f32,
}

impl Sphere {
    #[inline]
    pub fn new(radius: f32) -> Self {
        Self { radius }
    }

    pub fn volume(&self) -> f32 {
        4.0 / 3.0 * PI * self.radius.powi(3)
    }
}
