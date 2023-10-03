use super::queuefamilies::QueueFamilyIndices;
use crate::constants::{DEVICE_EXTENSIONS, VALIDATION_ENABLED, VALIDATION_LAYER};
use anyhow::Result;
use std::collections::HashSet;
use vulkanalia::prelude::v1_0::*;

pub struct LogicalDevice {
    pub device: Device,
    pub graphics_queue: vk::Queue,
    pub present_queue: vk::Queue,
}

impl LogicalDevice {
    pub unsafe fn new(
        instance: &Instance,
        physical_device: vk::PhysicalDevice,
        surface: vk::SurfaceKHR,
    ) -> Result<Self> {
        let indices = QueueFamilyIndices::get(instance, surface, physical_device)?;
        let unique_indices = HashSet::from([indices.graphics, indices.present]);
        let queue_priorities = &[1.0];
        let queue_infos = unique_indices
            .iter()
            .map(|i| {
                vk::DeviceQueueCreateInfo::builder()
                    .queue_family_index(*i)
                    .queue_priorities(queue_priorities)
            })
            .collect::<Vec<_>>();
        let layers = if VALIDATION_ENABLED {
            vec![VALIDATION_LAYER.as_ptr()]
        } else {
            vec![]
        };
        let extensions = DEVICE_EXTENSIONS
            .iter()
            .map(|n| n.as_ptr())
            .collect::<Vec<_>>();
        let features = vk::PhysicalDeviceFeatures::builder().sampler_anisotropy(true);
        let info = vk::DeviceCreateInfo::builder()
            .queue_create_infos(&queue_infos)
            .enabled_layer_names(&layers)
            .enabled_extension_names(&extensions)
            .enabled_features(&features);

        let device = instance.create_device(physical_device, &info, None)?;
        let graphics_queue = device.get_device_queue(indices.graphics, 0);
        let present_queue = device.get_device_queue(indices.present, 0);

        Ok(Self {
            device,
            graphics_queue,
            present_queue,
        })
    }
}
