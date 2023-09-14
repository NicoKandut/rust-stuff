const SOLID: u8 = 0b1000_0000;
const OPAQUE: u8 = 0b0100_0000;

/**
 * First 2 bits determine solidity and opacity, all other bits are IDs.
 */
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq)]
#[rustfmt::skip]
pub enum Material {
    Air   =                  0,
    Water =         OPAQUE | 1,
    Glass = SOLID |          2,
    Stone = SOLID | OPAQUE | 3,
    Grass = SOLID | OPAQUE | 4,
    Sand  = SOLID | OPAQUE | 5,
    Snow  = SOLID | OPAQUE | 6,
    Ice   = SOLID | OPAQUE | 7,
}

impl Material {
    pub fn is_solid(&self) -> bool {
        (*self as u8 & SOLID) != 0
    }

    pub fn is_opaque(&self) -> bool {
        (*self as u8 & OPAQUE) != 0
    }

    pub fn color(&self) -> [f32; 3] {
        match *self {
            Self::Air => [0.0, 0.0, 0.0],
            Self::Water => [0.0, 0.0, 1.0],
            Self::Glass => [1.0, 1.0, 1.0],
            Self::Stone => [0.3, 0.2, 0.2],
            Self::Grass => [0.1, 0.7, 0.3],
            Self::Sand => [0.75, 0.7, 0.50],
            Self::Snow => [0.8, 0.9, 1.],
            Self::Ice => [0.5, 0.5, 1.],
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

impl From<f32> for Material {
    fn from(value: f32) -> Self {
        match value {
            x if x > 0.75 => Self::Stone,
            x if x > 0.5 => Self::Grass,
            _ => Self::Air,
        }
    }
}
