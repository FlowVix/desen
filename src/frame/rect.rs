use lyon::{
    algorithms::rounded_polygon::add_rounded_polygon,
    lyon_tessellation::BuffersBuilder,
    math::{point, Box2D},
    path::{Path, Polygon, NO_ATTRIBUTES},
};

use crate::vertex::VertexConstructor;

use super::Frame;

pub struct RectBuilder<'a> {
    pub(crate) frame: &'a mut Frame,
    pub(crate) x: f32,
    pub(crate) y: f32,
    pub(crate) w: f32,
    pub(crate) h: f32,
    pub(crate) centered: bool,
    pub(crate) rounded: Option<f32>,
}

impl<'a> RectBuilder<'a> {
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
        self.w = w;
        self
    }
    #[inline]
    pub fn h(mut self, h: f32) -> Self {
        self.h = h;
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
        self.w = w;
        self.h = h;
        self
    }
    #[inline]
    pub fn centered(mut self) -> Self {
        self.centered = true;
        self
    }
    #[inline]
    pub fn rounded(mut self, r: f32) -> Self {
        self.rounded = Some(r);
        self
    }
}

impl<'a> Drop for RectBuilder<'a> {
    fn drop(&mut self) {
        let (x, y) = if self.centered {
            (self.x - self.w / 2.0, self.y - self.h / 2.0)
        } else {
            (self.x, self.y)
        };
        let (w, h) = (self.w, self.h);

        if let Some(r) = self.rounded {
            let rect_polygon = Polygon {
                points: &[
                    point(x, y),
                    point(x + w, y),
                    point(x + w, y + h),
                    point(x, y + h),
                ],
                closed: true,
            };
            let mut builder = Path::builder();

            add_rounded_polygon(&mut builder, rect_polygon, r, NO_ATTRIBUTES);
            //builder.add_polygon(arrow_polygon);
            let rect_path = builder.build();
            if self.frame.draw_fill {
                self.frame
                    .fill_tess
                    .tessellate_path(
                        &rect_path,
                        &self.frame.fill_options,
                        &mut BuffersBuilder::new(
                            &mut self.frame.geometry,
                            VertexConstructor::new_only_color(
                                self.frame.fill_color,
                                self.frame.transform,
                            ),
                        ),
                    )
                    .unwrap();
            }
            if self.frame.draw_stroke {
                // let mut stroke_tess = StrokeTessellator::new();
                self.frame
                    .stroke_tess
                    .tessellate_path(
                        &rect_path,
                        &self.frame.stroke_options,
                        &mut BuffersBuilder::new(
                            &mut self.frame.geometry,
                            VertexConstructor::new_only_color(
                                self.frame.stroke_color,
                                self.frame.transform,
                            ),
                        ),
                    )
                    .unwrap();
            }
        } else {
            if self.frame.draw_fill {
                // let mut fill_tess = FillTessellator::new();
                self.frame
                    .fill_tess
                    .tessellate_rectangle(
                        &Box2D::new(point(x, y), point(x + w, y + h)),
                        &self.frame.fill_options,
                        &mut BuffersBuilder::new(
                            &mut self.frame.geometry,
                            VertexConstructor::new_only_color(
                                self.frame.fill_color,
                                self.frame.transform,
                            ),
                        ),
                    )
                    .unwrap();
            }
            if self.frame.draw_stroke {
                // let mut stroke_tess = StrokeTessellator::new();
                self.frame
                    .stroke_tess
                    .tessellate_rectangle(
                        &Box2D::new(point(x, y), point(x + w, y + h)),
                        &self.frame.stroke_options,
                        &mut BuffersBuilder::new(
                            &mut self.frame.geometry,
                            VertexConstructor::new_only_color(
                                self.frame.stroke_color,
                                self.frame.transform,
                            ),
                        ),
                    )
                    .unwrap();
            }
        }
    }
}
