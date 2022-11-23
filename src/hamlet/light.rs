use ash::vk;
use nalgebra as na;

use crate::ceaser::buffer::Buffer;

pub struct DirectionalLight {
    pub direction: na::Vector3<f32>,
    pub ambient: [f32; 3], 
    pub diffuse: [f32; 3], 
    pub specular: [f32; 3],
}

pub struct PointLight {
    pub position: na::Point3<f32>,
    pub constant: f32,
    pub linear: f32,
    pub quadratic: f32,
    pub ambient: [f32; 3], 
    pub diffuse: [f32; 3], 
    pub specular: [f32; 3], 
}

pub enum Light {
    Directional(DirectionalLight),
    Point(PointLight),
}

impl From<PointLight> for Light {
    fn from(p: PointLight) -> Self {
        Light::Point(p)
    }
}

impl From<DirectionalLight> for Light {
    fn from(d: DirectionalLight) -> Self {
        Light::Directional(d)
    }
}

pub struct LightManager {
    directional_lights: Vec<DirectionalLight>,
    point_lights: Vec<PointLight>,
}

impl Default for LightManager {
    fn default() -> Self {
        LightManager {
            directional_lights: vec![],
            point_lights: vec![],
        }
    }
}

impl LightManager {
    pub fn add_light<T: Into<Light>>(&mut self, l: T) {
        use Light::*;
        match l.into() {
            Directional(dl) => {
                self.directional_lights.push(dl);
            }
            Point(pl) => {
                self.point_lights.push(pl);
            }
        }
    }

    pub fn update_buffer(
        &self,
        logical_device: &ash::Device,
        allocator: &mut gpu_allocator::vulkan::Allocator,
        buffer: &mut Buffer,
        descriptor_sets_light: &mut [vk::DescriptorSet],
    ) -> Result<(),  Box<dyn std::error::Error>> {
        let mut data: Vec<f32> = vec![];
        data.push(self.directional_lights.len() as f32);
        data.push(self.point_lights.len() as f32);
        for dl in &self.directional_lights {
            data.push(dl.direction.x);
            data.push(dl.direction.y);
            data.push(dl.direction.z);
            data.push(0.0);
            data.push(dl.ambient[0]);
            data.push(dl.ambient[1]);
            data.push(dl.ambient[2]);
            data.push(0.0);
            data.push(dl.diffuse[0]);
            data.push(dl.diffuse[1]);
            data.push(dl.diffuse[2]);
            data.push(0.0);
            data.push(dl.specular[0]);
            data.push(dl.specular[1]);
            data.push(dl.specular[2]);
            data.push(0.0);
        }
        for pl in &self.point_lights {
            data.push(pl.position.x);
            data.push(pl.position.y);
            data.push(pl.position.z);
            data.push(0.0);
            data.push(pl.constant);
            data.push(pl.linear);
            data.push(pl.quadratic);
            data.push(0.0);
            data.push(pl.ambient[0]);
            data.push(pl.ambient[1]);
            data.push(pl.ambient[2]);
            data.push(0.0);
            data.push(pl.diffuse[0]);
            data.push(pl.diffuse[1]);
            data.push(pl.diffuse[2]);
            data.push(0.0);
            data.push(pl.specular[0]);
            data.push(pl.specular[1]);
            data.push(pl.specular[2]);
            data.push(0.0);
        }
        buffer.fill(logical_device, allocator, &data)?;
        for descset in descriptor_sets_light {
            let buffer_infos = [vk::DescriptorBufferInfo {
                buffer: buffer.buffer,
                offset: 0,
                range: 4 * data.len() as u64,
            }];
            let desc_sets_write = [vk::WriteDescriptorSet::builder()
                .dst_set(*descset)
                .dst_binding(0)
                .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                .buffer_info(&buffer_infos)
                .build()];
            unsafe { logical_device.update_descriptor_sets(&desc_sets_write, &[]) };
        }
        Ok(())
    }
}