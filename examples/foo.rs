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
        // self.time += delta as f32;
    }

    fn render(&mut self, s: &mut Stage, delta: f64, data: &mut AppData) {
        self.button(s, 0.0, 100.0, 100.0, 50.0, |slef| {
            slef.show = !slef.show;
        });

        if self.show {
            s.fill_color = [1.0, 0.0, 0.0, 1.0];

            let x = s.temp("test", || 0.0);
            *x += 0.5;
            let x = *x;

            let y = s.temp("test", || 0u64);
            *y += 1;
            let y = *y;

            s.rect().x(x).y(y as f32).w(50.0).h(50.0).draw();
        }
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
