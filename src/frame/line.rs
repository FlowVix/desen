use lyon::{
    algorithms::rounded_polygon::add_rounded_polygon,
    geom::LineSegment,
    lyon_tessellation::BuffersBuilder,
    math::{point, Box2D},
    path::{Path, Polygon, NO_ATTRIBUTES},
};

use crate::vertex::VertexConstructor;

use super::Frame;

pub struct LineBuilder<'a> {
    pub(crate) frame: &'a mut Frame,
    pub(crate) x0: f32,
    pub(crate) y0: f32,
    pub(crate) x1: f32,
    pub(crate) y1: f32,
}

impl<'a> LineBuilder<'a> {
    #[inline]
    pub fn x0(mut self, x0: f32) -> Self {
        self.x0 = x0;
        self
    }
    #[inline]
    pub fn y0(mut self, y0: f32) -> Self {
        self.y0 = y0;
        self
    }
    #[inline]
    pub fn x1(mut self, x1: f32) -> Self {
        self.x1 = x1;
        self
    }
    #[inline]
    pub fn y1(mut self, y1: f32) -> Self {
        self.y1 = y1;
        self
    }
    #[inline]
    pub fn xy0(mut self, x: f32, y: f32) -> Self {
        self.x0 = x;
        self.y0 = y;
        self
    }
    #[inline]
    pub fn xy1(mut self, x: f32, y: f32) -> Self {
        self.x1 = x;
        self.y1 = y;
        self
    }
}

impl<'a> Drop for LineBuilder<'a> {
    fn drop(&mut self) {
        if self.frame.draw_stroke {
            let mut builder = Path::builder();
            // builder.begin(point(x0, y0));
            // builder.line_to(point(x1, y1));
            // builder.close();
            builder.add_line_segment(&LineSegment {
                from: point(self.x0, self.y0),
                to: point(self.x1, self.y1),
            });
            let path = builder.build();
            self.frame
                .stroke_tess
                .tessellate_path(
                    &path,
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
