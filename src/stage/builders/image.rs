use crate::{Graphics, shaders::wgsl_main::structs::VertexInput};

#[must_use = "draw builders do nothing unless `.draw()` is called"]
pub struct ImageBuilder<'a> {
    pub(crate) graphics: &'a mut Graphics,
    pub x: f32,
    pub y: f32,
    pub w: Option<f32>,
    pub h: Option<f32>,
    pub centered: bool,
    pub tinted: bool,
    pub(crate) crop: Option<(f32, f32, f32, f32)>,
}

impl ImageBuilder<'_> {
    #[inline]
    pub fn x(mut self, x: f32) -> Self {
        self.x = x;
        self
    }
    #[inline]
    pub fn y(mut self, y: f32) -> Self {
        self.y = y;
        self
    }
    #[inline]
    pub fn xy(mut self, x: f32, y: f32) -> Self {
        self.x = x;
        self.y = y;
        self
    }

    #[inline]
    pub fn w(mut self, w: f32) -> Self {
        self.w = Some(w);
        self
    }
    #[inline]
    pub fn h(mut self, h: f32) -> Self {
        self.h = Some(h);
        self
    }
    #[inline]
    pub fn wh(mut self, w: f32, h: f32) -> Self {
        self.w = Some(w);
        self.h = Some(h);
        self
    }

    #[inline]
    pub fn centered(mut self, centered: bool) -> Self {
        self.centered = centered;
        self
    }
    #[inline]
    pub fn tinted(mut self, tinted: bool) -> Self {
        self.tinted = tinted;
        self
    }
    #[inline]
    pub fn cropped(mut self, x: f32, y: f32, w: f32, h: f32) -> Self {
        self.crop = Some((x, y, w, h));
        self
    }

    pub fn draw(self) {
        if let Some(tex) = self.graphics.current_texture {
            let texw = tex.width as f32;
            let texh = tex.height as f32;
            let (w, h) = if let Some(region) = self.crop {
                (self.w.unwrap_or(region.2), self.h.unwrap_or(region.3))
            } else {
                (self.w.unwrap_or(texw), self.h.unwrap_or(texh))
            };

            let (uv_x0, uv_y0, uv_x1, uv_y1) = if let Some(region) = self.crop {
                (
                    region.0 / texw,
                    region.1 / texh,
                    (region.0 + region.2) / texw,
                    (region.1 + region.3) / texh,
                )
            } else {
                (0.0, 0.0, 1.0, 1.0)
            };

            let color = if self.tinted {
                self.graphics.fill_color
            } else {
                [1.0; 4]
            };

            let mut points = [
                ([0.0, 0.0], [uv_x0, uv_y1]),
                ([w, 0.0], [uv_x1, uv_y1]),
                ([w, h], [uv_x1, uv_y0]),
                ([0.0, h], [uv_x0, uv_y0]),
            ]
            .map(|([x, y], uv)| ([x + self.x, y + self.y], uv));
            if self.centered {
                for (i, _) in &mut points {
                    i[0] -= w / 2.0;
                    i[1] -= h / 2.0;
                }
            }
            self.graphics.add_geometry(
                points.map(|(pos, uv)| {
                    VertexInput::new(
                        pos,
                        color,
                        uv,
                        self.graphics.transform.matrix2.x_axis.to_array(),
                        self.graphics.transform.matrix2.y_axis.to_array(),
                        self.graphics.transform.translation.to_array(),
                    )
                }),
                [0, 1, 2, 2, 3, 0],
            );
        }
    }
}
