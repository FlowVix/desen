// use winit::{
//     event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
//     event_loop::{ControlFlow, EventLoop},
//     window::WindowBuilder,
// };

// use crate::ctx::Context;

use std::marker::PhantomData;
use std::time::{Duration, Instant};

pub use app::App;
use frame::Frame;
use state::{WindowedAppInfo, WindowedAppState};

#[cfg(feature = "html-canvas")]
use state::{CanvasAppInfo, CanvasAppState};

use winit::{
    dpi::PhysicalSize,
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

mod app;
pub mod color;
pub mod frame;
pub mod state;
pub mod texture;
mod util;
mod vertex;

pub use winit::*;

pub fn run_app_windowed<
    I: WindowedAppInfo,
    S: WindowedAppState<I> + 'static,
    F: FnOnce(App, winit::window::Window) -> S,
>(
    init_state: F,
) -> ! {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_inner_size(PhysicalSize::new(600, 600))
        // .with_decorations(false)
        // .with_resizable(true)
        .build(&event_loop)
        .unwrap();

    // let mut loader = ResourceLoader::new();
    let app = App::new_windowed(&window);

    let mut state = init_state(app, window);

    let time = Instant::now();
    let mut last_time = 0.0;

    let mut frame = Frame::new();

    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == state.get_info().get_window().id() => {
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
                        state.get_info().get_app().resize(*physical_size);
                    }
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        // new_inner_size is &mut so w have to dereference it twice
                        state.get_info().get_app().resize(**new_inner_size);
                    }
                    _ => {}
                }
                state.event(event)
            }
            Event::RedrawRequested(window_id)
                if window_id == state.get_info().get_window().id() =>
            {
                let now = time.elapsed().as_secs_f32();
                let delta = now - last_time;
                last_time = now;

                frame.reset();
                state.view(&mut frame, delta);

                match state.get_info().get_app().render(&frame) {
                    Ok(_) => {}
                    // Reconfigure the surface if it's lost or outdated
                    Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                        let new_size = (
                            state.get_info().get_app().config.width,
                            state.get_info().get_app().config.height,
                        );

                        state.get_info().get_app().resize(new_size.into())
                    }
                    // The system is out of memory, we should probably quit
                    Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                    // We're ignoring timeouts
                    Err(wgpu::SurfaceError::Timeout) => println!("Surface timeout"),
                }

                // *control_flow =
                //     ControlFlow::WaitUntil(Instant::now() + Duration::from_secs_f64(1.0 / 30.0))
            }
            Event::MainEventsCleared => {
                // RedrawRequested will only trigger once, unless we manually
                // request it.
                state.get_info().get_window().request_redraw();
            }
            _ => {}
        }
    });
}

#[cfg(feature = "html-canvas")]
pub struct CanvasAppBundle<S, I> {
    pub state: S,
    frame: Frame,
    _p: PhantomData<I>,
}

#[cfg(feature = "html-canvas")]
impl<S, I> std::ops::Deref for CanvasAppBundle<S, I> {
    type Target = S;

    fn deref(&self) -> &Self::Target {
        &self.state
    }
}
#[cfg(feature = "html-canvas")]
impl<S, I> std::ops::DerefMut for CanvasAppBundle<S, I> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.state
    }
}

#[cfg(feature = "html-canvas")]
impl<S, I: CanvasAppInfo> CanvasAppBundle<S, I>
where
    S: CanvasAppState<I>,
{
    pub fn render(&mut self, delta: f32) {
        let frame_p = &self.frame as *const _;

        self.frame.reset();
        self.state.view(&mut self.frame, delta);

        match self.get_info().get_app().render(unsafe { &*frame_p }) {
            Ok(_) => {}
            Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                let v = (
                    self.get_info().get_app().config.width,
                    self.get_info().get_app().config.height,
                )
                    .into();
                self.get_info().get_app().resize(v)
            }
            Err(wgpu::SurfaceError::OutOfMemory) => {
                println!("out of memory")
            }
            Err(wgpu::SurfaceError::Timeout) => println!("Surface timeout"),
        }
    }
    // pub fn resize(&mut self, width: u32, height: u32) {
    //     self.app.resize((width, height).into());
    // }
    // pub fn get_size(&mut self) -> (u32, u32) {
    //     (self.app.config.width, self.app.config.height)
    // }
}

#[cfg(feature = "html-canvas")]
#[cfg(all(target_arch = "wasm32", not(target_os = "emscripten")))]
pub fn new_app_canvas<I: CanvasAppInfo, S: CanvasAppState<I>, F: FnOnce(App) -> S>(
    canvas: web_sys::HtmlCanvasElement,
    init_state: F,
) -> CanvasAppBundle<S, I> {
    let app = App::new_canvas(canvas);

    let state = init_state(app);

    let frame = Frame::new();

    CanvasAppBundle {
        state,
        frame,
        _p: PhantomData,
    }
}
