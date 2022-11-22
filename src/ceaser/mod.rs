use ash::vk;
use ash::vk::CommandBuffer;
use command_buffer::create_command_buffers;
use gpu_allocator::{vulkan::{Allocator, AllocatorCreateDesc, AllocationCreateDesc, Allocation}, MemoryLocation};
use std::mem::ManuallyDrop;
use winit::window::Window;

pub mod command_buffer;
pub mod device;
pub mod instance;
pub mod logical;
pub mod pipeline;
pub mod queue;
pub mod render_pass;
pub mod surface;
pub mod swap_chain;
pub mod buffer;

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
    pub command_buffers: Vec<CommandBuffer>,
    pub allocator: Allocator,
    buffers: Vec<buffer::Buffer>,
}

impl Ceaser {
    pub fn new(window: Window) -> Result<Ceaser, Box<dyn std::error::Error>> {
        let entry = unsafe { ash::Entry::load()? };
        let layer_names = vec!["VK_LAYER_KHRONOS_validation"];
        let instance = instance::init_instance(&entry, &layer_names, &window)?;
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

        let mut allocator = Allocator::new(&AllocatorCreateDesc {
            instance: instance.clone(),
            device: logical_device.clone(),
            physical_device: device.physical_device.clone(),
            debug_settings: Default::default(),
            buffer_device_address: false,
        })?;

        let buffer_data_1 = &[
            0.5f32, 0.0f32, 0.0f32, 1.0f32, 0.0f32, 0.2f32, 0.0f32, 1.0f32, -0.5f32, 0.0f32,
            0.0f32, 1.0f32, -0.9f32, -0.9f32, 0.0f32, 1.0f32, 0.3f32, -0.8f32, 0.0f32, 1.0f32,
            0.0f32, -0.6f32, 0.0f32, 1.0f32,
        ];
        let buffer1 = buffer::Buffer::new(
            &logical_device,
            &mut allocator,
            (buffer_data_1.len() * 4) as u64,
            vk::BufferUsageFlags::VERTEX_BUFFER,
            MemoryLocation::CpuToGpu,
        )?;
        buffer1.fill(
            &allocator,
            buffer_data_1,
        )?;
        let buffer_data_2 = &[
            15.0f32, 0.0f32, 1.0f32, 0.0f32, 1.0f32, 15.0f32, 0.0f32, 1.0f32, 0.0f32, 1.0f32,
            15.0f32, 0.0f32, 1.0f32, 0.0f32, 1.0f32, 1.0f32, 0.8f32, 0.7f32, 0.0f32, 1.0f32,
            1.0f32, 0.8f32, 0.7f32, 0.0f32, 1.0f32, 1.0f32, 0.8f32, 0.7f32, 0.0f32, 1.0f32,
        ];

        let buffer2 = buffer::Buffer::new(
            &logical_device,
            &mut allocator,
            (buffer_data_2.len() * 4) as u64,
            vk::BufferUsageFlags::VERTEX_BUFFER,
            MemoryLocation::CpuToGpu,
        )?;
        buffer2.fill(
            &allocator,
            buffer_data_2,
        )?;

        let command_buffers = create_command_buffers(&logical_device, &pools, swapchain.amount_of_images as usize)?;

        command_buffer::fill_command_buffers(
            &command_buffers,
            &logical_device,
            &render_pass,
            &swapchain,
            &pipeline,
            &buffer1.buffer,
            &buffer2.buffer
        )?;
        
        let buffers = vec![buffer1, buffer2];

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
            command_buffers,
            allocator,
            buffers,
        })
    }
}

impl Drop for Ceaser {
    fn drop(&mut self) {
        unsafe {
            self.logical_device
                .device_wait_idle()
                .expect("something wrong while waiting");
            
            for b in &self.buffers {
                self.logical_device.destroy_buffer(b.buffer, None);
            }
            self.pools.cleanup(&self.logical_device);
            self.pipeline.cleanup(&self.logical_device);
            self.logical_device
                .destroy_render_pass(self.render_pass, None);
            self.swapchain.cleanup(&self.logical_device);
            self.logical_device.destroy_device(None);
            std::mem::ManuallyDrop::drop(&mut self.surfaces);
            std::mem::ManuallyDrop::drop(&mut self.debug);
            self.instance.destroy_instance(None)
        };
    }
}
