pub mod logical;
pub mod memory;
pub mod physical;
pub mod queuefamilies;
pub mod sustainabilityerror;
pub mod swapchainsupport;

use self::{logical::LogicalDevice, physical::pick_physical_device};
use anyhow::Result;
use vulkanalia::prelude::v1_0::*;

pub unsafe fn pick_device(
    instance: &Instance,
    surface: vk::SurfaceKHR,
) -> Result<(LogicalDevice, vk::PhysicalDevice)> {
    let physical_device = pick_physical_device(instance, surface)?;
    let logical_device = LogicalDevice::new(instance, physical_device, surface)?;
    Ok((logical_device, physical_device))
}
