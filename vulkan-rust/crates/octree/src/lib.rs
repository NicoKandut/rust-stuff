#![feature(test)]

extern crate test;

mod next;

/**
 * Generic node in an octree
 */
#[derive(Clone, PartialEq, Debug)]
pub enum Node<Children, T: Copy + PartialEq> {
    Empty,
    Sparse(Children),
    Full(T),
}

// Specific nodes for all levels in the octree
pub type L1Node<T> = Node<[Option<T>; 8], T>;
pub type L2Node<T> = Node<Box<[L1Node<T>; 8]>, T>;
pub type L3Node<T> = Node<Box<[L2Node<T>; 8]>, T>;
pub type L4Node<T> = Node<Box<[L3Node<T>; 8]>, T>;
pub type L5Node<T> = Node<Box<[L4Node<T>; 8]>, T>;
pub type L6Node<T> = Node<Box<[L5Node<T>; 8]>, T>;

pub trait LeafAccess<T: Copy + PartialEq> {
    const LEVEL: usize;
    const SIZE: usize;
    const SHIFT: usize;
    const CHILD_MASK: usize;

    fn get(&self, x: usize, y: usize, z: usize) -> Option<T>;
    fn set(&mut self, x: usize, y: usize, z: usize, m: T);
}

macro_rules! impl_leaf_access {
    ($level:literal, $node:ident, $child:ident) => {
        impl<T: Copy + PartialEq> LeafAccess<T> for $node<T> {
            const LEVEL: usize = $level;
            const SIZE: usize = 1 << $level;
            const SHIFT: usize = Self::LEVEL - 1;
            const CHILD_MASK: usize = (1 << Self::SHIFT) - 1;

            fn get(&self, x: usize, y: usize, z: usize) -> Option<T> {
                debug_assert!(x < Self::SIZE);
                debug_assert!(y < Self::SIZE);
                debug_assert!(z < Self::SIZE);

                match self {
                    Self::Full(data) => Some(*data),
                    Self::Sparse(children) => {
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

            fn set(&mut self, x: usize, y: usize, z: usize, m: T) {
                debug_assert!(x < Self::SIZE);
                debug_assert!(y < Self::SIZE);
                debug_assert!(z < Self::SIZE);

                let i = 0
                    | ((z >> Self::SHIFT) & 1) << 2
                    | ((y >> Self::SHIFT) & 1) << 1
                    | ((x >> Self::SHIFT) & 1) << 0;
                debug_assert!(i < 8);

                match self {
                    Self::Full(data) => {
                        if m != *data {
                            let mut children = [
                                $child::Full(*data),
                                $child::Full(*data),
                                $child::Full(*data),
                                $child::Full(*data),
                                $child::Full(*data),
                                $child::Full(*data),
                                $child::Full(*data),
                                $child::Full(*data),
                            ];

                            children[i].set(
                                x & Self::CHILD_MASK,
                                y & Self::CHILD_MASK,
                                z & Self::CHILD_MASK,
                                m,
                            );
                            *self = Self::Sparse(Box::new(children))
                        }
                    }
                    Self::Sparse(children) => {
                        children[i].set(
                            x & Self::CHILD_MASK,
                            y & Self::CHILD_MASK,
                            z & Self::CHILD_MASK,
                            m,
                        );
                        if children.iter().all(|data| match *data {
                            $child::Full(data) => data == m,
                            _ => false,
                        }) {
                            *self = Self::Full(m);
                            return;
                        }

                        // sparse to empty
                        if children.iter().all(|data| match *data {
                            $child::Empty => true,
                            _ => false,
                        }) {
                            *self = Self::Empty;
                        }
                    }
                    Self::Empty => {
                        let mut children = [
                            $child::Empty,
                            $child::Empty,
                            $child::Empty,
                            $child::Empty,
                            $child::Empty,
                            $child::Empty,
                            $child::Empty,
                            $child::Empty,
                        ];

                        children[i].set(
                            x & Self::CHILD_MASK,
                            y & Self::CHILD_MASK,
                            z & Self::CHILD_MASK,
                            m,
                        );
                        *self = Self::Sparse(Box::new(children))
                    }
                };
            }
        }
    };
}

impl<T: Copy + PartialEq> LeafAccess<T> for L1Node<T> {
    const LEVEL: usize = 1;
    const SIZE: usize = 2;
    const SHIFT: usize = 0;
    const CHILD_MASK: usize = 0;

    fn get(&self, x: usize, y: usize, z: usize) -> Option<T> {
        debug_assert!(x < Self::SIZE);
        debug_assert!(y < Self::SIZE);
        debug_assert!(z < Self::SIZE);

        match self {
            Self::Full(data) => Some(*data),
            Self::Sparse(data) => data[z << 2 | y << 1 | x],
            Self::Empty => None,
        }
    }

    fn set(&mut self, x: usize, y: usize, z: usize, m: T) {
        debug_assert!(x < Self::SIZE);
        debug_assert!(y < Self::SIZE);
        debug_assert!(z < Self::SIZE);

        let i = z << 2 | y << 1 | x;

        match self {
            Self::Full(material) => {
                // full to sparse
                if m != *material {
                    let mut data = [Some(*material); 8];
                    data[i] = Some(m);
                    *self = Self::Sparse(data)
                }
            }
            Self::Sparse(data) => {
                data[i] = Some(m);
                // sparse to full
                if data.iter().all(|mat| *mat == Some(m)) {
                    *self = Self::Full(m);
                    return;
                }

                // sparse to empty
                if data.iter().all(|mat| mat.is_none()) {
                    *self = Self::Empty;
                }
            }
            Self::Empty => {
                // empty to sparse
                let mut data = [None; 8];
                data[i] = Some(m);
                *self = Self::Sparse(data)
            }
        }
    }
}

impl_leaf_access!(2, L2Node, L1Node);
impl_leaf_access!(3, L3Node, L2Node);
impl_leaf_access!(4, L4Node, L3Node);
impl_leaf_access!(5, L5Node, L4Node);
impl_leaf_access!(6, L6Node, L5Node);

#[cfg(test)]
mod tests {
    use rand::Rng;
    use std::mem::size_of;
    use test::Bencher;

    use crate::{L1Node, L2Node, L3Node, L4Node, L5Node, L6Node, LeafAccess};

    #[test]
    fn can_find_voxel() {
        let mut tree = L6Node::<u8>::Empty;

        // first insert
        assert_eq!(None, tree.get(0, 0, 0));
        tree.set(0, 0, 0, 1);
        assert_eq!(Some(1), tree.get(0, 0, 0));

        // insert in same leaf
        assert_eq!(None, tree.get(1, 0, 1));
        tree.set(1, 0, 1, 1);
        assert_eq!(Some(1), tree.get(1, 0, 1));

        // different branch
        assert_eq!(None, tree.get(2, 2, 2));
        tree.set(2, 2, 2, 1);
        assert_eq!(Some(1), tree.get(2, 2, 2));

        // highest
        assert_eq!(None, tree.get(63, 63, 63));
        tree.set(63, 63, 63, 1);
        assert_eq!(Some(1), tree.get(63, 63, 63));

        // center
        assert_eq!(None, tree.get(32, 32, 32));
        tree.set(32, 32, 32, 1);
        assert_eq!(Some(1), tree.get(32, 32, 32));
    }

    #[bench]
    fn fixed_inserts(b: &mut Bencher) {
        let mut tree = L6Node::<u8>::Empty;

        b.iter(|| {
            test::black_box(tree.set(34, 12, 27, 1));
        });
    }

    #[bench]
    fn random_inserts(b: &mut Bencher) {
        let mut tree = L6Node::<u8>::Empty;

        let mut random = rand::thread_rng();

        b.iter(|| {
            let x = random.gen_range(0..64);
            let y = random.gen_range(0..64);
            let z = random.gen_range(0..64);

            test::black_box(tree.set(x, y, z, 1));
        });
    }

    #[bench]
    fn full_inserts(b: &mut Bencher) {
        let mut tree = L6Node::<u8>::Empty;

        b.iter(|| {
            test::black_box(for z in 0..64 {
                for y in 0..64 {
                    for x in 0..64 {
                        tree.set(x, y, z, 1)
                    }
                }
            });
        });
    }

    #[test]
    fn enum_transitions_1() {
        let mut vd1 = L1Node::<u8>::Empty;
        let m = 1;

        vd1.set(0, 0, 0, m);
        assert_eq!(
            vd1,
            L1Node::Sparse([Some(1), None, None, None, None, None, None, None]),
        );

        vd1.set(1, 0, 0, m);
        assert_eq!(
            vd1,
            L1Node::Sparse([Some(1), Some(1), None, None, None, None, None, None]),
        );

        vd1.set(0, 1, 0, m);
        assert_eq!(
            vd1,
            L1Node::Sparse([Some(1), Some(1), Some(1), None, None, None, None, None]),
        );

        vd1.set(1, 1, 0, m);
        assert_eq!(
            vd1,
            L1Node::Sparse([Some(1), Some(1), Some(1), Some(1), None, None, None, None])
        );

        vd1.set(0, 0, 1, m);
        assert_eq!(
            vd1,
            L1Node::Sparse([
                Some(1),
                Some(1),
                Some(1),
                Some(1),
                Some(1),
                None,
                None,
                None
            ])
        );

        vd1.set(1, 0, 1, m);
        assert_eq!(
            vd1,
            L1Node::Sparse([
                Some(1),
                Some(1),
                Some(1),
                Some(1),
                Some(1),
                Some(1),
                None,
                None
            ])
        );

        vd1.set(0, 1, 1, m);
        assert_eq!(
            vd1,
            L1Node::Sparse([
                Some(1),
                Some(1),
                Some(1),
                Some(1),
                Some(1),
                Some(1),
                Some(1),
                None
            ])
        );

        vd1.set(1, 1, 1, m);
        assert_eq!(vd1, L1Node::Full(1));
    }

    #[test]
    fn full_merge() {
        let mut tree = L6Node::<u8>::Empty;
        let m = 1;

        assert!(tree == L6Node::Empty);

        let size = L6Node::<u8>::SIZE;

        for z in 0..size {
            for y in 0..size {
                for x in 0..size {
                    tree.set(x, y, z, m)
                }
            }
        }

        assert!(tree == L6Node::Full(m));
    }

    #[test]
    fn sizes() {
        assert_eq!(16, size_of::<L1Node<u8>>());
        assert_eq!(16, size_of::<L2Node<u8>>());
        assert_eq!(16, size_of::<L3Node<u8>>());
        assert_eq!(16, size_of::<L4Node<u8>>());
        assert_eq!(16, size_of::<L5Node<u8>>());
        assert_eq!(16, size_of::<L6Node<u8>>());
    }
}
