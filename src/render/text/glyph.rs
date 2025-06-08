use etagere::AllocId;
use glam::Affine2;

use crate::{
    render::{gpu::GPUData, text::atlas::create_atlases_bind_group},
    shaders::wgsl_main,
};

/// some glyphs are just a mask where each pixel is a 0-255 value, like most text
/// other glyphs are full color, like emojis
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ContentType {
    Mask,
    Color,
}
impl ContentType {
    /// how many values are used per pixel
    pub fn channel_count(self) -> usize {
        match self {
            Self::Mask => 1,
            Self::Color => 4,
        }
    }

    /// you get it
    pub fn texture_format(self) -> wgpu::TextureFormat {
        match self {
            Self::Mask => wgpu::TextureFormat::R8Unorm,
            Self::Color => wgpu::TextureFormat::Rgba8Unorm,
        }
    }
}

/// data about a glyph in the atlases
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GlyphCacheStatus {
    InAtlas {
        x: u16,
        y: u16,
        content_type: ContentType,
        alloc_id: AllocId,
    },
    ZeroSized,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GlyphData {
    pub width: u16,
    pub height: u16,
    pub top: i16,
    pub left: i16,
    pub cache_status: GlyphCacheStatus,
}

pub fn prepare_glyph(
    physical: cosmic_text::PhysicalGlyph,
    line_y: f32,
    gpu_data: &mut GPUData,
    color: [f32; 4],
    transform: Affine2,
    offset_x: f32,
    offset_y: f32,
) -> Option<[wgsl_main::structs::InstanceInput; 2]> {
    let data = if let Some(d) = gpu_data.mask_atlas.glyph_cache.get(&physical.cache_key) {
        gpu_data.mask_atlas.glyphs_in_use.insert(physical.cache_key);
        d
    } else if let Some(d) = gpu_data.color_atlas.glyph_cache.get(&physical.cache_key) {
        gpu_data
            .color_atlas
            .glyphs_in_use
            .insert(physical.cache_key);
        d
    } else {
        let image = gpu_data
            .swash_cache
            .get_image_uncached(&mut gpu_data.font_system, physical.cache_key)?;

        let content_type = match image.content {
            cosmic_text::SwashContent::Color => ContentType::Color,
            cosmic_text::SwashContent::Mask => ContentType::Mask,
            cosmic_text::SwashContent::SubpixelMask => ContentType::Mask,
        };

        let nonzero = image.placement.width > 0 && image.placement.height > 0;

        let mut atlas = match content_type {
            ContentType::Mask => &mut gpu_data.mask_atlas,
            ContentType::Color => &mut gpu_data.color_atlas,
        };

        let cache_status = if nonzero {
            let alloc = loop {
                match atlas.try_alloc(
                    image.placement.width as usize,
                    image.placement.height as usize,
                ) {
                    Some(a) => break a,
                    None => {
                        if !atlas.grow(
                            &gpu_data.device,
                            &gpu_data.queue,
                            &mut gpu_data.font_system,
                            &mut gpu_data.swash_cache,
                        ) {
                            // full atlas
                            return None;
                        }

                        gpu_data.text_atlas_bind_group = create_atlases_bind_group(
                            &gpu_data.device,
                            &gpu_data.mask_atlas,
                            &gpu_data.color_atlas,
                        );

                        atlas = match content_type {
                            ContentType::Mask => &mut gpu_data.mask_atlas,
                            ContentType::Color => &mut gpu_data.color_atlas,
                        };
                    }
                }
            };
            let atlas_min = alloc.rectangle.min;

            gpu_data.queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &atlas.texture.texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d {
                        x: atlas_min.x as u32,
                        y: atlas_min.y as u32,
                        z: 0,
                    },
                    aspect: wgpu::TextureAspect::All,
                },
                &image.data,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(image.placement.width * atlas.channel_count() as u32),
                    rows_per_image: None,
                },
                wgpu::Extent3d {
                    width: image.placement.width,
                    height: image.placement.height,
                    depth_or_array_layers: 1,
                },
            );

            GlyphCacheStatus::InAtlas {
                x: atlas_min.x as u16,
                y: atlas_min.y as u16,
                content_type,
                alloc_id: alloc.id,
            }
        } else {
            GlyphCacheStatus::ZeroSized
        };

        atlas.glyphs_in_use.insert(physical.cache_key);
        atlas
            .glyph_cache
            .get_or_insert(physical.cache_key, || GlyphData {
                width: image.placement.width as u16,
                height: image.placement.height as u16,
                top: image.placement.top as i16,
                left: image.placement.left as i16,
                cache_status,
            })
    };

    let x = physical.x as f32 + data.left as f32;
    let y = line_y.round() + physical.y as f32 - data.top as f32;

    let (atlas_x, atlas_y, content_type) = match data.cache_status {
        GlyphCacheStatus::InAtlas {
            x, y, content_type, ..
        } => (x as f32, y as f32, content_type),
        GlyphCacheStatus::ZeroSized => return None,
    };

    let width = data.width as f32;
    let height = data.height as f32;

    let points = [[0.0, 0.0], [width, 0.0], [width, -height], [0.0, -height]]
        .map(|[p0, p1]| [p0 + x + offset_x, p1 - y + offset_y]);
    let uv = [[0.0, 0.0], [width, 0.0], [width, height], [0.0, height]]
        .map(|[p0, p1]| [p0 + atlas_x, p1 + atlas_y]);

    Some([
        wgsl_main::structs::InstanceInput::new(
            points[0],
            points[1],
            points[2],
            color,
            color,
            color,
            uv[0],
            uv[1],
            uv[2],
            transform.matrix2.x_axis.to_array(),
            transform.matrix2.y_axis.to_array(),
            transform.translation.to_array(),
            if content_type == ContentType::Mask {
                1
            } else {
                2
            },
        ),
        wgsl_main::structs::InstanceInput::new(
            points[2],
            points[3],
            points[0],
            color,
            color,
            color,
            uv[2],
            uv[3],
            uv[0],
            transform.matrix2.x_axis.to_array(),
            transform.matrix2.y_axis.to_array(),
            transform.translation.to_array(),
            if content_type == ContentType::Mask {
                1
            } else {
                2
            },
        ),
    ])
}
