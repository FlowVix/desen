use std::{
    sync::Arc,
    thread,
    time::{Duration, Instant},
};

use glam::vec2;
use slotmap::SlotMap;
use winit::window::Window;

use crate::{AppData, AppState, Stage, render::gpu::GPUData, state::AppBundle};

struct App<S> {
    window: Arc<Window>,
    bundle: AppBundle<S>,
}

struct AppHandler<S> {
    app: Option<App<S>>,
}

impl<S> winit::application::ApplicationHandler for AppHandler<S>
where
    S: AppState,
{
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if self.app.is_none() {
            let window = Arc::new(
                event_loop
                    .create_window(Window::default_attributes())
                    .unwrap(),
            );

            let window_size = window.inner_size();
            let gpu_data = pollster::block_on(GPUData::new(
                window.clone(),
                window_size.width,
                window_size.height,
            ));

            let mut data = AppData {
                // window,
                gpu_data,
                loaded_textures: SlotMap::default(),
            };
            let state = S::setup(&mut data);

            let stage = Stage::new();

            self.app = Some(App {
                window,
                bundle: AppBundle {
                    data,
                    stage,
                    state,

                    last_render: Instant::now(),
                },
            })
        }
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        if let Some(app) = &mut self.app {
            match event {
                winit::event::WindowEvent::CloseRequested => {
                    event_loop.exit();
                }
                winit::event::WindowEvent::Resized(to) => {
                    app.bundle.data.gpu_data.resize(to.width, to.height);
                }
                winit::event::WindowEvent::RedrawRequested => {
                    app.bundle.stage.start();

                    let now = Instant::now();
                    let delta = now - app.bundle.last_render;
                    app.bundle.last_render = now;

                    app.bundle.stage.delta = delta.as_secs_f64();

                    app.bundle
                        .state
                        .render(&mut app.bundle.stage, &mut app.bundle.data);

                    app.bundle
                        .data
                        .gpu_data
                        .render(&app.bundle.stage, &app.bundle.data.loaded_textures);

                    app.bundle.data.gpu_data.mask_atlas.clear_in_use();
                    app.bundle.data.gpu_data.color_atlas.clear_in_use();

                    app.window.request_redraw();
                }
                winit::event::WindowEvent::CursorMoved { position, .. } => {
                    let w_size = app.window.inner_size();

                    app.bundle.stage.mouse_pos = vec2(
                        position.x as f32 - w_size.width as f32 / 2.0,
                        -(position.y as f32 - w_size.height as f32 / 2.0),
                    );
                }
                winit::event::WindowEvent::MouseInput { state, button, .. } => {
                    match (button, state.is_pressed()) {
                        (winit::event::MouseButton::Left, true) => {
                            app.bundle.stage.mouse_down =
                                app.bundle.stage.find_top_old_sense().map(|v| v.id);
                        }
                        (winit::event::MouseButton::Left, false) => {
                            app.bundle.stage.mouse_down = None;
                        }
                        (winit::event::MouseButton::Right, true) => {
                            app.bundle.stage.right_mouse_down =
                                app.bundle.stage.find_top_old_sense().map(|v| v.id);
                        }
                        (winit::event::MouseButton::Right, false) => {
                            app.bundle.stage.right_mouse_down = None;
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
    }

    fn device_event(
        &mut self,
        _event_loop: &winit::event_loop::ActiveEventLoop,
        device_id: winit::event::DeviceId,
        event: winit::event::DeviceEvent,
    ) {
        // if let Some(app) = &mut self.app {
        //     #[allow(clippy::needless_return)]
        //     if app
        //         .bundle
        //         .state
        //         .device_event(&event, device_id, &mut app.bundle.data)
        //     {
        //         return;
        //     };
        // }
    }
}

pub fn run_app_windowed<S: AppState>() {
    let event_loop = winit::event_loop::EventLoop::builder().build().unwrap();
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);

    let mut app = AppHandler::<S> { app: None };

    event_loop.run_app(&mut app).unwrap();
}
