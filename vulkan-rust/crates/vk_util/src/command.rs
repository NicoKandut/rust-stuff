use crate::{appdata::AppData, device::queuefamilies::QueueFamilyIndices};
use anyhow::Result;
use vulkanalia::prelude::v1_0::*;

pub unsafe fn create_command_pools(
    instance: &Instance,
    device: &Device,
    data: &mut AppData,
) -> Result<()> {
    data.command_pool = create_command_pool(instance, device, data)?;

    let num_images = data.swapchain_images.len();
    for _ in 0..num_images {
        let command_pool = create_command_pool(instance, device, data)?;
        data.command_pools.push(command_pool);
    }

    Ok(())
}

pub unsafe fn create_command_pool(
    instance: &Instance,
    device: &Device,
    data: &mut AppData,
) -> Result<vk::CommandPool> {
    let indices = QueueFamilyIndices::get(instance, data, data.physical_device)?;

    let info = vk::CommandPoolCreateInfo::builder()
        .flags(vk::CommandPoolCreateFlags::TRANSIENT)
        .queue_family_index(indices.graphics);

    Ok(device.create_command_pool(&info, None)?)
}

pub unsafe fn create_command_buffers(device: &Device, data: &mut AppData) -> Result<()> {
    let num_images = data.swapchain_images.len();
    for image_index in 0..num_images {
        let allocate_info = vk::CommandBufferAllocateInfo::builder()
            .command_pool(data.command_pools[image_index])
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(2);

        let command_buffers = device.allocate_command_buffers(&allocate_info)?;
        data.command_buffers_transfer.push(command_buffers[0]);
        data.command_buffers.push(command_buffers[1]);
    }

    data.secondary_command_buffers = [Default::default(), Default::default(), Default::default()];

    Ok(())
}
