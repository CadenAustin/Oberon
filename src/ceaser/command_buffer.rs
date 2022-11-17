use ash::vk;

use crate::{queue::Pools, swap_chain::Swapchain, pipeline::Pipeline};



pub fn create_command_buffers(
    logical_device: &ash::Device,
    pools: &Pools,
    amount: usize,
) -> Result<Vec<vk::CommandBuffer>, vk::Result> {
    let commandbuf_allocate_info = vk::CommandBufferAllocateInfo::builder()
        .command_pool(pools.commandpool_graphics)
        .command_buffer_count(amount as u32);
    unsafe { logical_device.allocate_command_buffers(&commandbuf_allocate_info) }
}

pub fn fill_command_buffers(
    command_buffers: &[vk::CommandBuffer],
    logical_device: &ash::Device,
    render_pass: &vk::RenderPass,
    swapchain: &Swapchain,
    pipeline: &Pipeline,
    vb1: &vk::Buffer,
    vb2: &vk::Buffer,
) -> Result<(), vk::Result> {
    for (i, &command_buffer) in command_buffers.iter().enumerate() {
        let command_buffer_begininfo = vk::CommandBufferBeginInfo::builder();
        unsafe {
            logical_device.begin_command_buffer(command_buffer, &command_buffer_begininfo)?;
        }

        let clearvalues = [vk::ClearValue {
            color: vk::ClearColorValue {
                float32: [0.0, 0.0, 0.08, 1.0],
            },
        }];
        let render_pass_begininfo = vk::RenderPassBeginInfo::builder()
            .render_pass(*render_pass)
            .framebuffer(swapchain.framebuffers[i])
            .render_area(vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: swapchain.extent,
            })
            .clear_values(&clearvalues);
        
        unsafe {
            logical_device.cmd_begin_render_pass(
                command_buffer,
                &render_pass_begininfo,
                vk::SubpassContents::INLINE,
            );

            logical_device.cmd_bind_pipeline(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                pipeline.pipeline,
            );

            logical_device.cmd_bind_vertex_buffers(command_buffer, 0, &[*vb1], &[0]);
            logical_device.cmd_bind_vertex_buffers(command_buffer, 1, &[*vb2], &[0]);
            logical_device.cmd_draw(command_buffer, 6, 1, 0, 0);
            logical_device.cmd_end_render_pass(command_buffer);
            logical_device.end_command_buffer(command_buffer)?;
        }
    }

    Ok(())
}