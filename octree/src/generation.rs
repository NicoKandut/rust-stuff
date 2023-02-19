use crate::octree::Material;
use noise::{NoiseFn, Seedable, SuperSimplex};

type coord = usize;

pub struct MaterialGenerator {
  noise: SuperSimplex,
}

pub fn world_to_noise_single(x: coord) -> f64 {
  x as f64 / 31.0 + 0.19
}

pub fn world_to_noise_coord(xyz: [coord; 3]) -> [f64; 3] {
  [
    world_to_noise_single(xyz[0]),
    world_to_noise_single(xyz[0]),
    world_to_noise_single(xyz[0]),
  ]
}

pub fn map_to_material(xyz: [coord; 3], density: f64) -> Material {
  if density > 0.5 {
    Material::STONE
  } else {
    Material::AIR
  }
}

impl MaterialGenerator {
  pub fn new(seed: u32) -> Self {
    let noise = SuperSimplex::default();
    noise.set_seed(seed);

    MaterialGenerator { noise: noise }
  }

  fn getDensity(&self, xyz: [coord; 3]) -> f64 {
    self.noise.get(world_to_noise_coord(xyz))
  }

  pub fn get_material_at(&self, xyz: [coord; 3]) -> Material {
    map_to_material(xyz, self.getDensity(xyz))
  }

  pub fn get_chunk(&self, start: [coord; 3]) -> [Material; 64] {
    let [x0, y0, z0] = start;
    let mut chunk = [Material::AIR; 64];

    for z in z0..z0 + 4 {
      for y in y0..y0 + 4 {
        for x in x0..x0 + 4 {
          chunk[x + y * x + z * x * y] = self.get_material_at([x, y, z])
        }
      }
    }

    chunk
  }
}
