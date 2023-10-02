use crate::{
    appdata::AppData,
    device::memory::get_memory_type_index,
    image::{begin_single_time_commands, end_single_time_commands},
};
use anyhow::Result;
use graphics::{Mesh, Vertex};
use std::{mem::size_of, ptr::copy_nonoverlapping as memcpy};
use vulkanalia::{
    prelude::v1_0::*,
    vk::{Buffer, DeviceMemory, PhysicalDevice},
};
use world::MeshId;

pub unsafe fn create_vertex_buffer(
    instance: &Instance,
    device: &Device,
    data: &mut AppData,
) -> Result<()> {
    let size = (size_of::<Vertex>() * data.vertices.len()) as u64;

    let (staging_buffer, staging_buffer_memory) = create_buffer(
        instance,
        device,
        data,
        size,
        vk::BufferUsageFlags::TRANSFER_SRC,
        vk::MemoryPropertyFlags::HOST_COHERENT | vk::MemoryPropertyFlags::HOST_VISIBLE,
    )?;

    let memory = device.map_memory(staging_buffer_memory, 0, size, vk::MemoryMapFlags::empty())?;

    memcpy(data.vertices.as_ptr(), memory.cast(), data.vertices.len());

    device.unmap_memory(staging_buffer_memory);

    let (vertex_buffer, vertex_buffer_memory) = create_buffer(
        instance,
        device,
        data,
        size,
        vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::VERTEX_BUFFER,
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
    )?;

    data.vertex_buffer = vertex_buffer;
    data.vertex_buffer_memory = vertex_buffer_memory;

    copy_buffer(device, data, staging_buffer, vertex_buffer, size)?;
    device.destroy_buffer(staging_buffer, None);
    device.free_memory(staging_buffer_memory, None);

    Ok(())
}

pub unsafe fn create_chunk_buffers(
    instance: &Instance,
    device: &Device,
    id: MeshId,
    mesh: Mesh,
    data: &mut AppData,
) -> Result<()> {
    create_chunk_vertex_buffer(instance, device, id, &mesh.vertices, data)?;
    create_chunk_index_buffer(instance, device, data, id, &mesh.indices)?;
    Ok(())
}

pub unsafe fn create_chunk_vertex_buffer(
    instance: &Instance,
    device: &Device,
    id: MeshId,
    vertices: &Vec<Vertex>,
    data: &mut AppData,
) -> Result<()> {
    let size = (size_of::<Vertex>() * vertices.len()) as u64;

    let (staging_buffer, staging_buffer_memory) = create_buffer(
        instance,
        device,
        data,
        size,
        vk::BufferUsageFlags::TRANSFER_SRC,
        vk::MemoryPropertyFlags::HOST_COHERENT | vk::MemoryPropertyFlags::HOST_VISIBLE,
    )?;

    let memory = device.map_memory(staging_buffer_memory, 0, size, vk::MemoryMapFlags::empty())?;

    memcpy(vertices.as_ptr(), memory.cast(), vertices.len());

    device.unmap_memory(staging_buffer_memory);

    let (vertex_buffer, vertex_buffer_memory) = create_buffer(
        instance,
        device,
        data,
        size,
        vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::VERTEX_BUFFER,
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
    )?;

    // TODO: not great causes stutter
    device.device_wait_idle().unwrap();

    let prev_buffer = data.chunk_vertex_buffers.insert(id.clone(), vertex_buffer);
    if let Some(b) = prev_buffer {
        // TODO: is destroying correct?
        device.destroy_buffer(b, None);
    }

    let prev_memory = data
        .chunk_vertex_buffers_memory
        .insert(id.clone(), vertex_buffer_memory);
    if let Some(m) = prev_memory {
        // TODO: is freeing correct?
        device.free_memory(m, None);
    }

    copy_buffer(device, data, staging_buffer, vertex_buffer, size)?;
    device.destroy_buffer(staging_buffer, None);
    device.free_memory(staging_buffer_memory, None);

    Ok(())
}

pub unsafe fn create_buffer(
    instance: &Instance,
    device: &Device,
    data: &AppData,
    size: vk::DeviceSize,
    usage: vk::BufferUsageFlags,
    properties: vk::MemoryPropertyFlags,
) -> Result<(vk::Buffer, vk::DeviceMemory)> {
    // println!("{LOG_VK} Creating buffer");
    let buffer_info = vk::BufferCreateInfo::builder()
        .size(size)
        .usage(usage)
        .sharing_mode(vk::SharingMode::EXCLUSIVE);

    let buffer = device.create_buffer(&buffer_info, None)?;

    let requirements = device.get_buffer_memory_requirements(buffer);

    let memory_info = vk::MemoryAllocateInfo::builder()
        .allocation_size(requirements.size)
        .memory_type_index(get_memory_type_index(
            instance,
            &data.physical_device,
            properties,
            requirements,
        )?);

    let buffer_memory = device.allocate_memory(&memory_info, None)?;

    device.bind_buffer_memory(buffer, buffer_memory, 0)?;

    Ok((buffer, buffer_memory))
}

pub unsafe fn copy_buffer(
    device: &Device,
    data: &AppData,
    source: vk::Buffer,
    destination: vk::Buffer,
    size: vk::DeviceSize,
) -> Result<()> {
    let command_buffer = begin_single_time_commands(device, data)?;

    let regions = vk::BufferCopy::builder().size(size);

    device.cmd_copy_buffer(command_buffer, source, destination, &[regions]);

    end_single_time_commands(device, data, command_buffer)?;

    Ok(())
}

pub unsafe fn create_index_buffer(
    instance: &Instance,
    device: &Device,
    data: &mut AppData,
) -> Result<()> {
    let size = (size_of::<u32>() * data.indices.len()) as u64;

    let (staging_buffer, staging_buffer_memory) = create_buffer(
        instance,
        device,
        data,
        size,
        vk::BufferUsageFlags::TRANSFER_SRC,
        vk::MemoryPropertyFlags::HOST_COHERENT | vk::MemoryPropertyFlags::HOST_VISIBLE,
    )?;

    let memory = device.map_memory(staging_buffer_memory, 0, size, vk::MemoryMapFlags::empty())?;

    memcpy(data.indices.as_ptr(), memory.cast(), data.indices.len());

    device.unmap_memory(staging_buffer_memory);

    let (index_buffer, index_buffer_memory) = create_buffer(
        instance,
        device,
        data,
        size,
        vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::INDEX_BUFFER,
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
    )?;

    data.index_buffer = index_buffer;
    data.index_buffer_memory = index_buffer_memory;

    copy_buffer(device, data, staging_buffer, index_buffer, size)?;

    device.destroy_buffer(staging_buffer, None);
    device.free_memory(staging_buffer_memory, None);

    Ok(())
}

pub unsafe fn create_chunk_index_buffer(
    instance: &Instance,
    device: &Device,
    data: &mut AppData,
    id: MeshId,
    indices: &Vec<u32>,
) -> Result<()> {
    let size = (size_of::<u32>() * indices.len()) as u64;

    let (staging_buffer, staging_buffer_memory) = create_buffer(
        instance,
        device,
        data,
        size,
        vk::BufferUsageFlags::TRANSFER_SRC,
        vk::MemoryPropertyFlags::HOST_COHERENT | vk::MemoryPropertyFlags::HOST_VISIBLE,
    )?;

    let memory = device.map_memory(staging_buffer_memory, 0, size, vk::MemoryMapFlags::empty())?;

    memcpy(indices.as_ptr(), memory.cast(), indices.len());

    device.unmap_memory(staging_buffer_memory);

    let (index_buffer, index_buffer_memory) = create_buffer(
        instance,
        device,
        data,
        size,
        vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::INDEX_BUFFER,
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
    )?;

    // TODO: not great causes stutter
    device.device_wait_idle().unwrap();

    let prev_buffer = data.chunk_index_buffer.insert(id, index_buffer);
    if let Some(b) = prev_buffer {
        // TODO: is destroying correct? Yes but do it at a later poit with a queue
        device.destroy_buffer(b, None);
    }

    let prev_memory = data
        .chunk_index_buffer_memory
        .insert(id.clone(), index_buffer_memory);
    if let Some(m) = prev_memory {
        // TODO: is freeing correct?
        device.free_memory(m, None);
    }

    copy_buffer(device, data, staging_buffer, index_buffer, size)?;

    device.destroy_buffer(staging_buffer, None);
    device.free_memory(staging_buffer_memory, None);

    Ok(())
}

pub unsafe fn create_vertex_buffer_cmd(
    instance: &Instance,
    device: &Device,
    id: MeshId,
    vertices: &Vec<Vertex>,
    physical_device: &PhysicalDevice,
) -> Result<(Buffer, DeviceMemory, Buffer, DeviceMemory)> {
    let size = (size_of::<Vertex>() * vertices.len()) as u64;

    let (staging_buffer, staging_buffer_memory) = create_buffer2(
        instance,
        device,
        physical_device,
        size,
        vk::BufferUsageFlags::TRANSFER_SRC,
        vk::MemoryPropertyFlags::HOST_COHERENT | vk::MemoryPropertyFlags::HOST_VISIBLE,
    )?;

    let memory = device.map_memory(staging_buffer_memory, 0, size, vk::MemoryMapFlags::empty())?;

    memcpy(vertices.as_ptr(), memory.cast(), vertices.len());

    device.unmap_memory(staging_buffer_memory);

    let (vertex_buffer, vertex_buffer_memory) = create_buffer2(
        instance,
        device,
        physical_device,
        size,
        vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::VERTEX_BUFFER,
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
    )?;

    Ok((
        staging_buffer,
        staging_buffer_memory,
        vertex_buffer,
        vertex_buffer_memory,
    ))
}

pub unsafe fn create_typed_buffer_cmd<T>(
    instance: &Instance,
    device: &Device,
    id: MeshId,
    entries: &[T],
    physical_device: &PhysicalDevice,
    usage_flags: vk::BufferUsageFlags,
) -> Result<(Buffer, DeviceMemory, Buffer, DeviceMemory)> {
    let size = (size_of::<T>() * entries.len()) as u64;

    let (staging_buffer, staging_buffer_memory) = create_buffer2(
        instance,
        device,
        physical_device,
        size,
        vk::BufferUsageFlags::TRANSFER_SRC,
        vk::MemoryPropertyFlags::HOST_COHERENT | vk::MemoryPropertyFlags::HOST_VISIBLE,
    )?;

    let memory = device.map_memory(staging_buffer_memory, 0, size, vk::MemoryMapFlags::empty())?;

    memcpy(entries.as_ptr(), memory.cast(), entries.len());

    device.unmap_memory(staging_buffer_memory);

    let (buffer, buffer_memory) = create_buffer2(
        instance,
        device,
        physical_device,
        size,
        vk::BufferUsageFlags::TRANSFER_DST | usage_flags,
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
    )?;

    Ok((staging_buffer, staging_buffer_memory, buffer, buffer_memory))
}

unsafe fn create_buffer2(
    instance: &Instance,
    device: &Device,
    physical_device: &PhysicalDevice,
    size: vk::DeviceSize,
    usage: vk::BufferUsageFlags,
    properties: vk::MemoryPropertyFlags,
) -> Result<(vk::Buffer, vk::DeviceMemory)> {
    // println!("{LOG_VK} Creating buffer");
    let buffer_info = vk::BufferCreateInfo::builder()
        .size(size)
        .usage(usage)
        .sharing_mode(vk::SharingMode::EXCLUSIVE);

    let buffer = device.create_buffer(&buffer_info, None)?;

    let requirements = device.get_buffer_memory_requirements(buffer);

    let memory_type_index =
        get_memory_type_index(instance, physical_device, properties, requirements)?;

    let memory_info = vk::MemoryAllocateInfo::builder()
        .allocation_size(requirements.size)
        .memory_type_index(memory_type_index);

    let buffer_memory = device.allocate_memory(&memory_info, None)?;

    device.bind_buffer_memory(buffer, buffer_memory, 0)?;

    Ok((buffer, buffer_memory))
}
