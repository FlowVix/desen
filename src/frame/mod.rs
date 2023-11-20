pub mod circle;
pub mod ellipse;
pub mod line;
pub mod rect;
pub mod texture;

use lyon::{
    algorithms::rounded_polygon::add_rounded_polygon,
    geom::LineSegment,
    lyon_tessellation::{
        BuffersBuilder, FillOptions, FillTessellator, StrokeOptions, StrokeTessellator,
        VertexBuffers,
    },
    math::{point, vector, Angle, Box2D},
    path::{Path, Polygon, NO_ATTRIBUTES},
};
use nalgebra::Matrix3;

use crate::{
    color::Color,
    texture::LoadedTexture,
    vertex::{Vertex, VertexConstructor},
};

use self::{
    circle::CircleBuilder, ellipse::EllipseBuilder, line::LineBuilder, rect::RectBuilder,
    texture::TextureBuilder,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlendMode {
    Normal,
    Additive,
    AdditiveSquaredAlpha,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DrawCall {
    pub(crate) start_index: u32,
    pub(crate) blend_mode: Option<BlendMode>,
    pub(crate) texture: Option<LoadedTexture>,
}

pub struct Frame {
    pub(crate) geometry: VertexBuffers<Vertex, u32>,

    pub(crate) fill_options: FillOptions,
    pub(crate) stroke_options: StrokeOptions,

    pub(crate) fill_tess: FillTessellator,
    pub(crate) stroke_tess: StrokeTessellator,

    pub(crate) fill_color: (f32, f32, f32, f32),
    pub(crate) draw_fill: bool,
    pub(crate) stroke_color: (f32, f32, f32, f32),
    pub(crate) draw_stroke: bool,

    pub(crate) transform: FrameTransformMatrix,
    pub(crate) transform_stack: Vec<FrameTransformMatrix>,

    pub(crate) draw_calls: Vec<DrawCall>,

    pub(crate) current_blend_mode: BlendMode,
    pub(crate) current_texture: Option<LoadedTexture>,
}

impl Frame {
    pub(crate) fn new() -> Self {
        Self {
            geometry: VertexBuffers::with_capacity(250000, 250000 / 3),
            fill_options: FillOptions::tolerance(0.5),
            stroke_options: StrokeOptions::tolerance(0.5).with_line_width(2.0),
            fill_tess: FillTessellator::new(),
            stroke_tess: StrokeTessellator::new(),
            fill_color: (0.3, 0.3, 0.3, 1.0),
            draw_fill: true,
            stroke_color: (0.8, 0.8, 0.8, 1.0),
            draw_stroke: true,
            transform: FrameTransformMatrix::identity(),
            transform_stack: vec![],
            draw_calls: vec![DrawCall {
                start_index: 0,
                blend_mode: Some(BlendMode::Normal),
                texture: None,
            }],
            current_blend_mode: BlendMode::Normal,
            current_texture: None,
            // font_system: FontSystem::new(),
            // swash_cache: SwashCache::new(),
        }
    }
    pub(crate) fn reset(&mut self) {
        self.geometry.vertices.clear();
        self.geometry.indices.clear();
        self.fill_options = FillOptions::tolerance(0.5);
        self.stroke_options = StrokeOptions::tolerance(0.5).with_line_width(2.0);
        self.fill_color = (0.3, 0.3, 0.3, 1.0);
        self.stroke_color = (0.8, 0.8, 0.8, 1.0);
        self.draw_fill = true;
        self.draw_stroke = true;
        self.transform = FrameTransformMatrix::identity();
        self.transform_stack.clear();
        self.draw_calls.clear();
        self.draw_calls.push(DrawCall {
            start_index: 0,
            blend_mode: Some(BlendMode::Normal),
            texture: None,
        });
        self.current_blend_mode = BlendMode::Normal;
        self.current_texture = None;
    }

    pub fn tolerance(&mut self, t: f32) {
        self.fill_options.tolerance = t;
        self.stroke_options.tolerance = t;
    }

    pub fn fill(&mut self, color: Color) {
        self.draw_fill = true;
        self.fill_color = color.to_rgba();
    }
    pub fn stroke(&mut self, color: Color) {
        self.draw_stroke = true;
        self.stroke_color = color.to_rgba();
    }
    pub fn stroke_weight(&mut self, weight: f32) {
        self.draw_stroke = true;
        self.stroke_options.line_width = weight;
    }
    pub fn no_fill(&mut self) {
        self.draw_fill = false;
    }
    pub fn no_stroke(&mut self) {
        self.draw_stroke = false;
    }

    pub fn rect(&mut self) -> RectBuilder {
        RectBuilder {
            frame: self,
            x: 0.0,
            y: 0.0,
            w: 10.0,
            h: 10.0,
            centered: false,
            rounded: None,
        }
    }
    pub fn circle(&mut self) -> CircleBuilder {
        CircleBuilder {
            frame: self,
            x: 0.0,
            y: 0.0,
            radius: 10.0,
        }
    }
    pub fn ellipse(&mut self) -> EllipseBuilder {
        EllipseBuilder {
            frame: self,
            x: 0.0,
            y: 0.0,
            w: 10.0,
            h: 10.0,
        }
    }
    pub fn line(&mut self) -> LineBuilder {
        LineBuilder {
            frame: self,
            x0: 0.0,
            y0: 0.0,
            x1: 0.0,
            y1: 0.0,
        }
    }

    pub fn translate(&mut self, x: f32, y: f32) {
        self.transform(FrameTransform::Translate { x, y })
    }
    pub fn rotate(&mut self, angle: f32) {
        self.transform(FrameTransform::Rotate(angle))
    }
    pub fn scale(&mut self, x: f32, y: f32) {
        self.transform(FrameTransform::Scale { x, y })
    }
    pub fn skew(&mut self, x: f32, y: f32) {
        self.transform(FrameTransform::Skew { x, y })
    }
    pub fn transform(&mut self, t: FrameTransform) {
        self.transform *= t.mat()
    }
    pub fn push(&mut self) {
        self.transform_stack.push(self.transform);
    }
    pub fn pop(&mut self) {
        if let Some(t) = self.transform_stack.pop() {
            self.transform = t
        }
    }

    pub fn texture(&mut self) -> TextureBuilder {
        TextureBuilder {
            frame: self,
            x: 0.0,
            y: 0.0,
            w: None,
            h: None,
            centered: false,
            tinted: false,
            crop: None,
        }
    }

    pub fn set_blend_mode(&mut self, mode: BlendMode) {
        self.draw_calls.push(DrawCall {
            start_index: self.geometry.indices.len() as u32,
            blend_mode: Some(mode),
            texture: None,
        });
        self.current_blend_mode = mode;
    }
    pub fn set_current_texture(&mut self, tex: LoadedTexture) {
        self.draw_calls.push(DrawCall {
            start_index: self.geometry.indices.len() as u32,
            blend_mode: None,
            texture: Some(tex),
        });
        self.current_texture = Some(tex)
    }

    pub fn get_transform(&self) -> FrameTransform {
        FrameTransform::Custom(self.transform)
    }
    pub fn set_transform(&mut self, t: FrameTransform) {
        self.transform = t.mat();
    }
}

pub type FrameTransformMatrix = Matrix3<f32>;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FrameTransform {
    Translate { x: f32, y: f32 },
    Rotate(f32),
    Scale { x: f32, y: f32 },
    Skew { x: f32, y: f32 },
    Custom(FrameTransformMatrix),
}

impl FrameTransform {
    #[inline]
    pub fn mat(self) -> FrameTransformMatrix {
        match self {
            #[rustfmt::skip]
            FrameTransform::Translate { x, y } => {
                FrameTransformMatrix::new(
                    1.0, 0.0, x,
                    0.0, 1.0, y,
                    0.0, 0.0, 1.0,
                )
            },
            #[rustfmt::skip]
            FrameTransform::Rotate(angle) => {
                let cos = angle.cos();
                let sin = angle.sin();

                FrameTransformMatrix::new(
                    cos, -sin, 0.0,
                    sin, cos, 0.0,
                    0.0, 0.0, 1.0,
                )
            }
            #[rustfmt::skip]
            FrameTransform::Scale { x, y } => {
                FrameTransformMatrix::new(
                    x, 0.0, 0.0,
                    0.0, y, 0.0,
                    0.0, 0.0, 1.0,
                )
            },
            #[rustfmt::skip]
            FrameTransform::Skew { x, y } => {
                FrameTransformMatrix::new(
                    1.0, x, 0.0,
                    y, 1.0, 0.0,
                    0.0, 0.0, 1.0,
                )
            },
            #[rustfmt::skip]
            FrameTransform::Custom(m) => m,
        }
    }
    pub fn series_mat<T>(v: T) -> FrameTransformMatrix
    where
        T: IntoIterator<Item = Self>,
    {
        let mut out = FrameTransformMatrix::identity();
        for i in v {
            out *= i.mat()
        }
        out
    }
}
