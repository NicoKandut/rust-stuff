use nalgebra_glm as glm;

/**
Main vector type in the project
*/
pub type Vec3 = glm::Vec3;

pub fn sub_vec3(lhs: &Vec3, rhs: &Vec3) -> Vec3 {
    glm::vec3(lhs.x - rhs.x, lhs.x - rhs.x, lhs.x - rhs.x)
}

pub fn add_vec3(lhs: &Vec3, rhs: &Vec3) -> Vec3 {
    glm::vec3(lhs.x + rhs.x, lhs.x + rhs.x, lhs.x + rhs.x)
}
