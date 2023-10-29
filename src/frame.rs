use lyon::{
    algorithms::rounded_polygon::add_rounded_polygon,
    geom::LineSegment,
    lyon_tessellation::{
        BuffersBuilder, FillOptions, FillTessellator, StrokeOptions, StrokeTessellator,
        VertexBuffers,
    },
    math::{point, Box2D},
    path::{Path, Polygon, NO_ATTRIBUTES},
};
use nalgebra::Matrix3;

use crate::{
    state::LoadedTexture,
    vertex::{Vertex, VertexConstructor},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlendMode {
    Normal,
    Additive,
    AdditiveSquaredAlpha,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DrawChunk {
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

    pub(crate) transform: Matrix3<f32>,
    pub(crate) transform_stack: Vec<Matrix3<f32>>,

    pub(crate) draw_chunks: Vec<DrawChunk>,

    pub(crate) current_blend_mode: BlendMode,
    pub(crate) current_texture: Option<LoadedTexture>,
}

impl Frame {
    pub(crate) fn new() -> Self {
        Self {
            // draw_commands: vec![],
            geometry: VertexBuffers::with_capacity(250000, 250000 / 3),
            fill_options: FillOptions::tolerance(0.5),
            stroke_options: StrokeOptions::tolerance(0.5).with_line_width(2.0),
            fill_tess: FillTessellator::new(),
            stroke_tess: StrokeTessellator::new(),
            fill_color: (0.3, 0.3, 0.3, 1.0),
            draw_fill: true,
            stroke_color: (0.8, 0.8, 0.8, 1.0),
            draw_stroke: true,
            transform: Matrix3::<f32>::identity(),
            transform_stack: vec![],
            draw_chunks: vec![DrawChunk {
                start_index: 0,
                blend_mode: Some(BlendMode::Normal),
                texture: None,
            }],
            current_blend_mode: BlendMode::Normal,
            current_texture: None,
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
        self.transform = Matrix3::<f32>::identity();
        self.transform_stack.clear();
        self.draw_chunks.clear();
        self.draw_chunks.push(DrawChunk {
            start_index: 0,
            blend_mode: Some(BlendMode::Normal),
            texture: None,
        });
        self.current_blend_mode = BlendMode::Normal;
        self.current_texture = None;
    }

    pub fn fill(&mut self, r: u8, g: u8, b: u8, a: u8) {
        // let (r, g, b) = to_linear_rgb(r, g, b);
        self.draw_fill = true;
        self.fill_color = (
            r as f32 / 255.0,
            g as f32 / 255.0,
            b as f32 / 255.0,
            a as f32 / 255.0,
        )
    }
    pub fn stroke(&mut self, r: u8, g: u8, b: u8, a: u8) {
        // let (r, g, b) = to_linear_rgb(r, g, b);

        self.draw_stroke = true;
        self.stroke_color = (
            r as f32 / 255.0,
            g as f32 / 255.0,
            b as f32 / 255.0,
            a as f32 / 255.0,
        )
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
    // pub fn clear(&mut self, r: u8, g: u8, b: u8) {
    //     self.draw_commands.push(DrawCommand::Clear(r, g, b))
    // }
    pub fn circle(&mut self, x: f32, y: f32, radius: f32) {
        if self.draw_fill {
            // let mut fill_tess = FillTessellator::new();
            self.fill_tess
                .tessellate_circle(
                    point(x, y),
                    radius,
                    &self.fill_options,
                    &mut BuffersBuilder::new(
                        &mut self.geometry,
                        VertexConstructor::new_only_color(self.fill_color, self.transform),
                    ),
                )
                .unwrap();
        }
        if self.draw_stroke {
            // let mut stroke_tess = StrokeTessellator::new();
            self.stroke_tess
                .tessellate_circle(
                    point(x, y),
                    radius,
                    &self.stroke_options,
                    &mut BuffersBuilder::new(
                        &mut self.geometry,
                        VertexConstructor::new_only_color(self.stroke_color, self.transform),
                    ),
                )
                .unwrap();
        }
    }
    pub fn rect(&mut self, x: f32, y: f32, w: f32, h: f32) {
        if self.draw_fill {
            // let mut fill_tess = FillTessellator::new();
            self.fill_tess
                .tessellate_rectangle(
                    &Box2D::new(point(x, y), point(x + w, y + h)),
                    &self.fill_options,
                    &mut BuffersBuilder::new(
                        &mut self.geometry,
                        VertexConstructor::new_only_color(self.fill_color, self.transform),
                    ),
                )
                .unwrap();
        }
        if self.draw_stroke {
            // let mut stroke_tess = StrokeTessellator::new();
            self.stroke_tess
                .tessellate_rectangle(
                    &Box2D::new(point(x, y), point(x + w, y + h)),
                    &self.stroke_options,
                    &mut BuffersBuilder::new(
                        &mut self.geometry,
                        VertexConstructor::new_only_color(self.stroke_color, self.transform),
                    ),
                )
                .unwrap();
        }
    }
    pub fn rounded_rect(&mut self, x: f32, y: f32, w: f32, h: f32, r: f32) {
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
        if self.draw_fill {
            self.fill_tess
                .tessellate_path(
                    &rect_path,
                    &self.fill_options,
                    &mut BuffersBuilder::new(
                        &mut self.geometry,
                        VertexConstructor::new_only_color(self.fill_color, self.transform),
                    ),
                )
                .unwrap();
        }
        if self.draw_stroke {
            // let mut stroke_tess = StrokeTessellator::new();
            self.stroke_tess
                .tessellate_path(
                    &rect_path,
                    &self.stroke_options,
                    &mut BuffersBuilder::new(
                        &mut self.geometry,
                        VertexConstructor::new_only_color(self.stroke_color, self.transform),
                    ),
                )
                .unwrap();
        }
    }

    pub fn translate(&mut self, x: f32, y: f32) {
        self.transform *= Matrix3::new(1.0, 0.0, x, 0.0, 1.0, y, 0.0, 0.0, 1.0);
    }
    pub fn rotate(&mut self, angle: f32) {
        let cos = angle.cos();
        let sin = angle.sin();

        self.transform *= Matrix3::new(cos, -sin, 0.0, sin, cos, 0.0, 0.0, 0.0, 1.0);
    }
    pub fn scale(&mut self, x: f32, y: f32) {
        self.transform *= Matrix3::new(x, 0.0, 0.0, 0.0, y, 0.0, 0.0, 0.0, 1.0);
    }
    pub fn push(&mut self) {
        self.transform_stack.push(self.transform);
    }
    pub fn pop(&mut self) {
        if let Some(t) = self.transform_stack.pop() {
            self.transform = t
        }
    }

    pub fn line(&mut self, x0: f32, y0: f32, x1: f32, y1: f32) {
        if self.draw_stroke {
            let mut builder = Path::builder();
            // builder.begin(point(x0, y0));
            // builder.line_to(point(x1, y1));
            // builder.close();
            builder.add_line_segment(&LineSegment {
                from: point(x0, y0),
                to: point(x1, y1),
            });
            let path = builder.build();
            self.stroke_tess
                .tessellate_path(
                    &path,
                    &self.stroke_options,
                    &mut BuffersBuilder::new(
                        &mut self.geometry,
                        VertexConstructor::new_only_color(self.stroke_color, self.transform),
                    ),
                )
                .unwrap();
        }
    }

    pub fn draw_image(&mut self, x: f32, y: f32, w: Option<f32>, h: Option<f32>, tinted: bool) {
        if let Some(tex) = self.current_texture {
            let w = w.unwrap_or(tex.w as f32);
            let h = h.unwrap_or(tex.h as f32);

            let (x0, y0, x1, y1) = (0.0, 0.0, 1.0, 1.0);

            let vert_count = self.geometry.vertices.len() as u32;

            if !tinted {
                self.geometry.vertices.extend(&[
                    VertexConstructor::new_textured((x0, y1), self.transform).with_pos(x, y),
                    VertexConstructor::new_textured((x1, y1), self.transform).with_pos(x + w, y),
                    VertexConstructor::new_textured((x1, y0), self.transform)
                        .with_pos(x + w, y + h),
                    VertexConstructor::new_textured((x0, y0), self.transform).with_pos(x, y + h),
                ]);
            } else {
                self.geometry.vertices.extend(&[
                    VertexConstructor::new_textured_tinted(
                        self.fill_color,
                        (x0, y1),
                        self.transform,
                    )
                    .with_pos(x, y),
                    VertexConstructor::new_textured_tinted(
                        self.fill_color,
                        (x1, y1),
                        self.transform,
                    )
                    .with_pos(x + w, y),
                    VertexConstructor::new_textured_tinted(
                        self.fill_color,
                        (x1, y0),
                        self.transform,
                    )
                    .with_pos(x + w, y + h),
                    VertexConstructor::new_textured_tinted(
                        self.fill_color,
                        (x0, y0),
                        self.transform,
                    )
                    .with_pos(x, y + h),
                ]);
            }
            self.geometry.indices.extend(&[
                vert_count,
                2 + vert_count,
                1 + vert_count,
                3 + vert_count,
                2 + vert_count,
                vert_count,
            ])
        }
    }

    pub fn draw_image_cropped(
        &mut self,
        x: f32,
        y: f32,
        w: Option<f32>,
        h: Option<f32>,
        region: (f32, f32, f32, f32),
        tinted: bool,
    ) {
        if let Some(tex) = self.current_texture {
            let texw = tex.w as f32;
            let texh = tex.h as f32;
            let w = w.unwrap_or(region.2 - region.0);
            let h = h.unwrap_or(region.3 - region.1);

            let (x0, y0, x1, y1) = (
                region.0 / texw,
                region.1 / texh,
                region.2 / texw,
                region.3 / texh,
            );

            let vert_count = self.geometry.vertices.len() as u32;

            if !tinted {
                self.geometry.vertices.extend(&[
                    VertexConstructor::new_textured((x0, y1), self.transform).with_pos(x, y),
                    VertexConstructor::new_textured((x1, y1), self.transform).with_pos(x + w, y),
                    VertexConstructor::new_textured((x1, y0), self.transform)
                        .with_pos(x + w, y + h),
                    VertexConstructor::new_textured((x0, y0), self.transform).with_pos(x, y + h),
                ]);
            } else {
                self.geometry.vertices.extend(&[
                    VertexConstructor::new_textured_tinted(
                        self.fill_color,
                        (x0, y1),
                        self.transform,
                    )
                    .with_pos(x, y),
                    VertexConstructor::new_textured_tinted(
                        self.fill_color,
                        (x1, y1),
                        self.transform,
                    )
                    .with_pos(x + w, y),
                    VertexConstructor::new_textured_tinted(
                        self.fill_color,
                        (x1, y0),
                        self.transform,
                    )
                    .with_pos(x + w, y + h),
                    VertexConstructor::new_textured_tinted(
                        self.fill_color,
                        (x0, y0),
                        self.transform,
                    )
                    .with_pos(x, y + h),
                ]);
            }
            self.geometry.indices.extend(&[
                vert_count,
                2 + vert_count,
                1 + vert_count,
                3 + vert_count,
                2 + vert_count,
                vert_count,
            ])
        }
    }
    pub fn set_blend_mode(&mut self, mode: BlendMode) {
        self.draw_chunks.push(DrawChunk {
            start_index: self.geometry.indices.len() as u32,
            blend_mode: Some(mode),
            texture: None,
        });
        self.current_blend_mode = mode;
    }
    pub fn set_current_texture(&mut self, tex: LoadedTexture) {
        self.draw_chunks.push(DrawChunk {
            start_index: self.geometry.indices.len() as u32,
            blend_mode: None,
            texture: Some(tex),
        });
        self.current_texture = Some(tex)
    }
}
