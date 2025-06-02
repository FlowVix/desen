use std::{
    any::{Any, TypeId},
    collections::{HashMap, HashSet},
    hash::Hash,
    rc::Rc,
    thread,
    time::Duration,
};

use desen::{AppData, AppState, Stage, TextureInfo, run_app};

use dioxus_devtools::subsecond;
use image::ImageReader;

struct State {
    time: f32,

    tex: TextureInfo,
    show: bool,
}

impl AppState for State {
    fn setup(data: &mut AppData) -> Self {
        let img: image::DynamicImage = ImageReader::open("examples/uv.png")
            .unwrap()
            .decode()
            .unwrap();
        let tex = data.load_texture_rgba(&img.to_rgba8(), img.width(), img.height(), false);
        Self {
            time: 0.0,
            tex,
            show: true,
        }
    }

    fn fixed_update(&mut self, delta: f64, data: &mut AppData) {
        self.time += delta as f32;
    }

    fn render(&mut self, s: &mut Stage, delta: f64, data: &mut AppData) {
        s.draw_stroke = true;
        s.stroke_weight = 20.0;
        s.line()
            .x1(0.0)
            .y1(0.0)
            .x2(self.time.cos() * 100.0)
            .y2(self.time.sin() * 100.0)
            .draw();
    }
}

impl State {
    fn button(
        &mut self,
        s: &mut Stage,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        mut onclick: impl FnMut(&mut Self),
    ) {
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
            onclick(self);
        }
    }
}

fn main() {
    run_app::<State>(100);
}
