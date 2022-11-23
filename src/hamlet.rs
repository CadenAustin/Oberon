use crate::ceaser::buffer::Buffer;
use ash::vk;
use nalgebra as na;

pub mod light;

#[derive(Debug, Clone)]
pub struct InvalidHandle;
impl std::fmt::Display for InvalidHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "invalid handle")
    }
}
impl std::error::Error for InvalidHandle {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

#[repr(C)]
pub struct InstanceData {
    pub model_matrix: [[f32; 4]; 4],
    pub inverse_modelmatrix: [[f32; 4]; 4],
    pub color: [f32; 3],
    pub metallic: f32,
    pub roughness: f32,
}

impl InstanceData {
    pub fn from_matrix_and_color(
        model_matrix: na::Matrix4<f32>,
        color: [f32; 3],
        metallic: f32,
        roughness: f32,
    ) -> InstanceData {
        InstanceData {
            model_matrix: model_matrix.into(),
            inverse_modelmatrix: model_matrix.try_inverse().unwrap().into(),
            color,
            metallic,
            roughness,
        }
    }
}

#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub struct VertexData {
    pub position: [f32; 3],
    pub normal: [f32; 3],
}

impl VertexData {
    pub fn midpoint(a: &VertexData, b: &VertexData) -> VertexData {
        VertexData {
            position: [
                0.5 * (a.position[0] + b.position[0]),
                0.5 * (a.position[1] + b.position[1]),
                0.5 * (a.position[2] + b.position[2]),
            ],
            normal: normalize([
                0.5 * (a.normal[0] + b.normal[0]),
                0.5 * (a.normal[1] + b.normal[1]),
                0.5 * (a.normal[2] + b.normal[2]),
            ]),
        }
    }
}

pub fn normalize(v: [f32; 3]) -> [f32; 3] {
    let l = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    [v[0] / l, v[1] / l, v[2] / l]
}

pub struct Model<V, I> {
    pub vertexdata: Vec<V>,
    pub indexdata: Vec<u32>,
    pub handle_to_index: std::collections::HashMap<usize, usize>,
    pub handles: Vec<usize>,
    pub instances: Vec<I>,
    pub first_invisible: usize,
    pub next_handle: usize,
    pub vertexbuffer: Option<Buffer>,
    pub indexbuffer: Option<Buffer>,
    pub instancebuffer: Option<Buffer>,
}

#[allow(dead_code)]
impl<V, I> Model<V, I> {
    pub fn get(&self, handle: usize) -> Option<&I> {
        if let Some(&index) = self.handle_to_index.get(&handle) {
            self.instances.get(index)
        } else {
            None
        }
    }
    pub fn get_mut(&mut self, handle: usize) -> Option<&mut I> {
        if let Some(&index) = self.handle_to_index.get(&handle) {
            self.instances.get_mut(index)
        } else {
            None
        }
    }
    pub fn swap_by_handle(&mut self, handle1: usize, handle2: usize) -> Result<(), InvalidHandle> {
        if handle1 == handle2 {
            return Ok(());
        }
        if let (Some(&index1), Some(&index2)) = (
            self.handle_to_index.get(&handle1),
            self.handle_to_index.get(&handle2),
        ) {
            self.handles.swap(index1, index2);
            self.instances.swap(index1, index2);
            self.handle_to_index.insert(index1, handle2);
            self.handle_to_index.insert(index2, handle1);
            Ok(())
        } else {
            Err(InvalidHandle)
        }
    }
    pub fn swap_by_index(&mut self, index1: usize, index2: usize) {
        if index1 == index2 {
            return;
        }
        let handle1 = self.handles[index1];
        let handle2 = self.handles[index2];
        self.handles.swap(index1, index2);
        self.instances.swap(index1, index2);
        self.handle_to_index.insert(index1, handle2);
        self.handle_to_index.insert(index2, handle1);
    }

    pub fn is_visible(&self, handle: usize) -> Result<bool, InvalidHandle> {
        if let Some(index) = self.handle_to_index.get(&handle) {
            Ok(index < &self.first_invisible)
        } else {
            Err(InvalidHandle)
        }
    }
    pub fn make_visible(&mut self, handle: usize) -> Result<(), InvalidHandle> {
        //if already visible: do nothing
        if let Some(&index) = self.handle_to_index.get(&handle) {
            if index < self.first_invisible {
                return Ok(());
            }
            //else: move to position first_invisible and increase value of first_invisible
            self.swap_by_index(index, self.first_invisible);
            self.first_invisible += 1;
            Ok(())
        } else {
            Err(InvalidHandle)
        }
    }
    pub fn make_invisible(&mut self, handle: usize) -> Result<(), InvalidHandle> {
        //if already invisible: do nothing
        if let Some(&index) = self.handle_to_index.get(&handle) {
            if index >= self.first_invisible {
                return Ok(());
            }
            //else: move to position before first_invisible and decrease value of first_invisible
            self.swap_by_index(index, self.first_invisible - 1);
            self.first_invisible -= 1;
            Ok(())
        } else {
            Err(InvalidHandle)
        }
    }

    pub fn insert(&mut self, element: I) -> usize {
        let handle = self.next_handle;
        self.next_handle += 1;
        let index = self.instances.len();
        self.instances.push(element);
        self.handles.push(handle);
        self.handle_to_index.insert(handle, index);
        handle
    }

    pub fn insert_visibly(&mut self, element: I) -> usize {
        let new_handle = self.insert(element);
        self.make_visible(new_handle).ok(); //can't go wrong, see previous line
        new_handle
    }

    pub fn remove(&mut self, handle: usize) -> Result<I, InvalidHandle> {
        if let Some(&index) = self.handle_to_index.get(&handle) {
            if index < self.first_invisible {
                self.swap_by_index(index, self.first_invisible - 1);
                self.first_invisible -= 1;
            }
            self.swap_by_index(self.first_invisible, self.instances.len() - 1);
            self.handles.pop();
            self.handle_to_index.remove(&handle);
            //must be Some(), otherwise we couldn't have found an index
            Ok(self.instances.pop().unwrap())
        } else {
            Err(InvalidHandle)
        }
    }

    pub fn update_vertexbuffer(
        &mut self,
        logical_device: &ash::Device,
        allocator: &mut gpu_allocator::vulkan::Allocator,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(buffer) = &mut self.vertexbuffer {
            buffer.fill(logical_device, allocator, &self.vertexdata)?;
            Ok(())
        } else {
            let bytes = (self.vertexdata.len() * std::mem::size_of::<V>()) as u64;
            let mut buffer = Buffer::new(
                logical_device,
                allocator,
                bytes,
                vk::BufferUsageFlags::VERTEX_BUFFER,
                gpu_allocator::MemoryLocation::CpuToGpu,
            )?;
            buffer.fill(logical_device, allocator, &self.vertexdata)?;
            self.vertexbuffer = Some(buffer);
            Ok(())
        }
    }

    pub fn update_indexbuffer(
        &mut self,
        logical_device: &ash::Device,
        allocator: &mut gpu_allocator::vulkan::Allocator,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(buffer) = &mut self.indexbuffer {
            buffer.fill(logical_device, allocator, &self.indexdata)?;
            Ok(())
        } else {
            let bytes = (self.indexdata.len() * std::mem::size_of::<V>()) as u64;
            let mut buffer = Buffer::new(
                logical_device,
                allocator,
                bytes,
                vk::BufferUsageFlags::INDEX_BUFFER,
                gpu_allocator::MemoryLocation::CpuToGpu,
            )?;
            buffer.fill(logical_device, allocator, &self.indexdata)?;
            self.indexbuffer = Some(buffer);
            Ok(())
        }
    }

    pub fn update_instancebuffer(
        &mut self,
        logical_device: &ash::Device,
        allocator: &mut gpu_allocator::vulkan::Allocator,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(buffer) = &mut self.instancebuffer {
            buffer.fill(
                logical_device,
                allocator,
                &self.instances[0..self.first_invisible],
            )?;
            Ok(())
        } else {
            let bytes = (self.first_invisible * std::mem::size_of::<I>()) as u64;
            if bytes > 0 {
                let mut buffer = Buffer::new(
                    logical_device,
                    allocator,
                    bytes,
                    vk::BufferUsageFlags::VERTEX_BUFFER,
                    gpu_allocator::MemoryLocation::CpuToGpu,
                )?;
                buffer.fill(
                    logical_device,
                    allocator,
                    &self.instances[0..self.first_invisible],
                )?;
                self.instancebuffer = Some(buffer);
            }
            Ok(())
        }
    }

    pub fn draw(&self, logical_device: &ash::Device, commandbuffer: vk::CommandBuffer) {
        if let Some(vertexbuffer) = &self.vertexbuffer {
            if let Some(indexbuffer) = &self.indexbuffer {
                if let Some(instancebuffer) = &self.instancebuffer {
                    if self.first_invisible > 0 {
                        unsafe {
                            logical_device.cmd_bind_index_buffer(
                                commandbuffer,
                                indexbuffer.buffer,
                                0,
                                vk::IndexType::UINT32,
                            );
                            logical_device.cmd_bind_vertex_buffers(
                                commandbuffer,
                                0,
                                &[vertexbuffer.buffer],
                                &[0],
                            );
                            logical_device.cmd_bind_vertex_buffers(
                                commandbuffer,
                                1,
                                &[instancebuffer.buffer],
                                &[0],
                            );
                            logical_device.cmd_draw_indexed(
                                commandbuffer,
                                self.indexdata.len() as u32,
                                self.first_invisible as u32,
                                0,
                                0,
                                0,
                            );
                        }
                    }
                }
            }
        }
    }
}

mod primitives;
