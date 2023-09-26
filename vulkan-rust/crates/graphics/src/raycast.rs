use geometry::Ray;
use std::ops::Range;

pub trait Raycast {
    fn cast_ray(&self, ray: &Ray, limit: &Range<f32>) -> Option<f32>;
}
