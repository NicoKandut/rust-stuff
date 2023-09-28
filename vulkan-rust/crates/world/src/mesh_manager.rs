use crate::chunk_id::ChunkId;
use graphics::Mesh;
use std::collections::BTreeMap;

pub struct MeshManager {
    pub meshes: BTreeMap<ChunkId, Box<Mesh>>,
}

impl MeshManager {
    pub fn new() -> Self {
        Self {
            meshes: BTreeMap::new(),
        }
    }

    pub fn insert(&mut self, id: &ChunkId, mesh: Mesh) {
        self.meshes.insert(*id, Box::new(mesh));
    }

    pub fn get(&self, id: &ChunkId) -> Option<&Mesh> {
        match self.meshes.get(id) {
            Some(mesh_box) => Some(mesh_box),
            None => None,
        }
    }

    pub fn get_all(&self) -> &BTreeMap<ChunkId, Box<Mesh>> {
        &self.meshes
    }

    pub fn remove(&mut self, id: &ChunkId) {
        self.meshes.remove(id);
    }
}
