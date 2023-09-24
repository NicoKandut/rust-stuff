use crate::{chunk_id::ChunkId, world_position::WorldPosition};

#[derive(Clone)]
pub struct WorldSeed(u64);

impl WorldSeed {
    pub const fn new(seed: u64) -> Self {
        Self(seed)
    }
}

impl From<&WorldSeed> for u64 {
    fn from(value: &WorldSeed) -> Self {
        value.0
    }
}

impl From<&WorldSeed> for i32 {
    fn from(value: &WorldSeed) -> Self {
        value.0 as i32
    }
}

#[derive(Clone)]
pub struct PositionalSeed {
    world_seed: WorldSeed,
    position: WorldPosition,
}

impl PositionalSeed {
    pub fn new(world_seed: &WorldSeed, position: &WorldPosition) -> Self {
        Self {
            world_seed: world_seed.clone(),
            position: position.clone(),
        }
    }

    pub fn for_chunk(world_seed: &WorldSeed, chunk_id: &ChunkId) -> Self {
        Self {
            world_seed: world_seed.clone(),
            position: WorldPosition::from(chunk_id),
        }
    }

    pub fn pos(&self) -> &WorldPosition {
        &self.position
    }

    pub fn value(&self) -> u64 {
        let x = (self.position.x as u64) << 0;
        let y = (self.position.y as u64) << 16;
        let z = (self.position.z as u64) << 32;
        let w = self.world_seed.0;

        w ^ x ^ y ^ z
    }

    pub fn world_seed(&self) -> &WorldSeed {
        &self.world_seed
    }
}
