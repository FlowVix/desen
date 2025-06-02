pub mod data;

use std::{
    sync::Arc,
    thread,
    time::{Duration, Instant},
};

use data::AppData;
use glam::vec2;
use slotmap::SlotMap;
use winit::window::Window;

use crate::{Stage, render::gpu::GPUData};

// use crate::render::state::RenderState;

pub trait AppState {
    fn setup(data: &mut AppData) -> Self;

    fn fixed_update(&mut self, delta: f64, data: &mut AppData);
    fn render(&mut self, s: &mut Stage, delta: f64, data: &mut AppData);

    fn window_event(&mut self, event: &winit::event::WindowEvent, data: &mut AppData) -> bool {
        false
    }
    fn device_event(
        &mut self,
        event: &winit::event::DeviceEvent,
        device_id: winit::event::DeviceId,
        data: &mut AppData,
    ) -> bool {
        false
    }
}

struct App<S> {
    data: AppData,
    stage: Stage,
    state: S,

    last_render: Instant,
    last_update: Instant,
}

struct AppHandler<S> {
    app: Option<App<S>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum CustomEvent {
    FixedUpdate,
}

impl<S> winit::application::ApplicationHandler<CustomEvent> for AppHandler<S>
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
                window,
                gpu_data,
                loaded_textures: SlotMap::default(),
            };
            let state = S::setup(&mut data);

            let stage = Stage::new();

            self.app = Some(App {
                data,
                stage,
                state,

                last_render: Instant::now(),
                last_update: Instant::now(),
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
            if app.state.window_event(&event, &mut app.data) {
                return;
            };
            match event {
                winit::event::WindowEvent::CloseRequested => {
                    event_loop.exit();
                }
                winit::event::WindowEvent::Resized(to) => {
                    app.data.gpu_data.resize(to.width, to.height);
                }
                winit::event::WindowEvent::RedrawRequested => {
                    app.stage.reset();

                    let now = Instant::now();
                    let delta = now - app.last_render;
                    app.last_render = now;

                    app.state
                        .render(&mut app.stage, delta.as_secs_f64(), &mut app.data);

                    app.data
                        .gpu_data
                        .render(&app.stage, &app.data.loaded_textures);

                    app.data.gpu_data.mask_atlas.clear_in_use();
                    app.data.gpu_data.color_atlas.clear_in_use();

                    app.data.window.request_redraw();
                }
                winit::event::WindowEvent::CursorMoved { position, .. } => {
                    let w_size = app.data.window.inner_size();

                    app.stage.mouse_pos = vec2(
                        position.x as f32 - w_size.width as f32 / 2.0,
                        -(position.y as f32 - w_size.height as f32 / 2.0),
                    );
                }
                winit::event::WindowEvent::MouseInput { state, button, .. } => {
                    match (button, state.is_pressed()) {
                        (winit::event::MouseButton::Left, true) => {
                            app.stage.mouse_down = app.stage.find_top_old_sense().map(|v| v.id);
                        }
                        (winit::event::MouseButton::Left, false) => {
                            app.stage.mouse_down = None;
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
        if let Some(app) = &mut self.app {
            #[allow(clippy::needless_return)]
            if app.state.device_event(&event, device_id, &mut app.data) {
                return;
            };
        }
    }

    fn user_event(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop, event: CustomEvent) {
        if let Some(app) = &mut self.app {
            match event {
                CustomEvent::FixedUpdate => {
                    let now = Instant::now();
                    let delta = now - app.last_update;
                    app.last_update = now;

                    app.state.fixed_update(delta.as_secs_f64(), &mut app.data);
                }
            }
        }
    }
}

pub fn run_app<S: AppState>(fixed_update_rate: u32) {
    let event_loop = winit::event_loop::EventLoop::<CustomEvent>::with_user_event()
        .build()
        .unwrap();
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);

    let mut app = AppHandler::<S> { app: None };

    thread::spawn({
        let proxy = event_loop.create_proxy();
        move || {
            let interval = Duration::from_secs_f64(1.0 / fixed_update_rate as f64);
            loop {
                spin_sleep::sleep(interval);
                if proxy.send_event(CustomEvent::FixedUpdate).is_err() {
                    break;
                }
            }
        }
    });

    event_loop.run_app(&mut app).unwrap();
}
