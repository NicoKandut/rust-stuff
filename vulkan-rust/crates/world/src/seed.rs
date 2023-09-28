use rand::{thread_rng, Rng};

use crate::{chunk_id::ChunkId, world_position::WorldPosition};

#[derive(Clone)]
pub struct WorldSeed(u64);

impl WorldSeed {
    pub const fn new(seed: u64) -> Self {
        Self(seed)
    }

    pub fn random() -> Self {
        Self(thread_rng().gen())
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

#[derive(Clone)]
pub struct ChunkSeed {
    world_seed: WorldSeed,
    id: ChunkId,
}

impl ChunkSeed {
    pub fn new(world_seed: &WorldSeed, id: &ChunkId) -> Self {
        Self {
            world_seed: world_seed.clone(),
            id: id.clone(),
        }
    }

    pub fn id(&self) -> &ChunkId {
        &self.id
    }

    pub fn value(&self) -> u64 {
        let x = (self.id.x as u64) << 0;
        let y = (self.id.y as u64) << 16;
        let z = (self.id.z as u64) << 32;
        let w = self.world_seed.0;

        w ^ x ^ y ^ z
    }

    pub fn world_seed(&self) -> &WorldSeed {
        &self.world_seed
    }
}
