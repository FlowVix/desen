use std::sync::Arc;

use slotmap::{SlotMap, new_key_type};

use crate::{
    render::{gpu::GPUData, texture::Texture},
    shaders::wgsl_main,
};

new_key_type! {
    pub struct TextureKey;
}

pub struct LoadedTexture {
    pub(crate) texture: Texture,
    pub(crate) bind_group: wgsl_main::globals::BindGroup1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TextureInfo {
    pub(crate) key: TextureKey,
    pub(crate) width: u32,
    pub(crate) height: u32,
}
impl TextureInfo {
    pub fn key(&self) -> TextureKey {
        self.key
    }
    pub fn width(&self) -> u32 {
        self.width
    }
    pub fn height(&self) -> u32 {
        self.height
    }
}

pub type TextureMap = SlotMap<TextureKey, LoadedTexture>;
pub struct AppData {
    pub(crate) window: Arc<winit::window::Window>,
    pub(crate) loaded_textures: TextureMap,

    pub(crate) gpu_data: GPUData,
}
impl AppData {
    pub fn window(&self) -> &winit::window::Window {
        &self.window
    }
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
