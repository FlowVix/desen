use lyon::{
    lyon_tessellation::BuffersBuilder,
    math::{point, vector, Angle, Box2D},
};

use crate::vertex::VertexConstructor;

use super::Frame;

pub struct EllipseBuilder<'a> {
    pub(crate) frame: &'a mut Frame,
    pub(crate) x: f32,
    pub(crate) y: f32,
    pub(crate) w: f32,
    pub(crate) h: f32,
}

impl<'a> EllipseBuilder<'a> {
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
}

impl<'a> Drop for EllipseBuilder<'a> {
    fn drop(&mut self) {
        if self.frame.draw_fill {
            // let mut fill_tess = FillTessellator::new();
            self.frame
                .fill_tess
                .tessellate_ellipse(
                    point(self.x, self.y),
                    vector(self.w, self.h),
                    Angle::zero(),
                    lyon::path::Winding::Positive,
                    &self.frame.fill_options,
                    &mut BuffersBuilder::new(
                        &mut self.frame.geometry,
                        VertexConstructor::new_only_color(
                            self.frame.fill_color,
                            self.frame.transform,
                            self.frame.current_texture_group,
                        ),
                    ),
                )
                .unwrap()
        }
        if self.frame.draw_stroke {
            self.frame
                .stroke_tess
                .tessellate_ellipse(
                    point(self.x, self.y),
                    vector(self.w, self.h),
                    Angle::zero(),
                    lyon::path::Winding::Positive,
                    &self.frame.stroke_options,
                    &mut BuffersBuilder::new(
                        &mut self.frame.geometry,
                        VertexConstructor::new_only_color(
                            self.frame.fill_color,
                            self.frame.transform,
                            self.frame.current_texture_group,
                        ),
                    ),
                )
                .unwrap()
        }
    }
}
