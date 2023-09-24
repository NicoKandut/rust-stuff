use crate::seed::PositionalSeed;
use crate::slice::CubeSlice;
use crate::traits::{Data3D, Generate, Voxelize};
use crate::{
    gen::tree::Tree, terrain_noise, world_parameters::SEA_LEVEL, world_position::WorldPosition,
    ChunkData, CHUNK_SIZE, CHUNK_SIZE_SAFE_SQUARED,
};
use crate::{WorldSeed, CHUNK_SIZE_SAFE};
use gamedata::material::Material;

use super::boulder::Boulder;

const CAVE_THRESHOLD: f32 = 0.002;

pub struct Chunk {
    seed: PositionalSeed,
    #[allow(unused)]
    rainfall: [f32; 4],
    temperature: [f32; 4],
}

impl Generate<PositionalSeed> for Chunk {
    fn generate(seed: PositionalSeed) -> Self {
        let mut rng = fastrand::Rng::with_seed(seed.value());
        let mut rainfall = [0.0; 4];
        let mut temperature = [0.0; 4];

        let temp_noise = terrain_noise::chunk_temperature(seed.world_seed(), seed.pos());

        for i in 0..4 {
            rainfall[i] = rng.f32();
            temperature[i] = temp_noise[i];
        }

        Self {
            seed,
            rainfall,
            temperature,
        }
    }
}

pub struct ChunkVoxels {
    pub voxels: CubeSlice<Material, CHUNK_SIZE_SAFE>,
    pub voxel_count: usize,
}

impl Voxelize<ChunkVoxels> for Chunk {
    fn voxelize(&self) -> ChunkVoxels {
        let chunk_start = self.seed.pos() - 1;
        let mut voxels = CubeSlice::default();
        let mut voxel_count = 0;

        generate_height(
            self.seed.world_seed(),
            &chunk_start,
            &mut voxels,
            &mut voxel_count,
        );
        generate_water(self.seed.pos(), &mut voxels, &mut voxel_count);
        generate_caves(
            self.seed.world_seed(),
            &chunk_start,
            &mut voxels,
            &mut voxel_count,
        );
        generate_surface(&mut voxels, &self.temperature);
        generate_trees(&chunk_start, &mut voxels);
        generate_boulders(&chunk_start, &mut voxels);
        // generate_frame(&mut voxels);

        ChunkVoxels {
            voxels,
            voxel_count,
        }
    }
}

#[allow(unused)]
fn generate_full(data: &mut CubeSlice<Material, CHUNK_SIZE_SAFE>) {
    for x in 1..(CHUNK_SIZE_SAFE - 1) {
        for y in 1..(CHUNK_SIZE_SAFE - 1) {
            for z in 1..(CHUNK_SIZE_SAFE - 1) {
                data.set(x, y, z, Material::Stone);
            }
        }
    }
}

#[allow(unused)]
fn generate_frame(data: &mut CubeSlice<Material, CHUNK_SIZE_SAFE>) {
    for d in 0..3 {
        let u = (d + 1) % 3;
        let v = (d + 2) % 3;
        let mut x = [1, 1, 1];
        for i in 1..65 {
            x[d] = i;

            x[u] = 1;
            x[v] = 1;
            data.set(x[0], x[1], x[2], Material::Stone);
            x[u] = 1;
            x[v] = 64;
            data.set(x[0], x[1], x[2], Material::Stone);
            x[u] = 64;
            x[v] = 1;
            data.set(x[0], x[1], x[2], Material::Stone);
            x[u] = 64;
            x[v] = 64;
            data.set(x[0], x[1], x[2], Material::Stone);
        }
    }
}

pub fn compress(data: &CubeSlice<Material, CHUNK_SIZE_SAFE>) -> ChunkData {
    let mut result = ChunkData::default();
    for x in 0..CHUNK_SIZE {
        for y in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                result.set(x, y, z, data.get(x + 1, y + 1, z + 1))
            }
        }
    }
    result
}

fn generate_trees(offset: &WorldPosition, data: &mut CubeSlice<Material, CHUNK_SIZE_SAFE>) {
    if offset.z < 0 {
        return;
    }

    let mut rng = fastrand::Rng::with_seed((offset.x + offset.y) as u64);

    let nr_trees = 4;
    let tree_step = CHUNK_SIZE / nr_trees;

    for ix in 0..nr_trees {
        for iy in 0..nr_trees {
            if rng.f32() < 0.2 {
                continue;
            }

            let x = ix * tree_step + rng.usize(1..tree_step - 1);
            let y = iy * tree_step + rng.usize(1..tree_step - 1);
            let mut prev_material = data.get(x, y, 0);
            for z in 1..CHUNK_SIZE_SAFE {
                if !prev_material.is_solid() {
                    break;
                }

                let cur_material = data.get(x, y, z);
                if !cur_material.is_solid() && prev_material.is_solid() {
                    let tree = Tree::generate(z as u64);
                    let tree_voxels = tree.voxelize();
                    let r = tree.radius as isize;
                    tree_voxels.write_into(data, x as isize - r, y as isize - r, z as isize - r);
                }

                prev_material = cur_material;
            }
        }
    }
}

fn generate_boulders(offset: &WorldPosition, data: &mut CubeSlice<Material, CHUNK_SIZE_SAFE>) {
    let mut rng = fastrand::Rng::with_seed((offset.x + offset.y) as u64);

    let nr_boulders = 2;
    let step = CHUNK_SIZE / nr_boulders;

    for ix in 0..nr_boulders {
        for iy in 0..nr_boulders {
            if rng.f32() < 0.2 {
                continue;
            }

            let x = ix * step + rng.usize(1..step - 1);
            let y = iy * step + rng.usize(1..step - 1);
            let mut prev_material = data.get(x, y, 0);
            for z in 1..CHUNK_SIZE_SAFE {
                if !prev_material.is_solid() {
                    break;
                }

                let cur_material = data.get(x, y, z);
                if !cur_material.is_solid() && prev_material.is_solid() {
                    let boulder = Boulder::generate(z as u64);
                    let boulder_voxels = boulder.voxelize();
                    let offset_x = (boulder.width / 2) as isize;
                    let offset_y = (boulder.depth / 2) as isize;
                    let offset_z = (boulder.height / 2) as isize;
                    boulder_voxels.write_into(
                        data,
                        x as isize - offset_x,
                        y as isize - offset_y,
                        z as isize - offset_z,
                    );
                }

                prev_material = cur_material;
            }
        }
    }
}

fn generate_surface(data: &mut CubeSlice<Material, CHUNK_SIZE_SAFE>, temperature: &[f32; 4]) {
    for y in 0..CHUNK_SIZE_SAFE {
        for x in 0..CHUNK_SIZE_SAFE {
            let temperature = chunk_bilerp(temperature, x as i32 - 1, y as i32 - 1);
            let mut blocks_below_ground = 0;
            for z in (0..CHUNK_SIZE_SAFE).rev() {
                if data.get(x, y, z).is_solid() {
                    blocks_below_ground += 1;
                    let block = if blocks_below_ground == 1 {
                        if temperature > 0.0 {
                            Material::Sand
                        } else {
                            Material::Grass
                        }
                    } else {
                        Material::Dirt
                    };

                    data.set(x, y, z, block);
                }

                if blocks_below_ground == 4 {
                    break;
                }
            }
        }
    }
}

#[allow(unused)]
fn generate_caves(
    seed: &WorldSeed,
    chunk_start: &WorldPosition,
    data: &mut CubeSlice<Material, CHUNK_SIZE_SAFE>,
    block_count: &mut usize,
) {
    let cave_noise = terrain_noise::caves(
        seed,
        chunk_start.x - 1,
        chunk_start.y - 1,
        chunk_start.z - 1,
        CHUNK_SIZE_SAFE,
    );
    let cave_slice = &cave_noise[..];
    for z in 0..CHUNK_SIZE_SAFE {
        for y in 0..CHUNK_SIZE_SAFE {
            for x in 0..CHUNK_SIZE_SAFE {
                if cave_slice[z * CHUNK_SIZE_SAFE_SQUARED + y * CHUNK_SIZE_SAFE + x]
                    < CAVE_THRESHOLD
                {
                    let material = data.get(x, y, z);
                    if material.is_solid() {
                        data.set(x, y, z, Material::Air);
                        *block_count -= 1;
                    }
                }
            }
        }
    }
}

fn generate_water(
    id: &WorldPosition,
    data: &mut CubeSlice<Material, CHUNK_SIZE_SAFE>,
    block_count: &mut usize,
) {
    if id.z < SEA_LEVEL {
        for y in 0..CHUNK_SIZE_SAFE {
            for x in 0..CHUNK_SIZE_SAFE {
                for z in 1..(CHUNK_SIZE_SAFE - 1) {
                    if data.get(x, y, z).is_fillable() {
                        data.set(x, y, z, Material::Water);
                        *block_count += 1;
                    }
                }
            }
        }
    }
}

fn generate_height(
    seed: &WorldSeed,
    chunk_start: &WorldPosition,
    data: &mut CubeSlice<Material, CHUNK_SIZE_SAFE>,
    block_count: &mut usize,
) {
    let height_noise =
        terrain_noise::height(seed, chunk_start.x - 1, chunk_start.y - 1, CHUNK_SIZE_SAFE);
    // let height_noise = terrain_noise::wave(chunk_start.x - 1, chunk_start.y - 1, CHUNK_SIZE_SAFE);
    let height_slice = &height_noise[..];
    for y in 0..CHUNK_SIZE_SAFE {
        for x in 0..CHUNK_SIZE_SAFE {
            let height = height_slice[y * CHUNK_SIZE_SAFE + x];
            let height_in_chunk =
                ((height) as i32 - chunk_start.z).clamp(0, CHUNK_SIZE_SAFE as i32) as usize;
            for z in 0..height_in_chunk {
                data.set(x, y, z, Material::Stone);
                *block_count += 1;
            }
        }
    }
}

#[cfg(test)]
mod test {
    use test::Bencher;

    use super::Chunk;
    use crate::{
        chunk_id::ChunkId,
        seed::{PositionalSeed, WorldSeed},
        traits::{Generate, Voxelize},
        world_position::WorldPosition,
    };

    const WORLD_SEED: WorldSeed = WorldSeed::new(17);

    #[test]
    fn generates() {
        let id = ChunkId::new(0, 0, 0);
        let position = WorldPosition::from(&id);
        let chunk = Chunk::generate(PositionalSeed::new(&WORLD_SEED, &position));
        let voxel_data = chunk.voxelize();
        assert!(voxel_data.voxel_count == 0)
    }

    #[bench]
    fn bench_chunk_generation(b: &mut Bencher) {
        let mut x = 0;

        b.iter(|| {
            let id = ChunkId::new(x, 0, 0);
            let position = WorldPosition::from(&id);
            x += 1;
            test::black_box({
                Chunk::generate(PositionalSeed::new(&WORLD_SEED, &position)).voxelize();
            });
        });
    }
}

pub fn chunk_bilerp(corners: &[f32; 4], x: i32, y: i32) -> f32 {
    let xr1 = glm::smoothstep(0.0, CHUNK_SIZE as f32, x as f32);
    let xr2 = 1.0 - xr1;
    let x1 = corners[3] * xr1 + corners[2] * xr2;
    let x2 = corners[1] * xr1 + corners[0] * xr2;

    let yr1 = glm::smoothstep(0.0, CHUNK_SIZE as f32, y as f32);
    let yr2 = 1.0 - yr1;
    let y1 = x1 * yr1 + x2 * yr2;

    y1
}
