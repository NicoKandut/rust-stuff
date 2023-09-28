use crate::slice::Slice3;
use crate::traits::{Data3D, Generate, Voxelize};
use gamedata::material::Material;

pub struct Boulder {
    pub height: usize,
    pub width: usize,
    pub depth: usize,
}

impl Generate<u64> for Boulder {
    fn generate(seed: u64) -> Self {
        let mut rng = fastrand::Rng::with_seed(seed);

        let height = rng.usize(5..15);
        let width = rng.usize(5..15);
        let depth = rng.usize(5..15);

        Self {
            height,
            width,
            depth,
        }
    }
}

impl Voxelize<Slice3<Material>> for Boulder {
    fn voxelize(&self) -> Slice3<Material> {
        let mut voxels = Slice3::new(self.width.into(), self.depth.into(), self.height.into());

        let half_depth = (self.depth / 2) as usize;

        for z in 0..self.height {
            let y_size = (1.0 - (z as f32 / self.height as f32).powf(2.0)).sqrt();
            let y_lower = half_depth - (half_depth as f32 * y_size) as usize;
            let y_upper = half_depth + (half_depth as f32 * y_size) as usize;
            for y in y_lower..y_upper as usize {
                let half_width = ((y_upper - y_lower) / 2) as usize;
                let x_size = (1.0 - (y as f32 / self.width as f32).powf(2.0)).sqrt();
                let x_lower = half_width - (half_width as f32 * x_size) as usize;
                let x_upper = half_width + (half_width as f32 * x_size) as usize;
                for x in x_lower..x_upper as usize {
                    if x < self.width && y < self.depth && z < self.height {
                        voxels.set(x, y, z, Material::Stone);
                    }
                }
            }
        }

        voxels
    }
}
