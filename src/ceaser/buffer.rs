use std::fmt::Error;

use ash::vk;

pub struct Buffer {
    pub buffer: vk::Buffer,
    pub allocation: gpu_allocator::vulkan::Allocation,
}

impl Buffer {
    pub fn new(
        logical_device: &ash::Device,
        allocator: &mut gpu_allocator::vulkan::Allocator,
        size_in_bytes: u64,
        usage: vk::BufferUsageFlags,
        memory_usage: gpu_allocator::MemoryLocation,
    ) -> Result<Buffer, Box<dyn std::error::Error>> {
        let vk_info = vk::BufferCreateInfo::builder()
            .size(size_in_bytes)
            .usage(usage);

        let buffer = unsafe { logical_device.create_buffer(&vk_info, None) }.unwrap();
        let requirements = unsafe { logical_device.get_buffer_memory_requirements(buffer) };

        let allocation = allocator
            .allocate(&gpu_allocator::vulkan::AllocationCreateDesc {
                name: "Example allocation",
                requirements,
                location: memory_usage,
                linear: true, // Buffers are always linear
            })
            .unwrap();

        unsafe {
            logical_device
                .bind_buffer_memory(buffer, allocation.memory(), allocation.offset())
                .unwrap()
        };
        Ok(Buffer {
            buffer,
            allocation,
        })
    }
    pub fn fill<T: Sized>(
        &self,
        allocator: &gpu_allocator::vulkan::Allocator,
        data: &[T],
    ) -> Result<(),  Box<dyn std::error::Error>> {
        let data_ptr: *mut T = self.allocation.mapped_ptr().unwrap().cast().as_ptr();
        unsafe { data_ptr.copy_from_nonoverlapping(data.as_ptr(), data.len()) };
        Ok(())
    }
}