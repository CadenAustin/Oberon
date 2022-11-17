use ash::{vk, version::InstanceV1_0};

pub struct Device {
    pub physical_device: vk::PhysicalDevice,
    pub physical_device_properties: vk::PhysicalDeviceProperties,
}

impl Device {
    pub fn new(instance: &ash::Instance) -> Result<Device, vk::Result> {
        let phys_devs = unsafe { instance.enumerate_physical_devices()? };
        let (physical_device, physical_device_properties) = {
            let mut chosen = None;
            for p in phys_devs {
                let properties = unsafe { instance.get_physical_device_properties(p) };
                if properties.device_type == vk::PhysicalDeviceType::DISCRETE_GPU {
                    chosen = Some((p, properties));
                }
            }
            chosen.unwrap()
        };

        Ok(Self {physical_device, physical_device_properties})

    }
}