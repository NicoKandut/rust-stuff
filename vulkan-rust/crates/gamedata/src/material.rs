const SOLID: u8 = 0b1000_0000;
const OPAQUE: u8 = 0b0100_0000;

fn rgb(r: u8, g: u8, b: u8) -> [f32; 3] {
    [r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0]
}

/**
 * First 2 bits determine solidity and opacity, all other bits are IDs.
 */
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq)]
#[rustfmt::skip]
pub enum Material {
    Unset  =                  0,
    Air    =                  1,
    Water  =         OPAQUE | 2,
    Glass  = SOLID |          3,
    Stone  = SOLID | OPAQUE | 4,
    Grass  = SOLID | OPAQUE | 5,
    Sand   = SOLID | OPAQUE | 6,
    Snow   = SOLID | OPAQUE | 7,
    Ice    = SOLID | OPAQUE | 8,
    Wood   = SOLID | OPAQUE | 9,
    Leaves = SOLID | OPAQUE | 10,
    Dirt   = SOLID | OPAQUE | 11,
    Debug  = SOLID | OPAQUE | 12,
}

impl Material {
    pub fn is_solid(&self) -> bool {
        (*self as u8 & SOLID) != 0
    }

    pub fn is_opaque(&self) -> bool {
        (*self as u8 & OPAQUE) != 0
    }

    pub fn is_fillable(&self) -> bool {
        return (*self as u8) < 2;
    }

    pub fn color(&self) -> [f32; 3] {
        match *self {
            Self::Air | Self::Unset => [0.0, 0.0, 0.0],
            Self::Water => rgb(23, 98, 203),
            Self::Glass => [1.0, 1.0, 1.0],
            Self::Stone => [0.3, 0.2, 0.2],
            Self::Grass => [0.1, 0.7, 0.3],
            Self::Sand => [0.8, 0.7, 0.5],
            Self::Snow => [0.8, 0.9, 1.],
            Self::Ice => [0.5, 0.5, 1.0],
            Self::Wood => [0.5, 0.3, 0.0],
            Self::Leaves => [0.0, 0.5, 0.0],
            Self::Dirt => [0.3, 0.2, 0.0],
            Self::Debug => rgb(255, 0, 0),
        }
    }

    pub fn from_noise(value: f32) -> Self {
        match value {
            x if x > 0.75 => Self::Stone,
            x if x > 0.5 => Self::Grass,
            _ => Self::Air,
        }
    }
}

impl Default for Material {
    fn default() -> Self {
        Material::Unset
    }
}

impl From<f32> for Material {
    fn from(value: f32) -> Self {
        match value {
            x if x > 0.75 => Self::Stone,
            x if x > 0.5 => Self::Grass,
            _ => Self::Air,
        }
    }
}
