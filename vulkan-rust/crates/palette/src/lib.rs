extern crate nalgebra_glm as glm;

macro_rules! color {
    ($name:tt, $r:literal, $g:literal, $b:literal, $a:literal) => {
        #[inline(always)]
        pub fn $name() -> glm::Vec4 {
            glm::Vec4::new(
                $r as f32 / 255.0,
                $g as f32 / 255.0,
                $b as f32 / 255.0,
                $a as f32 / 255.0,
            )
        }
    };
}

color!(red, 255, 0, 0, 255);
color!(white, 255, 255, 255, 255);
color!(transparent, 0, 0, 0, 0);

color!(water, 10, 98, 225, 128);
color!(stone, 91, 93, 108, 255);
color!(grass, 130, 186, 23, 255);
color!(sand, 195, 194, 155, 255);
color!(wood, 133, 97, 56, 255);
color!(leaves, 37, 95, 36, 255);
color!(dirt, 155, 132, 69, 255);
color!(sky, 80, 120, 254, 255);
