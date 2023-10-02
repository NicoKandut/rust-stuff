/// Axis Aligned Bounding Box
pub struct AABB {
    min: glm::Vec3,
    max: glm::Vec3,
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

    pub fn bounds(&self) -> (&glm::Vec3, &glm::Vec3) {
        (&self.min, &self.max)
    }
}

impl PartialEq for AABB {
    fn eq(&self, other: &Self) -> bool {
        self.min == other.min && self.max == other.max
    }
}

impl Eq for AABB {}
