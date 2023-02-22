use crate::{fixed_tree::ChunkData, CHUNK_SIZE};
use std::{
    collections::BTreeMap,
    ops::{Add, AddAssign},
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ChunkId {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct WorldPosition {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl WorldPosition {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }
}

impl Add<i32> for WorldPosition {
    type Output = Self;

    fn add(self, rhs: i32) -> Self::Output {
        Self::new(self.x + rhs, self.y + rhs, self.z + rhs)
    }
}

impl Add<Self> for WorldPosition {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }
}

impl AddAssign<i32> for WorldPosition {
    fn add_assign(&mut self, rhs: i32) {
        self.x += rhs;
        self.y += rhs;
        self.z += rhs;
    }
}

impl ChunkId {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }
}

impl From<ChunkId> for WorldPosition {
    fn from(id: ChunkId) -> Self {
        Self::new(
            id.x * CHUNK_SIZE as i32,
            id.y * CHUNK_SIZE as i32,
            id.z * CHUNK_SIZE as i32,
        )
    }
}

pub struct ChunkManager {
    chunks: BTreeMap<ChunkId, Box<ChunkData>>,
}

impl ChunkManager {
    pub fn new() -> Self {
        Self {
            chunks: BTreeMap::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.chunks.len()
    }

    pub fn insert(&mut self, id: &ChunkId, data: ChunkData) {
        self.chunks.insert(id.clone(), Box::new(data));
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

    pub(crate) fn get_all(&self) -> Vec<(&ChunkId, &Box<ChunkData>)> {
        self.chunks.iter().filter(|c| !c.1.is_empty()).collect()
    }
}

#[cfg(test)]
mod test {
    use gamedata::material::Material;

    use crate::fixed_tree::ChunkData;

    use super::{ChunkId, ChunkManager};

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

    #[test]
    fn mutate() {
        let mut cm = ChunkManager::new();
        let id = ChunkId::new(0, 0, 0);
        cm.insert(&id, ChunkData::default());
        let chunk = cm.get_mut(&id).unwrap();

        chunk.set(0, 0, 0, Material::Stone);
        assert_eq!(chunk.get(0, 0, 0), Some(Material::Stone));
    }
}
