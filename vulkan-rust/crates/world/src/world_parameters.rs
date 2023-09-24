// WORLD
pub const SEA_LEVEL: i32 = 0;

// CHUNKS
pub const CHUNK_SIZE: usize = 64;
pub const CHUNK_SIZE_SQUARED: usize = CHUNK_SIZE * CHUNK_SIZE;
pub const CHUNK_SIZE_CUBED: usize = CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE;

pub const CHUNK_SIZE_I: i32 = CHUNK_SIZE as i32;
pub const CHUNK_SIZE_SQUARED_I: i32 = CHUNK_SIZE_SQUARED as i32;
pub const CHUNK_SIZE_CUBED_I: i32 = CHUNK_SIZE_CUBED as i32;

pub const CHUNK_SIZE_SAFE: usize = CHUNK_SIZE + 2;
pub const CHUNK_SIZE_SAFE_SQUARED: usize = CHUNK_SIZE_SAFE * CHUNK_SIZE_SAFE;
pub const CHUNK_SIZE_SAFE_CUBED: usize = CHUNK_SIZE_SAFE * CHUNK_SIZE_SAFE * CHUNK_SIZE_SAFE;
