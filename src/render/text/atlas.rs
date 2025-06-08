use std::collections::HashSet;

use etagere::{Allocation, BucketedAtlasAllocator, size2};
use lru::LruCache;

use crate::{
    render::texture::Texture,
    shaders::{self, wgsl_main},
};

use super::glyph::{ContentType, GlyphCacheStatus, GlyphData};

#[allow(dead_code)]
pub struct GlyphAtlas {
    pub typ: ContentType,

    pub texture: Texture,
    pub texture_size: u32,
    pub max_texture_size: u32,

    pub packer: BucketedAtlasAllocator,
    pub glyph_cache: LruCache<cosmic_text::CacheKey, GlyphData>,
    /// hashset of glyphs that are currently in use (only gets cleared during a render rebuild)
    pub glyphs_in_use: HashSet<cosmic_text::CacheKey>,
}

impl GlyphAtlas {
    const START_SIZE: u32 = 256;

    pub fn new(device: &wgpu::Device, typ: ContentType) -> Self {
        let max_texture_size = device.limits().max_texture_dimension_2d;
        let texture_size = Self::START_SIZE.min(max_texture_size);

        let packer = BucketedAtlasAllocator::new(size2(texture_size as i32, texture_size as i32));

        let texture = Texture::blank(
            device,
            typ.texture_format(),
            texture_size,
            texture_size,
            wgpu::FilterMode::Nearest,
            wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            1,
            1,
        );

        let glyph_cache = LruCache::unbounded();
        let glyphs_in_use = HashSet::new();

        Self {
            typ,
            texture,
            texture_size,
            max_texture_size,
            packer,
            glyph_cache,
            glyphs_in_use,
        }
    }
    /// try to allocate space in the atlas
    pub fn try_alloc(&mut self, width: usize, height: usize) -> Option<Allocation> {
        let size = size2(width as i32, height as i32);

        loop {
            let alloc = self.packer.allocate(size);

            if alloc.is_some() {
                return alloc;
            }

            let (key, alloc_id) = loop {
                let (key, value) = self.glyph_cache.peek_lru()?;
                if let GlyphCacheStatus::InAtlas { alloc_id, .. } = value.cache_status {
                    break (key, alloc_id);
                }

                if self.glyphs_in_use.contains(key) {
                    return None;
                }

                _ = self.glyph_cache.pop_lru();
            };

            if self.glyphs_in_use.contains(key) {
                return None;
            }

            _ = self.glyph_cache.pop_lru().unwrap();
            self.packer.deallocate(alloc_id);
        }
    }

    pub fn channel_count(&self) -> usize {
        self.typ.channel_count()
    }

    pub fn grow(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        font_system: &mut cosmic_text::FontSystem,
        cache: &mut cosmic_text::SwashCache,
    ) -> bool {
        if self.texture_size >= self.max_texture_size {
            return false;
        }

        const GROWTH_FACTOR: u32 = 2;
        let new_size = (self.texture_size * GROWTH_FACTOR).min(self.max_texture_size);

        self.packer.grow(size2(new_size as i32, new_size as i32));

        self.texture = Texture::blank(
            device,
            self.typ.texture_format(),
            new_size,
            new_size,
            wgpu::FilterMode::Nearest,
            wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            1,
            1,
        );

        for (&cache_key, glyph) in &self.glyph_cache {
            let (x, y) = match glyph.cache_status {
                GlyphCacheStatus::InAtlas { x, y, .. } => (x, y),
                GlyphCacheStatus::ZeroSized => continue,
            };

            let (image_data, width, height) = {
                let image = cache.get_image_uncached(font_system, cache_key).unwrap();
                let width = image.placement.width as usize;
                let height = image.placement.height as usize;

                (image.data, width, height)
            };

            queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &self.texture.texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d {
                        x: x as u32,
                        y: y as u32,
                        z: 0,
                    },
                    aspect: wgpu::TextureAspect::All,
                },
                &image_data,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(width as u32 * self.typ.channel_count() as u32),
                    rows_per_image: None,
                },
                wgpu::Extent3d {
                    width: width as u32,
                    height: height as u32,
                    depth_or_array_layers: 1,
                },
            );
        }

        self.texture_size = new_size;

        true
    }

    pub fn clear_in_use(&mut self) {
        self.glyphs_in_use.clear();
    }
}

pub fn create_atlases_bind_group(
    device: &wgpu::Device,
    mask: &GlyphAtlas,
    color: &GlyphAtlas,
) -> wgsl_main::globals::BindGroup2 {
    let params = wgsl_main::globals::BindGroup2EntriesEntriesParams {
        TEXT_MASK_T: &mask.texture.view,
        TEXT_MASK_S: &mask.texture.sampler,
        TEXT_COLOR_T: &color.texture.view,
        TEXT_COLOR_S: &color.texture.sampler,
    };

    wgsl_main::globals::BindGroup2::from_bindings(
        device,
        wgsl_main::globals::BindGroup2Entries::new(params),
    )
}
