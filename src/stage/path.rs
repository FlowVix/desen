use lyon::{
    math::{Angle, Box2D},
    path::{
        BuilderImpl, Winding,
        builder::{BorderRadii, NoAttributes},
    },
};

#[derive(Clone)]
pub struct Path {
    pub(crate) inner: lyon::path::Path,
}

#[derive(Clone)]
pub struct PathBuilder {
    pub(crate) inner: NoAttributes<BuilderImpl>,
}
impl Default for PathBuilder {
    fn default() -> Self {
        Self::new()
    }
}
impl PathBuilder {
    pub fn new() -> Self {
        Self {
            inner: lyon::path::Path::builder(),
        }
    }
    pub fn add_circle(&mut self, center: [f32; 2], radius: f32) {
        self.inner
            .add_circle(center.into(), radius, Winding::Positive);
    }
    pub fn add_ellipse(&mut self, center: [f32; 2], radii: [f32; 2], x_rot: f32) {
        self.inner.add_ellipse(
            center.into(),
            radii.into(),
            Angle { radians: x_rot },
            Winding::Positive,
        );
    }
    pub fn add_rectangle(&mut self, pos: [f32; 2], size: [f32; 2]) {
        self.inner.add_rectangle(
            &Box2D::from_origin_and_size(pos.into(), size.into()),
            Winding::Positive,
        );
    }
    pub fn add_rounded_rectangle(&mut self, pos: [f32; 2], size: [f32; 2], radii: [f32; 4]) {
        self.inner.add_rounded_rectangle(
            &Box2D::from_origin_and_size(pos.into(), size.into()),
            &BorderRadii {
                top_left: radii[0],
                top_right: radii[1],
                bottom_right: radii[2],
                bottom_left: radii[3],
            },
            Winding::Positive,
        );
    }
    pub fn begin(&mut self, at: [f32; 2]) {
        self.inner.begin(at.into());
    }
    pub fn line_to(&mut self, pos: [f32; 2]) {
        self.inner.line_to(pos.into());
    }
    pub fn quadratic_bezier_to(&mut self, pos: [f32; 2], ctrl: [f32; 2]) {
        self.inner.quadratic_bezier_to(ctrl.into(), pos.into());
    }
    pub fn cubic_bezier_to(&mut self, pos: [f32; 2], ctrl1: [f32; 2], ctrl2: [f32; 2]) {
        self.inner
            .cubic_bezier_to(ctrl1.into(), ctrl2.into(), pos.into());
    }
    pub fn end(&mut self, close: bool) {
        self.inner.end(close);
    }
    pub fn build(self) -> Path {
        Path {
            inner: self.inner.build(),
        }
    }
}
