use std::sync::Arc;

use slotmap::{SlotMap, new_key_type};

use crate::render::{gpu::GPUData, shaders::wgsl_main, texture::Texture};

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
    // pub fn key(&self) -> TextureKey {
    //     self.key
    // }
    pub fn width(&self) -> u32 {
        self.width
    }
    pub fn height(&self) -> u32 {
        self.height
    }
}

pub type TextureMap = SlotMap<TextureKey, LoadedTexture>;
