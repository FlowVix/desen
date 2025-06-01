use std::{thread, time::Duration};

use desen::{AppData, AppState, Stage, TextureInfo, run_app};

use dioxus_devtools::subsecond;
use image::ImageReader;

struct State {
    time: f32,
    tex: TextureInfo,
}

fn button(s: &mut Stage, x: f32, y: f32, w: f32, h: f32, mut onclick: impl FnMut()) {
    s.draw_stroke = false;
    s.draw_fill = true;
    let sense = s.rect_sense().x(x).y(y).w(w).h(h).test();

    s.fill_color = [0.4, 0.4, 0.4, 1.0];
    if sense.hovering {
        s.fill_color = [0.5, 0.5, 0.5, 1.0];
    }
    if sense.holding {
        s.fill_color = [0.3, 0.3, 0.3, 1.0];
    }
    s.rect().x(x).y(y).w(w).h(h).draw();
    if sense.click_ended {
        onclick();
    }
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
        subsecond::call(|| {
            button(s, 0.0, 0.0, 100.0, 50.0, || {
                println!("gaga");
            });
        })
    }
}

fn fact(v: u32) -> u32 {
    if v < 2 {
        return 1;
    }
    return v * fact(v - 1);
}

fn main() {
    dioxus_devtools::connect_subsecond();
    loop {
        subsecond::call(|| {
            thread::sleep(Duration::from_secs(2));
            println!("gagaeee {}", fact(6));
        })
    }
    // run_app::<State>(100);
}
