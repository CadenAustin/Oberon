use ash::vk;
use winit::event::{Event, WindowEvent};
extern crate ceaser;


fn main() -> Result<(), Box<dyn std::error::Error>> {
    let eventloop = winit::event_loop::EventLoop::new();
    let window = winit::window::Window::new(&eventloop)?;
    let mut ceaser = ceaser::Ceaser::new(window)?;

    eventloop.run(move |event, _, controlflow| match event {
        Event::WindowEvent {
            event: WindowEvent::CloseRequested,
            ..
        } => {
            *controlflow = winit::event_loop::ControlFlow::Exit;
        }
        Event::RedrawRequested(_) => {
            ceaser.swapchain.current_image = (ceaser.swapchain.current_image + 1) % ceaser.swapchain.amount_of_images as usize;
            let (image_index, _) = unsafe {
                ceaser
                    .swapchain
                    .swapchain_loader
                    .acquire_next_image(
                        ceaser.swapchain.swapchain,
                        std::u64::MAX,
                        ceaser.swapchain.image_available[ceaser.swapchain.current_image],
                        vk::Fence::null(),
                    )
                    .expect("image acquisition trouble")
            };

            unsafe {
                ceaser.logical_device.wait_for_fences(&[ceaser.swapchain.may_begin_drawing[ceaser.swapchain.current_image]], true, std::u64::MAX).expect("fence-waiting");
                ceaser.logical_device.reset_fences(&[ceaser.swapchain.may_begin_drawing[ceaser.swapchain.current_image]]).expect("resetting fences");
            }
            let semaphores_available = [ceaser.swapchain.image_available[ceaser.swapchain.current_image]];
            let waiting_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
            let semaphores_finished = [ceaser.swapchain.rendering_finished[ceaser.swapchain.current_image]];
            let command_buffers = [ceaser.command_buffers[image_index as usize]];
            let submit_info = [vk::SubmitInfo::builder()
                .wait_semaphores(&semaphores_available)
                .wait_dst_stage_mask(&waiting_stages)
                .command_buffers(&command_buffers)
                .signal_semaphores(&semaphores_finished)
                .build()];

            unsafe {
                ceaser
                    .logical_device
                    .queue_submit(
                        ceaser.queues.graphics_queue,
                        &submit_info,
                        ceaser.swapchain.may_begin_drawing[ceaser.swapchain.current_image],
                    )
                    .expect("queue submission");
            }

            let swapchains = [ceaser.swapchain.swapchain];
            let indices = [image_index];
            let present_info = vk::PresentInfoKHR::builder()
                .wait_semaphores(&semaphores_finished)
                .swapchains(&swapchains)
                .image_indices(&indices);
            unsafe {
                ceaser
                    .swapchain
                    .swapchain_loader
                    .queue_present(ceaser.queues.graphics_queue, &present_info)
                    .expect("queue presentation");
            };

            
        }
        Event::MainEventsCleared => {
            ceaser.window.request_redraw();
        }
        _ => {}
    });
}