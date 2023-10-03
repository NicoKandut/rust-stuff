use super::{queuefamilies::QueueFamilyIndices, swapchainsupport::SwapchainSupport};
use crate::{constants::DEVICE_EXTENSIONS, device::sustainabilityerror::SuitabilityError};
use anyhow::{anyhow, Result};
use std::collections::HashSet;
use vulkanalia::prelude::v1_0::*;

pub unsafe fn pick_physical_device(
    instance: &Instance,
    surface: vk::SurfaceKHR,
) -> Result<vk::PhysicalDevice> {
    instance
        .enumerate_physical_devices()?
        .into_iter()
        .find(|physical_device| check_physical_device(instance, surface, *physical_device).is_ok()) // TODO: rank devices and take best
        .ok_or(anyhow!("Failed to find suitable physical device."))
}

unsafe fn check_physical_device(
    instance: &Instance,
    surface: vk::SurfaceKHR,
    physical_device: vk::PhysicalDevice,
) -> Result<()> {
    // check if queue families are supported
    QueueFamilyIndices::get(instance, surface, physical_device)?;

    check_physical_device_extensions(instance, physical_device)?;

    let support = SwapchainSupport::get(instance, surface, physical_device)?;
    if support.formats.is_empty() || support.present_modes.is_empty() {
        return Err(anyhow!(SuitabilityError("Insufficient swapchain support.")));
    }

    let features = instance.get_physical_device_features(physical_device);
    if features.sampler_anisotropy != vk::TRUE {
        return Err(anyhow!(SuitabilityError("No sampler anisotropy.")));
    }

    Ok(())
}

unsafe fn check_physical_device_extensions(
    instance: &Instance,
    physical_device: vk::PhysicalDevice,
) -> Result<()> {
    let extensions = instance
        .enumerate_device_extension_properties(physical_device, None)?
        .iter()
        .map(|e| e.extension_name)
        .collect::<HashSet<_>>();
    if DEVICE_EXTENSIONS.iter().all(|e| extensions.contains(e)) {
        Ok(())
    } else {
        Err(anyhow!(SuitabilityError(
            "Missing required device extensions."
        )))
    }
}
