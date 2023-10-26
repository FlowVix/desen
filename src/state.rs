use std::path::Path;

use image::DynamicImage;
use texture_packer::{
    exporter::ImageExporter, importer::ImageImporter, TexturePacker, TexturePackerConfig,
};
use winit::{event::WindowEvent, window::Window};

use crate::frame::Frame;

pub trait WindowedAppState: AppState {
    fn init(window: Window, loader: &mut ResourceLoader) -> Self;
    fn get_window(&self) -> &Window;
    fn event(&mut self, event: &WindowEvent);
}
#[cfg(feature = "html-canvas")]
pub trait CanvasAppState: AppState {
    fn init(loader: &mut ResourceLoader) -> Self;
}
pub trait AppState {
    fn update(&mut self, delta: f32);
    fn view(&self, frame: &mut Frame);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LoadedTexture {
    pub(crate) x: u32,
    pub(crate) y: u32,
    pub(crate) w: u32,
    pub(crate) h: u32,
}

impl LoadedTexture {
    pub(crate) fn atlas_coords(&self, atlas_width: f32, atlas_height: f32) -> (f32, f32, f32, f32) {
        (
            self.x as f32 / atlas_width,
            self.y as f32 / atlas_height,
            (self.x + self.w) as f32 / atlas_width,
            (self.y + self.h) as f32 / atlas_height,
        )
    }
}

pub struct ResourceLoader<'a> {
    packer: TexturePacker<'a, image::DynamicImage, usize>,
}

impl<'a> ResourceLoader<'a> {
    pub(crate) fn new() -> Self {
        let config = TexturePackerConfig {
            max_width: 4096,
            max_height: 4096,
            allow_rotation: false,
            border_padding: 2,
            trim: false,
            ..Default::default()
        };
        let packer = TexturePacker::new_skyline(config);
        Self { packer }
    }

    fn load_texture(&mut self, texture: DynamicImage) -> LoadedTexture {
        let key = self.packer.get_frames().len();
        self.packer.pack_own(key, texture).unwrap();
        let frame = self.packer.get_frame(&key).unwrap().frame;
        LoadedTexture {
            x: frame.x,
            y: frame.y,
            w: frame.w,
            h: frame.h,
        }
    }

    pub fn load_texture_path(&mut self, path_str: &str) -> LoadedTexture {
        let path = Path::new(path_str);
        let texture = ImageImporter::import_from_file(path)
            .unwrap_or_else(|_| panic!("Unable to import image at {:?}", path));

        self.load_texture(texture)
    }
    pub fn load_texture_bytes(&mut self, bytes: &[u8]) -> LoadedTexture {
        let texture = ImageImporter::import_from_memory(bytes)
            .unwrap_or_else(|_| panic!("Unable to import image"));

        self.load_texture(texture)
    }

    pub fn build_atlas(&self) -> DynamicImage {
        ImageExporter::export(&self.packer).unwrap()
    }
}
