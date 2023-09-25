use std::collections::{BTreeMap, BTreeSet};

use crate::{chunk_id::ChunkId, ChunkData};

pub struct ChunkManager {
    ids: BTreeSet<ChunkId>,
    chunks: BTreeMap<ChunkId, Box<ChunkData>>,
}

impl ChunkManager {
    pub fn new() -> Self {
        Self {
            ids: BTreeSet::new(),
            chunks: BTreeMap::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.chunks.len()
    }

    pub fn insert_data(&mut self, id: &ChunkId, data: ChunkData) {
        self.chunks.insert(id.clone(), Box::new(data));
    }

    pub fn set_requested(&mut self, id: &ChunkId) {
        self.ids.insert(id.clone());
    }

    pub fn remove(&mut self, id: &ChunkId) {
        self.ids.remove(id);
    }

    pub fn get_data(&self, id: &ChunkId) -> Option<&ChunkData> {
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

    pub fn ids(&self) -> &BTreeSet<ChunkId> {
        &self.ids
    }

    pub(crate) fn get_all(&self) -> Vec<(&ChunkId, &Box<ChunkData>)> {
        self.chunks.iter().filter(|c| !c.1.is_empty()).collect()
    }

    pub(crate) fn reset(&mut self) {
        self.ids.clear();
        self.chunks.clear();
    }
}

#[cfg(test)]
mod test {
    use super::{ChunkId, ChunkManager};
    use crate::{traits::Data3D, ChunkData};
    use gamedata::material::Material;

    #[test]
    fn insert() {
        let mut cm = ChunkManager::new();
        cm.insert_data(&ChunkId::new(0, 0, 0), ChunkData::default());
        assert_eq!(cm.len(), 1);
    }

    #[test]
    fn get() {
        let mut cm = ChunkManager::new();
        let id = ChunkId::new(0, 0, 0);
        cm.insert_data(&id, ChunkData::default());
        let chunk = cm.get_data(&id);
        assert!(chunk.is_some());

        let id2 = ChunkId::new(0, 1, 0);
        let chunk = cm.get_data(&id2);
        assert!(chunk.is_none());
    }

    #[test]
    fn mutate() {
        let mut cm = ChunkManager::new();
        let id = ChunkId::new(0, 0, 0);
        cm.insert_data(&id, ChunkData::default());
        let chunk = cm.get_mut(&id).unwrap();

        chunk.set(0, 0, 0, Material::Stone);
        assert_eq!(chunk.get(0, 0, 0), Material::Stone);
    }
}
