#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
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
