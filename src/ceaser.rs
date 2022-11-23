use ash::vk::{self, CommandBuffer, DescriptorPool, DescriptorSet};
use command_buffer::create_command_buffers;
use gpu_allocator::{
    vulkan::{Allocator, AllocatorCreateDesc}
};
use nalgebra as na;
use std::mem::ManuallyDrop;
use winit::window::Window;

use crate::hamlet::{InstanceData, Model, VertexData};

use self::buffer::Buffer;

pub mod buffer;
pub mod command_buffer;
pub mod device;
pub mod instance;
pub mod logical;
pub mod pipeline;
pub mod queue;
pub mod render_pass;
pub mod surface;
pub mod swap_chain;
pub mod camera;

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
    pub models: Vec<Model<VertexData, InstanceData>>,
    pub uniform_buffer: Buffer,
    pub descriptor_pool: DescriptorPool,
    pub descriptor_sets_camera: Vec<vk::DescriptorSet>, 
    pub descriptor_sets_light: Vec<vk::DescriptorSet>, 
    pub light_buffer: Buffer,
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
        let mut allocator = Allocator::new(&AllocatorCreateDesc {
            instance: instance.clone(),
            device: logical_device.clone(),
            physical_device: device.physical_device.clone(),
            debug_settings: Default::default(),
            buffer_device_address: false,
        })?;

        let mut swapchain = swap_chain::Swapchain::new(
            &instance,
            device.physical_device,
            &logical_device,
            &surfaces,
            &queue_families,
            &queues,
            &mut allocator,
        )?;

        let render_pass = render_pass::init_render_pass(
            &logical_device,
            surfaces
                .get_formats(device.physical_device)?
                .first()
                .unwrap()
                .format,
        )?;

        swapchain.create_framebuffers(&logical_device, render_pass)?;

        let pipeline = pipeline::Pipeline::new(&logical_device, &swapchain, &render_pass)?;

        let pools = queue::Pools::new(&logical_device, &queue_families)?;

        let command_buffers =
            create_command_buffers(&logical_device, &pools, swapchain.amount_of_images as usize)?;

        let mut uniform_buffer = Buffer::new(
            &logical_device,
            &mut allocator,
            128,
            vk::BufferUsageFlags::UNIFORM_BUFFER,
            gpu_allocator::MemoryLocation::CpuToGpu,
        )?;
        let camera_transforms: [[[f32; 4]; 4]; 2] = [
            na::Matrix4::identity().into(),
            na::Matrix4::identity().into(),
        ];
        uniform_buffer.fill(&logical_device, &mut allocator, &camera_transforms)?;
        
        let mut light_buffer = Buffer::new(
            &logical_device,
            &mut allocator,
            8,
            vk::BufferUsageFlags::STORAGE_BUFFER,
            gpu_allocator::MemoryLocation::CpuToGpu,
        )?;
        light_buffer.fill(&logical_device, &mut allocator, &[0.,0.])?;

        let pool_sizes = [
            vk::DescriptorPoolSize {
                ty: vk::DescriptorType::UNIFORM_BUFFER,
                descriptor_count: swapchain.amount_of_images,
            },
            vk::DescriptorPoolSize {
                ty: vk::DescriptorType::STORAGE_BUFFER,
                descriptor_count: swapchain.amount_of_images,
            },
        ];
        let descriptor_pool_info = vk::DescriptorPoolCreateInfo::builder()
            .max_sets(2 * swapchain.amount_of_images) //
            .pool_sizes(&pool_sizes);
        let descriptor_pool =
            unsafe { logical_device.create_descriptor_pool(&descriptor_pool_info, None) }?;

        let desc_layouts_camera =
            vec![pipeline.descriptor_set_layouts[0]; swapchain.amount_of_images as usize];
        let descriptor_set_allocate_info_camera = vk::DescriptorSetAllocateInfo::builder()
            .descriptor_pool(descriptor_pool)
            .set_layouts(&desc_layouts_camera);
        let descriptor_sets_camera = unsafe {
            logical_device.allocate_descriptor_sets(&descriptor_set_allocate_info_camera)
        }?;

        for descset in &descriptor_sets_camera {
            let buffer_infos = [vk::DescriptorBufferInfo {
                buffer: uniform_buffer.buffer,
                offset: 0,
                range: 128,
            }];
            let desc_sets_write = [vk::WriteDescriptorSet::builder()
                .dst_set(*descset)
                .dst_binding(0)
                .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                .buffer_info(&buffer_infos)
                .build()];
            unsafe { logical_device.update_descriptor_sets(&desc_sets_write, &[]) };
        }
        let desc_layouts_light =
            vec![pipeline.descriptor_set_layouts[1]; swapchain.amount_of_images as usize];
        let descriptor_set_allocate_info_light = vk::DescriptorSetAllocateInfo::builder()
            .descriptor_pool(descriptor_pool)
            .set_layouts(&desc_layouts_light);
        let descriptor_sets_light = unsafe {
            logical_device.allocate_descriptor_sets(&descriptor_set_allocate_info_light)
        }?;

        for descset in &descriptor_sets_light {
            let buffer_infos = [vk::DescriptorBufferInfo {
                buffer: light_buffer.buffer,
                offset: 0,
                range: 8,
            }];
            let desc_sets_write = [vk::WriteDescriptorSet::builder()
                .dst_set(*descset)
                .dst_binding(0)
                .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                .buffer_info(&buffer_infos)
                .build()];
            unsafe { logical_device.update_descriptor_sets(&desc_sets_write, &[]) };
        }

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
            models: vec![],
            uniform_buffer,
            descriptor_pool,
            descriptor_sets_camera,
            descriptor_sets_light,
            light_buffer
        })
    }

    pub fn update_commandbuffer(&mut self, index: usize) -> Result<(), vk::Result> {
        let commandbuffer = self.command_buffers[index];
        let commandbuffer_begininfo = vk::CommandBufferBeginInfo::builder();
        unsafe {
            self.logical_device
                .begin_command_buffer(commandbuffer, &commandbuffer_begininfo)?;
        }
        let clearvalues = [
            vk::ClearValue {
                color: vk::ClearColorValue {
                    float32: [0.0, 0.0, 0.08, 1.0],
                },
            },
            vk::ClearValue {
                depth_stencil: vk::ClearDepthStencilValue {
                    depth: 1.0,
                    stencil: 0,
                },
            },
        ];
        let renderpass_begininfo = vk::RenderPassBeginInfo::builder()
            .render_pass(self.render_pass)
            .framebuffer(self.swapchain.framebuffers[index])
            .render_area(vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: self.swapchain.extent,
            })
            .clear_values(&clearvalues);
        unsafe {
            self.logical_device.cmd_begin_render_pass(
                commandbuffer,
                &renderpass_begininfo,
                vk::SubpassContents::INLINE,
            );
            self.logical_device.cmd_bind_pipeline(
                commandbuffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.pipeline.pipeline,
            );

            self.logical_device.cmd_bind_descriptor_sets(
                commandbuffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.pipeline.layout,
                0,
                &[
                    self.descriptor_sets_camera[index],
                    self.descriptor_sets_light[index],
                ],
                &[],
            );

            for m in &self.models {
                m.draw(&self.logical_device, commandbuffer);
            }
            self.logical_device.cmd_end_render_pass(commandbuffer);
            self.logical_device.end_command_buffer(commandbuffer)?;
        }
        Ok(())
    }
}

impl Drop for Ceaser {
    fn drop(&mut self) {
        unsafe {
            self.logical_device
                .device_wait_idle()
                .expect("something wrong while waiting");
            for m in &self.models {
                if let Some(_vb) = &m.vertexbuffer {
                    //Do later
                    println!("Remove Buffer vb")
                }
                if let Some(_ixb) = &m.indexbuffer {
                    println!("Remove Buffer ixb")
                }
                if let Some(_ib) = &m.instancebuffer {
                    println!("Remove Buffer ib")
                }
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
