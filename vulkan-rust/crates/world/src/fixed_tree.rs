use gamedata::material::Material;

#[derive(Clone, PartialEq, Debug)]
pub enum VoxelData1 {
    Empty,
    Sparse { children: [Option<Material>; 8] },
    Full { data: Material },
}

#[derive(Clone)]
pub enum VoxelData2 {
    Empty,
    Sparse { children: Box<[VoxelData1; 8]> },
    Full { data: Material },
}

#[derive(Clone)]
pub enum VoxelData3 {
    Empty,
    Sparse { children: Box<[VoxelData2; 8]> },
    Full { data: Material },
}

#[derive(Clone)]
pub enum VoxelData4 {
    Empty,
    Sparse { children: Box<[VoxelData3; 8]> },
    Full { data: Material },
}

#[derive(Clone)]
pub enum VoxelData5 {
    Empty,
    Sparse { children: Box<[VoxelData4; 8]> },
    Full { data: Material },
}

#[derive(Clone)]
pub enum VoxelData6 {
    Empty,
    Sparse { children: Box<[VoxelData5; 8]> },
    Full { data: Material },
}

impl VoxelData1 {
    pub const LEVEL: usize = 1;
    pub const SIZE: usize = 2;

    pub fn get(&self, x: usize, y: usize, z: usize) -> Option<Material> {
        debug_assert!(x < Self::SIZE);
        debug_assert!(y < Self::SIZE);
        debug_assert!(z < Self::SIZE);

        match self {
            Self::Full { data } => Some(*data),
            Self::Sparse { children: data } => {
                let i = z << 2 | y << 1 | x << 0;
                debug_assert!(i < 8);
                data[i]
            }
            Self::Empty => None,
        }
    }

    pub fn set(&mut self, x: usize, y: usize, z: usize, m: Material) {
        debug_assert!(x < Self::SIZE);
        debug_assert!(y < Self::SIZE);
        debug_assert!(z < Self::SIZE);

        let i = z << 2 | y << 1 | x << 0;
        debug_assert!(i < 8);

        match self {
            Self::Full { data: material } => {
                if m != *material {
                    let mut data = [Some(*material); 8];
                    data[i] = Some(m);
                    *self = Self::Sparse { children: data }
                }
            }
            Self::Sparse { children: data } => {
                data[i] = Some(m);
                if data.iter().all(|mat| *mat == Some(m)) {
                    *self = Self::Full { data: m }
                }
            }
            Self::Empty => {
                let mut data = [None; 8];
                data[i] = Some(m);
                *self = Self::Sparse { children: data }
            }
        }
    }
}

impl VoxelData2 {
    pub const LEVEL: usize = 2;
    pub const SHIFT: usize = 1;
    pub const CHILD_MASK: usize = 0b1;
    pub const SIZE: usize = 4;

    pub fn get(&self, x: usize, y: usize, z: usize) -> Option<Material> {
        debug_assert!(x < Self::SIZE);
        debug_assert!(y < Self::SIZE);
        debug_assert!(z < Self::SIZE);

        match self {
            Self::Full { data } => Some(*data),
            Self::Sparse { children } => {
                let i = 0
                    | ((z >> Self::SHIFT) & 1) << 2
                    | ((y >> Self::SHIFT) & 1) << 1
                    | ((x >> Self::SHIFT) & 1) << 0;
                debug_assert!(i < 8);
                children[i].get(
                    x & Self::CHILD_MASK,
                    y & Self::CHILD_MASK,
                    z & Self::CHILD_MASK,
                )
            }
            Self::Empty => None,
        }
    }

    pub fn set(&mut self, x: usize, y: usize, z: usize, m: Material) {
        debug_assert!(x < Self::SIZE);
        debug_assert!(y < Self::SIZE);
        debug_assert!(z < Self::SIZE);

        let i = 0
            | ((z >> Self::SHIFT) & 1) << 2
            | ((y >> Self::SHIFT) & 1) << 1
            | ((x >> Self::SHIFT) & 1) << 0;
        debug_assert!(i < 8);

        match self {
            Self::Full { data } => {
                if m != *data {
                    let mut children = [
                        VoxelData1::Full { data: *data },
                        VoxelData1::Full { data: *data },
                        VoxelData1::Full { data: *data },
                        VoxelData1::Full { data: *data },
                        VoxelData1::Full { data: *data },
                        VoxelData1::Full { data: *data },
                        VoxelData1::Full { data: *data },
                        VoxelData1::Full { data: *data },
                    ];

                    children[i].set(
                        x & Self::CHILD_MASK,
                        y & Self::CHILD_MASK,
                        z & Self::CHILD_MASK,
                        m,
                    );
                    *self = Self::Sparse {
                        children: Box::new(children),
                    }
                }
            }
            Self::Sparse { children } => {
                children[i].set(
                    x & Self::CHILD_MASK,
                    y & Self::CHILD_MASK,
                    z & Self::CHILD_MASK,
                    m,
                );
                if children.iter().all(|data| match *data {
                    VoxelData1::Full { data } => data == m,
                    _ => false,
                }) {
                    *self = Self::Full { data: m }
                }
            }
            Self::Empty => {
                let mut children = [
                    VoxelData1::Empty,
                    VoxelData1::Empty,
                    VoxelData1::Empty,
                    VoxelData1::Empty,
                    VoxelData1::Empty,
                    VoxelData1::Empty,
                    VoxelData1::Empty,
                    VoxelData1::Empty,
                ];

                children[i].set(
                    x & Self::CHILD_MASK,
                    y & Self::CHILD_MASK,
                    z & Self::CHILD_MASK,
                    m,
                );
                *self = Self::Sparse {
                    children: Box::new(children),
                }
            }
        };
    }
}

impl VoxelData3 {
    pub const LEVEL: usize = 3;
    pub const SHIFT: usize = 2;
    pub const CHILD_MASK: usize = 0b11;

    pub const SIZE: usize = 1 << Self::LEVEL;

    pub fn get(&self, x: usize, y: usize, z: usize) -> Option<Material> {
        debug_assert!(x < Self::SIZE);
        debug_assert!(y < Self::SIZE);
        debug_assert!(z < Self::SIZE);

        match self {
            Self::Full { data } => Some(*data),
            Self::Sparse { children } => {
                let i = 0
                    | ((z >> Self::SHIFT) & 1) << 2
                    | ((y >> Self::SHIFT) & 1) << 1
                    | ((x >> Self::SHIFT) & 1) << 0;
                debug_assert!(i < 8);
                children[i].get(
                    x & Self::CHILD_MASK,
                    y & Self::CHILD_MASK,
                    z & Self::CHILD_MASK,
                )
            }
            Self::Empty => None,
        }
    }

    pub fn set(&mut self, x: usize, y: usize, z: usize, m: Material) {
        debug_assert!(x < Self::SIZE);
        debug_assert!(y < Self::SIZE);
        debug_assert!(z < Self::SIZE);

        let i = 0
            | ((z >> Self::SHIFT) & 1) << 2
            | ((y >> Self::SHIFT) & 1) << 1
            | ((x >> Self::SHIFT) & 1) << 0;
        debug_assert!(i < 8);

        match self {
            Self::Full { data } => {
                if m != *data {
                    let mut children = [
                        VoxelData2::Full { data: *data },
                        VoxelData2::Full { data: *data },
                        VoxelData2::Full { data: *data },
                        VoxelData2::Full { data: *data },
                        VoxelData2::Full { data: *data },
                        VoxelData2::Full { data: *data },
                        VoxelData2::Full { data: *data },
                        VoxelData2::Full { data: *data },
                    ];

                    children[i].set(
                        x & Self::CHILD_MASK,
                        y & Self::CHILD_MASK,
                        z & Self::CHILD_MASK,
                        m,
                    );
                    *self = Self::Sparse {
                        children: Box::new(children),
                    }
                }
            }
            Self::Sparse { children } => {
                children[i].set(
                    x & Self::CHILD_MASK,
                    y & Self::CHILD_MASK,
                    z & Self::CHILD_MASK,
                    m,
                );
                if children.iter().all(|data| match *data {
                    VoxelData2::Full { data } => data == m,
                    _ => false,
                }) {
                    *self = Self::Full { data: m }
                }
            }
            Self::Empty => {
                let mut children = [
                    VoxelData2::Empty,
                    VoxelData2::Empty,
                    VoxelData2::Empty,
                    VoxelData2::Empty,
                    VoxelData2::Empty,
                    VoxelData2::Empty,
                    VoxelData2::Empty,
                    VoxelData2::Empty,
                ];

                children[i].set(
                    x & Self::CHILD_MASK,
                    y & Self::CHILD_MASK,
                    z & Self::CHILD_MASK,
                    m,
                );
                *self = Self::Sparse {
                    children: Box::new(children),
                }
            }
        };
    }
}

impl VoxelData4 {
    pub const LEVEL: usize = 4;
    pub const SHIFT: usize = 3;
    pub const CHILD_MASK: usize = 0b111;

    pub const SIZE: usize = 1 << Self::LEVEL;

    pub fn get(&self, x: usize, y: usize, z: usize) -> Option<Material> {
        debug_assert!(x < Self::SIZE);
        debug_assert!(y < Self::SIZE);
        debug_assert!(z < Self::SIZE);

        match self {
            Self::Full { data } => Some(*data),
            Self::Sparse { children } => {
                let i = 0
                    | ((z >> Self::SHIFT) & 1) << 2
                    | ((y >> Self::SHIFT) & 1) << 1
                    | ((x >> Self::SHIFT) & 1) << 0;
                debug_assert!(i < 8);
                children[i].get(
                    x & Self::CHILD_MASK,
                    y & Self::CHILD_MASK,
                    z & Self::CHILD_MASK,
                )
            }
            Self::Empty => None,
        }
    }

    pub fn set(&mut self, x: usize, y: usize, z: usize, m: Material) {
        debug_assert!(x < Self::SIZE);
        debug_assert!(y < Self::SIZE);
        debug_assert!(z < Self::SIZE);

        let i = 0
            | ((z >> Self::SHIFT) & 1) << 2
            | ((y >> Self::SHIFT) & 1) << 1
            | ((x >> Self::SHIFT) & 1) << 0;
        debug_assert!(i < 8);

        match self {
            Self::Full { data } => {
                if m != *data {
                    let mut children = [
                        VoxelData3::Full { data: *data },
                        VoxelData3::Full { data: *data },
                        VoxelData3::Full { data: *data },
                        VoxelData3::Full { data: *data },
                        VoxelData3::Full { data: *data },
                        VoxelData3::Full { data: *data },
                        VoxelData3::Full { data: *data },
                        VoxelData3::Full { data: *data },
                    ];

                    children[i].set(
                        x & Self::CHILD_MASK,
                        y & Self::CHILD_MASK,
                        z & Self::CHILD_MASK,
                        m,
                    );
                    *self = Self::Sparse {
                        children: Box::new(children),
                    }
                }
            }
            Self::Sparse { children } => {
                children[i].set(
                    x & Self::CHILD_MASK,
                    y & Self::CHILD_MASK,
                    z & Self::CHILD_MASK,
                    m,
                );
                if children.iter().all(|data| match *data {
                    VoxelData3::Full { data } => data == m,
                    _ => false,
                }) {
                    *self = Self::Full { data: m }
                }
            }
            Self::Empty => {
                let mut children = [
                    VoxelData3::Empty,
                    VoxelData3::Empty,
                    VoxelData3::Empty,
                    VoxelData3::Empty,
                    VoxelData3::Empty,
                    VoxelData3::Empty,
                    VoxelData3::Empty,
                    VoxelData3::Empty,
                ];

                children[i].set(
                    x & Self::CHILD_MASK,
                    y & Self::CHILD_MASK,
                    z & Self::CHILD_MASK,
                    m,
                );
                *self = Self::Sparse {
                    children: Box::new(children),
                }
            }
        };
    }
}

impl VoxelData5 {
    pub const LEVEL: usize = 5;
    pub const SHIFT: usize = 4;
    pub const CHILD_MASK: usize = 0b1111;

    pub const SIZE: usize = 1 << Self::LEVEL;

    pub fn get(&self, x: usize, y: usize, z: usize) -> Option<Material> {
        debug_assert!(x < Self::SIZE);
        debug_assert!(y < Self::SIZE);
        debug_assert!(z < Self::SIZE);

        match self {
            Self::Full { data } => Some(*data),
            Self::Sparse { children } => {
                let i = 0
                    | ((z >> Self::SHIFT) & 1) << 2
                    | ((y >> Self::SHIFT) & 1) << 1
                    | ((x >> Self::SHIFT) & 1) << 0;
                debug_assert!(i < 8);
                children[i].get(
                    x & Self::CHILD_MASK,
                    y & Self::CHILD_MASK,
                    z & Self::CHILD_MASK,
                )
            }
            Self::Empty => None,
        }
    }

    pub fn set(&mut self, x: usize, y: usize, z: usize, m: Material) {
        debug_assert!(x < Self::SIZE);
        debug_assert!(y < Self::SIZE);
        debug_assert!(z < Self::SIZE);

        let i = 0
            | ((z >> Self::SHIFT) & 1) << 2
            | ((y >> Self::SHIFT) & 1) << 1
            | ((x >> Self::SHIFT) & 1) << 0;
        debug_assert!(i < 8);

        match self {
            Self::Full { data } => {
                if m != *data {
                    let mut children = [
                        VoxelData4::Full { data: *data },
                        VoxelData4::Full { data: *data },
                        VoxelData4::Full { data: *data },
                        VoxelData4::Full { data: *data },
                        VoxelData4::Full { data: *data },
                        VoxelData4::Full { data: *data },
                        VoxelData4::Full { data: *data },
                        VoxelData4::Full { data: *data },
                    ];

                    children[i].set(
                        x & Self::CHILD_MASK,
                        y & Self::CHILD_MASK,
                        z & Self::CHILD_MASK,
                        m,
                    );
                    *self = Self::Sparse {
                        children: Box::new(children),
                    }
                }
            }
            Self::Sparse { children } => {
                children[i].set(
                    x & Self::CHILD_MASK,
                    y & Self::CHILD_MASK,
                    z & Self::CHILD_MASK,
                    m,
                );
                if children.iter().all(|data| match *data {
                    VoxelData4::Full { data } => data == m,
                    _ => false,
                }) {
                    *self = Self::Full { data: m }
                }
            }
            Self::Empty => {
                let mut children = [
                    VoxelData4::Empty,
                    VoxelData4::Empty,
                    VoxelData4::Empty,
                    VoxelData4::Empty,
                    VoxelData4::Empty,
                    VoxelData4::Empty,
                    VoxelData4::Empty,
                    VoxelData4::Empty,
                ];

                children[i].set(
                    x & Self::CHILD_MASK,
                    y & Self::CHILD_MASK,
                    z & Self::CHILD_MASK,
                    m,
                );
                *self = Self::Sparse {
                    children: Box::new(children),
                }
            }
        };
    }
}

impl VoxelData6 {
    pub const LEVEL: usize = 6;
    pub const SHIFT: usize = 5;
    pub const CHILD_MASK: usize = 0b11111;

    pub const SIZE: usize = 1 << Self::LEVEL;

    pub fn get(&self, x: usize, y: usize, z: usize) -> Option<Material> {
        debug_assert!(x < Self::SIZE);
        debug_assert!(y < Self::SIZE);
        debug_assert!(z < Self::SIZE);

        match self {
            Self::Full { data } => Some(*data),
            Self::Sparse { children } => {
                let i = 0
                    | ((z >> Self::SHIFT) & 1) << 2
                    | ((y >> Self::SHIFT) & 1) << 1
                    | ((x >> Self::SHIFT) & 1) << 0;
                debug_assert!(i < 8);
                children[i].get(
                    x & Self::CHILD_MASK,
                    y & Self::CHILD_MASK,
                    z & Self::CHILD_MASK,
                )
            }
            Self::Empty => None,
        }
    }

    pub fn set(&mut self, x: usize, y: usize, z: usize, m: Material) {
        debug_assert!(x < Self::SIZE);
        debug_assert!(y < Self::SIZE);
        debug_assert!(z < Self::SIZE);

        let i = 0
            | ((z >> Self::SHIFT) & 1) << 2
            | ((y >> Self::SHIFT) & 1) << 1
            | ((x >> Self::SHIFT) & 1) << 0;
        debug_assert!(i < 8);

        match self {
            Self::Full { data } => {
                if m != *data {
                    let mut children = [
                        VoxelData5::Full { data: *data },
                        VoxelData5::Full { data: *data },
                        VoxelData5::Full { data: *data },
                        VoxelData5::Full { data: *data },
                        VoxelData5::Full { data: *data },
                        VoxelData5::Full { data: *data },
                        VoxelData5::Full { data: *data },
                        VoxelData5::Full { data: *data },
                    ];
                    children[i].set(
                        x & Self::CHILD_MASK,
                        y & Self::CHILD_MASK,
                        z & Self::CHILD_MASK,
                        m,
                    );
                    *self = Self::Sparse {
                        children: Box::new(children),
                    }
                }
            }
            Self::Sparse { children } => {
                children[i].set(
                    x & Self::CHILD_MASK,
                    y & Self::CHILD_MASK,
                    z & Self::CHILD_MASK,
                    m,
                );
                if children.iter().all(|data| match *data {
                    VoxelData5::Full { data } => data == m,
                    _ => false,
                }) {
                    *self = Self::Full { data: m }
                }
            }
            Self::Empty => {
                let mut children = [
                    VoxelData5::Empty,
                    VoxelData5::Empty,
                    VoxelData5::Empty,
                    VoxelData5::Empty,
                    VoxelData5::Empty,
                    VoxelData5::Empty,
                    VoxelData5::Empty,
                    VoxelData5::Empty,
                ];

                children[i].set(
                    x & Self::CHILD_MASK,
                    y & Self::CHILD_MASK,
                    z & Self::CHILD_MASK,
                    m,
                );
                *self = Self::Sparse {
                    children: Box::new(children),
                }
            }
        };
    }
}

#[derive(Clone)]
pub struct ChunkData {
    data: VoxelData6,
}

impl ChunkData {
    pub fn default() -> Self {
        Self {
            data: VoxelData6::Empty,
        }
    }

    pub fn get(&self, x: usize, y: usize, z: usize) -> Option<Material> {
        self.data.get(x, y, z)
    }

    pub fn set(&mut self, x: usize, y: usize, z: usize, m: Material) {
        self.data.set(x, y, z, m)
    }

    pub(crate) fn is_empty(&self) -> bool {
        match self.data {
            VoxelData6::Empty => true,
            _ => false,
        }
    }
}

#[cfg(test)]
mod test {
    use gamedata::material::Material;
    use rand::Rng;
    use test::Bencher;

    use super::{ChunkData, VoxelData1, VoxelData6};

    #[test]
    fn can_find_voxel() {
        let mut tree = ChunkData {
            data: VoxelData6::Empty,
        };

        // first insert
        assert_eq!(None, tree.get(0, 0, 0));
        tree.set(0, 0, 0, Material::Stone);
        assert_eq!(Some(Material::Stone), tree.get(0, 0, 0));

        // insert in same leaf
        assert_eq!(None, tree.get(1, 0, 1));
        tree.set(1, 0, 1, Material::Stone);
        assert_eq!(Some(Material::Stone), tree.get(1, 0, 1));

        // different branch
        assert_eq!(None, tree.get(2, 2, 2));
        tree.set(2, 2, 2, Material::Stone);
        assert_eq!(Some(Material::Stone), tree.get(2, 2, 2));

        // highest
        assert_eq!(None, tree.get(63, 63, 63));
        tree.set(63, 63, 63, Material::Stone);
        assert_eq!(Some(Material::Stone), tree.get(63, 63, 63));

        // center
        assert_eq!(None, tree.get(32, 32, 32));
        tree.set(32, 32, 32, Material::Stone);
        assert_eq!(Some(Material::Stone), tree.get(32, 32, 32));
    }

    #[bench]
    fn fixed_inserts(b: &mut Bencher) {
        let mut tree = ChunkData {
            data: VoxelData6::Empty,
        };

        b.iter(|| {
            test::black_box(tree.set(34, 12, 27, Material::Stone));
        });
    }

    #[bench]
    fn random_inserts(b: &mut Bencher) {
        let mut tree = ChunkData {
            data: VoxelData6::Empty,
        };

        let mut random = rand::thread_rng();

        b.iter(|| {
            let x = random.gen_range(0..64);
            let y = random.gen_range(0..64);
            let z = random.gen_range(0..64);

            test::black_box(tree.set(x, y, z, Material::Stone));
        });
    }

    #[bench]
    fn full_inserts(b: &mut Bencher) {
        let mut tree = ChunkData {
            data: VoxelData6::Empty,
        };

        b.iter(|| {
            test::black_box(for z in 0..64 {
                for y in 0..64 {
                    for x in 0..64 {
                        tree.set(x, y, z, Material::Stone)
                    }
                }
            });
        });
    }

    #[test]
    fn enum_transitions_1() {
        let mut vd1 = VoxelData1::Empty;
        let m = Material::Stone;

        vd1.set(0, 0, 0, m);
        assert_eq!(
            vd1,
            VoxelData1::Sparse {
                children: [
                    Some(Material::Stone),
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None
                ]
            },
            "0 failed"
        );

        vd1.set(1, 0, 0, m);
        assert_eq!(
            vd1,
            VoxelData1::Sparse {
                children: [
                    Some(Material::Stone),
                    Some(Material::Stone),
                    None,
                    None,
                    None,
                    None,
                    None,
                    None
                ]
            },
            "1 failed"
        );

        vd1.set(0, 1, 0, m);
        assert_eq!(
            vd1,
            VoxelData1::Sparse {
                children: [
                    Some(Material::Stone),
                    Some(Material::Stone),
                    Some(Material::Stone),
                    None,
                    None,
                    None,
                    None,
                    None
                ]
            },
            "2 failed"
        );

        vd1.set(1, 1, 0, m);
        assert_eq!(
            vd1,
            VoxelData1::Sparse {
                children: [
                    Some(Material::Stone),
                    Some(Material::Stone),
                    Some(Material::Stone),
                    Some(Material::Stone),
                    None,
                    None,
                    None,
                    None
                ]
            }
        );

        vd1.set(0, 0, 1, m);
        assert_eq!(
            vd1,
            VoxelData1::Sparse {
                children: [
                    Some(Material::Stone),
                    Some(Material::Stone),
                    Some(Material::Stone),
                    Some(Material::Stone),
                    Some(Material::Stone),
                    None,
                    None,
                    None
                ]
            }
        );

        vd1.set(1, 0, 1, m);
        assert_eq!(
            vd1,
            VoxelData1::Sparse {
                children: [
                    Some(Material::Stone),
                    Some(Material::Stone),
                    Some(Material::Stone),
                    Some(Material::Stone),
                    Some(Material::Stone),
                    Some(Material::Stone),
                    None,
                    None
                ]
            }
        );

        vd1.set(0, 1, 1, m);
        assert_eq!(
            vd1,
            VoxelData1::Sparse {
                children: [
                    Some(Material::Stone),
                    Some(Material::Stone),
                    Some(Material::Stone),
                    Some(Material::Stone),
                    Some(Material::Stone),
                    Some(Material::Stone),
                    Some(Material::Stone),
                    None
                ]
            }
        );

        vd1.set(1, 1, 1, m);
        assert_eq!(
            vd1,
            VoxelData1::Full {
                data: Material::Stone
            }
        );
    }
}
