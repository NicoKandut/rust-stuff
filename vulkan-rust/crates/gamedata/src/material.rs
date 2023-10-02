use std::mem::variant_count;

const SOLID: u8 = 0b1000_0000;
const OPAQUE: u8 = 0b0100_0000;
const ID_MASK: u8 = 0b0011_1111;

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
    Water  =                  2,
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
    pub fn is_surface(&self) -> bool {
        *self == Material::Grass || *self == Material::Sand
    }

    #[inline]
    pub fn is_invisible(&self) -> bool {
        *self == Material::Unset || *self == Material::Air
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
        // match *self {
        //     Self::Air => palette::transparent(),
        //     Self::Unset => palette::transparent(),
        //     Self::Water => palette::water(),
        //     Self::Glass => palette::transparent(),
        //     Self::Stone => palette::stone(),
        //     Self::Grass => palette::grass(),
        //     Self::Sand => palette::sand(),
        //     Self::Snow => palette::white(),
        //     Self::Ice => palette::white(),
        //     Self::Wood => palette::wood(),
        //     Self::Leaves => palette::leaves(),
        //     Self::Dirt => palette::dirt(),
        //     Self::Debug => palette::red(),
        // }

        glm::vec4(u8::from(*self) as f32, 0.0, 0.0, 1.0)
    }

    pub fn color_bytes(&self) -> [u8; 4] {
        match *self {
            Self::Unset => palette::TRANSPARENT,
            Self::Air => palette::TRANSPARENT,
            Self::Water => palette::WATER,
            Self::Glass => palette::TRANSPARENT,
            Self::Stone => palette::STONE,
            Self::Grass => palette::GRASS,
            Self::Sand => palette::SAND,
            Self::Snow => palette::WHITE,
            Self::Ice => palette::WHITE,
            Self::Wood => palette::WOOD,
            Self::Leaves => palette::LEAVES,
            Self::Dirt => palette::DIRT,
            Self::Debug => palette::RED,
        }

        // [u8::from(*self), 0, 0, 255]
    }

    pub const ALL: [Material; variant_count::<Material>()] = [
        Self::Unset,
        Self::Air,
        Self::Water,
        Self::Glass,
        Self::Stone,
        Self::Grass,
        Self::Sand,
        Self::Snow,
        Self::Ice,
        Self::Wood,
        Self::Leaves,
        Self::Dirt,
        Self::Debug,
    ];
}

impl From<u8> for Material {
    fn from(value: u8) -> Self {
        Self::ALL[value as usize]
    }
}

impl From<Material> for u8 {
    fn from(value: Material) -> Self {
        value as u8 & ID_MASK
    }
}
