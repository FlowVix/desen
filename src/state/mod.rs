pub mod texture;
pub mod windowed;

use std::time::Instant;

use crate::{
    Stage, TextureInfo,
    render::{gpu::GPUData, shaders::wgsl_main, texture::Texture},
    state::texture::{LoadedTexture, TextureMap},
};

pub trait AppState {
    fn setup(data: &mut AppData) -> Self;

    fn render(&mut self, s: &mut Stage, data: &mut AppData);
}

pub struct AppData {
    // pub(crate) window: Arc<winit::window::Window>,
    pub(crate) loaded_textures: TextureMap,

    pub(crate) gpu_data: GPUData,
}
impl AppData {
    // pub fn window(&self) -> &winit::window::Window {
    //     &self.window
    // }
    pub fn load_texture_rgba(
        &mut self,
        rgba: &[u8],
        width: u32,
        height: u32,
        nearest_neighbor: bool,
    ) -> TextureInfo {
        let texture = Texture::from_rgba(
            &self.gpu_data.device,
            &self.gpu_data.queue,
            rgba,
            width,
            height,
            if nearest_neighbor {
                wgpu::FilterMode::Nearest
            } else {
                wgpu::FilterMode::Linear
            },
            wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        );
        let bind_group = wgsl_main::globals::BindGroup1::from_bindings(
            &self.gpu_data.device,
            wgsl_main::globals::BindGroup1Entries::new(
                wgsl_main::globals::BindGroup1EntriesEntriesParams {
                    TEX_T: &texture.view,
                    TEX_S: &texture.sampler,
                },
            ),
        );
        let key = self.loaded_textures.insert(LoadedTexture {
            texture,
            bind_group,
        });
        TextureInfo { key, width, height }
    }
    pub fn remove_texture(&mut self, texture: TextureInfo) {
        self.loaded_textures.remove(texture.key);
    }
}

struct AppBundle<S> {
    data: AppData,
    stage: Stage,
    state: S,

    last_render: Instant,
}
