use crate::slice::Slice3;
use crate::traits::{Data3D, Generate, Voxelize};
use gamedata::material::Material;

pub struct Tree {
    pub height: u8,
    pub radius: u8,
}

impl Generate<u64> for Tree {
    fn generate(seed: u64) -> Self {
        let mut rng = fastrand::Rng::with_seed(seed);

        let height = rng.u8(5..15);
        let max_radius = (height - 1) / 2;
        let radius = rng.u8(2..=max_radius);

        Self { height, radius }
    }
}

impl Voxelize<Slice3<Material>> for Tree {
    fn voxelize(&self) -> Slice3<Material> {
        let width = (self.radius * 2 + 1) as usize;
        let height = self.height as usize;
        let mut voxels = Slice3::new(width, width, height);

        let radius = self.radius as usize;

        // lower leaves
        for z in (height - 2 * radius)..(height - radius) {
            for y in 0..width {
                for x in 0..width {
                    voxels.set(x, y, z, Material::Leaves);
                }
            }
        }

        // upper leaves
        let half_radius = radius / 2;
        for z in (height - radius)..height {
            for y in (radius - half_radius)..=(radius + half_radius) {
                for x in (radius - half_radius)..=(radius + half_radius) {
                    voxels.set(x, y, z, Material::Leaves);
                }
            }
        }

        // trunk
        for z in 0..(height - radius) {
            voxels.set(radius, radius, z, Material::Wood);
        }

        voxels
    }
}
