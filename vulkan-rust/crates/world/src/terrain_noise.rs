use crate::{chunk_manager::WorldPosition, CHUNK_SIZE};
use simdnoise::NoiseBuilder;

// frequency: 0.1123,
// octaves: 5,
// gain: 2.0,
// seed: 1337,
// lacunarity: 0.5,

// const SEA_LEVEL: i32 = 0;

const TERRAIN_FREQ: f32 = 1.0 / 170.0;
const OVERHANG_FREQ: f32 = 1.0 / 17.0;
const TERRAIN_HEIGHT: f32 = 30.0;
// const BASE: f32 = 100.0;

pub fn height_fade(z: i32) -> f32 {
    1.0 - (z.clamp(0, 100) as f32 / 50.0)
}

pub fn composite_3d(offset: &WorldPosition) -> Vec<(f32, f32)> {
    let x = offset.x as f32;
    let y = offset.y as f32;
    // let z = offset.z as f32;

    let base_height = NoiseBuilder::fbm_2d_offset(x, CHUNK_SIZE, y, CHUNK_SIZE)
        .with_freq(0.091)
        .with_octaves(5)
        .with_gain(2.0)
        .with_lacunarity(0.5)
        .generate();

    let ridge = NoiseBuilder::ridge_2d_offset(x, CHUNK_SIZE, y, CHUNK_SIZE)
        .with_freq(0.002)
        .with_octaves(3)
        .with_gain(4.0)
        .with_lacunarity(0.5)
        .generate();

    return base_height.0.iter().cloned().zip(ridge.0).collect();
}
