use crate::{
    appdata::AppData,
    device::{queuefamilies::QueueFamilyIndices, swapchainsupport::SwapchainSupport},
    image::create_image_view,
};
use anyhow::Result;
use vulkanalia::{prelude::v1_0::*, vk::KhrSwapchainExtension};
use winit::window::Window;

pub fn get_swapchain_surface_format(formats: &[vk::SurfaceFormatKHR]) -> vk::SurfaceFormatKHR {
    formats
        .iter()
        .cloned()
        .find(|f| {
            f.format == vk::Format::B8G8R8A8_SRGB
                && f.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
        })
        .unwrap_or_else(|| formats[0])
}

pub fn get_swapchain_present_mode(present_modes: &[vk::PresentModeKHR]) -> vk::PresentModeKHR {
    present_modes
        .iter()
        .cloned()
        .find(|m| *m == vk::PresentModeKHR::MAILBOX)
        .unwrap_or(vk::PresentModeKHR::FIFO)
}

pub fn get_swapchain_extent(
    window: &Window,
    capabilities: vk::SurfaceCapabilitiesKHR,
) -> vk::Extent2D {
    if capabilities.current_extent.width != u32::MAX {
        capabilities.current_extent
    } else {
        let size = window.inner_size();
        let clamp = |min: u32, max: u32, v: u32| min.max(max.min(v));
        vk::Extent2D::builder()
            .width(clamp(
                capabilities.min_image_extent.width,
                capabilities.max_image_extent.width,
                size.width,
            ))
            .height(clamp(
                capabilities.min_image_extent.height,
                capabilities.max_image_extent.height,
                size.height,
            ))
            .build()
    }
}

pub unsafe fn create_swapchain(
    window: &Window,
    instance: &Instance,
    device: &Device,
    data: &mut AppData,
) -> Result<()> {
    let indices = QueueFamilyIndices::get(instance, data.surface, data.physical_device)?;
    let support = SwapchainSupport::get(instance, data.surface, data.physical_device)?;

    let surface_format = get_swapchain_surface_format(&support.formats);
    let present_mode = get_swapchain_present_mode(&support.present_modes);
    let extent = get_swapchain_extent(window, support.capabilities);

    let mut image_count = support.capabilities.min_image_count + 1;
    if support.capabilities.max_image_count != 0
        && image_count > support.capabilities.max_image_count
    {
        image_count = support.capabilities.max_image_count;
    }

    let mut queue_family_indices = vec![];
    let image_sharing_mode = if indices.graphics != indices.present {
        queue_family_indices.push(indices.graphics);
        queue_family_indices.push(indices.present);
        vk::SharingMode::CONCURRENT
    } else {
        vk::SharingMode::EXCLUSIVE
    };

    let info = vk::SwapchainCreateInfoKHR::builder()
        .surface(data.surface)
        .min_image_count(image_count)
        .image_format(surface_format.format)
        .image_color_space(surface_format.color_space)
        .image_extent(extent)
        .image_array_layers(1)
        .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
        .image_sharing_mode(image_sharing_mode)
        .queue_family_indices(&queue_family_indices)
        .pre_transform(support.capabilities.current_transform)
        .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
        .present_mode(present_mode)
        .clipped(true)
        .old_swapchain(vk::SwapchainKHR::null());

    data.swapchain_format = surface_format.format;
    data.swapchain_extent = extent;
    data.swapchain = device.create_swapchain_khr(&info, None)?;
    data.swapchain_images = device.get_swapchain_images_khr(data.swapchain)?;

    Ok(())
}

pub unsafe fn create_swapchain_image_views(device: &Device, data: &mut AppData) -> Result<()> {
    data.swapchain_image_views = data
        .swapchain_images
        .iter()
        .map(|i| {
            create_image_view(
                device,
                *i,
                data.swapchain_format,
                vk::ImageAspectFlags::COLOR,
            )
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(())
}
