use crate::{WorldPosition, CHUNK_SIZE_I};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ChunkId {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl ChunkId {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    pub fn dist2(lhs: &ChunkId, rhs: &ChunkId) -> i32 {
        let diff = [lhs.x - rhs.x, lhs.y - rhs.y, lhs.z - rhs.z];

        diff[0].pow(2) + diff[1].pow(2) + diff[2].pow(2)
    }
}

impl From<&ChunkId> for glm::Vec3 {
    fn from(value: &ChunkId) -> Self {
        let x = (value.x * CHUNK_SIZE_I) as f32;
        let y = (value.y * CHUNK_SIZE_I) as f32;
        let z = (value.z * CHUNK_SIZE_I) as f32;
        Self::new(x, y, z)
    }
}

impl From<&glm::Vec3> for ChunkId {
    fn from(value: &glm::Vec3) -> Self {
        Self::new(
            value.x.div_euclid(CHUNK_SIZE_I as f32).floor() as i32,
            value.y.div_euclid(CHUNK_SIZE_I as f32).floor() as i32,
            value.z.div_euclid(CHUNK_SIZE_I as f32).floor() as i32,
        )
    }
}

impl From<&WorldPosition> for ChunkId {
    fn from(value: &WorldPosition) -> Self {
        Self::new(
            value.x.div_euclid(CHUNK_SIZE_I),
            value.y.div_euclid(CHUNK_SIZE_I),
            value.z.div_euclid(CHUNK_SIZE_I),
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::{ChunkId, WorldPosition};

    #[test]
    fn negative_pos_to_chunk() {
        let position = WorldPosition::new(-1, 0, 64);
        assert_eq!(ChunkId::from(&position), ChunkId::new(-1, 0, 1));
    }
}
