use std::fmt::format;
use std::fmt::Display;

#[repr(u8)]
#[derive(Clone, Copy)]
pub enum Material {
  AIR,
  GRASS,
  DIRT,
  STONE,
  SNOW,
  SAND,
  WOOD,
  WATER,
}

impl From<Material> for &str {
  fn from(mat: Material) -> Self {
    format!("{}", (mat as u8))
  }
}

impl Display for Material {
  fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
    let s: &str = (*self).into();
    write!("{}", s)
  }
}

pub struct Node {
  value: Material,
  level: u32,
  children: [Option<Box<Node>>; 8],
}

pub fn pow(a: u32, x: u32) -> u32 {
  let mut result = if x == 0 { 1 } else { a };
  for _ in [2..x] {
    result *= a;
  }

  return result;
}

/// The size grows exponetially with each level
pub fn level_to_size(level: u32) -> u32 {
  return 2u32.pow(level);
}

pub fn generate_random_tree(level: u32) -> Node {
  return if level > 0 {
    Node {
      level: level,
      value: Material::WOOD,
      children: [
        Some(Box::new(generate_random_tree(level - 1))),
        Some(Box::new(generate_random_tree(level - 1))),
        Some(Box::new(generate_random_tree(level - 1))),
        Some(Box::new(generate_random_tree(level - 1))),
        Some(Box::new(generate_random_tree(level - 1))),
        Some(Box::new(generate_random_tree(level - 1))),
        Some(Box::new(generate_random_tree(level - 1))),
        Some(Box::new(generate_random_tree(level - 1))),
      ],
    }
  } else {
    Node {
      level: level,
      value: Material::WOOD,
      children: [None, None, None, None, None, None, None, None],
    }
  };
}
