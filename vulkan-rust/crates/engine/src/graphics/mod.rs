use nalgebra_glm as glm;
use std::mem::size_of;
use vulkanalia::vk::{self, HasBuilder};
use std::hash::{Hash, Hasher};

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Vertex {
    pub pos: glm::Vec3,
    pub color: glm::Vec3,
}

impl Vertex {
    pub fn new(pos: glm::Vec3, color: glm::Vec3) -> Self {
        Self {
            pos,
            color,
        }
    }

    pub fn binding_description() -> vk::VertexInputBindingDescription {
        vk::VertexInputBindingDescription::builder()
            .binding(0)
            .stride(size_of::<Vertex>() as u32)
            .input_rate(vk::VertexInputRate::VERTEX)
            .build()
    }

    pub fn attribute_descriptions() -> [vk::VertexInputAttributeDescription; 2] {
        let pos = vk::VertexInputAttributeDescription::builder()
            .binding(0)
            .location(0)
            .format(vk::Format::R32G32B32_SFLOAT)
            .offset(0)
            .build();
        let color = vk::VertexInputAttributeDescription::builder()
            .binding(0)
            .location(1)
            .format(vk::Format::R32G32B32_SFLOAT)
            .offset(size_of::<glm::Vec3>() as u32)
            .build();

        [pos, color]
    }
}

impl PartialEq for Vertex {
  fn eq(&self, other: &Self) -> bool {
      self.pos == other.pos && self.color == other.color
  }
}

impl Eq for Vertex {}

impl Hash for Vertex {
  fn hash<H: Hasher>(&self, state: &mut H) {
      self.pos[0].to_bits().hash(state);
      self.pos[1].to_bits().hash(state);
      self.pos[2].to_bits().hash(state);
      self.color[0].to_bits().hash(state);
      self.color[1].to_bits().hash(state);
      self.color[2].to_bits().hash(state);
  }
}