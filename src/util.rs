/// range is 0-1 for hsv and rgb
pub fn hsv_to_rgb(h: f32, s: f32, v: f32) -> (f32, f32, f32) {
    let h = h * 6.0;

    let c = v * s;
    let x = c * (1.0 - (h.rem_euclid(2.0) - 1.0).abs());

    let (r, g, b) = if (0.0..1.0).contains(&h) {
        (c, x, 0.0)
    } else if (1.0..2.0).contains(&h) {
        (x, c, 0.0)
    } else if (2.0..3.0).contains(&h) {
        (0.0, c, x)
    } else if (3.0..4.0).contains(&h) {
        (0.0, x, c)
    } else if (4.0..5.0).contains(&h) {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };

    let m = v - c;
    (r + m, g + m, b + m)
}
