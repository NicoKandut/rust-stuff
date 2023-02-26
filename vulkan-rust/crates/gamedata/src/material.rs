const SOLID: u8 = 0b1000_0000;
const OPAQUE: u8 = 0b0100_0000;

/**
 * First 2 bits determine solidity and opacity.
 */
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Material {
    //  passable, see-through
    Air = 0,
    Water = 1,

    // solid, see-though
    Glass,

    // opaque, solid
    Stone = SOLID | OPAQUE | 1,
    Grass = SOLID | OPAQUE | 2,
}

impl Material {
    pub fn is_solid(&self) -> bool {
        (self.clone() as u8 & SOLID) != 0
    }

    pub fn is_opaque(&self) -> bool {
        (self.clone() as u8 & OPAQUE) != 0
    }

    pub fn color(&self) -> [f32; 3] {
        match *self {
            Material::Air => panic!("Air has no color!"),
            Material::Water => [0.0, 0.0, 1.0],
            Material::Glass => [1.0, 1.0, 1.0],
            Material::Stone => [0.3, 0.2, 0.2],
            Material::Grass => [0.2, 0.7, 0.2],
        }
    }

    pub fn from_noise(value: f32) -> Self {
        match value {
            x if x > 0.75 => Material::Stone,
            x if x > 0.5 => Material::Grass,
            _ => Material::Air,
        }
    }
}

impl From<f32> for Material {
    fn from(value: f32) -> Self {
        match value {
            x if x > 0.75 => Material::Stone,
            x if x > 0.5 => Material::Grass,
            _ => Material::Air,
        }
    }
}
