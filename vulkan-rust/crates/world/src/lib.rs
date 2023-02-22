#![feature(test)]

extern crate test;

pub mod chunk_generator;
pub mod chunk_manager;
pub mod fixed_tree;

use chunk_generator::ChunkGenerator;
use chunk_manager::{ChunkId, ChunkManager};
use fixed_tree::{ChunkData, VoxelData6};
use gamedata::material::Material;

pub const CHUNK_SIZE: usize = VoxelData6::SIZE;
pub const CHUNK_SIZE_SQUARED: usize = CHUNK_SIZE * CHUNK_SIZE;
pub const CHUNK_SIZE_CUBED: usize = CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE;

pub const CHUNK_SIZE_SAFE: usize = CHUNK_SIZE + 2;
pub const CHUNK_SIZE_SAFE_SQUARED: usize = CHUNK_SIZE_SAFE * CHUNK_SIZE_SAFE;
pub const CHUNK_SIZE_SAFE_CUBED: usize = CHUNK_SIZE_SAFE * CHUNK_SIZE_SAFE * CHUNK_SIZE_SAFE;

pub type ChunkDataCube = [[[Material; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE];

pub struct World {
    generator: ChunkGenerator,
    manager: ChunkManager,
}

impl World {
    pub fn new() -> Self {
        Self {
            generator: ChunkGenerator::new(),
            manager: ChunkManager::new(),
        }
    }

    pub fn get_chunks(&self) -> Vec<(&ChunkId, &Box<ChunkData>)> {
        self.manager.get_all()
    }

    pub fn load(&mut self, id: &ChunkId) {
        let data = self.generator.generate(id);
        self.manager.insert(id, data);
    }
}
