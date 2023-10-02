use std::{
    collections::{BTreeMap, HashMap, VecDeque},
    mem::size_of,
};

use anyhow::Context;
use geometry::AABB;
use graphics::{Frustum, Mesh, Vertex};
use vulkanalia::{prelude::v1_0::*, vk};
use world::MeshId;

use crate::{buffer::create_typed_buffer_cmd, pipeline};

struct Queue {
    command_pools: Vec<vk::CommandPool>,
    command_buffers: Vec<vk::CommandBuffer>,

    mesh_create_commands: VecDeque<(MeshId, Mesh)>,
    meshes: BTreeMap<MeshId, usize>,
    mesh_vertex_buffers: HashMap<MeshId, vk::Buffer>,
    mesh_vertex_memory: HashMap<MeshId, vk::Buffer>,
    mesh_index_buffers: HashMap<MeshId, vk::Buffer>,
    mesh_index_memory: HashMap<MeshId, vk::Buffer>,
    mesh_commands: Vec<HashMap<MeshId, vk::CommandBuffer>>,

    delete_commands: Vec<vk::CommandBuffer>,
    one_time_commands: Vec<vk::CommandBuffer>,
}

impl Queue {
    pub fn new(num_images: usize) -> Self {
        Self {
            command_pools: Vec::with_capacity(num_images),
            command_buffers: Vec::with_capacity(num_images),

            mesh_create_commands: VecDeque::new(),
            meshes: BTreeMap::new(),
            mesh_vertex_buffers: Default::default(),
            mesh_vertex_memory: Default::default(),
            mesh_index_buffers: Default::default(),
            mesh_index_memory: Default::default(),
            mesh_commands: Vec::with_capacity(num_images),

            delete_commands: Default::default(),
            one_time_commands: Default::default(),
        }
    }

    pub fn add_mesh(&mut self, mesh_id: MeshId, mesh: Mesh) {
        self.mesh_create_commands.push_back((mesh_id, mesh));
    }

    pub fn add_delete_command(&mut self, command_buffer: vk::CommandBuffer) {
        self.delete_commands.push(command_buffer)
    }

    pub fn add_single_time_command(&mut self, command_buffer: vk::CommandBuffer) {
        self.one_time_commands.push(command_buffer);
    }

    pub unsafe fn update_command_buffer(
        &mut self,
        image_index: usize,
        device: &Device,
        render_pass_begin_info: &vk::RenderPassBeginInfo,
        view: &glm::Mat4,
        projection: &glm::Mat4,
        instance: &Instance,
        physical_device: &vk::PhysicalDevice,
        pipeline: vk::Pipeline,
        pipeline_layout: vk::PipelineLayout,
        render_pass: vk::RenderPass,
        framebuffer: vk::Framebuffer,
        descriptor_set: vk::DescriptorSet,
    ) -> anyhow::Result<()> {
        let mut command_buffer =
            self.begin_commands(image_index, device, render_pass_begin_info)?;

        self.add_mesh_create_commands(command_buffer, instance, device, physical_device);
        self.add_mesh_commands(
            projection,
            view,
            image_index,
            device,
            command_buffer,
            pipeline,
            pipeline_layout,
            render_pass,
            framebuffer,
            descriptor_set,
        );

        device.cmd_end_render_pass(command_buffer);
        device.end_command_buffer(command_buffer)?;

        Ok(())
    }

    unsafe fn begin_commands(
        &mut self,
        image_index: usize,
        device: &Device,
        render_pass_begin_info: &vk::RenderPassBeginInfo,
    ) -> Result<vk::CommandBuffer, anyhow::Error> {
        let command_pool = self.command_pools[image_index];
        device.reset_command_pool(command_pool, vk::CommandPoolResetFlags::empty())?;
        let command_buffer = self.command_buffers[image_index];
        let info = vk::CommandBufferBeginInfo::builder();
        device.begin_command_buffer(command_buffer, &info)?;
        device.cmd_begin_render_pass(
            command_buffer,
            render_pass_begin_info,
            vk::SubpassContents::SECONDARY_COMMAND_BUFFERS,
        );
        Ok(command_buffer)
    }

    unsafe fn add_mesh_create_commands(
        &mut self,
        command_buffer: vk::CommandBuffer,
        instance: &Instance,
        device: &Device,
        physical_device: &vk::PhysicalDevice,
    ) {
        if !self.mesh_create_commands.is_empty() {
            let command_buffers = Vec::with_capacity(self.mesh_create_commands.len());

            for (id, mesh) in self.mesh_create_commands.drain(..) {
                // VERTEX
                let (v_staging_buffer, v_staging_memory, vertex_buffer, vertex_memory) =
                    create_typed_buffer_cmd(
                        instance,
                        device,
                        id,
                        &mesh.vertices,
                        physical_device,
                        vk::BufferUsageFlags::VERTEX_BUFFER,
                    )
                    .expect("Create staging and target buffers");
                add_copy_buffer_cmd::<Vertex>(
                    &mesh,
                    device,
                    command_buffer,
                    v_staging_buffer,
                    vertex_buffer,
                    mesh.vertices.len(),
                );

                // INDEX
                let (i_staging_buffer, i_staging_memory, index_buffer, index_memory) =
                    create_typed_buffer_cmd(
                        instance,
                        device,
                        id,
                        &mesh.indices,
                        physical_device,
                        vk::BufferUsageFlags::INDEX_BUFFER,
                    )
                    .expect("Create staging and target buffers");
                add_copy_buffer_cmd::<u32>(
                    &mesh,
                    device,
                    command_buffer,
                    i_staging_buffer,
                    index_buffer,
                    mesh.indices.len(),
                );
            }

            device.cmd_execute_commands(command_buffer, &command_buffers[..]);
        }
    }

    unsafe fn add_mesh_commands(
        &mut self,
        projection: &glm::Mat4,
        view: &glm::Mat4,
        image_index: usize,
        device: &Device,
        command_buffer: vk::CommandBuffer,
        pipeline: vk::Pipeline,
        pipeline_layout: vk::PipelineLayout,
        render_pass: vk::RenderPass,
        framebuffer: vk::Framebuffer,
        descriptor_set: vk::DescriptorSet,
    ) {
        let frustum = Frustum::from_mat4(projection * view);
        let secondary_command_buffers = self
            .meshes
            .iter()
            .filter(|(id, ..)| frustum.intersects_aabb(&AABB::from(id.chunk_id())))
            .map(|(id, index_count)| {
                self.update_mesh_command_buffer(
                    image_index,
                    id,
                    *index_count,
                    device,
                    pipeline,
                    pipeline_layout,
                    render_pass,
                    framebuffer,
                    descriptor_set,
                )
                .expect("Failed to update mesh command buffer")
            })
            .collect::<Vec<_>>();

        if secondary_command_buffers.len() > 0 {
            device.cmd_execute_commands(command_buffer, &secondary_command_buffers[..]);
        }
    }

    unsafe fn update_mesh_command_buffer(
        &mut self,
        image_index: usize,
        id: &MeshId,
        index_count: usize,
        device: &Device,
        pipeline: vk::Pipeline,
        pipeline_layout: vk::PipelineLayout,
        render_pass: vk::RenderPass,
        framebuffer: vk::Framebuffer,
        descriptor_set: vk::DescriptorSet,
    ) -> anyhow::Result<vk::CommandBuffer> {
        let command_buffers = &mut self.mesh_commands[image_index];
        if !command_buffers.contains_key(id) {
            let allocate_info = vk::CommandBufferAllocateInfo::builder()
                .command_pool(self.command_pools[image_index])
                .level(vk::CommandBufferLevel::SECONDARY)
                .command_buffer_count(1);

            let command_buffer = device.allocate_command_buffers(&allocate_info)?[0];

            command_buffers.insert(*id, command_buffer);
        }

        let cmd_buffer = command_buffers[id];
        let vertex_buffer = self.mesh_vertex_buffers[id];
        let model = glm::translate(&glm::identity(), &id.offset());
        let (_, model_bytes, _) = model.as_slice().align_to::<u8>();

        // let time = self.start.elapsed().as_secs_f32() / 20.0;
        let time: f32 = 0.0;
        let time_bytes = time.to_le_bytes();

        let inheritance_info = vk::CommandBufferInheritanceInfo::builder()
            .render_pass(render_pass)
            .subpass(0)
            .framebuffer(framebuffer);

        let info = vk::CommandBufferBeginInfo::builder()
            .flags(vk::CommandBufferUsageFlags::RENDER_PASS_CONTINUE)
            .inheritance_info(&inheritance_info);

        device.begin_command_buffer(cmd_buffer, &info)?;

        device.cmd_bind_pipeline(cmd_buffer, vk::PipelineBindPoint::GRAPHICS, pipeline);
        device.cmd_bind_vertex_buffers(cmd_buffer, 0, &[vertex_buffer], &[0]);
        device.cmd_bind_index_buffer(
            cmd_buffer,
            self.mesh_index_buffers[id],
            0,
            vk::IndexType::UINT32,
        );
        device.cmd_bind_descriptor_sets(
            cmd_buffer,
            vk::PipelineBindPoint::GRAPHICS,
            pipeline_layout,
            0,
            &[],
            &[],
        );
        device.cmd_push_constants(
            cmd_buffer,
            pipeline_layout,
            vk::ShaderStageFlags::VERTEX,
            0,
            model_bytes,
        );
        device.cmd_push_constants(
            cmd_buffer,
            pipeline_layout,
            vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT,
            64,
            &time_bytes,
        );
        device.cmd_draw_indexed(cmd_buffer, index_count as u32, 1, 0, 0, 0);
        device.end_command_buffer(cmd_buffer)?;

        Ok(cmd_buffer)
    }
}

unsafe fn add_copy_buffer_cmd<T>(
    mesh: &Mesh,
    device: &Device,
    command_buffer: vk::CommandBuffer,
    staging_buffer: vk::Buffer,
    vertex_buffer: vk::Buffer,
    size: usize,
) {
    let size = (size_of::<T>() * size) as u64;
    let regions = vk::BufferCopy::builder().size(size);
    device.cmd_copy_buffer(command_buffer, staging_buffer, vertex_buffer, &[regions]);
}
