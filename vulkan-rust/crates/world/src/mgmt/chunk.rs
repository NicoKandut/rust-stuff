use std::collections::HashMap;

use gamedata::material::Material;

use crate::{chunk_id::ChunkId, traits::Data3D, ChunkData, WorldPosition, CHUNK_SIZE_I};

pub struct ChunkManager {
    chunks: HashMap<ChunkId, ChunkData>,
}

impl ChunkManager {
    pub fn new() -> Self {
        Self {
            chunks: Default::default(),
        }
    }

    pub fn len(&self) -> usize {
        self.chunks.len()
    }

    pub fn insert(&mut self, id: &ChunkId, data: ChunkData) {
        self.chunks.insert(id.clone(), data);
    }

    pub fn set_block(&mut self, x: i32, y: i32, z: i32, m: Material) -> Result<ChunkId, ()> {
        let position = WorldPosition::new(x, y, z);
        let id = ChunkId::from(&position);
        let pos_in_chunk = position.rem_euclid(CHUNK_SIZE_I);
        if let Some(data) = self.chunks.get_mut(&id) {
            data.set(
                pos_in_chunk.x as usize,
                pos_in_chunk.y as usize,
                pos_in_chunk.z as usize,
                m,
            );
            Ok(id)
        } else {
            Err(())
        }
    }

    pub fn remove(&mut self, id: &ChunkId) {
        self.chunks.remove(id);
    }

    pub fn get(&self, id: &ChunkId) -> Option<&ChunkData> {
        match self.chunks.get(id) {
            Some(chunk_box) => Some(chunk_box),
            None => None,
        }
    }

    pub fn get_mut(&mut self, id: &ChunkId) -> Option<&mut ChunkData> {
        match self.chunks.get_mut(id) {
            Some(chunk_box) => Some(chunk_box),
            None => None,
        }
    }

    pub(crate) fn reset(&mut self) {
        self.chunks.clear();
    }
}

#[cfg(test)]
mod test {
    use super::{ChunkId, ChunkManager};
    use crate::ChunkData;

    #[test]
    fn insert() {
        let mut cm = ChunkManager::new();
        cm.insert(&ChunkId::new(0, 0, 0), ChunkData::default());
        assert_eq!(cm.len(), 1);
    }

    #[test]
    fn get() {
        let mut cm = ChunkManager::new();
        let id = ChunkId::new(0, 0, 0);
        cm.insert(&id, ChunkData::default());
        let chunk = cm.get(&id);
        assert!(chunk.is_some());

        let id2 = ChunkId::new(0, 1, 0);
        let chunk = cm.get(&id2);
        assert!(chunk.is_none());
    }
}
