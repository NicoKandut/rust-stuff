use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

use crate::{chunk_id::ChunkId, ChunkData, ChunkIdAndData, ChunkUpdateData};

pub struct ChunkManager {
    ids: BTreeSet<ChunkId>,
    chunks: BTreeMap<ChunkId, Arc<ChunkData>>,
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
        self.chunks.insert(id.clone(), Arc::new(data));
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

    pub(crate) fn reset(&mut self) {
        self.ids.clear();
        self.chunks.clear();
    }

    pub fn get_update_data(&self, id: &ChunkId) -> ChunkUpdateData {
        let adjecent_ids = id.get_adjecent();

        ChunkUpdateData {
            chunk: ChunkIdAndData {
                id: *id,
                data: self.chunks.get(id).cloned(),
            },
            adjecent: [
                ChunkIdAndData {
                    id: adjecent_ids[0],
                    data: self.chunks.get(&adjecent_ids[0]).cloned(),
                },
                ChunkIdAndData {
                    id: adjecent_ids[1],
                    data: self.chunks.get(&adjecent_ids[1]).cloned(),
                },
                ChunkIdAndData {
                    id: adjecent_ids[2],
                    data: self.chunks.get(&adjecent_ids[2]).cloned(),
                },
                ChunkIdAndData {
                    id: adjecent_ids[3],
                    data: self.chunks.get(&adjecent_ids[3]).cloned(),
                },
                ChunkIdAndData {
                    id: adjecent_ids[4],
                    data: self.chunks.get(&adjecent_ids[4]).cloned(),
                },
                ChunkIdAndData {
                    id: adjecent_ids[5],
                    data: self.chunks.get(&adjecent_ids[5]).cloned(),
                },
            ],
        }
    }
}

#[cfg(test)]
mod test {
    use super::{ChunkId, ChunkManager};
    use crate::ChunkData;

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
}
