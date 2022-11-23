use std::fmt::Error;

use ash::vk;

pub struct Buffer {
    pub buffer: vk::Buffer,
    pub allocation: gpu_allocator::vulkan::Allocation,
    pub size_in_bytes: u64,
    pub usage: vk::BufferUsageFlags,
    pub memory_usage: gpu_allocator::MemoryLocation
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
            size_in_bytes,
            usage,
            memory_usage
        })
    }
    pub fn fill<T: Sized>(
        &mut self,
        logical_device: &ash::Device,
        allocator: &mut gpu_allocator::vulkan::Allocator,
        data: &[T],
    ) -> Result<(),  Box<dyn std::error::Error>> {
        let bytes_to_write = (data.len() * std::mem::size_of::<T>()) as u64;
        if bytes_to_write > self.size_in_bytes {
            let newbuffer = Buffer::new(
                logical_device,
                allocator,
                bytes_to_write,
                self.usage,
                self.memory_usage,
            )?;
            *self = newbuffer;
        }
        let data_ptr: *mut T = self.allocation.mapped_ptr().unwrap().cast().as_ptr();
        unsafe { data_ptr.copy_from_nonoverlapping(data.as_ptr(), data.len()) };
        Ok(())
    }
}