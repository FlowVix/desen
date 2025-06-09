#![deny(unused_must_use, unnameable_types)]

mod render;
mod stage;
mod state;
mod util;

pub use stage::{BlendMode, Stage, color::Color, sense::Interactions};
pub use state::{AppState, data::AppData, data::TextureInfo, run_app};
pub use winit;
