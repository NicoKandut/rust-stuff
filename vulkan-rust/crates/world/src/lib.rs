#![feature(test)]

extern crate nalgebra_glm as glm;
extern crate test;

pub mod chunk_generator;
pub mod chunk_manager;
pub mod mesh_generator;
pub mod terrain_noise;

use std::collections::BTreeSet;

use chunk_generator::ChunkGenerator;
use chunk_manager::{ChunkId, ChunkManager};
use gamedata::material::Material;
use octree::{L6Node, LeafAccess};

pub const CHUNK_SIZE: usize = L6Node::<Material>::SIZE;
pub const CHUNK_SIZE_SQUARED: usize = CHUNK_SIZE * CHUNK_SIZE;
pub const CHUNK_SIZE_CUBED: usize = CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE;

pub const CHUNK_SIZE_SAFE: usize = CHUNK_SIZE + 2;
pub const CHUNK_SIZE_SAFE_SQUARED: usize = CHUNK_SIZE_SAFE * CHUNK_SIZE_SAFE;
pub const CHUNK_SIZE_SAFE_CUBED: usize = CHUNK_SIZE_SAFE * CHUNK_SIZE_SAFE * CHUNK_SIZE_SAFE;

pub type ChunkDataCube = [[[Material; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE];

pub struct World {
    pub generator: ChunkGenerator,
    pub manager: ChunkManager,
}

impl World {
    pub fn new() -> Self {
        Self {
            generator: ChunkGenerator::new(),
            manager: ChunkManager::new(),
        }
    }

    pub fn ids(&self) -> &BTreeSet<ChunkId> {
        self.manager.ids()
    }

    pub fn get_chunks(&self) -> Vec<(&ChunkId, &Box<ChunkData>)> {
        self.manager.get_all()
    }

    pub fn load(&mut self, id: &ChunkId) {
        let data = self.generator.generate(id);
        self.manager.insert_data(id, data);
    }

    pub fn intersects_point(&self, p: [f32; 3]) -> bool {
        let id = ChunkId::new(
            p[0] as i32 / CHUNK_SIZE as i32,
            p[1] as i32 / CHUNK_SIZE as i32,
            p[2] as i32 / CHUNK_SIZE as i32,
        );

        let pos = [
            ((p[0] - id.x as f32) as usize + CHUNK_SIZE) % CHUNK_SIZE,
            ((p[1] - id.y as f32) as usize + CHUNK_SIZE) % CHUNK_SIZE,
            ((p[2] - id.z as f32) as usize + CHUNK_SIZE) % CHUNK_SIZE,
        ];

        if let Some(chunk) = self.manager.get_data(&id) {
            if let Some(material) = chunk.get(pos[0], pos[1], pos[2]) {
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
