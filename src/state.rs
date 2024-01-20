use winit::{event::WindowEvent, window::Window};

use crate::{app::App, frame::Frame};

pub trait WindowedAppInfo {
    fn init(app: App, window: Window) -> Self;
    fn get_app(&mut self) -> &mut App;
    fn get_window(&mut self) -> &mut Window;
}
pub trait WindowedAppState<I: WindowedAppInfo>: AppState {
    fn init(info: I) -> Self;
    fn get_info(&mut self) -> &mut I;
    fn event(&mut self, event: &WindowEvent);
}

#[cfg(feature = "html-canvas")]
pub trait CanvasAppInfo {
    fn init(app: App) -> Self;
    fn get_app(&mut self) -> &mut App;
}
#[cfg(feature = "html-canvas")]
pub trait CanvasAppState<I: CanvasAppInfo>: AppState {
    fn init(info: I) -> Self;
    fn get_info(&mut self) -> &mut I;
}
pub trait AppState {
    fn view(&mut self, frame: &mut Frame, delta: f32);
}
// #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
// pub struct LoadedTexture {
//     pub(crate) idx: usize,
//     pub(crate) w: u32,
//     pub(crate) h: u32,
// }

// pub struct ResourceLoader {
//     pub(crate) textures: Vec<DynamicImage>,
// }

// use image::io::Reader as ImageReader;
// impl ResourceLoader {
//     pub(crate) fn new() -> Self {
//         let mut out = Self { textures: vec![] };
//         out.load_texture_bytes(include_bytes!("./funny.png"));
//         out
//     }

//     fn load_texture(&mut self, texture: DynamicImage) -> LoadedTexture {
//         let out = LoadedTexture {
//             idx: self.textures.len(),
//             w: texture.width(),
//             h: texture.height(),
//         };
//         self.textures.push(texture);
//         out
//     }

//     pub fn load_texture_path<T: AsRef<Path>>(&mut self, path_str: T) -> LoadedTexture {
//         let img = ImageReader::open(path_str).unwrap().decode().unwrap();

//         self.load_texture(img)
//     }
//     pub fn load_texture_bytes(&mut self, bytes: &[u8]) -> LoadedTexture {
//         let img = ImageReader::new(Cursor::new(bytes))
//             .with_guessed_format()
//             .unwrap()
//             .decode()
//             .unwrap();

//         self.load_texture(img)
//     }
// }
