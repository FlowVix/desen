// use winit::{
//     event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
//     event_loop::{ControlFlow, EventLoop},
//     window::WindowBuilder,
// };

// use crate::ctx::Context;

use std::time::Instant;

use app::App;
use frame::Frame;
use state::{AppState, ResourceLoader};
use winit::{
    dpi::PhysicalSize,
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

mod app;
pub mod frame;
pub mod state;
pub mod texture;
mod vertex;

pub use winit::*;

pub fn run_app<S: AppState + 'static>() -> ! {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_inner_size(PhysicalSize::new(600, 600))
        // .with_decorations(false)
        // .with_resizable(true)
        .build(&event_loop)
        .unwrap();

    let mut loader = ResourceLoader::new();
    let mut state = S::init(window, &mut loader);

    let atlas = loader.build_atlas();
    let atlas_size = (atlas.width() as f32, atlas.height() as f32);
    let mut app = App::new(state.get_window(), atlas);

    let time = Instant::now();
    let mut last_time = 0.0;

    let mut frame = Frame::new(atlas_size);

    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == state.get_window().id() => {
                match event {
                    WindowEvent::CloseRequested
                    | WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::Escape),
                                ..
                            },
                        ..
                    } => *control_flow = ControlFlow::Exit,
                    WindowEvent::Resized(physical_size) => {
                        app.resize(*physical_size);
                    }
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        // new_inner_size is &mut so w have to dereference it twice
                        app.resize(**new_inner_size);
                    }
                    _ => state.event(event),
                }
            }
            Event::RedrawRequested(window_id) if window_id == state.get_window().id() => {
                let now = time.elapsed().as_secs_f32();
                let delta = now - last_time;
                last_time = now;

                state.update(delta);

                frame.reset();
                state.view(&mut frame);

                match app.render(&frame) {
                    Ok(_) => {}
                    // Reconfigure the surface if it's lost or outdated
                    Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                        app.resize(app.size)
                    }
                    // The system is out of memory, we should probably quit
                    Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                    // We're ignoring timeouts
                    Err(wgpu::SurfaceError::Timeout) => println!("Surface timeout"),
                }
            }
            Event::MainEventsCleared => {
                // RedrawRequested will only trigger once, unless we manually
                // request it.
                state.get_window().request_redraw();
            }
            _ => {}
        }
    });
}
