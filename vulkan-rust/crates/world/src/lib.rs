#![feature(test)]
#![feature(generic_const_exprs)]

extern crate nalgebra_glm as glm;
extern crate test;

pub mod chunk_generator;
pub mod chunk_id;
pub mod chunk_manager;
pub mod mesh_generator;
pub mod mesh_manager;
pub mod terrain_noise;
pub mod world_parameters;
pub mod world_position;

use std::collections::BTreeSet;

use chunk_generator::ChunkGenerator;
use chunk_id::ChunkId;
use chunk_manager::ChunkManager;
use gamedata::material::Material;
use graphics::AABB;
use mesh_manager::MeshManager;
use octree::{L6Node, LeafAccess};
use world_position::WorldPosition;

pub const CHUNK_SIZE: usize = L6Node::<Material>::SIZE;
pub const CHUNK_SIZE_SQUARED: usize = CHUNK_SIZE * CHUNK_SIZE;
pub const CHUNK_SIZE_CUBED: usize = CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE;

pub const CHUNK_SIZE_I: i32 = CHUNK_SIZE as i32;
pub const CHUNK_SIZE_SQUARED_I: i32 = CHUNK_SIZE_SQUARED as i32;
pub const CHUNK_SIZE_CUBED_I: i32 = CHUNK_SIZE_CUBED as i32;

pub const CHUNK_SIZE_SAFE: usize = CHUNK_SIZE + 2;
pub const CHUNK_SIZE_SAFE_SQUARED: usize = CHUNK_SIZE_SAFE * CHUNK_SIZE_SAFE;
pub const CHUNK_SIZE_SAFE_CUBED: usize = CHUNK_SIZE_SAFE * CHUNK_SIZE_SAFE * CHUNK_SIZE_SAFE;

pub type ChunkDataCube = [[[Material; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE];

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
    pub generator: ChunkGenerator,
    pub chunk_manager: ChunkManager,
    pub mesh_manager: MeshManager,
}

impl World {
    pub fn new() -> Self {
        Self {
            generator: ChunkGenerator::new(),
            chunk_manager: ChunkManager::new(),
            mesh_manager: MeshManager::new(),
        }
    }

    pub fn ids(&self) -> &BTreeSet<ChunkId> {
        self.chunk_manager.ids()
    }

    pub fn get_chunks(&self) -> Vec<(&ChunkId, &Box<ChunkData>)> {
        self.chunk_manager.get_all()
    }

    pub fn load(&mut self, id: &ChunkId) {
        let data = self.generator.generate(id);
        let compact = self.generator.compress(&data.0);
        self.chunk_manager.insert_data(id, compact);
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

        if let Some(chunk) = self.chunk_manager.get_data(&id) {
            if let Some(material) = chunk.get(
                position_in_chunk[0],
                position_in_chunk[1],
                position_in_chunk[2],
            ) {
                material.is_solid()
            } else {
                false
            }
        } else {
            false
        }
    }
}

#[derive(Clone)]
pub struct ChunkData(L6Node<Material>);

impl ChunkData {
    pub fn default() -> Self {
        Self(L6Node::Empty)
    }

    pub fn get(&self, x: usize, y: usize, z: usize) -> Option<Material> {
        if x >= CHUNK_SIZE || y >= CHUNK_SIZE || z >= CHUNK_SIZE {
            None // todo remove
        } else {
            self.0.get(x, y, z)
        }
    }

    pub fn set(&mut self, x: usize, y: usize, z: usize, m: Material) {
        self.0.set(x, y, z, m)
    }

    pub fn is_empty(&self) -> bool {
        match self {
            Self(L6Node::Empty) => true,
            _ => false,
        }
    }

    pub fn needs_mesh(&self) -> bool {
        match self {
            Self(L6Node::<Material>::Empty) => false,
            Self(L6Node::<Material>::Full(m)) => !m.is_opaque(),
            _ => true,
        }
    }
}
