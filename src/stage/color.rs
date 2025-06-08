use palette::{FromColor, Hsv, Srgba, WithAlpha};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}
impl Color {
    // MARK: Constructors
    #[inline]
    pub fn from_array(arr: [f32; 4]) -> Self {
        Self {
            r: arr[0],
            g: arr[1],
            b: arr[2],
            a: arr[3],
        }
    }
    #[inline]
    pub fn from_array8(arr: [u8; 4]) -> Self {
        Self::from_array(arr.map(|v| v as f32 / 255.0))
    }

    #[inline]
    pub fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self::rgba(r, g, b, 1.0)
    }
    #[inline]
    pub fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self::from_array([r, g, b, a])
    }
    #[inline]
    pub fn rgb8(r: u8, g: u8, b: u8) -> Self {
        Self::rgba8(r, g, b, 255)
    }
    #[inline]
    pub fn rgba8(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self::from_array8([r, g, b, a])
    }

    #[inline]
    pub fn hsv(h: f32, s: f32, v: f32) -> Self {
        Self::hsva(h, s, v, 1.0)
    }
    #[inline]
    pub fn hsva(h: f32, s: f32, v: f32, a: f32) -> Self {
        let hsv = Hsv::new_srgb(h * 360.0, s, v).with_alpha(a);
        let rgb = Srgba::from_color(hsv.into_format::<f32, f32>());
        Self::from_array([rgb.red, rgb.green, rgb.blue, rgb.alpha])
    }
    #[inline]
    pub fn hsv2(h: f32, s: f32, v: f32) -> Self {
        Self::hsva2(h, s, v, 100.0)
    }
    #[inline]
    pub fn hsva2(h: f32, s: f32, v: f32, a: f32) -> Self {
        let hsv = Hsv::new_srgb(h, s / 100.0, v / 100.0).with_alpha(a / 100.0);
        let rgb = Srgba::from_color(hsv.into_format::<f32, f32>());
        Self::from_array([rgb.red, rgb.green, rgb.blue, rgb.alpha])
    }

    // MARK: Getters
    #[inline]
    pub fn to_array(&self) -> [f32; 4] {
        [self.r, self.g, self.b, self.a]
    }
    #[inline]
    pub fn to_array8(&self) -> [u8; 4] {
        [self.r, self.g, self.b, self.a].map(|v| (v * 255.0) as u8)
    }

    #[inline]
    pub fn r8(&self) -> u8 {
        (self.r * 255.0) as u8
    }
    #[inline]
    pub fn g8(&self) -> u8 {
        (self.g * 255.0) as u8
    }
    #[inline]
    pub fn b8(&self) -> u8 {
        (self.b * 255.0) as u8
    }
    #[inline]
    pub fn a8(&self) -> u8 {
        (self.a * 255.0) as u8
    }

    // MARK: Setters
    #[inline]
    pub fn set_r8(&mut self, to: u8) {
        self.r = to as f32 / 255.0;
    }
    #[inline]
    pub fn set_g8(&mut self, to: u8) {
        self.g = to as f32 / 255.0;
    }
    #[inline]
    pub fn set_b8(&mut self, to: u8) {
        self.b = to as f32 / 255.0;
    }
    #[inline]
    pub fn set_a8(&mut self, to: u8) {
        self.a = to as f32 / 255.0;
    }
}
