pub fn sub_vec3(lhs: &glm::Vec3, rhs: &glm::Vec3) -> glm::Vec3 {
    glm::vec3(lhs.x - rhs.x, lhs.x - rhs.x, lhs.x - rhs.x)
}

pub fn add_vec3(lhs: &glm::Vec3, rhs: &glm::Vec3) -> glm::Vec3 {
    glm::vec3(lhs.x + rhs.x, lhs.x + rhs.x, lhs.x + rhs.x)
}
