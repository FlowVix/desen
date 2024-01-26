use lyon::{
    algorithms::rounded_polygon::add_rounded_polygon,
    lyon_tessellation::BuffersBuilder,
    math::{point, Box2D},
    path::{Path, Polygon, NO_ATTRIBUTES},
};

use crate::vertex::VertexConstructor;

use super::Frame;

pub struct TextureBuilder<'a> {
    pub(crate) frame: &'a mut Frame,
    pub(crate) x: f32,
    pub(crate) y: f32,
    pub(crate) w: Option<f32>,
    pub(crate) h: Option<f32>,
    pub(crate) centered: bool,
    pub(crate) tinted: bool,
    pub(crate) crop: Option<(f32, f32, f32, f32)>,
}

impl<'a> TextureBuilder<'a> {
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
    pub fn xy(mut self, x: f32, y: f32) -> Self {
        self.x = x;
        self.y = y;
        self
    }
    #[inline]
    pub fn wh(mut self, w: f32, h: f32) -> Self {
        self.w = Some(w);
        self.h = Some(h);
        self
    }
    #[inline]
    pub fn centered(mut self) -> Self {
        self.centered = true;
        self
    }
    #[inline]
    pub fn tinted(mut self) -> Self {
        self.tinted = true;
        self
    }
    #[inline]
    pub fn cropped(mut self, region: (f32, f32, f32, f32)) -> Self {
        self.crop = Some(region);
        self
    }
}

impl<'a> Drop for TextureBuilder<'a> {
    fn drop(&mut self) {
        if let Some(tex) = self.frame.current_textures[self.frame.current_texture_group as usize] {
            let texw = tex.width as f32;
            let texh = tex.height as f32;
            let (w, h) = if let Some(region) = self.crop {
                (self.w.unwrap_or(region.2), self.h.unwrap_or(region.3))
            } else {
                (self.w.unwrap_or(texw), self.h.unwrap_or(texh))
            };

            let (x, y) = if self.centered {
                (self.x - w / 2.0, self.y - h / 2.0)
            } else {
                (self.x, self.y)
            };

            let (x0, y0, x1, y1) = if let Some(region) = self.crop {
                (
                    region.0 / texw,
                    region.1 / texh,
                    (region.0 + region.2) / texw,
                    (region.1 + region.3) / texh,
                )
            } else {
                (0.0, 0.0, 1.0, 1.0)
            };

            let vert_count = self.frame.geometry.vertices.len() as u32;

            if !self.tinted {
                self.frame.geometry.vertices.extend(&[
                    VertexConstructor::new_textured(
                        (x0, y1),
                        self.frame.transform,
                        self.frame.current_texture_group,
                    )
                    .with_pos(x, y),
                    VertexConstructor::new_textured(
                        (x1, y1),
                        self.frame.transform,
                        self.frame.current_texture_group,
                    )
                    .with_pos(x + w, y),
                    VertexConstructor::new_textured(
                        (x1, y0),
                        self.frame.transform,
                        self.frame.current_texture_group,
                    )
                    .with_pos(x + w, y + h),
                    VertexConstructor::new_textured(
                        (x0, y0),
                        self.frame.transform,
                        self.frame.current_texture_group,
                    )
                    .with_pos(x, y + h),
                ]);
            } else {
                self.frame.geometry.vertices.extend(&[
                    VertexConstructor::new_textured_tinted(
                        self.frame.fill_color,
                        (x0, y1),
                        self.frame.transform,
                        self.frame.current_texture_group,
                    )
                    .with_pos(x, y),
                    VertexConstructor::new_textured_tinted(
                        self.frame.fill_color,
                        (x1, y1),
                        self.frame.transform,
                        self.frame.current_texture_group,
                    )
                    .with_pos(x + w, y),
                    VertexConstructor::new_textured_tinted(
                        self.frame.fill_color,
                        (x1, y0),
                        self.frame.transform,
                        self.frame.current_texture_group,
                    )
                    .with_pos(x + w, y + h),
                    VertexConstructor::new_textured_tinted(
                        self.frame.fill_color,
                        (x0, y0),
                        self.frame.transform,
                        self.frame.current_texture_group,
                    )
                    .with_pos(x, y + h),
                ]);
            }
            self.frame.geometry.indices.extend(&[
                vert_count,
                2 + vert_count,
                1 + vert_count,
                3 + vert_count,
                2 + vert_count,
                vert_count,
            ])
        }
    }
}
