use crate::{
    chunk_manager::{ChunkId, WorldPosition},
    fixed_tree::ChunkData,
    terrain_noise, CHUNK_SIZE,
};
use gamedata::material::Material;

pub struct ChunkGenerator {}
impl ChunkGenerator {
    pub fn new() -> Self {
        Self {}
    }

    pub fn generate(&self, id: &ChunkId) -> ChunkData {
        let WorldPosition {
            x: sx,
            y: sy,
            z: sz,
        } = id.clone().into();

        let mut data = ChunkData::default();

        let mut voxel_pos = WorldPosition::new(sx, sy, sz);
        let noise = terrain_noise::composite_3d(&voxel_pos);
        let mut noise_iter = noise.iter();

        for y in 0..CHUNK_SIZE {
            voxel_pos.y = sy + y as i32;
            for x in 0..CHUNK_SIZE {
                voxel_pos.x = sx + x as i32;
                let (bh, r) = noise_iter.next().unwrap().to_owned();
                let world_height = (20. + (50. * bh) + (10. * r)) as i32;

                let chunk_height = (world_height - sz).clamp(0, CHUNK_SIZE as i32) as usize;

                for z in 0..chunk_height {
                    data.set(
                        x,
                        y,
                        z,
                        if z == chunk_height - 1 {
                            Material::Grass
                        } else {
                            Material::Stone
                        },
                    );
                }
            }
        }

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
