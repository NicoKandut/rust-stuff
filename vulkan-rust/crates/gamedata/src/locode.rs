use nalgebra_glm as glm;

use crate::vector::Vec3;

pub const ROOT_LOCATION_CODE: u64 = 0b1;

pub struct LocationCode {}

impl LocationCode {
    pub fn from_vec(vector: Vec3) -> u64 {
        u64::from(vector.x.is_sign_positive()) << 2
            | u64::from(vector.y.is_sign_positive()) << 1
            | u64::from(vector.z.is_sign_positive())
    }

    pub fn child_index_to_vec(child_index: u64) -> Vec3 {
        assert!(child_index < 8, "Child index too big");

        let code = child_index as u8;

        glm::vec3(
            -0.5 + f32::from(code >> 2),
            -0.5 + f32::from(code >> 1 & 0b1),
            -0.5 + f32::from(code & 0b1),
        )
    }
}

pub fn get_level(location_code: u64) -> u32 {
    (location_code.leading_zeros() - 42) / 3
}

pub fn get_child_code(location_code: u64, child_index: u64) -> u64 {
    assert!(child_index < 8, "Invalid child index");
    assert!(location_code > 0, "Invalid location code");
    (location_code << 3) | child_index
}

// pub fn get_parent_code(location_code: u64) -> u64 {
//     assert!(location_code > 0, "Invalid location code");
//     return location_code >> 3;
// }
