use desen::{AppData, AppState, Stage, TextureInfo, run_app};
use dioxus_devtools::subsecond;
use image::ImageReader;

struct State {
    time: f32,
    tex: TextureInfo,
}

impl AppState for State {
    fn setup(data: &mut AppData) -> Self {
        let img: image::DynamicImage = ImageReader::open("examples/uv.png")
            .unwrap()
            .decode()
            .unwrap();
        let tex = data.load_texture_rgba(&img.to_rgba8(), img.width(), img.height(), false);
        Self { time: 0.0, tex }
    }

    fn fixed_update(&mut self, delta: f64, data: &mut AppData) {
        self.time += delta as f32;
    }

    fn render(&mut self, s: &mut Stage, delta: f64, data: &mut AppData) {
        s.draw_stroke = true;
        s.draw_fill = true;

        s.fill_color = [0.0, 0.0, 1.0, 1.0];
        s.stroke_color = [0.0, 1.0, 0.0, 1.0];
        s.ellipse().w(self.time * 20.0).h(self.time * 20.0).draw();
    }
}

fn main() {
    dioxus_devtools::connect_subsecond();
    run_app::<State>(100);
}
