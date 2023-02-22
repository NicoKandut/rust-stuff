use crate::{
    chunk_manager::{ChunkId, WorldPosition},
    fixed_tree::ChunkData,
    CHUNK_SIZE,
};
use gamedata::material::Material;
use simdnoise::NoiseBuilder;

pub struct ChunkGenerator {
    frequency: f32,
    octaves: u8,
    gain: f32,
    seed: i32,
    lacunarity: f32,
}

impl ChunkGenerator {
    pub fn new() -> Self {
        Self {
            frequency: 0.5,
            octaves: 5,
            gain: 2.0,
            seed: 1337,
            lacunarity: 0.5,
        }
    }

    pub fn generate(&self, id: &ChunkId) -> ChunkData {
        let world_pos: WorldPosition = id.clone().into();

        let (samples, ..) = NoiseBuilder::turbulence_3d_offset(
            world_pos.x as f32,
            CHUNK_SIZE,
            world_pos.y as f32,
            CHUNK_SIZE,
            world_pos.z as f32,
            CHUNK_SIZE,
        )
        .with_freq(self.frequency)
        .with_gain(self.gain)
        .with_lacunarity(self.lacunarity)
        .with_octaves(self.octaves)
        .with_seed(self.seed)
        .generate();

        let mut data = ChunkData::default();
        let mut x = 0;
        let mut y = 0;
        let mut z = 0;

        samples.iter().map(|s| Material::from(*s)).for_each(|m| {
            let border = (x == 0 || x == CHUNK_SIZE - 1) as u8
                + (y == 0 || y == CHUNK_SIZE - 1) as u8
                + (z == 0 || z == CHUNK_SIZE - 1) as u8;
            if border > 1 {
                data.set(x, y, z, Material::Stone);
            } else if m != Material::Air {
                data.set(x, y, z, m);
            }

            x += 1;
            y += (x == CHUNK_SIZE) as usize;
            z += (y == CHUNK_SIZE) as usize;
            x %= CHUNK_SIZE;
            y %= CHUNK_SIZE;
            z %= CHUNK_SIZE;
        });

        data
    }
}

#[cfg(test)]
mod test {
    use test::Bencher;

    use super::ChunkGenerator;
    use crate::chunk_manager::ChunkId;

    #[test]
    fn generates() {
        let cg = ChunkGenerator::new();
        let id = ChunkId::new(0, 0, 0);
        let data = cg.generate(&id);
        assert!(!data.is_empty())
    }

    #[bench]
    fn bench_chunk_generation(b: &mut Bencher) {
        let cg = ChunkGenerator::new();
        let mut x = 0;

        b.iter(|| {
            let id = ChunkId::new(x, 0, 0);
            x += 1;
            test::black_box(cg.generate(&id));
        });
    }
}
