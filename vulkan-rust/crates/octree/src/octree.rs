#![allow(dead_code)]
#![allow(unused_variables)]

use std::simd::usizex8;

use gamedata::{material::Material, vector::Vec3};

use crate::vec;

pub type Size = f32;
pub type Point = [f32; 3];
pub type NodeId = usize;
pub type Children = [Node; 8];

// fn vec_div_scalar(v: Point, s: Size) -> Point {
//     [v[0] / s, v[1] / s, v[2] / s]
// }

#[derive(Debug)]
pub struct Ray {
    pub origin: Point,
    pub dir: Point,
    pub inv_dir: Point,
    pub v_mask: [bool; 3],
}

impl Ray {
    pub fn new(origin: Point, dir: Point) -> Self {
        Self {
            origin,
            dir,
            inv_dir: vec::inv(origin),
            v_mask: [
                dir[0].is_sign_negative(),
                dir[1].is_sign_negative(),
                dir[2].is_sign_negative(),
            ],
        }
    }
}

#[derive(Debug)]
pub struct Octree {
    root: usize,
    arena: Vec<Node>,
}

#[derive(Debug)]
pub struct Node {
    pub data: Material,
    pub center: Point,
    pub half_size: Size,
    children: Option<[NodeId; 8]>,
    parent: Option<NodeId>,
}

impl Node {
    pub fn new(center: Point, half_size: Size) -> Self {
        Self {
            data: Material::Air,
            center,
            half_size,
            children: None,
            parent: None,
        }
    }
}

impl Octree {
    pub fn new(size: Size) -> Self {
        let mut arena = vec![Node {
            data: Material::Air,
            center: [0., 0., 0.],
            half_size: size / 2.0,
            children: None,
            parent: None,
        }];

        arena.reserve(1000);

        Self { root: 0, arena }
    }

    pub fn len(&self) -> usize {
        self.arena.len()
    }

    pub fn split_node(&mut self, node_id: NodeId) -> Result<usize, ()> {
        let next_id = self.arena.len();
        let children = self.update_node(node_id, next_id)?;
        self.arena.extend(children);

        Ok(next_id)
    }

    fn update_node(&mut self, node_id: NodeId, next_id: usize) -> Result<Children, ()> {
        if let Some(node) = self.arena.get_mut(node_id) {
            let base_id = usizex8::splat(next_id);
            let offset = usizex8::from_array([0, 1, 2, 3, 4, 5, 6, 7]);
            let ids = base_id + offset;

            node.children = Some(ids.into());

            let children = create_children(node, node_id);

            Ok(children)
        } else {
            Err(())
        }
    }

    pub fn set(&mut self, node_id: NodeId, data: Material) -> Result<(), ()> {
        if let Some(node) = self.arena.get_mut(node_id) {
            node.data = data;
            Ok(())
        } else {
            Err(())
        }
    }

    pub(crate) fn ray_cast(&self, ray: &Ray) -> Option<RaycastOutcome> {
        let mut node_id = 0;
        let mut result = None;

        while let Some(node) = self.arena.get(node_id) {
            if intersect(node, ray) {
                result = Some(RaycastOutcome {
                    node_id,
                    half_size: node.half_size,
                });

                if let Some(children) = node.children {
                    node_id = children[0]
                } else {
                    break;
                }
            }
        }

        result
    }

    pub fn set_at(&mut self, target: Point, mat: Material) {
        self.arena
            .iter_mut()
            .find(|n| (n.children == None && intersect_point(n, &target)))
            .expect("No node at position")
            .data = mat;
    }

    pub fn get_leaves(&self) -> Vec<&Node> {
        self.arena.iter().filter(|n| n.children == None).collect()
    }

    pub fn intersects_box(&self, center: &Vec3, half_size: &Vec3) -> bool {
        false
    }
}

fn intersect_point(node: &Node, point: &Point) -> bool {
    let (min, max) = get_aabb(node);

    let mut outside: bool = false;

    for i in 0..3 {
        outside |= point[i] < min[i];
        outside |= point[i] > max[i];
    }

    return !outside;
}

pub(crate) struct RaycastOutcome {
    node_id: NodeId,
    half_size: Size,
}

pub(crate) fn create_children(node: &Node, node_id: usize) -> [Node; 8] {
    let parent = Some(node_id);
    let half_size = node.half_size / 2.0;
    let child_dirs = [
        [-1., -1., -1.],
        [-1., -1., 1.],
        [-1., 1., -1.],
        [-1., 1., 1.],
        [1., -1., -1.],
        [1., -1., 1.],
        [1., 1., -1.],
        [1., 1., 1.],
    ];

    let children = child_dirs
        .map(|dir| vec::mul_s(dir, half_size))
        .map(|o| vec::add(o, node.center))
        .map(|center| Node::new_child_of(parent, node.data.clone(), center, half_size));
    children
}

pub(crate) fn intersect(node: &Node, ray: &Ray) -> bool {
    let (node_min, node_max) = get_aabb(node);

    let t0 = vec::mul(vec::sub(node_min, ray.origin), ray.inv_dir);
    let t1 = vec::mul(vec::sub(node_max, ray.origin), ray.inv_dir);

    let mut tmin = f32::MIN;
    let mut tmax = f32::MAX;

    for i in 0..3 {
        tmin = f32::min(tmin.max(t0[i]), tmin.max(t1[i]));
        tmax = f32::max(tmax.min(t0[i]), tmax.min(t1[i]));
    }

    return tmax >= tmin;
}

fn get_aabb(node: &Node) -> ([f32; 3], [f32; 3]) {
    let node_min = vec::sub_s(node.center, node.half_size);
    let node_max = vec::add_s(node.center, node.half_size);
    (node_min, node_max)
}

impl Node {
    pub(crate) fn new_child_of(
        parent: Option<NodeId>,
        data: Material,
        center: Point,
        half_size: Size,
    ) -> Self {
        Self {
            data,
            center,
            half_size,
            children: None,
            parent,
        }
    }
}

// fn get_child_center(parent_center: Point, parent_half_size: Size, child_index: u8) -> Point {
//     let x_up = child_index >> 0 & 0b1;
//     let y_up = child_index >> 1 & 0b1;
//     let z_up = child_index >> 2 & 0b1;

//     let x_diff = (-0.5 + x_up as Size) * parent_half_size;
//     let y_diff = (-0.5 + y_up as Size) * parent_half_size;
//     let z_diff = (-0.5 + z_up as Size) * parent_half_size;

//     let diff = f32x4::from_array([x_diff, y_diff, z_diff, 0.0]);

//     parent_center + diff
// }

#[cfg(test)]
mod ray_cast_test {
    use super::{Octree, Ray};

    #[test]
    fn finds_root() {
        let tree = Octree::new(1.0);
        let ray = Ray::new([-2., 0., 0.], [1., 0., 0.]);

        let result = tree.ray_cast(&ray).expect("root was not hit");
        assert_eq!(0.5, result.half_size);
        assert_eq!(0, result.node_id);
    }

    #[test]
    fn finds_root_child() {
        let mut tree = Octree::new(1.0);
        tree.split_node(0).expect("split failed");

        let ray = Ray::new([-2., 0.5, 0.5], [1., 0., 0.]);

        let result = tree.ray_cast(&ray).expect("root was not hit");
        assert_eq!(0.25, result.half_size);
        assert_eq!(1, result.node_id);
    }
}
