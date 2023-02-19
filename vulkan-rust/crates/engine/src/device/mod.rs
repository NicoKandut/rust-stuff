mod logical;
mod physical;
pub mod queuefamilies;
mod sustainabilityerror;
pub mod swapchainsupport;

use self::{logical::create_logical_device, physical::pick_physical_device};
use crate::AppData;
use anyhow::Result;
use vulkanalia::{Device, Instance};

pub unsafe fn pick_device(instance: &Instance, data: &mut AppData) -> Result<Device> {
    pick_physical_device(instance, data)?;
    create_logical_device(instance, data)
}
