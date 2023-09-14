use std::f32::consts::PI;

use crate::{world_position::WorldPosition, CHUNK_SIZE, CHUNK_SIZE_SAFE};
use simdnoise::NoiseBuilder;

// frequency: 0.1123,
// octaves: 5,
// gain: 2.0,
// seed: 1337,
// lacunarity: 0.5,

// const TERRAIN_FREQ: f32 = 1.0 / 170.0;
// const OVERHANG_FREQ: f32 = 1.0 / 17.0;
// const TERRAIN_HEIGHT: f32 = 30.0;
// const BASE: f32 = 100.0;

pub fn height_fade(z: i32) -> f32 {
    1.0 - (z.clamp(0, 100) as f32 / 50.0)
}

pub fn height(offset_x: i32, offset_y: i32, size: usize) -> Vec<f32> {
    let (mut noise, _min, _max) =
        NoiseBuilder::fbm_2d_offset(offset_x as f32, size, offset_y as f32, size)
            .with_freq(0.17)
            .with_octaves(11)
            .with_gain(2.0)
            .with_lacunarity(0.5)
            .generate();

    noise.iter_mut().for_each(|x| *x *= 10.0);

    noise
}

pub fn composite_3d(offset: &WorldPosition) -> Vec<f32> {
    let x = offset.x as f32;
    let y = offset.y as f32;
    // let z = offset.z as f32;

    let base_height = NoiseBuilder::fbm_2d_offset(x, CHUNK_SIZE_SAFE, y, CHUNK_SIZE_SAFE)
        .with_freq(1.)
        .with_octaves(11)
        .with_gain(2.0)
        .with_lacunarity(0.5)
        .generate();

    let ridge = NoiseBuilder::ridge_2d_offset(x, CHUNK_SIZE_SAFE, y, CHUNK_SIZE_SAFE)
        .with_freq(0.0002)
        .with_octaves(4)
        .with_gain(4.0)
        .with_lacunarity(0.5)
        .generate();

    return base_height.0;
}

pub fn temperature(position: &WorldPosition) -> Vec<f32> {
    let x = position.x as f32;
    let y = position.y as f32;
    let z = position.z as f32;

    let (temp, min, max) = NoiseBuilder::fbm_3d_offset(x, CHUNK_SIZE, y, CHUNK_SIZE, z, CHUNK_SIZE)
        .with_freq(0.008)
        .with_octaves(3)
        .with_gain(2.0)
        .with_lacunarity(0.5)
        .generate();

    println!("Temperature: {:?} to {:?}", min, max);

    temp
}

pub fn flat(_x: i32, _y: i32, size: usize) -> Vec<f32> {
    vec![0.0; size * size]
}

pub fn wave(x_start: i32, y_start: i32, size: usize) -> Vec<f32> {
    let mut result = vec![0.0; size * size];
    let scale = (2.0 * PI) / (CHUNK_SIZE as f32);

    for y in 0..size {
        for x in 0..size {
            result[y * size + x] = 0.0
                + ((x_start + x as i32) as f32 * scale).sin() * 5.0
                + ((y_start + y as i32) as f32 * scale).sin() * 5.0;
        }
    }

    result
}

pub fn caves(x: i32, y: i32, z: i32, size: usize) -> Vec<f32> {
    let (noise, _min, _max) =
        NoiseBuilder::turbulence_3d_offset(x as f32, size, y as f32, size, z as f32, size)
            .with_freq(0.008)
            .with_octaves(3)
            .with_gain(2.0)
            .with_lacunarity(0.5)
            .generate();

    noise
}
