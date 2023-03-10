pub mod logical;
pub mod memory;
pub mod physical;
pub mod queuefamilies;
pub mod sustainabilityerror;
pub mod swapchainsupport;

use self::{logical::create_logical_device, physical::pick_physical_device};
use crate::appdata::AppData;
use anyhow::Result;
use vulkanalia::prelude::v1_0::*;

pub unsafe fn pick_device(instance: &Instance, data: &mut AppData) -> Result<Device> {
    pick_physical_device(instance, data)?;
    create_logical_device(instance, data)
}
