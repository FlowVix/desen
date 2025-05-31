use desen::{AppData, AppState, Stage, run_app};
use dioxus_devtools::subsecond;

struct State {
    time: f32,
}

impl AppState for State {
    fn setup(data: &mut AppData) -> Self {
        Self { time: 0.0 }
    }

    fn fixed_update(&mut self, delta: f64, data: &mut AppData) {
        self.time += delta as f32;
    }

    fn render(&mut self, s: &mut Stage, delta: f64, data: &mut AppData) {
        s.draw_stroke = false;
        s.draw_fill = true;
        s.rotate(0.5);
        s.translate(100.0, 0.0);

        let check = |s: &mut Stage, x: f32, y: f32, w: f32, h: f32, msg: &str| {
            s.rect().x(x).y(y).w(w).h(h).draw();
            if s.ellipse_sense().x(x).y(y).w(w).h(h).test().clicked {
                println!("{}", msg);
            }
        };

        s.fill_color = [1.0, 0.0, 0.0, 1.0];
        check(s, 0.0, 0.0, 300.0, 100.0, "A");
        // s.fill_color = [0.0, 0.0, 1.0, 1.0];
        // check(s, 40.0, 20.0, 100.0, 100.0, "BBB");

        // s.tri()
        //     .a([0.0, 0.0])
        //     .b([100.0, 0.0])
        //     .c([100.0, 100.0])
        //     .color_a([1.0, 0.0, 0.0, 1.0])
        //     .color_b([0.0, 1.0, 0.0, 1.0])
        //     .color_c([0.0, 0.0, 1.0, 1.0])
        //     .draw();
    }
}

fn main() {
    dioxus_devtools::connect_subsecond();
    run_app::<State>(100);
}
