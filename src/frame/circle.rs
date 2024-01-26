use lyon::{
    lyon_tessellation::BuffersBuilder,
    math::{point, Box2D},
};

use crate::vertex::VertexConstructor;

use super::Frame;

pub struct CircleBuilder<'a> {
    pub(crate) frame: &'a mut Frame,
    pub(crate) x: f32,
    pub(crate) y: f32,
    pub(crate) radius: f32,
}

impl<'a> CircleBuilder<'a> {
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
    pub fn radius(mut self, r: f32) -> Self {
        self.radius = r;
        self
    }
    #[inline]
    pub fn xy(mut self, x: f32, y: f32) -> Self {
        self.x = x;
        self.y = y;
        self
    }
}

impl<'a> Drop for CircleBuilder<'a> {
    fn drop(&mut self) {
        if self.frame.draw_fill {
            // let mut fill_tess = FillTessellator::new();
            self.frame
                .fill_tess
                .tessellate_circle(
                    point(self.x, self.y),
                    self.radius,
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
                .unwrap();
        }
        if self.frame.draw_stroke {
            // let mut stroke_tess = StrokeTessellator::new();
            self.frame
                .stroke_tess
                .tessellate_circle(
                    point(self.x, self.y),
                    self.radius,
                    &self.frame.stroke_options,
                    &mut BuffersBuilder::new(
                        &mut self.frame.geometry,
                        VertexConstructor::new_only_color(
                            self.frame.stroke_color,
                            self.frame.transform,
                            self.frame.current_texture_group,
                        ),
                    ),
                )
                .unwrap();
        }
    }
}
