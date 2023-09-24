use nalgebra_glm::Vec3;

use crate::AABB;

#[derive(Clone, Debug)]
pub struct Ray {
    pub origin: Vec3,
    pub direction: Vec3,
    direction_inverse: Vec3,
}

impl Ray {
    pub fn new(origin: Vec3, direction: Vec3) -> Self {
        Self {
            origin,
            direction,
            direction_inverse: Vec3::new(1.0 / direction.x, 1.0 / direction.y, 1.0 / direction.z),
        }
    }

    pub fn point_on_ray(&self, distance: f32) -> Vec3 {
        self.origin + distance * self.direction
    }

    pub fn collides_with_aabb(&self, aabb: &AABB) -> Option<f32> {
        let tx1 = (aabb.min.x - self.origin.x) * self.direction_inverse.x;
        let tx2 = (aabb.max.x - self.origin.x) * self.direction_inverse.x;
        let tmin = f32::min(tx1, tx2);
        let tmax = f32::max(tx1, tx2);

        let ty1 = (aabb.min.y - self.origin.y) * self.direction_inverse.y;
        let ty2 = (aabb.max.y - self.origin.y) * self.direction_inverse.y;

        let tmin = f32::max(tmin, f32::min(ty1, ty2));
        let tmax = f32::min(tmax, f32::max(ty1, ty2));

        let tz1 = (aabb.min.z - self.origin.z) * self.direction_inverse.z;
        let tz2 = (aabb.max.z - self.origin.z) * self.direction_inverse.z;

        let tmin = f32::max(tmin, f32::min(tz1, tz2));
        let tmax = f32::min(tmax, f32::max(tz1, tz2));

        if tmax >= tmin {
            Some(tmin)
        } else {
            None
        }
    }
}
