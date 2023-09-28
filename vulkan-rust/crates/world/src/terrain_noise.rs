use std::f32::consts::PI;

use crate::{world_position::WorldPosition, WorldSeed, CHUNK_SIZE, CHUNK_SIZE_I};
use simdnoise::NoiseBuilder;

mod noise_id {
    pub const HEIGHT: i32 = 0;
    pub const TEMPERATURE: i32 = 1;
    pub const CAVES: i32 = 2;
    pub const RAINFALL: i32 = 3;
}

pub fn height(seed: &WorldSeed, offset_x: i32, offset_y: i32, size: usize) -> Vec<f32> {
    let seed_offset_x = u64::from(seed) & 0xFFFF;
    let seed_offset_y = (u64::from(seed) >> 32) & 0xFFFF;
    let x = offset_x as f32 + seed_offset_x as f32;
    let y = offset_y as f32 + seed_offset_y as f32;

    let (mut noise, _min, _max) = NoiseBuilder::fbm_2d_offset(x as f32, size, y as f32, size)
        .with_seed(noise_id::HEIGHT + i32::from(seed))
        .with_freq(0.17)
        .with_octaves(11)
        .with_gain(2.0)
        .with_lacunarity(0.5)
        .generate();

    noise.iter_mut().for_each(|x| *x *= 10.0);

    noise
}

pub fn chunk_temperature(seed: &WorldSeed, position: &WorldPosition) -> Vec<f32> {
    let x = (position.x / CHUNK_SIZE_I) as f32;
    let y = (position.y / CHUNK_SIZE_I) as f32;

    let (temp, ..) = NoiseBuilder::fbm_2d_offset(x, 2, y, 2)
        .with_seed(noise_id::TEMPERATURE + i32::from(seed))
        .with_freq(0.08)
        .with_octaves(3)
        .with_gain(2.0)
        .with_lacunarity(0.5)
        .generate();

    // println!("Temperature: {:?} to {:?}", min, max);

    debug_assert!(temp.len() == 4);

    temp
}

pub fn chunk_rainfall(seed: &WorldSeed, position: &WorldPosition) -> Vec<f32> {
    let x = (position.x / CHUNK_SIZE_I) as f32;
    let y = (position.y / CHUNK_SIZE_I) as f32;

    let (temp, ..) = NoiseBuilder::fbm_2d_offset(x, 2, y, 2)
        .with_seed(noise_id::RAINFALL + i32::from(seed))
        .with_freq(0.2)
        .with_octaves(3)
        .with_gain(2.0)
        .with_lacunarity(0.5)
        .generate();

    // println!("Temperature: {:?} to {:?}", min, max);

    debug_assert!(temp.len() == 4);

    temp
}

#[allow(unused)]
pub fn flat(_x: i32, _y: i32, size: usize) -> Vec<f32> {
    vec![0.0; size * size]
}

#[allow(unused)]
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

pub fn caves(seed: &WorldSeed, offset_x: i32, offset_y: i32, z: i32, size: usize) -> Vec<f32> {
    let seed_offset_x = u64::from(seed) & 0xFFFF;
    let seed_offset_y = (u64::from(seed) >> 32) & 0xFFFF;
    let x = offset_x as f32 + seed_offset_x as f32;
    let y = offset_y as f32 + seed_offset_y as f32;

    let (noise, _min, _max) =
        NoiseBuilder::turbulence_3d_offset(x as f32, size, y as f32, size, z as f32, size)
            .with_seed(noise_id::CAVES + i32::from(seed))
            .with_freq(0.005)
            .with_octaves(2)
            .with_gain(2.0)
            .with_lacunarity(0.5)
            .generate();

    // println!("Cave noise: ({}, {})", _min, _max);

    noise
}
