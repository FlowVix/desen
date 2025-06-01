use std::{
    any::Any,
    collections::{HashMap, HashSet},
    hash::Hash,
    rc::Rc,
    thread,
    time::Duration,
};

use desen::{AppData, AppState, Stage, TextureInfo, run_app};

use dioxus_devtools::subsecond;
use image::ImageReader;
#[derive(Debug, Default)]
pub struct TempStates<K, T> {
    pub(crate) map: HashMap<K, T>,
    pub(crate) marked: HashSet<K>,
    pub(crate) safe: HashSet<K>,
}
impl<K: Eq + Hash + Clone, T> TempStates<K, T> {
    fn start(&mut self) {
        std::mem::swap(&mut self.marked, &mut self.safe);
        self.safe.clear();
    }
    fn finish(&mut self) {
        for i in &self.marked {
            self.map.remove(i);
        }
    }
    pub fn temp(&mut self, id: K, init_fn: impl Fn() -> T) -> &mut T {
        self.marked.remove(&id);
        self.safe.insert(id.clone());
        self.map.entry(id).or_insert_with(init_fn)
    }
}

struct State {
    time: f32,

    temps: TempStates<i32, f32>,

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
            temps: TempStates::default(),
            tex,
            show: true,
        }
    }

    fn fixed_update(&mut self, delta: f64, data: &mut AppData) {
        self.time += delta as f32;
    }

    fn render(&mut self, s: &mut Stage, delta: f64, data: &mut AppData) {
        // self.temps.start();

        // self.button(s, 0.0, 100.0, 100.0, 50.0, |slef| {
        //     slef.show = !slef.show;
        // });

        // if self.show {
        //     s.fill_color = [1.0, 0.0, 0.0, 1.0];

        //     let v = self.temps.temp(69, || 0.0);
        //     *v += 0.5;

        //     s.rect().x(*v).w(50.0).h(50.0).draw();
        // }

        // self.temps.finish();

        s.fill_color = [0.2, 0.2, 0.2, 1.0];
        s.rect().w(150.0 + self.time * 50.0).h(-500.0).draw();
        s.fill_color = [1.0; 4];
        s.text()
            .app_data(data)
            .text("Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt 😍😍😍😍😂😂\n\n\nhello hello hello hello hello hello ")
            .w(150.0 + self.time * 50.0)
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
