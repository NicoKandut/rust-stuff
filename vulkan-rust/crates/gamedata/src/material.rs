const SOLID: u8 = 0b1000_0000;
const OPAQUE: u8 = 0b0100_0000;

/**
 * First 2 bits determine solidity and opacity, all other bits are IDs.
 */
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Default)]
#[rustfmt::skip]
pub enum Material {
    #[default]
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
    #[inline]
    pub fn is_solid(&self) -> bool {
        (*self as u8 & SOLID) != 0
    }

    #[inline]
    pub fn is_opaque(&self) -> bool {
        (*self as u8 & OPAQUE) != 0
    }

    #[inline]
    pub fn is_fillable(&self) -> bool {
        return (*self as u8) < 2;
    }

    pub fn color(&self) -> glm::Vec4 {
        match *self {
            Self::Air | Self::Unset => palette::transparent(),
            Self::Water => palette::water(),
            Self::Glass => palette::transparent(),
            Self::Stone => palette::stone(),
            Self::Grass => palette::grass(),
            Self::Sand => palette::sand(),
            Self::Snow => palette::white(),
            Self::Ice => palette::white(),
            Self::Wood => palette::wood(),
            Self::Leaves => palette::leaves(),
            Self::Dirt => palette::dirt(),
            Self::Debug => palette::red(),
        }
    }
}
