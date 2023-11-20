use crate::util::hsv_to_rgb;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Color {
    Rgb8(u8, u8, u8),
    Rgba8(u8, u8, u8, u8),

    Rgb(f32, f32, f32),
    Rgba(f32, f32, f32, f32),

    Hsv(f32, f32, f32),
    Hsva(f32, f32, f32, f32),

    Hsv2(f32, f32, f32),
    Hsva2(f32, f32, f32, f32),
}

impl Color {
    pub(crate) fn to_rgba(self) -> (f32, f32, f32, f32) {
        let (h, s, v, a) = match self {
            Color::Rgb8(r, g, b) => {
                return (r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, 1.0)
            }
            Color::Rgba8(r, g, b, a) => {
                return (
                    r as f32 / 255.0,
                    g as f32 / 255.0,
                    b as f32 / 255.0,
                    a as f32 / 255.0,
                )
            }
            Color::Rgb(r, g, b) => return (r, g, b, 1.0),
            Color::Rgba(r, g, b, a) => return (r, g, b, a),
            Color::Hsv(h, s, v) => (h, s, v, 1.0),
            Color::Hsva(h, s, v, a) => (h, s, v, a),
            Color::Hsv2(h, s, v) => (h / 360.0, s / 100.0, v / 100.0, 1.0),
            Color::Hsva2(h, s, v, a) => (h / 360.0, s / 100.0, v / 100.0, a / 100.0),
        };
        let (r, g, b) = hsv_to_rgb(h, s, v);
        (r, g, b, a)
    }
}
