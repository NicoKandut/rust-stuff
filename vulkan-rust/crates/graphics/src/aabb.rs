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

    pub fn with_size(min: glm::Vec3, size: glm::Vec3) -> Self {
        Self {
            min,
            max: min + size,
        }
    }

    pub fn new_cube(size: f32) -> Self {
        let min = glm::Vec3::new(0.0, 0.0, 0.0);
        let max = glm::Vec3::new(size, size, size);

        Self { min, max }
    }
}
