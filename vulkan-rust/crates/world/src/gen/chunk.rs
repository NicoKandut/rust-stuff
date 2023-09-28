use super::boulder::Boulder;
use crate::seed::ChunkSeed;
use crate::slice::CubeSlice;
use crate::traits::{Data3D, Generate, Voxelize};
use crate::{
    terrain_noise, world_parameters::SEA_LEVEL, world_position::WorldPosition, ChunkData,
    CHUNK_SIZE, CHUNK_SIZE_SAFE_SQUARED,
};
use crate::{ChunkId, CHUNK_SIZE_CUBED, CHUNK_SIZE_SAFE, CHUNK_SIZE_SAFE_I};
use gamedata::material::Material;
use resources::prelude::CACHE;

const CAVE_THRESHOLD: f32 = 0.002;
const TREE_PATH: &str = "D:/Projects/rust-stuff/vulkan-rust/assets/tree.vox";

pub struct Chunk {
    seed: ChunkSeed,
    start: WorldPosition,
    #[allow(unused)]
    rainfall: [f32; 4],
    temperature: [f32; 4],
}

impl Generate<ChunkSeed> for Chunk {
    fn generate(seed: ChunkSeed) -> Self {
        let mut rng = fastrand::Rng::with_seed(seed.value());
        let mut rainfall = [0.0; 4];
        let mut temperature = [0.0; 4];
        let start = WorldPosition::from(seed.id());

        let temp_noise = terrain_noise::chunk_temperature(seed.world_seed(), &start);

        for i in 0..4 {
            rainfall[i] = rng.f32();
            temperature[i] = temp_noise[i];
        }

        Self {
            seed,
            start,
            rainfall,
            temperature,
        }
    }
}

pub struct GeneratedChunk {
    pub id: ChunkId,
    pub voxels: CubeSlice<Material, CHUNK_SIZE_SAFE>,
    pub needs_mesh: bool,
    pub overflow: Vec<(i32, i32, i32, Material)>,
}

impl Voxelize<GeneratedChunk> for Chunk {
    fn voxelize(&self) -> GeneratedChunk {
        let id = self.seed.id();
        let mut voxels = CubeSlice::default();
        let mut voxel_count = 0;
        let mut opaque = true;
        let mut overflow = vec![];

        self.generate_height(&mut voxels, &mut voxel_count);
        self.generate_water(&mut voxels, &mut voxel_count, &mut opaque);
        self.generate_surface(&mut voxels, &self.temperature);
        self.generate_caves(&mut voxels, &mut voxel_count);
        self.generate_trees(&mut voxels, &mut voxel_count, &mut overflow);
        self.generate_boulders(&mut voxels, &mut voxel_count, &mut overflow);
        // self.generate_frame(&mut voxels, &mut voxel_count);
        // self.generate_full(&mut voxels, &mut voxel_count);

        let needs_mesh = !opaque || voxel_count > 0 && voxel_count < CHUNK_SIZE_CUBED;

        GeneratedChunk {
            id: *id,
            voxels,
            needs_mesh,
            overflow,
        }
    }
}

impl Chunk {
    fn generate_height(
        &self,
        data: &mut CubeSlice<Material, CHUNK_SIZE_SAFE>,
        block_count: &mut usize,
    ) {
        let height_noise = terrain_noise::height(
            self.seed.world_seed(),
            self.start.x - 1,
            self.start.y - 1,
            CHUNK_SIZE_SAFE,
        );
        // let height_noise = terrain_noise::wave(chunk_start.x - 1, chunk_start.y - 1, CHUNK_SIZE_SAFE);
        let height_slice = &height_noise[..];
        for y in 0..CHUNK_SIZE_SAFE {
            for x in 0..CHUNK_SIZE_SAFE {
                let height = height_slice[y * CHUNK_SIZE_SAFE + x];
                let height_in_chunk =
                    ((height) as i32 - self.start.z).clamp(0, CHUNK_SIZE_SAFE as i32) as usize;
                for z in 0..height_in_chunk {
                    data.set(x, y, z, Material::Stone);
                    if in_chunk_data(x, y, z) {
                        *block_count += 1;
                    }
                }
            }
        }
    }

    fn generate_water(
        &self,
        data: &mut CubeSlice<Material, CHUNK_SIZE_SAFE>,
        block_count: &mut usize,
        opaque: &mut bool,
    ) {
        if self.start.z < SEA_LEVEL {
            for y in 0..CHUNK_SIZE_SAFE {
                for x in 0..CHUNK_SIZE_SAFE {
                    for z in 1..(CHUNK_SIZE_SAFE - 1) {
                        if data.get(x, y, z).is_fillable() {
                            data.set(x, y, z, Material::Water);
                            if in_chunk_data(x, y, z) {
                                *block_count += 1;
                                *opaque = false;
                            }
                        }
                    }
                }
            }
        }
    }

    fn generate_surface(
        &self,
        data: &mut CubeSlice<Material, CHUNK_SIZE_SAFE>,
        temperature: &[f32; 4],
    ) {
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

    fn generate_caves(
        &self,
        data: &mut CubeSlice<Material, CHUNK_SIZE_SAFE>,
        block_count: &mut usize,
    ) {
        let cave_noise = terrain_noise::caves(
            self.seed.world_seed(),
            self.start.x - 1,
            self.start.y - 1,
            self.start.z - 1,
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
                            if in_chunk_data(x, y, z) {
                                *block_count -= 1;
                            }
                        }
                    }
                }
            }
        }
    }

    fn generate_trees(
        &self,
        data: &mut CubeSlice<Material, CHUNK_SIZE_SAFE>,
        block_count: &mut usize,
        overflow: &mut Vec<(i32, i32, i32, Material)>,
    ) {
        // No underground trees
        if self.start.z < 0 {
            return;
        }

        let mut rng = fastrand::Rng::with_seed(self.seed.value());

        let nr_trees = 4;
        let tree_step = CHUNK_SIZE / nr_trees;

        for ix in 0..nr_trees {
            for iy in 0..nr_trees {
                if rng.f32() < 0.2 {
                    continue;
                }

                let x = ix * tree_step + rng.usize(1..tree_step - 1);
                let y = iy * tree_step + rng.usize(1..tree_step - 1);
                let mut prev_material = data.get(x, y, CHUNK_SIZE_SAFE - 1);
                for z in (1..CHUNK_SIZE_SAFE).rev() {
                    if prev_material.is_solid() {
                        break;
                    }

                    let cur_material = data.get(x, y, z);
                    if cur_material.is_solid() && !prev_material.is_solid() {
                        // let tree = Tree::generate(z as u64);
                        // let tree_voxels = tree.voxelize();
                        let r = 2;
                        // tree_voxels.write_into(data, x as isize - r, y as isize - r, z as isize - r);

                        let vox = CACHE.get_vox(TREE_PATH);

                        for voxel in rng.choice(vox.models.iter()).unwrap().voxels.iter() {
                            let material = Material::from(voxel.color_index.0 - 1);

                            let voxel_x = voxel.point.x as i32 - r + x as i32;
                            let voxel_y = voxel.point.y as i32 - r + y as i32;
                            let voxel_z = voxel.point.z as i32 - r + z as i32 + 3;

                            if voxel_x >= 0
                                && voxel_x < CHUNK_SIZE_SAFE_I
                                && voxel_y >= 0
                                && voxel_y < CHUNK_SIZE_SAFE_I
                                && voxel_z >= 0
                                && voxel_z < CHUNK_SIZE_SAFE_I
                            {
                                if data.get(voxel_x as usize, voxel_y as usize, voxel_z as usize)
                                    != Default::default()
                                {
                                    if in_chunk_data(
                                        voxel_x as usize,
                                        voxel_y as usize,
                                        voxel_z as usize,
                                    ) {
                                        *block_count += 1;
                                    }
                                }

                                data.set(
                                    voxel_x as usize,
                                    voxel_y as usize,
                                    voxel_z as usize,
                                    material,
                                );
                            }

                            if voxel_x <= 0
                                || voxel_x >= CHUNK_SIZE_SAFE_I - 1
                                || voxel_y <= 0
                                || voxel_y >= CHUNK_SIZE_SAFE_I - 1
                                || voxel_z <= 0
                                || voxel_z >= CHUNK_SIZE_SAFE_I - 1
                            {
                                overflow.push((
                                    self.start.x - 1 + voxel_x,
                                    self.start.y - 1 + voxel_y,
                                    self.start.z - 1 + voxel_z,
                                    material,
                                ))
                            }
                        }
                    }

                    prev_material = cur_material;
                }
            }
        }
    }

    fn generate_boulders(
        &self,
        data: &mut CubeSlice<Material, CHUNK_SIZE_SAFE>,
        voxel_count: &mut usize,
        overflow: &mut Vec<(i32, i32, i32, Material)>,
    ) {
        let mut rng = fastrand::Rng::with_seed(self.seed.value());

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
                        let offset_x = (boulder.width / 2) as i32;
                        let offset_y = (boulder.depth / 2) as i32;
                        let offset_z = (boulder.height / 2) as i32;
                        boulder_voxels.write_into(
                            data,
                            x as i32 - offset_x,
                            y as i32 - offset_y,
                            z as i32 - offset_z,
                            voxel_count,
                            overflow,
                        );
                    }

                    prev_material = cur_material;
                }
            }
        }
    }

    #[allow(unused)]
    fn generate_frame(
        &self,
        data: &mut CubeSlice<Material, CHUNK_SIZE_SAFE>,
        voxel_count: &mut usize,
    ) {
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

                *voxel_count += 4;
            }
        }
    }

    #[allow(unused)]
    fn generate_full(
        &self,
        data: &mut CubeSlice<Material, CHUNK_SIZE_SAFE>,
        voxel_count: &mut usize,
    ) {
        for x in 1..(CHUNK_SIZE_SAFE - 1) {
            for y in 1..(CHUNK_SIZE_SAFE - 1) {
                for z in 1..(CHUNK_SIZE_SAFE - 1) {
                    data.set(x, y, z, Material::Stone);
                    *voxel_count += 1;
                }
            }
        }
    }
}

pub fn compress(data: &CubeSlice<Material, CHUNK_SIZE_SAFE>) -> ChunkData {
    let mut result = ChunkData::default();
    for x in 0..CHUNK_SIZE {
        for y in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                let material = data.get(x + 1, y + 1, z + 1);
                if material != Default::default() {
                    result.set(x, y, z, material);
                }
            }
        }
    }

    result
}

#[cfg(test)]
mod test {
    use test::Bencher;

    use super::Chunk;
    use crate::{
        chunk_id::ChunkId,
        seed::{ChunkSeed, WorldSeed},
        traits::{Generate, Voxelize},
    };

    const WORLD_SEED: WorldSeed = WorldSeed::new(17);

    #[bench]
    fn generates_chunk(b: &mut Bencher) {
        let mut x = 0;

        b.iter(|| {
            let id = ChunkId::new(x, 0, 0);
            x += 1;
            test::black_box({
                Chunk::generate(ChunkSeed::new(&WORLD_SEED, &id)).voxelize();
            });
        });
    }
}

pub fn in_chunk_data(x: usize, y: usize, z: usize) -> bool {
    x >= 1
        && x < CHUNK_SIZE_SAFE - 1
        && y >= 1
        && y < CHUNK_SIZE_SAFE - 1
        && z >= 1
        && z < CHUNK_SIZE_SAFE - 1
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
