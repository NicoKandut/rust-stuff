use crate::{
    appdata::{
        create_descriptor_pool, create_descriptor_sets, create_framebuffers, create_sync_objects,
        AppData,
    },
    buffer::PreparedMesh,
    command::{create_command_buffers, create_command_pools},
    constants::{MAX_FRAMES_IN_FLIGHT, VALIDATION_ENABLED},
    device::{physical, pick_device},
    image::{
        create_depth_objects, create_texture_image, create_texture_image_view,
        create_texture_sampler,
    },
    instance::create_instance,
    pipeline::create_pipeline,
    renderpass::create_render_pass,
    swapchain::{create_swapchain, create_swapchain_image_views},
    uinform::{create_descriptor_set_layout, create_uniform_buffers, UniformBufferObject},
};
use anyhow::{anyhow, Result};
use geometry::AABB;
use graphics::{camera::FlyingCamera, Frustum};
use logging::{log, LOG_VULKAN};
use resources::prelude::CACHE;
use std::{
    collections::{BTreeMap, VecDeque},
    mem::size_of,
    ptr::copy_nonoverlapping as memcpy,
    time::Instant,
};
use vulkanalia::{
    loader::{LibloadingLoader, LIBRARY},
    prelude::v1_0::*,
    vk::ExtDebugUtilsExtension,
    vk::KhrSurfaceExtension,
    vk::KhrSwapchainExtension,
    window as vk_window,
};
use winit::window::Window;
use world::{MeshId, WorldPosition};

pub enum Delete {
    Mesh(MeshId),
    Buffer(vk::Buffer),
    DeviceMemory(vk::DeviceMemory),
}

#[derive(Clone, Debug)]
pub struct App {
    pub entry: Entry,
    pub instance: Instance,
    pub data: AppData,
    pub device: Device,
    pub frame: usize,
    pub resized: bool,
    pub start: Instant,
    pub render_distance: f32,
}

impl App {
    /// Creates our Vulkan app.
    pub unsafe fn create(window: &Window) -> Result<Self> {
        log!(*LOG_VULKAN, "Initializing vulkan");
        let start = Instant::now();
        let loader = LibloadingLoader::new(LIBRARY)?;
        let entry = Entry::new(loader).map_err(|b| anyhow!("{}", b))?;
        let mut data = AppData::default();

        log!(*LOG_VULKAN, "Creating instance");
        let instance = create_instance(window, &entry, &mut data)?;
        data.surface = vk_window::create_surface(&instance, window)?;
        let (logical_device, physical_device) = pick_device(&instance, data.surface)?;
        data.physical_device = physical_device;
        data.graphics_queue = logical_device.graphics_queue;
        data.present_queue = logical_device.present_queue;

        create_swapchain(window, &instance, &logical_device.device, &mut data)?;
        create_swapchain_image_views(&logical_device.device, &mut data)?;
        create_render_pass(&instance, &logical_device.device, &mut data)?;
        create_descriptor_set_layout(&logical_device.device, &mut data)?;
        create_pipeline(&logical_device.device, &mut data)?;
        create_command_pools(&instance, &logical_device.device, &mut data)?;
        create_depth_objects(&instance, &logical_device.device, &mut data)?;
        create_framebuffers(&logical_device.device, &mut data)?;
        create_texture_image(
            &instance,
            &logical_device.device,
            &mut data,
            CACHE.get_img("\\assets\\palette.png"),
        )?;
        create_texture_image_view(&logical_device.device, &mut data)?;
        create_texture_sampler(&logical_device.device, &mut data)?;
        // load_model(&mut data)?;
        // load_world(&mut data, voxels)?;
        // create_vertex_buffer(&instance, &device, &mut data)?;
        // create_index_buffer(&instance, &device, &mut data)?;
        create_uniform_buffers(&instance, &logical_device.device, &mut data)?;
        create_descriptor_pool(&logical_device.device, &mut data)?;
        create_descriptor_sets(&logical_device.device, &mut data)?;
        create_command_buffers(&logical_device.device, &mut data)?;
        create_sync_objects(&logical_device.device, &mut data)?;

        Ok(Self {
            entry,
            instance,
            data,
            device: logical_device.device,
            frame: 0,
            resized: false,
            start,
            render_distance: 10.0,
        })
    }

    /// Renders a frame for our Vulkan app.
    pub unsafe fn render(
        &mut self,
        window: &Window,
        meshes: &mut BTreeMap<MeshId, usize>,
        cam: &FlyingCamera,
        prepared_meshes: &mut Vec<(MeshId, PreparedMesh)>,
        deletion_queue: &mut VecDeque<Delete>,
    ) -> Result<()> {
        let in_flight_fence = self.data.in_flight_fences[self.frame];

        self.device
            .wait_for_fences(&[in_flight_fence], true, u64::MAX)?;

        let result = self.device.acquire_next_image_khr(
            self.data.swapchain,
            u64::MAX,
            self.data.image_available_semaphores[self.frame],
            vk::Fence::null(),
        );

        let image_index = match result {
            Ok((image_index, _)) => image_index as usize,
            Err(vk::ErrorCode::OUT_OF_DATE_KHR) => return self.recreate_swapchain(window),
            Err(e) => return Err(anyhow!(e)),
        };

        // println!("IMAGE INDEX: {image_index}");

        let image_in_flight = self.data.images_in_flight[image_index];
        if !image_in_flight.is_null() {
            self.device
                .wait_for_fences(&[image_in_flight], true, u64::MAX)?;
        }

        self.data.images_in_flight[image_index] = in_flight_fence;

        let vp = self.calculate_vp(cam);

        self.update_command_buffer(image_index, meshes, &vp, prepared_meshes, deletion_queue)?;
        self.update_uniform_buffer(image_index, vp, glm::vec3_to_vec4(&cam.cam.position))?;

        let wait_semaphores = &[self.data.image_available_semaphores[self.frame]];
        let wait_stages = &[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let command_buffers = &[
            self.data.command_buffers_transfer[image_index],
            self.data.command_buffers[image_index],
        ];
        let signal_semaphores = &[self.data.render_finished_semaphores[self.frame]];
        let submit_info = vk::SubmitInfo::builder()
            .wait_semaphores(wait_semaphores)
            .wait_dst_stage_mask(wait_stages)
            .command_buffers(command_buffers)
            .signal_semaphores(signal_semaphores);

        self.device.reset_fences(&[in_flight_fence])?;
        self.device
            .queue_submit(self.data.graphics_queue, &[submit_info], in_flight_fence)?;

        let swapchains = &[self.data.swapchain];
        let image_indices = &[image_index as u32];
        let present_info = vk::PresentInfoKHR::builder()
            .wait_semaphores(signal_semaphores)
            .swapchains(swapchains)
            .image_indices(image_indices);

        let result = self
            .device
            .queue_present_khr(self.data.present_queue, &present_info);
        let changed = result == Ok(vk::SuccessCode::SUBOPTIMAL_KHR)
            || result == Err(vk::ErrorCode::OUT_OF_DATE_KHR);
        if self.resized || changed {
            self.resized = false;
            self.recreate_swapchain(window)?;
        } else if let Err(e) = result {
            return Err(anyhow!(e));
        }

        self.frame = (self.frame + 1) % MAX_FRAMES_IN_FLIGHT;

        Ok(())
    }

    unsafe fn update_command_buffer(
        &mut self,
        image_index: usize,
        meshes: &mut BTreeMap<MeshId, usize>,
        vp: &(glm::Mat4, glm::Mat4),
        prepared_meshes: &mut Vec<(MeshId, PreparedMesh)>,
        deletion_queue: &mut VecDeque<Delete>,
    ) -> Result<()> {
        // println!("{LOG_VK} UPDATING ALL CMD BUFFERS");
        let command_pool = self.data.command_pools[image_index];
        self.device
            .reset_command_pool(command_pool, vk::CommandPoolResetFlags::empty())?;

        // NEW MESHES
        let command_buffer = self.data.command_buffers_transfer[image_index];
        let info = vk::CommandBufferBeginInfo::builder();
        self.device.begin_command_buffer(command_buffer, &info)?;
        let new_meshes = prepared_meshes
            .drain(0..prepared_meshes.len().min(2))
            .map(|(mesh_id, prepared_mesh)| {
                let regions = vk::BufferCopy::builder().size(prepared_mesh.vertex.size);
                self.device.cmd_copy_buffer(
                    command_buffer,
                    prepared_mesh.vertex.staging.buffer,
                    prepared_mesh.vertex.target.buffer,
                    &[regions],
                );

                let regions = vk::BufferCopy::builder().size(prepared_mesh.index.size);
                self.device.cmd_copy_buffer(
                    command_buffer,
                    prepared_mesh.index.staging.buffer,
                    prepared_mesh.index.target.buffer,
                    &[regions],
                );

                (mesh_id, prepared_mesh)
            })
            .collect::<Vec<_>>();
        self.device.end_command_buffer(command_buffer)?;

        // RENDER
        let command_buffer = self.data.command_buffers[image_index];
        let info = vk::CommandBufferBeginInfo::builder();
        self.device.begin_command_buffer(command_buffer, &info)?;
        let render_area = vk::Rect2D::builder()
            .offset(vk::Offset2D::default())
            .extent(self.data.swapchain_extent);

        let color_clear_value = vk::ClearValue {
            color: vk::ClearColorValue {
                float32: palette::sky().into(),
            },
        };

        let depth_clear_value = vk::ClearValue {
            depth_stencil: vk::ClearDepthStencilValue {
                depth: 1.0,
                stencil: 0,
            },
        };

        let clear_values = &[color_clear_value, depth_clear_value];
        let info = vk::RenderPassBeginInfo::builder()
            .render_pass(self.data.render_pass)
            .framebuffer(self.data.framebuffers[image_index])
            .render_area(render_area)
            .clear_values(clear_values);

        self.device.cmd_begin_render_pass(
            command_buffer,
            &info,
            vk::SubpassContents::SECONDARY_COMMAND_BUFFERS,
        );

        // EXISTING CHUNKS
        // println!("Rendering {} meshes", meshes.len());
        let (view, proj) = vp;
        let frustum = Frustum::from_mat4(&(proj * view));
        let secondary_command_buffers = meshes
            .iter()
            .filter(|(id, ..)| frustum.intersects_aabb(&AABB::from(id.chunk_id())))
            .filter_map(|(id, index_count)| {
                match self.update_secondary_command_buffer(image_index, id, *index_count) {
                    Ok(cmd_buffer) => Some(cmd_buffer),
                    Err(_) => {
                        // println!("Failed to update cmd buffer");
                        None
                    }
                }
            })
            .collect::<Vec<_>>();

        if secondary_command_buffers.len() > 0 {
            self.device
                .cmd_execute_commands(command_buffer, &secondary_command_buffers[..]);
        }

        self.device.cmd_end_render_pass(command_buffer);
        self.device.end_command_buffer(command_buffer)?;

        for (mesh_id, prepared_mesh) in new_meshes {
            self.register_mesh_ready(mesh_id, &prepared_mesh, deletion_queue);
            meshes.insert(
                mesh_id,
                prepared_mesh.index.size as usize / size_of::<u32>(),
            );
        }

        // println!("{LOG_VK} DONE UPDATING ALL CMD BUFFERS");

        Ok(())
    }

    fn register_mesh_ready(
        &mut self,
        mesh_id: MeshId,
        prepared_mesh: &PreparedMesh,
        deletion_queue: &mut VecDeque<Delete>,
    ) {
        let prev_buffer = self
            .data
            .chunk_vertex_buffers
            .insert(mesh_id, prepared_mesh.vertex.target.buffer);
        if let Some(b) = prev_buffer {
            deletion_queue.push_back(Delete::Buffer(b))
        }
        let prev_memory = self
            .data
            .chunk_vertex_buffers_memory
            .insert(mesh_id, prepared_mesh.vertex.target.memory);
        if let Some(m) = prev_memory {
            deletion_queue.push_back(Delete::DeviceMemory(m))
        }

        let prev_buffer = self
            .data
            .chunk_index_buffer
            .insert(mesh_id, prepared_mesh.index.target.buffer);
        if let Some(b) = prev_buffer {
            deletion_queue.push_back(Delete::Buffer(b))
        }
        let prev_memory = self
            .data
            .chunk_vertex_buffers_memory
            .insert(mesh_id, prepared_mesh.index.target.memory);
        if let Some(m) = prev_memory {
            deletion_queue.push_back(Delete::DeviceMemory(m))
        }
    }

    unsafe fn update_secondary_command_buffer(
        &mut self,
        image_index: usize,
        id: &MeshId,
        index_count: usize,
    ) -> Result<vk::CommandBuffer> {
        // println!("{LOG_VK} ImageIndex({image_index}) Updating sec buffer");
        let command_buffers = &mut self.data.secondary_command_buffers[image_index];
        if !command_buffers.contains_key(id) {
            // println!("{LOG_VK} Index({index}) Allocating seconary cmd buffer");
            let allocate_info = vk::CommandBufferAllocateInfo::builder()
                .command_pool(self.data.command_pools[image_index])
                .level(vk::CommandBufferLevel::SECONDARY)
                .command_buffer_count(1);

            let command_buffer = self.device.allocate_command_buffers(&allocate_info)?[0];

            command_buffers.insert(*id, command_buffer);
        }

        let cmd_buffer = command_buffers[id];
        let vertex_buffer = self.data.chunk_vertex_buffers[id];

        let start = WorldPosition::from(id.chunk_id());
        let x = start.x as f32;
        let y = start.y as f32;
        let mut z = start.z as f32;

        if id.is_water() {
            z -= 2.0 / 12.0;
        }

        // let x = start.x as f32 * 1.05;
        // let y = start.y as f32 * 1.05;
        // let z = start.z as f32 * 1.05;

        let model = glm::translate(&glm::identity(), &glm::vec3(x, y, z));
        let (_, model_bytes, _) = model.as_slice().align_to::<u8>();

        let time = self.start.elapsed().as_secs_f32() / 20.0;
        let time_bytes = time.to_le_bytes();

        let inheritance_info = vk::CommandBufferInheritanceInfo::builder()
            .render_pass(self.data.render_pass)
            .subpass(0)
            .framebuffer(self.data.framebuffers[image_index]);

        let info = vk::CommandBufferBeginInfo::builder()
            .flags(vk::CommandBufferUsageFlags::RENDER_PASS_CONTINUE)
            .inheritance_info(&inheritance_info);

        self.device.begin_command_buffer(cmd_buffer, &info)?;

        self.device.cmd_bind_pipeline(
            cmd_buffer,
            vk::PipelineBindPoint::GRAPHICS,
            self.data.pipeline,
        );
        self.device
            .cmd_bind_vertex_buffers(cmd_buffer, 0, &[vertex_buffer], &[0]);
        self.device.cmd_bind_index_buffer(
            cmd_buffer,
            self.data.chunk_index_buffer[id],
            0,
            vk::IndexType::UINT32,
        );
        self.device.cmd_bind_descriptor_sets(
            cmd_buffer,
            vk::PipelineBindPoint::GRAPHICS,
            self.data.pipeline_layout,
            0,
            &[self.data.descriptor_sets[image_index]],
            &[],
        );
        self.device.cmd_push_constants(
            cmd_buffer,
            self.data.pipeline_layout,
            vk::ShaderStageFlags::VERTEX,
            0,
            model_bytes,
        );
        self.device.cmd_push_constants(
            cmd_buffer,
            self.data.pipeline_layout,
            vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT,
            64,
            &time_bytes,
        );
        self.device
            .cmd_draw_indexed(cmd_buffer, index_count as u32, 1, 0, 0, 0);

        self.device.end_command_buffer(cmd_buffer)?;

        Ok(cmd_buffer)
    }

    unsafe fn update_uniform_buffer(
        &self,
        image_index: usize,
        vp: (glm::Mat4, glm::Mat4),
        player: glm::Vec4,
    ) -> Result<()> {
        let (view, proj) = vp;
        let ubo = UniformBufferObject { view, proj, player };
        let memory = self.device.map_memory(
            self.data.uniform_buffers_memory[image_index],
            0,
            size_of::<UniformBufferObject>() as u64,
            vk::MemoryMapFlags::empty(),
        )?;

        memcpy(&ubo, memory.cast(), 1);

        self.device
            .unmap_memory(self.data.uniform_buffers_memory[image_index]);

        Ok(())
    }

    fn calculate_vp(&self, fly_cam: &FlyingCamera) -> (glm::Mat4, glm::Mat4) {
        let view = fly_cam.cam.look_at();

        let mut proj = glm::perspective_rh_zo(
            self.data.swapchain_extent.width as f32 / self.data.swapchain_extent.height as f32,
            glm::radians(&glm::vec1(45.0))[0],
            0.1, // must be smaller than player collision radius
            2000.,
        );
        proj[(1, 1)] *= -1.0;

        (view, proj)
    }

    pub unsafe fn recreate_swapchain(&mut self, window: &Window) -> Result<()> {
        self.device.device_wait_idle()?;
        self.destroy_swapchain();
        create_swapchain(window, &self.instance, &self.device, &mut self.data)?;
        create_swapchain_image_views(&self.device, &mut self.data)?;
        create_render_pass(&self.instance, &self.device, &mut self.data)?;
        create_pipeline(&self.device, &mut self.data)?;
        create_depth_objects(&self.instance, &self.device, &mut self.data)?;
        create_framebuffers(&self.device, &mut self.data)?;
        create_uniform_buffers(&self.instance, &self.device, &mut self.data)?;
        create_descriptor_pool(&self.device, &mut self.data)?;
        create_descriptor_sets(&self.device, &mut self.data)?;
        create_command_buffers(&self.device, &mut self.data)?;
        println!("Swapchain swapped.");
        Ok(())
    }

    /// Destroys our Vulkan app.
    pub unsafe fn destroy(&mut self) {
        self.device.device_wait_idle().unwrap();

        self.destroy_swapchain();
        self.data
            .command_pools
            .iter()
            .for_each(|p| self.device.destroy_command_pool(*p, None));

        // self.device.destroy_sampler(self.data.texture_sampler, None);
        self.device
            .destroy_image_view(self.data.texture_image_view, None);
        self.device.destroy_image(self.data.texture_image, None);
        self.device
            .free_memory(self.data.texture_image_memory, None);

        self.device
            .destroy_descriptor_set_layout(self.data.descriptor_set_layout, None);

        self.free_chunk_memory();

        // index
        self.device.destroy_buffer(self.data.index_buffer, None);
        self.device.free_memory(self.data.index_buffer_memory, None);

        // vertex
        self.device.destroy_buffer(self.data.vertex_buffer, None);
        self.device
            .free_memory(self.data.vertex_buffer_memory, None);

        self.data
            .in_flight_fences
            .iter()
            .for_each(|f| self.device.destroy_fence(*f, None));
        self.data
            .render_finished_semaphores
            .iter()
            .for_each(|s| self.device.destroy_semaphore(*s, None));
        self.data
            .image_available_semaphores
            .iter()
            .for_each(|s| self.device.destroy_semaphore(*s, None));
        self.device
            .destroy_command_pool(self.data.command_pool, None);
        self.device.destroy_device(None);
        self.instance.destroy_surface_khr(self.data.surface, None);

        if VALIDATION_ENABLED {
            self.instance
                .destroy_debug_utils_messenger_ext(self.data.messenger, None);
        }

        self.instance.destroy_instance(None);
    }

    pub unsafe fn free_chunk_memory(&mut self) {
        for (_, buffer) in &self.data.chunk_index_buffer {
            self.device.destroy_buffer(*buffer, None);
        }
        for (_, memory) in &self.data.chunk_index_buffer_memory {
            self.device.free_memory(*memory, None);
        }

        for (_, buffer) in &self.data.chunk_vertex_buffers {
            self.device.destroy_buffer(*buffer, None);
        }
        for (_, memory) in &self.data.chunk_vertex_buffers_memory {
            self.device.free_memory(*memory, None);
        }

        self.data.chunk_index_buffer.clear();
        self.data.chunk_index_buffer_memory.clear();
        self.data.chunk_vertex_buffers.clear();
        self.data.chunk_vertex_buffers_memory.clear();
    }

    pub unsafe fn unload_single_chunk(&mut self, mesh_id: &MeshId) {
        if let Some(buffer) = self.data.chunk_index_buffer.remove(mesh_id) {
            self.device.destroy_buffer(buffer, None);
        }
        if let Some(memory) = self.data.chunk_index_buffer_memory.remove(mesh_id) {
            self.device.free_memory(memory, None);
        }

        if let Some(buffer) = self.data.chunk_vertex_buffers.remove(mesh_id) {
            self.device.destroy_buffer(buffer, None);
        }
        if let Some(memory) = self.data.chunk_vertex_buffers_memory.remove(mesh_id) {
            self.device.free_memory(memory, None);
        }
    }

    unsafe fn destroy_swapchain(&mut self) {
        self.device
            .destroy_image_view(self.data.depth_image_view, None);
        self.device.free_memory(self.data.depth_image_memory, None);
        self.device.destroy_image(self.data.depth_image, None);
        self.device
            .destroy_descriptor_pool(self.data.descriptor_pool, None);
        self.data
            .uniform_buffers
            .iter()
            .for_each(|b| self.device.destroy_buffer(*b, None));
        self.data
            .uniform_buffers_memory
            .iter()
            .for_each(|m| self.device.free_memory(*m, None));
        self.data
            .framebuffers
            .iter()
            .for_each(|f| self.device.destroy_framebuffer(*f, None));
        self.device.destroy_pipeline(self.data.pipeline, None);
        self.device
            .destroy_pipeline_layout(self.data.pipeline_layout, None);
        self.device.destroy_render_pass(self.data.render_pass, None);
        self.data
            .swapchain_image_views
            .iter()
            .for_each(|v| self.device.destroy_image_view(*v, None));
        self.device.destroy_swapchain_khr(self.data.swapchain, None);
    }
}
