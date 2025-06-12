use std::{
    any::{Any, TypeId},
    collections::{HashMap, HashSet},
    hash::Hash,
    rc::Rc,
    thread,
    time::Duration,
};

use desen::{
    AppData, AppState, BlendMode, Color, PathBuilder, Stage, TextureInfo, run_app_windowed,
};

use dioxus_devtools::subsecond;
use glam::FloatExt;
use image::ImageReader;
use palette::{FromColor, Hsv, Srgb, Srgba, WithAlpha};

struct State {
    time: f32,

    tex: TextureInfo,
    show: bool,

    count: u32,
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
            count: 0,
        }
    }

    fn render(&mut self, s: &mut Stage, data: &mut AppData) {
        self.time += s.delta() as f32;

        s.draw_stroke = false;

        s.fill_color = Color::rgba8(255, 255, 255, 127);

        s.skew(1.0, 0.0);

        s.rect().w(100.0).h(50.0).draw();
        let mut p = PathBuilder::new();
        p.add_rectangle([0.0, 0.0], [100.0, 50.0]);
        let p = p.build();
        s.add_clip(&p);

        s.fill_color = Color::rgb8(255, 0, 0);
        let mpos = s.mouse_world_pos();
        s.ellipse().x(mpos[0]).y(mpos[1]).w(20.0).h(20.0).draw();
    }
}

impl State {
    #[allow(clippy::too_many_arguments)]
    fn button(
        &mut self,
        s: &mut Stage,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        id: impl Hash + 'static,
        mut onclick: impl FnMut(&mut Self, &mut Stage),
    ) {
        s.draw_stroke = false;
        s.draw_fill = true;
        let sense = s.rect_sense().x(x).y(y).w(w).h(h).test();

        let color = s.temp(id, || 0.4);

        if sense.holding {
            *color = color.lerp(0.2, 0.1);
        } else if sense.hovering {
            *color = color.lerp(0.6, 0.1);
        } else {
            *color = color.lerp(0.4, 0.1);
        }

        let color = *color;
        s.fill_color = Color::rgba(color, color, color, 1.0);
        s.rect().x(x).y(y).w(w).h(h).draw();
        if sense.click_ended {
            onclick(self, s);
        }
    }
}

fn main() {
    run_app_windowed::<State>();
}
