#![feature(test)]

extern crate test;

pub mod chunk_generator;
pub mod chunk_manager;
pub mod fixed_tree;
pub mod terrain_noise;

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

    pub fn get_chunks(&self) -> Vec<(&ChunkId, &Box<ChunkData>)> {
        self.manager.get_all()
    }

    pub fn load(&mut self, id: &ChunkId) {
        let data = self.generator.generate(id);
        self.manager.insert(id, data);
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

        if let Some(chunk) = self.manager.get(&id) {
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
