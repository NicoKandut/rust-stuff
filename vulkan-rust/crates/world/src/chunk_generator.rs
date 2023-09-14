use crate::{
    chunk_id::ChunkId, terrain_noise, world_parameters::SEA_LEVEL, world_position::WorldPosition,
    ChunkData, CHUNK_SIZE, CHUNK_SIZE_SAFE_SQUARED, CHUNK_SIZE_SQUARED,
};
use gamedata::material::Material;
use simdnoise::NoiseBuilder;

pub type Slice2<T, const N: usize> = [[T; N]; N];
pub type Slice3<T, const N: usize> = [[[T; N]; N]; N];

pub fn reinterpret_as_2d<T, const N: usize>(slice: &[T]) -> &[[T; N]] {
    unsafe { std::mem::transmute::<&[T], &[[T; N]]>(slice) }
}

const CHUNK_SIZE_SAFE: usize = CHUNK_SIZE + 2;

pub struct ChunkGenerator {
    // pub seed: i32,
    // pub wip_chunk_height_map: Slice2<f32, CHUNK_SIZE_SAFE>,
    // pub wip_chunk_continentalness: Slice2<f32, 3>,
    // pub wip_chunk_rainfall: Slice2<f32, 3>,
    // pub wip_chunk_temperature: Slice2<f32, 3>,
    // pub wip_chunk_materials: Slice3<Material, CHUNK_SIZE_SAFE>,
}
impl ChunkGenerator {
    pub fn new() -> Self {
        Self {
            // seed: rand::thread_rng().gen_range(0..10000),
            // wip_chunk_height_map: [[0.0; CHUNK_SIZE_SAFE]; CHUNK_SIZE_SAFE],
            // wip_chunk_continentalness: [[0.0; 3]; 3],
            // wip_chunk_rainfall: [[0.0; 3]; 3],
            // wip_chunk_temperature: [[0.0; 3]; 3],
            // wip_chunk_materials: [[[Material::Air; CHUNK_SIZE_SAFE]; CHUNK_SIZE_SAFE];
            //     CHUNK_SIZE_SAFE],
        }
    }

    pub fn generate_inplace(&mut self, id: &ChunkId) {
        let WorldPosition {
            x: sx,
            y: sy,
            z: _sz,
        } = id.into();

        let (noise, min, max) = NoiseBuilder::fbm_2d_offset(sx as f32, 3, sy as f32, 3)
            .with_gain(0.5)
            .with_lacunarity(2.0)
            .with_octaves(4)
            .with_freq(0.0002)
            .generate();
        // self.wip_chunk_continentalness[1][1] = noise[4] - 0.01;

        let (noise, min, max) = NoiseBuilder::fbm_2d_offset(sx as f32, 3, sy as f32, 3)
            .with_gain(0.5)
            .with_lacunarity(2.0)
            .with_octaves(2)
            .with_freq(0.00008)
            .generate();
        // self.wip_chunk_temperature[1][1] = noise[4];

        let (noise, min, max) = NoiseBuilder::fbm_2d_offset(sx as f32, 3, sy as f32, 3)
            .with_gain(0.5)
            .with_lacunarity(2.0)
            .with_octaves(2)
            .with_freq(0.00009)
            .generate();
        // self.wip_chunk_rainfall[1][1] = noise[4];
    }

    pub fn generate(&self, id: &ChunkId) -> (Slice3<Material, CHUNK_SIZE_SAFE>, usize) {
        let chunk_start = &WorldPosition::from(id) - 1;
        let mut data = [[[Material::Air; CHUNK_SIZE_SAFE]; CHUNK_SIZE_SAFE]; CHUNK_SIZE_SAFE];
        let mut block_count = 0;

        // terrain height
        generate_height(&chunk_start, &mut data, &mut block_count);

        // waterfill
        generate_water(id, &mut data, &mut block_count);

        // caves
        // generate_caves(chunk_start, &mut data, &mut block_count);

        // surface
        generate_surface(&mut data);

        // self.generate_full(&mut data);
        // self.generate_frame(&mut data);
        // block_count = CHUNK_SIZE_CUBED;

        (data, block_count)
    }

    pub fn generate_full(&self, data: &mut Slice3<Material, CHUNK_SIZE_SAFE>) {
        for x in 1..(CHUNK_SIZE_SAFE - 1) {
            for y in 1..(CHUNK_SIZE_SAFE - 1) {
                for z in 1..(CHUNK_SIZE_SAFE - 1) {
                    data[x][y][z] = Material::Stone;
                }
            }
        }
    }

    pub fn generate_frame(&self, data: &mut Slice3<Material, CHUNK_SIZE_SAFE>) {
        for d in 0..3 {
            let u = (d + 1) % 3;
            let v = (d + 2) % 3;
            let mut x = [1, 1, 1];
            for i in 1..65 {
                x[d] = i;

                x[u] = 1;
                x[v] = 1;
                data[x[0]][x[1]][x[2]] = Material::Stone;
                x[u] = 1;
                x[v] = 64;
                data[x[0]][x[1]][x[2]] = Material::Stone;
                x[u] = 64;
                x[v] = 1;
                data[x[0]][x[1]][x[2]] = Material::Stone;
                x[u] = 64;
                x[v] = 64;
                data[x[0]][x[1]][x[2]] = Material::Stone;
            }
        }
    }

    pub fn compress(&self, data: &Slice3<Material, CHUNK_SIZE_SAFE>) -> ChunkData {
        let mut result = ChunkData::default();
        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                for z in 0..CHUNK_SIZE {
                    result.set(x, y, z, data[x + 1][y + 1][z + 1])
                }
            }
        }
        result
    }
}

fn generate_surface(data: &mut [[[Material; 66]; 66]; 66]) {
    for y in 0..CHUNK_SIZE_SAFE {
        for x in 0..CHUNK_SIZE_SAFE {
            let mut blocks_below_ground = 0;
            for z in (0..CHUNK_SIZE_SAFE).rev() {
                if data[x][y][z].is_solid() {
                    blocks_below_ground += 1;
                    if blocks_below_ground == 1 {
                        data[x][y][z] = Material::Grass;
                    } else {
                        data[x][y][z] = Material::Sand;
                    }
                }

                if blocks_below_ground == 4 {
                    break;
                }
            }
        }
    }
}

fn generate_caves(
    chunk_start: WorldPosition,
    data: &mut [[[Material; 66]; 66]; 66],
    block_count: &mut usize,
) {
    let cave_noise = terrain_noise::caves(
        chunk_start.x - 1,
        chunk_start.y - 1,
        chunk_start.z - 1,
        CHUNK_SIZE_SAFE,
    );
    let cave_slice = &cave_noise[..];
    for z in 0..CHUNK_SIZE_SAFE {
        for y in 0..CHUNK_SIZE_SAFE {
            for x in 0..CHUNK_SIZE_SAFE {
                if cave_slice[z * CHUNK_SIZE_SAFE_SQUARED + y * CHUNK_SIZE_SAFE + x] < 0.0
                    && data[x][y][z] != Material::Air
                {
                    data[x][y][z] = Material::Air;
                    *block_count -= 1;
                }
            }
        }
    }
}

fn generate_water(id: &ChunkId, data: &mut [[[Material; 66]; 66]; 66], block_count: &mut usize) {
    if id.z <= SEA_LEVEL {
        for y in 0..CHUNK_SIZE_SAFE {
            for x in 0..CHUNK_SIZE_SAFE {
                for z in 1..(CHUNK_SIZE_SAFE - 1) {
                    if data[x][y][z] == Material::Air {
                        data[x][y][z] = Material::Water;
                        *block_count += 1;
                    }
                }
            }
        }
    }
}

fn generate_height(
    chunk_start: &WorldPosition,
    data: &mut [[[Material; 66]; 66]; 66],
    block_count: &mut usize,
) {
    let height_noise = terrain_noise::height(chunk_start.x - 1, chunk_start.y - 1, CHUNK_SIZE_SAFE);
    let height_slice = &height_noise[..];
    for y in 0..CHUNK_SIZE_SAFE {
        for x in 0..CHUNK_SIZE_SAFE {
            let height = height_slice[y * CHUNK_SIZE_SAFE + x];
            let height_in_chunk =
                ((height) as i32 - chunk_start.z).clamp(0, CHUNK_SIZE_SAFE as i32) as usize;
            for z in 0..height_in_chunk {
                data[x][y][z] = Material::Stone;
                *block_count += 1;
            }
        }
    }
}

#[cfg(test)]
mod test {
    use test::Bencher;

    use super::ChunkGenerator;
    use crate::chunk_id::ChunkId;

    #[test]
    fn generates() {
        let cg = ChunkGenerator::new();
        let id = ChunkId::new(0, 0, 0);
        let (_, blocks) = cg.generate(&id);
        assert!(blocks == 0)
    }

    #[bench]
    fn bench_chunk_generation(b: &mut Bencher) {
        let cg: ChunkGenerator = ChunkGenerator::new();
        let mut x = 0;

        b.iter(|| {
            let id = ChunkId::new(x, 0, 0);
            x += 1;
            test::black_box(cg.generate(&id));
        });
    }
}
