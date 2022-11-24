use ash::vk;
use ceaser::{camera::Camera};
use hamlet::{InstanceData, Model, light::{LightManager, DirectionalLight, PointLight}};
use nalgebra as na;
use winit::event::{Event, WindowEvent};

mod ceaser;
mod hamlet;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let eventloop = winit::event_loop::EventLoop::new();
    let window = winit::window::Window::new(&eventloop)?;
    let mut ceaser = ceaser::Ceaser::new(window)?;
    let mut sphere = Model::sphere(3);
    for i in 0..10 {
        for j in 0..10 {
            sphere.insert_visibly(InstanceData::from_matrix_and_color(
                na::Matrix4::new_translation(&na::Vector3::new(i as f32 - 5., j as f32 + 5., 10.0))
                    * na::Matrix4::new_scaling(0.5),
                [0., 0., 0.8],
            ));
        }
    }

    sphere.update_vertexbuffer(&ceaser.logical_device, &mut ceaser.allocator)?;
    sphere.update_indexbuffer(&ceaser.logical_device, &mut ceaser.allocator)?;
    sphere.update_instancebuffer(&ceaser.logical_device, &mut ceaser.allocator)?;

    ceaser.models = vec![sphere];

    let mut lights = LightManager::default();
    lights.add_light(DirectionalLight {
        direction: na::Vector3::new(-0.2, -1.0, -0.3),
        ambient: na::Vector3::new(0.3, 0.24, 0.14),
        diffuse: na::Vector3::new( 0.7, 0.42, 0.26),
        specular: na::Vector3::new(0.5, 0.5, 0.5),
 
    });
    lights.add_light(PointLight {
        position: na::Point3::new(-0.2, -1.0, -0.3),
        constant: 1.0,
        linear: 0.09,
        quadratic: 0.032,
        ambient: na::Vector3::new(0.3, 0.24, 0.14),
        diffuse: na::Vector3::new( 0.7, 0.42, 0.26),
        specular: na::Vector3::new(0.5, 0.5, 0.5),
    });
    lights.update_buffer(&ceaser.logical_device, &mut ceaser.allocator, &mut ceaser.light_buffer, &mut ceaser.descriptor_sets_light)?;

    let mut camera = Camera::builder().build();

    eventloop.run(move |event, _, controlflow| match event {
        Event::WindowEvent {
            event: WindowEvent::CloseRequested,
            ..
        } => {
            *controlflow = winit::event_loop::ControlFlow::Exit;
        }
        Event::MainEventsCleared => {
            ceaser.window.request_redraw();
        }
        Event::RedrawRequested(_) => {
            ceaser.swapchain.current_image =
                (ceaser.swapchain.current_image + 1) % ceaser.swapchain.amount_of_images as usize;
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
                ceaser
                    .logical_device
                    .wait_for_fences(
                        &[ceaser.swapchain.may_begin_drawing[ceaser.swapchain.current_image]],
                        true,
                        std::u64::MAX,
                    )
                    .expect("fence-waiting");
                ceaser
                    .logical_device
                    .reset_fences(&[
                        ceaser.swapchain.may_begin_drawing[ceaser.swapchain.current_image]
                    ])
                    .expect("resetting fences");
            }

            camera.update_buffer(
                &ceaser.logical_device,
                &mut ceaser.allocator,
                &mut ceaser.uniform_buffer,
            );
            for m in &mut ceaser.models {
                m.update_instancebuffer(&ceaser.logical_device, &mut ceaser.allocator)
                    .expect("Error updating instance buffers");
            }

            ceaser
                .update_commandbuffer(image_index as usize)
                .expect("updating the command buffer");

            let semaphores_available =
                [ceaser.swapchain.image_available[ceaser.swapchain.current_image]];
            let waiting_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
            let semaphores_finished =
                [ceaser.swapchain.rendering_finished[ceaser.swapchain.current_image]];
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
        Event::WindowEvent {
            event: WindowEvent::KeyboardInput { input, .. },
            ..
        } => match input {
            winit::event::KeyboardInput {
                state: winit::event::ElementState::Pressed,
                virtual_keycode: Some(keycode),
                ..
            } => match keycode {
                winit::event::VirtualKeyCode::Right | winit::event::VirtualKeyCode::D => {
                    camera.turn_right(0.1);
                }
                winit::event::VirtualKeyCode::Left | winit::event::VirtualKeyCode::A => {
                    camera.turn_left(0.1);
                }
                winit::event::VirtualKeyCode::Q => {
                    camera.strafe_right(0.1);
                }
                winit::event::VirtualKeyCode::E => {
                    camera.strafe_left(0.1);
                }
                winit::event::VirtualKeyCode::Up | winit::event::VirtualKeyCode::W => {
                    camera.move_forward(0.05);
                }
                winit::event::VirtualKeyCode::Down | winit::event::VirtualKeyCode::S => {
                    camera.move_backward(0.05);
                }
                winit::event::VirtualKeyCode::PageUp => {
                    camera.turn_up(0.02);
                }
                winit::event::VirtualKeyCode::PageDown => {
                    camera.turn_down(0.02);
                }
                _ => {}
            },
            _ => {}
        },
        _ => {}
    });
}
