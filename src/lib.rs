#![deny(unused_must_use, unnameable_types)]

mod render;
mod stage;
mod state;
mod util;

pub use stage::{
    BlendMode, ClipID, Stage,
    color::Color,
    path::{Path, PathBuilder},
    sense::Interactions,
};
pub use state::{AppData, AppState, texture::TextureInfo, windowed::run_app_windowed};
pub use winit;
