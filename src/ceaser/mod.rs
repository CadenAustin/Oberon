use ash::version::DeviceV1_0;
use ash::version::InstanceV1_0;
use ash::vk;
use ash::vk::CommandBuffer;
use std::mem::ManuallyDrop;
use winit::window::Window;

pub mod device;
pub mod instance;
pub mod logical;
pub mod queue;
pub mod render_pass;
pub mod surface;
pub mod swap_chain;
pub mod pipeline;
pub mod command_buffer;

pub struct Ceaser {
    pub window: winit::window::Window,
    pub entry: ash::Entry,
    pub instance: ash::Instance,
    pub debug: ManuallyDrop<instance::Debug>,
    pub surfaces: ManuallyDrop<surface::Surface>,
    pub device: device::Device,
    pub queue_families: queue::QueueFamilies,
    pub queues: queue::Queues,
    pub logical_device: ash::Device,
    pub swapchain: swap_chain::Swapchain,
    pub render_pass: vk::RenderPass,
    pub pipeline: pipeline::Pipeline,
    pub pools: queue::Pools,
    pub command_buffers: Vec<CommandBuffer>
}

impl Ceaser {
    pub fn new(window: Window) -> Result<Ceaser, Box<dyn std::error::Error>> {
        let entry = ash::Entry::new()?;
        let layer_names = vec!["VK_LAYER_KHRONOS_validation"];
        let instance = instance::init_instance(&entry, &layer_names)?;
        let debug = instance::Debug::new(&entry, &instance)?;
        let surfaces = surface::Surface::new(&window, &entry, &instance)?;
        let device = device::Device::new(&instance)?;
        let queue_families =
            queue::QueueFamilies::new(&instance, device.physical_device, &surfaces)?;
        let (logical_device, queues) = logical::init_device_and_queues(
            &instance,
            device.physical_device,
            &queue_families,
            &layer_names,
        )?;
        let mut swapchain = swap_chain::Swapchain::new(
            &instance,
            device.physical_device,
            &logical_device,
            &surfaces,
            &queue_families,
            &queues,
        )?;

        let render_pass =
            render_pass::init_renderpass(&logical_device, device.physical_device, &surfaces)?;

        swapchain.create_framebuffers(&logical_device, render_pass)?;

        let pipeline = pipeline::Pipeline::new(&logical_device, &swapchain, &render_pass)?;
        
        let pools = queue::Pools::new(&logical_device, &queue_families)?;

        let command_buffers = command_buffer::create_command_buffers(&logical_device, &pools, swapchain.framebuffers.len())?;

        command_buffer::fill_command_buffers(&command_buffers,
            &logical_device,
            &render_pass,
            &swapchain,
            &pipeline
        )?;

        Ok(Self {
            window,
            entry,
            instance,
            debug: std::mem::ManuallyDrop::new(debug),
            surfaces: std::mem::ManuallyDrop::new(surfaces),
            device,
            queue_families,
            queues,
            logical_device,
            swapchain,
            render_pass,
            pipeline,
            pools,
            command_buffers
        })
    }
}

impl Drop for Ceaser {
    fn drop(&mut self) {
        unsafe {
            self.pools.cleanup(&self.logical_device);
            self.pipeline.cleanup(&self.logical_device);
            self.logical_device.destroy_render_pass(self.render_pass, None);
            self.swapchain.cleanup(&self.logical_device);
            self.logical_device.destroy_device(None);
            std::mem::ManuallyDrop::drop(&mut self.surfaces);
            std::mem::ManuallyDrop::drop(&mut self.debug);
            self.instance.destroy_instance(None)
        };
    }
}
