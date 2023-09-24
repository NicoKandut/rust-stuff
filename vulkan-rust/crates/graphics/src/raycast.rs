use std::ops::Range;

use crate::Ray;

pub trait Raycast {
    fn cast_ray(&self, ray: &Ray, limit: &Range<f32>) -> Option<f32>;
}
