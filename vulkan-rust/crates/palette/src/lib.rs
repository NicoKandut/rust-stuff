extern crate nalgebra_glm as glm;

pub const TRANSPARENT: [u8; 4] = [0, 0, 0, 0];
pub const RED: [u8; 4] = [255, 0, 0, 255];
pub const WHITE: [u8; 4] = [255, 255, 255, 255];
pub const WATER: [u8; 4] = [10, 98, 225, 128];
pub const STONE: [u8; 4] = [91, 93, 108, 255];
pub const GRASS: [u8; 4] = [130, 186, 23, 255];
pub const SAND: [u8; 4] = [195, 194, 155, 255];
pub const WOOD: [u8; 4] = [133, 97, 56, 255];
pub const LEAVES: [u8; 4] = [37, 95, 36, 255];
pub const DIRT: [u8; 4] = [155, 132, 69, 255];
pub const SKY: [u8; 4] = [80, 120, 254, 255];

macro_rules! color {
    ($name:tt, $rgba:expr) => {
        #[inline(always)]
        pub fn $name() -> glm::Vec4 {
            glm::Vec4::new(
                $rgba[0] as f32 / 255.0,
                $rgba[1] as f32 / 255.0,
                $rgba[2] as f32 / 255.0,
                $rgba[3] as f32 / 255.0,
            )
        }
    };
}

color!(transparent, TRANSPARENT);
color!(red, RED);
color!(white, WHITE);
color!(water, WATER);
color!(stone, STONE);
color!(grass, GRASS);
color!(sand, SAND);
color!(wood, WOOD);
color!(leaves, LEAVES);
color!(dirt, DIRT);
color!(sky, SKY);
