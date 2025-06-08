use image::GenericImageView;

pub struct Texture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
}

impl Texture {
    pub fn blank(
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        width: u32,
        height: u32,
        filter: wgpu::FilterMode,
        usage: wgpu::TextureUsages,
        mip_level_count: u32,
        sample_count: u32,
    ) -> Self {
        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("texture"),
            size,
            mip_level_count,
            sample_count,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage,
            view_formats: &[],
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: filter,
            min_filter: filter,
            mipmap_filter: filter,
            ..Default::default()
        });

        Self {
            texture,
            view,
            sampler,
        }
    }
    pub fn from_rgba(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        rgba: &[u8],
        width: u32,
        height: u32,
        filter: wgpu::FilterMode,
        usage: wgpu::TextureUsages,
    ) -> Self {
        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };
        let format = wgpu::TextureFormat::Rgba8Unorm;
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("texture"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage,
            view_formats: &[],
        });

        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                aspect: wgpu::TextureAspect::All,
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            rgba,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * width),
                rows_per_image: Some(height),
            },
            size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: filter,
            min_filter: filter,
            mipmap_filter: filter,
            ..Default::default()
        });

        Self {
            texture,
            view,
            sampler,
        }
    }
    // pub fn from_image(
    //     device: &wgpu::Device,
    //     queue: &wgpu::Queue,
    //     img: &image::DynamicImage,
    //     filter: wgpu::FilterMode,
    //     usage: wgpu::TextureUsages,
    // ) -> Self {
    //     let rgba = img.to_rgba8();
    //     let dimensions = img.dimensions();

    //     let size = wgpu::Extent3d {
    //         width: dimensions.0,
    //         height: dimensions.1,
    //         depth_or_array_layers: 1,
    //     };
    //     let format = wgpu::TextureFormat::Rgba8Unorm;
    //     let texture = device.create_texture(&wgpu::TextureDescriptor {
    //         label: Some("texure"),
    //         size,
    //         mip_level_count: 1,
    //         sample_count: 1,
    //         dimension: wgpu::TextureDimension::D2,
    //         format,
    //         usage,
    //         view_formats: &[],
    //     });

    //     queue.write_texture(
    //         wgpu::TexelCopyTextureInfo {
    //             aspect: wgpu::TextureAspect::All,
    //             texture: &texture,
    //             mip_level: 0,
    //             origin: wgpu::Origin3d::ZERO,
    //         },
    //         &rgba,
    //         wgpu::TexelCopyBufferLayout {
    //             offset: 0,
    //             bytes_per_row: Some(4 * dimensions.0),
    //             rows_per_image: Some(dimensions.1),
    //         },
    //         size,
    //     );

    //     let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    //     let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
    //         address_mode_u: wgpu::AddressMode::ClampToEdge,
    //         address_mode_v: wgpu::AddressMode::ClampToEdge,
    //         address_mode_w: wgpu::AddressMode::ClampToEdge,
    //         mag_filter: filter,
    //         min_filter: filter,
    //         mipmap_filter: filter,
    //         ..Default::default()
    //     });

    //     Self {
    //         texture,
    //         view,
    //         sampler,
    //     }
    // }
}
