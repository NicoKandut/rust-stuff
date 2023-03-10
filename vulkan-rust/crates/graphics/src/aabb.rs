/**
 * Axis Aligned Bounding Box
 */
pub struct AABB {
    pub min: glm::Vec3,
    pub max: glm::Vec3,
}

impl AABB {
    pub fn new(min: glm::Vec3, max: glm::Vec3) -> Self {
        Self { min, max }
    }
}
