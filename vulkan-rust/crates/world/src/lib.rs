#![feature(test)]

extern crate nalgebra_glm as glm;
extern crate test;

pub mod gen;
pub mod mesh_generator; // TODO: extract
pub mod mesh_manager; // TODO: extract
pub mod mgmt;
pub mod slice;
pub mod traits;

pub use chunk_id::{ChunkId, MeshId};
pub use mgmt::chunk::ChunkManager;
pub use seed::{ChunkSeed, PositionalSeed, WorldSeed};
pub use world_parameters::*;
pub use world_position::WorldPosition;

mod chunk_id;
mod seed;
mod terrain_noise;
mod world_parameters;
mod world_position;

use gamedata::material::Material;
use geometry::{Ray, AABB};
use glm::Vec3;
use octree::{L1Node, L2Node, L3Node, L4Node, L5Node, L6Node, LeafAccess};
use rand::{thread_rng, Rng};
use std::{ops::Range, sync::Arc};
use traits::Data3D;

pub struct ChunkIdAndData {
    pub id: ChunkId,
    pub data: Option<Arc<ChunkData>>,
}

pub struct ChunkUpdateData {
    pub chunk: ChunkIdAndData,
    pub adjecent: [ChunkIdAndData; 6],
}

impl From<&ChunkId> for AABB {
    fn from(id: &ChunkId) -> Self {
        let min = glm::vec3(
            (id.x * CHUNK_SIZE as i32) as f32,
            (id.y * CHUNK_SIZE as i32) as f32,
            (id.z * CHUNK_SIZE as i32) as f32,
        );
        let max = glm::vec3(
            min.x + CHUNK_SIZE as f32,
            min.y + CHUNK_SIZE as f32,
            min.z + CHUNK_SIZE as f32,
        );

        AABB::new(min, max)
    }
}

impl From<&ChunkId> for WorldPosition {
    fn from(id: &ChunkId) -> Self {
        Self::new(
            id.x * CHUNK_SIZE as i32,
            id.y * CHUNK_SIZE as i32,
            id.z * CHUNK_SIZE as i32,
        )
    }
}

pub struct World {
    pub seed: WorldSeed,
    pub chunk_manager: ChunkManager,
}

impl World {
    pub fn random() -> Self {
        Self {
            seed: WorldSeed::new(thread_rng().gen()),
            chunk_manager: ChunkManager::new(),
        }
    }

    pub fn new(seed: WorldSeed) -> Self {
        Self {
            seed,
            chunk_manager: ChunkManager::new(),
        }
    }

    pub fn intersects_point(&self, p: [f32; 3]) -> bool {
        let id = ChunkId::new(
            p[0] as i32 / CHUNK_SIZE as i32,
            p[1] as i32 / CHUNK_SIZE as i32,
            p[2] as i32 / CHUNK_SIZE as i32,
        );

        let position_in_chunk = [
            (p[0] as i32 % CHUNK_SIZE as i32) as usize,
            (p[1] as i32 % CHUNK_SIZE as i32) as usize,
            (p[2] as i32 % CHUNK_SIZE as i32) as usize,
        ];

        if let Some(chunk) = self.chunk_manager.get(&id) {
            chunk
                .get(
                    position_in_chunk[0],
                    position_in_chunk[1],
                    position_in_chunk[2],
                )
                .is_solid()
        } else {
            false
        }
    }

    pub fn reset(&mut self) {
        self.chunk_manager.reset();
    }
}

pub trait Raycast {
    fn cast_ray(&self, ray: &Ray, limit: &Range<f32>) -> Option<f32>;
}

impl Raycast for World {
    fn cast_ray(&self, ray: &Ray, limit: &Range<f32>) -> Option<f32> {
        // println!("Ray: {:?}", ray);
        let step_x = if ray.direction.x >= 0.0 { 1 } else { -1 };
        let step_y = if ray.direction.y >= 0.0 { 1 } else { -1 };
        let step_z = if ray.direction.z >= 0.0 { 1 } else { -1 };

        let mut chunk_id = ChunkId::from(&ray.origin);
        let mut distance = 0.0;

        while limit.contains(&distance) {
            if let Some(chunk_data) = self.chunk_manager.get(&chunk_id) {
                let chunk_pos = glm::Vec3::from(&chunk_id);
                let chunk_ray = Ray::new(ray.origin - chunk_pos, ray.direction);
                if let Some(distance) = chunk_data.cast_ray(&chunk_ray, limit) {
                    return Some(distance);
                }
            }

            let x_chunk_id = ChunkId::new(chunk_id.x + step_x, chunk_id.y, chunk_id.z);
            let x_aabb = AABB::from(&x_chunk_id);
            if let Some(step) = ray.collides_with_aabb(&x_aabb) {
                distance += step;
                chunk_id = x_chunk_id;
                continue;
            }

            let y_chunk_id = ChunkId::new(chunk_id.x, chunk_id.y + step_y, chunk_id.z);
            let y_aabb = AABB::from(&y_chunk_id);
            if let Some(step) = ray.collides_with_aabb(&y_aabb) {
                distance += step;
                chunk_id = y_chunk_id;
                continue;
            }

            let z_chunk_id = ChunkId::new(chunk_id.x, chunk_id.y, chunk_id.z + step_z);
            let z_aabb = AABB::from(&z_chunk_id);
            if let Some(step) = ray.collides_with_aabb(&z_aabb) {
                distance += step;
                chunk_id = z_chunk_id;
                continue;
            }

            break;
        }

        None
    }
}

type TopNode<T> = L6Node<T>;

#[derive(Clone, Default)]
pub struct ChunkData(TopNode<Material>, usize);

impl Data3D<Material> for ChunkData {
    fn set(&mut self, x: usize, y: usize, z: usize, value: Material) {
        self.0.set(x, y, z, value);
        self.1 += value.is_solid() as usize;
    }

    fn get(&self, x: usize, y: usize, z: usize) -> Material {
        if x >= CHUNK_SIZE || y >= CHUNK_SIZE || z >= CHUNK_SIZE {
            Default::default()
        } else {
            self.0.get(x, y, z).unwrap_or_default()
        }
    }
}

impl Raycast for ChunkData {
    fn cast_ray(&self, ray: &Ray, limit: &Range<f32>) -> Option<f32> {
        self.0.cast_ray(&ray, limit)
    }
}

impl ChunkData {
    pub fn is_empty(&self) -> bool {
        match self.0 {
            TopNode::Empty => true,
            _ => false,
        }
    }

    pub fn needs_mesh(&self) -> bool {
        match self.0 {
            TopNode::<Material>::Empty => false,
            TopNode::<Material>::Full(m) => !m.is_opaque(),
            TopNode::Sparse(_) => self.1 != CHUNK_SIZE_CUBED,
        }
    }
}

fn get_child_aabbs(size: usize) -> [AABB; 8] {
    let half = (size / 2) as f32;
    let extend = Vec3::new(half, half, half);

    [
        AABB::with_size(Vec3::new(0.0, 0.0, 0.0), extend),
        AABB::with_size(Vec3::new(half, 0.0, 0.0), extend),
        AABB::with_size(Vec3::new(0.0, half, 0.0), extend),
        AABB::with_size(Vec3::new(half, half, 0.0), extend),
        AABB::with_size(Vec3::new(0.0, 0.0, half), extend),
        AABB::with_size(Vec3::new(half, 0.0, half), extend),
        AABB::with_size(Vec3::new(0.0, half, half), extend),
        AABB::with_size(Vec3::new(half, half, half), extend),
    ]
}

impl Raycast for L1Node<Material> {
    fn cast_ray(&self, ray: &Ray, limit: &Range<f32>) -> Option<f32> {
        let aabb = AABB::new_cube(2.0);
        if let Some(entry_distance) = ray.collides_with_aabb(&aabb) {
            let distance = match &self {
                Self::Empty => None,
                Self::Full(m) => solid_distance_or_none(m, entry_distance),
                Self::Sparse(children) => {
                    let child_aabbs = get_child_aabbs(2);
                    let hits = get_leaf_hits(children, child_aabbs, ray);
                    hits.into_iter().next()
                }
            };

            distance.filter(|d| limit.contains(d))
        } else {
            None
        }
    }
}

impl Raycast for L2Node<Material> {
    fn cast_ray(&self, ray: &Ray, limit: &Range<f32>) -> Option<f32> {
        let aabb = AABB::new_cube(4.0);
        if let Some(entry_distance) = ray.collides_with_aabb(&aabb) {
            let distance = match &self {
                Self::Empty => None,
                Self::Full(m) => solid_distance_or_none(m, entry_distance),
                Self::Sparse(children) => cast_in_children(children, ray, 4, limit),
            };

            distance.filter(|d| limit.contains(d))
        } else {
            None
        }
    }
}

impl Raycast for L3Node<Material> {
    fn cast_ray(&self, ray: &Ray, limit: &Range<f32>) -> Option<f32> {
        let aabb = AABB::new_cube(8.0);
        if let Some(entry_distance) = ray.collides_with_aabb(&aabb) {
            let distance = match &self {
                Self::Empty => None,
                Self::Full(m) => solid_distance_or_none(m, entry_distance),
                Self::Sparse(children) => cast_in_children(children, ray, 8, limit),
            };

            distance.filter(|d| limit.contains(d))
        } else {
            None
        }
    }
}

impl Raycast for L4Node<Material> {
    fn cast_ray(&self, ray: &Ray, limit: &Range<f32>) -> Option<f32> {
        let aabb = AABB::new_cube(16.0);
        if let Some(entry_distance) = ray.collides_with_aabb(&aabb) {
            let distance = match &self {
                Self::Empty => None,
                Self::Full(m) => solid_distance_or_none(m, entry_distance),
                Self::Sparse(children) => cast_in_children(children, ray, 16, limit),
            };

            distance.filter(|d| limit.contains(d))
        } else {
            None
        }
    }
}

impl Raycast for L5Node<Material> {
    fn cast_ray(&self, ray: &Ray, limit: &Range<f32>) -> Option<f32> {
        let aabb = AABB::new_cube(32.0);
        if let Some(entry_distance) = ray.collides_with_aabb(&aabb) {
            let distance = match &self {
                Self::Empty => None,
                Self::Full(m) => solid_distance_or_none(m, entry_distance),
                Self::Sparse(children) => cast_in_children(children, ray, 32, limit),
            };

            distance.filter(|d| limit.contains(d))
        } else {
            None
        }
    }
}

impl Raycast for L6Node<Material> {
    fn cast_ray(&self, ray: &Ray, limit: &Range<f32>) -> Option<f32> {
        let aabb = AABB::new_cube(64.0);
        if let Some(entry_distance) = ray.collides_with_aabb(&aabb) {
            let distance = match &self {
                Self::Empty => None,
                Self::Full(m) => solid_distance_or_none(m, entry_distance),
                Self::Sparse(children) => cast_in_children(children, ray, 64, limit),
            };

            distance.filter(|d| limit.contains(d))
        } else {
            None
        }
    }
}

fn solid_distance_or_none(m: &Material, entry_distance: f32) -> Option<f32> {
    if m.is_solid() {
        Some(entry_distance)
    } else {
        None
    }
}

fn cast_in_children<T>(
    children: &Box<[T; 8]>,
    ray: &Ray,
    size: usize,
    limit: &Range<f32>,
) -> Option<f32>
where
    T: Raycast,
{
    let child_aabbs = get_child_aabbs(size);
    let hits = get_child_hits(children, child_aabbs, ray);
    let child_rays = get_child_rays(ray, size as f32);
    hits.into_iter()
        .find_map(|(child, index)| child.cast_ray(&child_rays[index], limit))
}

fn get_child_rays(ray: &Ray, size: f32) -> [Ray; 8] {
    let half = size / 2.0;
    [
        Ray::new(ray.origin + Vec3::new(0.0, 0.0, 0.0), ray.direction),
        Ray::new(ray.origin + Vec3::new(-half, 0.0, 0.0), ray.direction),
        Ray::new(ray.origin + Vec3::new(0.0, -half, 0.0), ray.direction),
        Ray::new(ray.origin + Vec3::new(-half, -half, 0.0), ray.direction),
        Ray::new(ray.origin + Vec3::new(0.0, 0.0, -half), ray.direction),
        Ray::new(ray.origin + Vec3::new(-half, 0.0, -half), ray.direction),
        Ray::new(ray.origin + Vec3::new(0.0, -half, -half), ray.direction),
        Ray::new(ray.origin + Vec3::new(-half, -half, -half), ray.direction),
    ]
}

fn get_leaf_hits(children: &[Option<Material>; 8], child_aabbs: [AABB; 8], ray: &Ray) -> Vec<f32> {
    let mut hits = Vec::with_capacity(4);
    for i in 0..8 {
        let child = children[i];
        let aabb = &child_aabbs[i];

        if child.map_or(false, |m| m.is_solid()) {
            if let Some(distance) = ray.collides_with_aabb(aabb) {
                hits.push(distance);
            }
        }
    }
    hits.sort_unstable_by(|a, b| a.total_cmp(&b));
    hits
}

fn get_child_hits<'a, T>(
    children: &'a [T; 8],
    child_aabbs: [AABB; 8],
    ray: &Ray,
) -> Vec<(&'a T, usize)> {
    let mut hit_children = Vec::<(usize, &T, f32)>::with_capacity(4);
    for i in 0..8 {
        let child = &children[i];
        let aabb = &child_aabbs[i];
        if let Some(distance) = ray.collides_with_aabb(aabb) {
            hit_children.push((i, child, distance));
        }
    }

    hit_children.sort_unstable_by(|a, b| a.2.total_cmp(&b.2));
    hit_children.into_iter().map(|(i, c, ..)| (c, i)).collect()
}

#[cfg(test)]
mod tests {
    use std::ops::Range;

    use gamedata::material::Material;
    use geometry::Ray;
    use nalgebra_glm::Vec3;
    use octree::{L1Node, L2Node, L3Node, L4Node, L5Node, L6Node};

    use crate::{ChunkData, ChunkId, Raycast, World, CHUNK_SIZE_CUBED};

    mod nodes {

        use gamedata::material::Material;
        use octree::{L1Node, L2Node};

        pub(crate) const L1_FULL_STONE: L1Node<Material> = L1Node::Full(Material::Stone);
        pub(crate) const L1_FULL_AIR: L1Node<Material> = L1Node::Full(Material::Air);
        pub(crate) const L1_EMPTY: L1Node<Material> = L1Node::Empty;
        pub(crate) const L1_SPARSE_0: L1Node<Material> = L1Node::Sparse([
            Some(Material::Stone),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        ]);
        pub(crate) const L1_SPARSE_1: L1Node<Material> = L1Node::Sparse([
            None,
            Some(Material::Stone),
            None,
            None,
            None,
            None,
            None,
            None,
        ]);
        pub(crate) const L1_SPARSE_2: L1Node<Material> = L1Node::Sparse([
            None,
            None,
            Some(Material::Stone),
            None,
            None,
            None,
            None,
            None,
        ]);
        pub(crate) const L1_SPARSE_3: L1Node<Material> = L1Node::Sparse([
            None,
            None,
            None,
            Some(Material::Stone),
            None,
            None,
            None,
            None,
        ]);
        pub(crate) const L1_SPARSE_4: L1Node<Material> = L1Node::Sparse([
            None,
            None,
            None,
            None,
            Some(Material::Stone),
            None,
            None,
            None,
        ]);
        pub(crate) const L1_SPARSE_5: L1Node<Material> = L1Node::Sparse([
            None,
            None,
            None,
            None,
            None,
            Some(Material::Stone),
            None,
            None,
        ]);
        pub(crate) const L1_SPARSE_6: L1Node<Material> = L1Node::Sparse([
            None,
            None,
            None,
            None,
            None,
            None,
            Some(Material::Stone),
            None,
        ]);
        pub(crate) const L1_SPARSE_7: L1Node<Material> = L1Node::Sparse([
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            Some(Material::Stone),
        ]);

        pub(crate) const L2_EMPTY: L2Node<Material> = L2Node::Empty;
        pub(crate) const L2_FULL_AIR: L2Node<Material> = L2Node::Full(Material::Air);
        pub(crate) const L2_FULL_STONE: L2Node<Material> = L2Node::Full(Material::Stone);
    }

    const LIMITS: Range<f32> = 0.0..100.0;

    #[test]
    fn cast_into_node1_full() {
        let ray = Ray::new(Vec3::new(-1.0, 1.0, 1.0), Vec3::new(1.0, 0.0, 0.0));
        assert_eq!(nodes::L1_FULL_STONE.cast_ray(&ray, &LIMITS), Some(1.0));
        assert_eq!(nodes::L1_FULL_AIR.cast_ray(&ray, &LIMITS), None);
    }

    #[test]
    fn cast_into_node1_empty() {
        let ray = Ray::new(Vec3::new(-1.0, 1.0, 1.0), Vec3::new(1.0, 0.0, 0.0));
        assert_eq!(nodes::L1_EMPTY.cast_ray(&ray, &LIMITS), None);
    }

    #[test]
    fn cast_into_node1_sparse() {
        let ray_x0 = Ray::new(Vec3::new(-1.0, 0.5, 0.5), Vec3::new(1.0, 0.0, 0.0));
        let ray_x1 = Ray::new(Vec3::new(-1.0, 1.5, 0.5), Vec3::new(1.0, 0.0, 0.0));
        let ray_x2 = Ray::new(Vec3::new(-1.0, 0.5, 1.5), Vec3::new(1.0, 0.0, 0.0));
        let ray_x3 = Ray::new(Vec3::new(-1.0, 1.5, 1.5), Vec3::new(1.0, 0.0, 0.0));

        assert_eq!(nodes::L1_SPARSE_0.cast_ray(&ray_x0, &LIMITS), Some(1.0));
        assert_eq!(nodes::L1_SPARSE_1.cast_ray(&ray_x0, &LIMITS), Some(2.0));
        assert_eq!(nodes::L1_SPARSE_2.cast_ray(&ray_x0, &LIMITS), None);
        assert_eq!(nodes::L1_SPARSE_3.cast_ray(&ray_x0, &LIMITS), None);
        assert_eq!(nodes::L1_SPARSE_4.cast_ray(&ray_x0, &LIMITS), None);
        assert_eq!(nodes::L1_SPARSE_5.cast_ray(&ray_x0, &LIMITS), None);
        assert_eq!(nodes::L1_SPARSE_6.cast_ray(&ray_x0, &LIMITS), None);
        assert_eq!(nodes::L1_SPARSE_7.cast_ray(&ray_x0, &LIMITS), None);

        assert_eq!(nodes::L1_SPARSE_0.cast_ray(&ray_x1, &LIMITS), None);
        assert_eq!(nodes::L1_SPARSE_1.cast_ray(&ray_x1, &LIMITS), None);
        assert_eq!(nodes::L1_SPARSE_2.cast_ray(&ray_x1, &LIMITS), Some(1.0));
        assert_eq!(nodes::L1_SPARSE_3.cast_ray(&ray_x1, &LIMITS), Some(2.0));
        assert_eq!(nodes::L1_SPARSE_4.cast_ray(&ray_x1, &LIMITS), None);
        assert_eq!(nodes::L1_SPARSE_5.cast_ray(&ray_x1, &LIMITS), None);
        assert_eq!(nodes::L1_SPARSE_6.cast_ray(&ray_x1, &LIMITS), None);
        assert_eq!(nodes::L1_SPARSE_7.cast_ray(&ray_x1, &LIMITS), None);

        assert_eq!(nodes::L1_SPARSE_0.cast_ray(&ray_x2, &LIMITS), None);
        assert_eq!(nodes::L1_SPARSE_1.cast_ray(&ray_x2, &LIMITS), None);
        assert_eq!(nodes::L1_SPARSE_2.cast_ray(&ray_x2, &LIMITS), None);
        assert_eq!(nodes::L1_SPARSE_3.cast_ray(&ray_x2, &LIMITS), None);
        assert_eq!(nodes::L1_SPARSE_4.cast_ray(&ray_x2, &LIMITS), Some(1.0));
        assert_eq!(nodes::L1_SPARSE_5.cast_ray(&ray_x2, &LIMITS), Some(2.0));
        assert_eq!(nodes::L1_SPARSE_6.cast_ray(&ray_x2, &LIMITS), None);
        assert_eq!(nodes::L1_SPARSE_7.cast_ray(&ray_x2, &LIMITS), None);

        assert_eq!(nodes::L1_SPARSE_0.cast_ray(&ray_x3, &LIMITS), None);
        assert_eq!(nodes::L1_SPARSE_1.cast_ray(&ray_x3, &LIMITS), None);
        assert_eq!(nodes::L1_SPARSE_2.cast_ray(&ray_x3, &LIMITS), None);
        assert_eq!(nodes::L1_SPARSE_3.cast_ray(&ray_x3, &LIMITS), None);
        assert_eq!(nodes::L1_SPARSE_4.cast_ray(&ray_x3, &LIMITS), None);
        assert_eq!(nodes::L1_SPARSE_5.cast_ray(&ray_x3, &LIMITS), None);
        assert_eq!(nodes::L1_SPARSE_6.cast_ray(&ray_x3, &LIMITS), Some(1.0));
        assert_eq!(nodes::L1_SPARSE_7.cast_ray(&ray_x3, &LIMITS), Some(2.0));
    }

    #[test]
    fn cast_into_node2_full() {
        let ray = Ray::new(Vec3::new(-1.0, 1.0, 1.0), Vec3::new(1.0, 0.0, 0.0));
        assert_eq!(nodes::L2_FULL_STONE.cast_ray(&ray, &LIMITS), Some(1.0));
        assert_eq!(nodes::L2_FULL_AIR.cast_ray(&ray, &LIMITS), None);
    }

    #[test]
    fn cast_into_node2_empty() {
        let ray = Ray::new(Vec3::new(-1.0, 1.0, 1.0), Vec3::new(1.0, 0.0, 0.0));
        assert_eq!(nodes::L2_EMPTY.cast_ray(&ray, &LIMITS), None);
    }

    #[test]
    fn cast_ray_into_node2_sparse() {
        let ray = Ray::new(Vec3::new(-1.0, 0.5, 0.5), Vec3::new(1.0, 0.0, 0.0));
        let l2_sparse_0: L2Node<Material> = L2Node::Sparse(Box::new([
            nodes::L1_FULL_STONE,
            nodes::L1_EMPTY,
            nodes::L1_EMPTY,
            nodes::L1_EMPTY,
            nodes::L1_EMPTY,
            nodes::L1_EMPTY,
            nodes::L1_EMPTY,
            nodes::L1_EMPTY,
        ]));
        let l2_sparse_1: L2Node<Material> = L2Node::Sparse(Box::new([
            nodes::L1_EMPTY,
            nodes::L1_FULL_STONE,
            nodes::L1_EMPTY,
            nodes::L1_EMPTY,
            nodes::L1_EMPTY,
            nodes::L1_EMPTY,
            nodes::L1_EMPTY,
            nodes::L1_EMPTY,
        ]));

        let l2_sparse_mixed: L2Node<Material> = L2Node::Sparse(Box::new([
            nodes::L1_EMPTY,
            nodes::L1_SPARSE_1,
            nodes::L1_EMPTY,
            nodes::L1_EMPTY,
            nodes::L1_EMPTY,
            nodes::L1_EMPTY,
            nodes::L1_EMPTY,
            nodes::L1_EMPTY,
        ]));

        assert_eq!(l2_sparse_0.cast_ray(&ray, &LIMITS), Some(1.0));
        assert_eq!(l2_sparse_1.cast_ray(&ray, &LIMITS), Some(3.0));
        assert_eq!(l2_sparse_mixed.cast_ray(&ray, &LIMITS), Some(4.0));
    }

    #[test]
    fn cast_into_l6() {
        let node = L6Node::Sparse(Box::new([
            L5Node::Empty,
            L5Node::Empty,
            L5Node::Empty,
            L5Node::Empty,
            L5Node::Empty,
            L5Node::Empty,
            L5Node::Empty,
            L5Node::Sparse(Box::new([
                L4Node::Empty,
                L4Node::Empty,
                L4Node::Empty,
                L4Node::Empty,
                L4Node::Empty,
                L4Node::Empty,
                L4Node::Empty,
                L4Node::Sparse(Box::new([
                    L3Node::Empty,
                    L3Node::Empty,
                    L3Node::Empty,
                    L3Node::Empty,
                    L3Node::Empty,
                    L3Node::Empty,
                    L3Node::Empty,
                    L3Node::Sparse(Box::new([
                        L2Node::Empty,
                        L2Node::Empty,
                        L2Node::Empty,
                        L2Node::Empty,
                        L2Node::Empty,
                        L2Node::Empty,
                        L2Node::Empty,
                        L2Node::Sparse(Box::new([
                            L1Node::Empty,
                            L1Node::Empty,
                            L1Node::Empty,
                            L1Node::Empty,
                            L1Node::Empty,
                            L1Node::Empty,
                            L1Node::Empty,
                            L1Node::Sparse([
                                None,
                                None,
                                None,
                                None,
                                None,
                                None,
                                None,
                                Some(Material::Stone),
                            ]),
                        ])),
                    ])),
                ])),
            ])),
        ]));

        let ray = Ray::new(Vec3::new(-1.0, 63.5, 63.5), Vec3::new(1.0, 0.0, 0.0));

        assert_eq!(node.cast_ray(&ray, &LIMITS), Some(64.0));
    }

    #[test]
    fn cast_into_full_negative_chunk() {
        let mut world = World::random();
        let chunk_id = ChunkId::new(-2, -2, -2);
        let data = ChunkData(L6Node::Full(Material::Stone), CHUNK_SIZE_CUBED);
        let ray = Ray::new(Vec3::new(32.0, 32.0, 32.0), Vec3::new(-1.0, -1.0, -1.0));

        world.chunk_manager.insert(&chunk_id, data);

        let hit = world
            .cast_ray(&ray, &LIMITS)
            .map(|distance| ray.point_on_ray(distance));
        assert_eq!(hit, Some(Vec3::new(-64.0, -64.0, -64.0)));
    }

    #[test]
    fn cast_into_sparse_negative_chunk() {
        let mut world = World::random();
        let chunk_id = ChunkId::new(-2, -2, -2);
        let data = ChunkData(L6Node::Full(Material::Stone), CHUNK_SIZE_CUBED);
        let ray = Ray::new(Vec3::new(32.0, 32.0, 32.0), Vec3::new(-1.0, -1.0, -1.0));

        world.chunk_manager.insert(&chunk_id, data);

        let hit = world
            .cast_ray(&ray, &LIMITS)
            .map(|distance| ray.point_on_ray(distance));
        assert_eq!(hit, Some(Vec3::new(-64.0, -64.0, -64.0)));
    }
}
