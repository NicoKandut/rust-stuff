use nalgebra_glm::vec4;
use std::{
    hash::{Hash, Hasher},
    mem::size_of,
};
use vulkanalia::vk::{self, HasBuilder};

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Vertex {
    pub pos_mat: glm::Vec4,
    pub normal: glm::Vec3,
}

impl Vertex {
    pub fn from_material(pos: glm::Vec3, material: u8, normal: glm::Vec3) -> Self {
        Self {
            pos_mat: vec4(pos.x, pos.y, pos.z, material as f32),
            normal,
        }
    }

    pub fn new(pos_mat: glm::Vec4, normal: glm::Vec3) -> Self {
        Self { pos_mat, normal }
    }

    pub fn binding_description() -> vk::VertexInputBindingDescription {
        vk::VertexInputBindingDescription::builder()
            .binding(0)
            .stride(size_of::<Vertex>() as u32)
            .input_rate(vk::VertexInputRate::VERTEX)
            .build()
    }

    pub fn attribute_descriptions() -> [vk::VertexInputAttributeDescription; 2] {
        let pos_mat = vk::VertexInputAttributeDescription::builder()
            .binding(0)
            .location(0)
            .format(vk::Format::R32G32B32A32_SFLOAT)
            .offset(0)
            .build();
        let normal = vk::VertexInputAttributeDescription::builder()
            .binding(0)
            .location(1)
            .format(vk::Format::R32G32B32_SFLOAT)
            .offset(size_of::<glm::Vec4>() as u32)
            .build();

        [pos_mat, normal]
    }
}

impl PartialEq for Vertex {
    fn eq(&self, other: &Self) -> bool {
        self.pos_mat == other.pos_mat && self.normal == other.normal
    }
}

impl Eq for Vertex {}

impl Hash for Vertex {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.pos_mat[0].to_bits().hash(state);
        self.pos_mat[1].to_bits().hash(state);
        self.pos_mat[2].to_bits().hash(state);
        self.normal[0].to_bits().hash(state);
        self.normal[1].to_bits().hash(state);
        self.normal[2].to_bits().hash(state);
    }
}
