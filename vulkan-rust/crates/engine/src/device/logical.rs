use super::queuefamilies::QueueFamilyIndices;
use crate::{AppData, DEVICE_EXTENSIONS, VALIDATION_ENABLED, VALIDATION_LAYER};
use anyhow::Result;
use std::collections::HashSet;
use vulkanalia::{
    vk::{self, DeviceV1_0, HasBuilder},
    Device, Instance,
};

pub unsafe fn create_logical_device(instance: &Instance, data: &mut AppData) -> Result<Device> {
    let indices = QueueFamilyIndices::get(instance, data, data.physical_device)?;
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

    let features = vk::PhysicalDeviceFeatures::builder().sampler_anisotropy(true);

    let extensions = DEVICE_EXTENSIONS
        .iter()
        .map(|n| n.as_ptr())
        .collect::<Vec<_>>();

    let info = vk::DeviceCreateInfo::builder()
        .queue_create_infos(&queue_infos)
        .enabled_layer_names(&layers)
        .enabled_extension_names(&extensions)
        .enabled_features(&features);

    let device = instance.create_device(data.physical_device, &info, None)?;

    data.graphics_queue = device.get_device_queue(indices.graphics, 0);
    data.present_queue = device.get_device_queue(indices.present, 0);

    Ok(device)
}
