use ash::vk;
use gpu_allocator::vulkan::{Allocator, AllocatorCreateDesc};

use crate::{surface::Surface, queue::{QueueFamilies, Queues}};

pub struct Swapchain {
    pub swapchain_loader: ash::extensions::khr::Swapchain,
    pub swapchain: vk::SwapchainKHR,
    pub images: Vec<vk::Image>,
    pub imageviews: Vec<vk::ImageView>,
    pub depth_image: vk::Image,                              
    pub depth_image_allocation: gpu_allocator::vulkan::Allocation,
    pub depth_imageview: vk::ImageView,
    pub framebuffers: Vec<vk::Framebuffer>,
    pub surface_format: vk::SurfaceFormatKHR,
    pub extent: vk::Extent2D,
    pub image_available: Vec<vk::Semaphore>,
    pub rendering_finished: Vec<vk::Semaphore>,
    pub may_begin_drawing: Vec<vk::Fence>,
    pub amount_of_images: u32,
    pub current_image: usize,
}

impl Swapchain {
    pub fn new(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        logical_device: &ash::Device,
        surfaces: &Surface,
        queue_families: &QueueFamilies,
        _queues: &Queues,
        allocator: &mut gpu_allocator::vulkan::Allocator,
    ) -> Result<Swapchain, vk::Result> {
        let surface_capabilities = surfaces.get_capabilities(physical_device)?;
        let extent = surface_capabilities.current_extent;
        let surface_present_modes = surfaces.get_present_modes(physical_device)?;
        let surface_format = *surfaces.get_formats(physical_device)?.first().unwrap();
        let queuefamilies = [queue_families.graphics_q_index.unwrap()];
        let swapchain_create_info = vk::SwapchainCreateInfoKHR::builder()
            .surface(surfaces.surface)
            .min_image_count(
                3.max(surface_capabilities.min_image_count)
                    .min(surface_capabilities.max_image_count),
            )
            .image_format(surface_format.format)
            .image_color_space(surface_format.color_space)
            .image_extent(extent)
            .image_array_layers(1)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
            .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
            .queue_family_indices(&queuefamilies)
            .pre_transform(surface_capabilities.current_transform)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(vk::PresentModeKHR::FIFO);
        let swapchain_loader = ash::extensions::khr::Swapchain::new(instance, logical_device);
        let swapchain = unsafe { swapchain_loader.create_swapchain(&swapchain_create_info, None)? };
        let swapchain_images = unsafe { swapchain_loader.get_swapchain_images(swapchain)? };
        let amount_of_images = swapchain_images.len() as u32;
        let mut swapchain_imageviews = Vec::with_capacity(swapchain_images.len());
        for image in &swapchain_images {
            let subresource_range = vk::ImageSubresourceRange::builder()
                .aspect_mask(vk::ImageAspectFlags::COLOR)
                .base_mip_level(0)
                .level_count(1)
                .base_array_layer(0)
                .layer_count(1);
            let imageview_create_info = vk::ImageViewCreateInfo::builder()
                .image(*image)
                .view_type(vk::ImageViewType::TYPE_2D)
                .format(vk::Format::B8G8R8A8_UNORM)
                .subresource_range(*subresource_range);
            let imageview =
                unsafe { logical_device.create_image_view(&imageview_create_info, None) }?;
            swapchain_imageviews.push(imageview);
        }
        
        let extent3d = vk::Extent3D {
            width: extent.width,
            height: extent.height,
            depth: 1,
        };
        let depth_image_info = vk::ImageCreateInfo::builder()
            .image_type(vk::ImageType::TYPE_2D)
            .format(vk::Format::D32_SFLOAT)
            .extent(extent3d)
            .mip_levels(1)
            .array_layers(1)
            .samples(vk::SampleCountFlags::TYPE_1)
            .tiling(vk::ImageTiling::OPTIMAL)
            .usage(vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT)
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .queue_family_indices(&queuefamilies);

        let depth_image = unsafe {
            logical_device.create_image(&depth_image_info, None)
        }.unwrap();
        let requirements = unsafe { logical_device.get_image_memory_requirements(depth_image) };
        let depth_image_allocation_info = gpu_allocator::vulkan::AllocationCreateDesc {
            name: "Z_Buffer Image",
            requirements,
            location: gpu_allocator::MemoryLocation::GpuOnly,
            linear: true,
        };
        let depth_image_allocation = allocator.allocate(&depth_image_allocation_info).unwrap();
        unsafe {
            logical_device.bind_image_memory(depth_image, depth_image_allocation.memory(), depth_image_allocation.offset());
        }
        let subresource_range = vk::ImageSubresourceRange::builder()
            .aspect_mask(vk::ImageAspectFlags::DEPTH)
            .base_mip_level(0)
            .level_count(1)
            .base_array_layer(0)
            .layer_count(1);
        let imageview_create_info = vk::ImageViewCreateInfo::builder()
            .image(depth_image)
            .view_type(vk::ImageViewType::TYPE_2D)
            .format(vk::Format::D32_SFLOAT)
            .subresource_range(*subresource_range);
        let depth_imageview =
            unsafe { logical_device.create_image_view(&imageview_create_info, None) }?;
        
        let mut image_available = vec![];
        let mut rendering_finished = vec![];
        let mut may_begin_drawing = vec![];
        let semaphoreinfo = vk::SemaphoreCreateInfo::builder();
        let fenceinfo = vk::FenceCreateInfo::builder().flags(vk::FenceCreateFlags::SIGNALED);
        for _ in 0..amount_of_images {
            let semaphore_available =
                unsafe { logical_device.create_semaphore(&semaphoreinfo, None) }?;
            let semaphore_finished =
                unsafe { logical_device.create_semaphore(&semaphoreinfo, None) }?;
            image_available.push(semaphore_available);
            rendering_finished.push(semaphore_finished);
            let fence = unsafe { logical_device.create_fence(&fenceinfo, None) }?;
            may_begin_drawing.push(fence);
        }
        Ok(Swapchain {
            swapchain_loader,
            swapchain,
            images: swapchain_images,
            imageviews: swapchain_imageviews,
            depth_image,
            depth_image_allocation,
            depth_imageview,
            framebuffers: vec![],
            surface_format,
            extent,
            amount_of_images,
            current_image:0,
            may_begin_drawing,
            image_available,
            rendering_finished,
        })
    }

    pub fn create_framebuffers(
        &mut self,
        logical_device: &ash::Device,
        renderpass: vk::RenderPass,
    ) -> Result<(), vk::Result> {
        for iv in &self.imageviews {
            let iview = [*iv, self.depth_imageview];
            let framebuffer_info = vk::FramebufferCreateInfo::builder()
                .render_pass(renderpass)
                .attachments(&iview)
                .width(self.extent.width)
                .height(self.extent.height)
                .layers(1);
            let fb = unsafe { logical_device.create_framebuffer(&framebuffer_info, None) }?;
            self.framebuffers.push(fb);
        }
        Ok(())
    }

    pub unsafe fn cleanup(&mut self, logical_device: &ash::Device) {
        logical_device.destroy_image_view(self.depth_imageview, None);
        for fence in &self.may_begin_drawing {
            logical_device.destroy_fence(*fence, None);
        }
        for semaphore in &self.image_available {
            logical_device.destroy_semaphore(*semaphore, None);
        }
        for semaphore in &self.rendering_finished {
            logical_device.destroy_semaphore(*semaphore, None);
        }
        for fb in &self.framebuffers {
            logical_device.destroy_framebuffer(*fb, None);
        }
        for iv in &self.imageviews {
            logical_device.destroy_image_view(*iv, None);
        }
        self.swapchain_loader
            .destroy_swapchain(self.swapchain, None)
    }
}

impl Drop for Swapchain {
    fn drop(&mut self) {
        unsafe {
            self.swapchain_loader
                .destroy_swapchain(self.swapchain, None)
        };
    }
}
