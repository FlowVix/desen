use wgpu::util::DeviceExt;

use crate::{
    Stage,
    render::{SAMPLE_COUNT, texture::Texture},
    shaders::wgsl_main,
    state::data::TextureMap,
};

pub struct GPUData {
    pub surface: wgpu::Surface<'static>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub surface_config: wgpu::SurfaceConfiguration,

    pub multisampled_frame_descriptor: wgpu::TextureDescriptor<'static>,

    pub main_pipeline: wgpu::RenderPipeline,

    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,

    pub globals_buffer: wgpu::Buffer,
    pub bind_group_0: wgsl_main::globals::BindGroup0,

    pub dummy_texture: wgsl_main::globals::BindGroup1,
}

impl GPUData {
    pub async fn new(
        target: impl Into<wgpu::SurfaceTarget<'static>>,
        width: u32,
        height: u32,
    ) -> Self {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::GL,
            flags: wgpu::InstanceFlags::all(),
            ..Default::default()
        });

        let surface = instance.create_surface(target).unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptionsBase {
                power_preference: wgpu::PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await
            .unwrap();
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("device_descriptor"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits {
                    // min_uniform_buffer_offset_alignment: 32,
                    // max_buffer_size: 268435456 * 6,
                    ..Default::default()
                },
                memory_hints: Default::default(),
                trace: wgpu::Trace::Off,
            })
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| !f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);
        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width,
            height,
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &surface_config);

        let multisampled_frame_descriptor = wgpu::TextureDescriptor {
            label: Some("multisampled_frame_descriptor"),
            size: wgpu::Extent3d {
                width: surface_config.width,
                height: surface_config.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: SAMPLE_COUNT,
            dimension: wgpu::TextureDimension::D2,
            format: surface_config.format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        };

        macro_rules! pipeline {
            ($name:ident, $shader_name:ident, $blend_state:expr, ( $($vertex_mode:ident),* )) => {
                let $name = {
                    let module = crate::shaders::$shader_name::create_shader_module(&device);

                    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                        label: Some("main_render_pipeline"),
                        layout: Some(&crate::shaders::$shader_name::create_pipeline_layout(&device)),
                        vertex: crate::shaders::make_vertex_state(
                            &module,
                            &crate::shaders::$shader_name::entries::vertex_entry_vs_main(
                                $(wgpu::VertexStepMode:: $vertex_mode),*
                            ),
                        ),
                        fragment: Some(crate::shaders::make_fragment_state(
                            &module,
                            &crate::shaders::$shader_name::entries::fragment_entry_fs_main(&[Some(
                                wgpu::ColorTargetState {
                                    format: surface_config.format,
                                    blend: Some($blend_state),
                                    write_mask: wgpu::ColorWrites::ALL,
                                },
                            )]),
                        )),
                        primitive: wgpu::PrimitiveState {
                            topology: wgpu::PrimitiveTopology::TriangleList,
                            strip_index_format: None,
                            front_face: wgpu::FrontFace::Cw,
                            cull_mode: None,
                            polygon_mode: wgpu::PolygonMode::Fill,
                            unclipped_depth: false,
                            conservative: false,
                        },
                        depth_stencil: None,
                        multisample: wgpu::MultisampleState {
                            count: SAMPLE_COUNT,
                            mask: !0,
                            alpha_to_coverage_enabled: false,
                        },
                        multiview: None,
                        cache: None,
                    })
                };
            };
        }
        let normal_state = wgpu::BlendState {
            color: wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::SrcAlpha,
                dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                operation: wgpu::BlendOperation::Add,
            },
            alpha: wgpu::BlendComponent::OVER,
        };

        pipeline!(main_pipeline, wgsl_main, normal_state, (Vertex, Instance));

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("vertex_buffer"),
            contents: bytemuck::cast_slice(&[
                wgsl_main::structs::VertexInput::new([0.0, 0.0]),
                wgsl_main::structs::VertexInput::new([1.0, 0.0]),
                wgsl_main::structs::VertexInput::new([0.0, 1.0]),
            ]),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("index_buffer"),
            contents: bytemuck::cast_slice(&[0u16, 1, 2]),
            usage: wgpu::BufferUsages::INDEX,
        });

        let globals_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("globals_buffer"),
            contents: bytemuck::cast_slice(&[wgsl_main::structs::Globals {
                screen_size: [0.0, 0.0],
            }]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group_0 = wgsl_main::globals::BindGroup0::from_bindings(
            &device,
            wgsl_main::globals::BindGroup0Entries::new(
                wgsl_main::globals::BindGroup0EntriesEntriesParams {
                    GLOBALS: globals_buffer.as_entire_buffer_binding(),
                },
            ),
        );

        Self {
            dummy_texture: {
                let tex = Texture::blank(
                    &device,
                    surface_format,
                    2,
                    2,
                    wgpu::FilterMode::Linear,
                    wgpu::TextureUsages::TEXTURE_BINDING,
                    1,
                );
                wgsl_main::globals::BindGroup1::from_bindings(
                    &device,
                    wgsl_main::globals::BindGroup1Entries::new(
                        wgsl_main::globals::BindGroup1EntriesEntriesParams {
                            TEX_T: &tex.view,
                            TEX_S: &tex.sampler,
                        },
                    ),
                )
            },
            surface,
            device,
            queue,
            surface_config,
            multisampled_frame_descriptor,
            globals_buffer,
            bind_group_0,
            main_pipeline,
            vertex_buffer,
            index_buffer,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            // tracing::span!("RenderState_resize");

            self.surface_config.width = width;
            self.surface_config.height = height;
            self.surface.configure(&self.device, &self.surface_config);

            self.queue.write_buffer(
                &self.globals_buffer,
                0,
                bytemuck::bytes_of(&wgsl_main::structs::Globals {
                    screen_size: [
                        self.surface_config.width as f32,
                        self.surface_config.height as f32,
                    ],
                }),
            );

            self.multisampled_frame_descriptor = wgpu::TextureDescriptor {
                label: Some("multisampled_frame_descriptor"),
                size: wgpu::Extent3d {
                    width: self.surface_config.width,
                    height: self.surface_config.height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: SAMPLE_COUNT,
                dimension: wgpu::TextureDimension::D2,
                format: self.surface_config.format,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            };
        }
    }

    pub fn render(&mut self, stage: &Stage, loaded_textures: &TextureMap) {
        let output = self.surface.get_current_texture().unwrap();
        let output_view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let multisample_view = self
            .device
            .create_texture(&self.multisampled_frame_descriptor)
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        let instance_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(&stage.instances),
                usage: wgpu::BufferUsages::VERTEX,
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &multisample_view,
                    resolve_target: Some(&output_view),
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            if !stage.instances.is_empty() {
                render_pass.set_pipeline(&self.main_pipeline);
                render_pass.set_bind_group(0, self.bind_group_0.get_bind_group(), &[]);
                render_pass.set_bind_group(1, self.dummy_texture.get_bind_group(), &[]);
                render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
                render_pass.set_vertex_buffer(1, instance_buffer.slice(..));
                render_pass
                    .set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);

                let num_instances = stage.instances.len() as u32;
                for (idx, pass) in stage.draw_calls.iter().enumerate() {
                    let start = pass.start_instance;
                    let end = stage
                        .draw_calls
                        .get(idx + 1)
                        .map(|p| p.start_instance)
                        .unwrap_or(num_instances);

                    if let Some(mode) = pass.set_blend_mode {
                        // render_pass.set_pipeline(match mode {
                        //     crate::frame::BlendMode::Normal => &self.render_pipeline_normal,
                        //     crate::frame::BlendMode::Additive => &self.render_pipeline_additive,
                        //     crate::frame::BlendMode::AdditiveSquaredAlpha => {
                        //         &self.render_pipeline_additive_squared_alpha
                        //     }
                        // });
                    }
                    if let Some(tex) = pass.set_texture {
                        render_pass.set_bind_group(
                            1,
                            loaded_textures[tex].bind_group.get_bind_group(),
                            &[],
                        );
                    }

                    render_pass.draw_indexed(0..3, 0, start..end);
                }
            }
        }
        self.queue.submit([encoder.finish()]);
        output.present();
    }
}
